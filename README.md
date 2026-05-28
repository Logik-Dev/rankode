# rankode

> **A deliberately over-engineered HEVC re-encoding queue processor.**
>
> This project is a learning lab — an excuse to explore **Domain-Driven Design**, idiomatic **Rust**, and **PostgreSQL as a pseudo-event bus** while building something concrete and useful. Expect hexagonal architecture where a simple script would do. That's the point.

---

## What it does

rankode scans your media library, extracts technical metadata via `ffprobe`, enriches files with movie info from Radarr, and decides whether re-encoding to HEVC is worthwhile based on compression potential and IMDb rating.

Candidates are sent as **iOS notifications** through Home Assistant. You approve or reject each one from your phone. After transcoding, your HA dashboard shows completed files with their disk gain so you can delete the original when ready.

PostgreSQL plays a dual role: **persistent storage** and **message queue** via `NOTIFY`/`LISTEN`. A trigger fires `pg_notify` on every event insert; workers pick it up and dispatch the next step.

---

## Quick Start

```bash
# 1. Run database migrations
cargo run -- migrate

# 2. Scan a media directory
cargo run -- scan /path/to/media

# 3. Start the watch daemon
cargo run -- watch
```

---

## Architecture

rankode follows **hexagonal architecture**: `domain/` holds traits (ports) and models, `infra/` holds all implementations, `application/` holds use cases. The three layers never import in the wrong direction.

```
┌──────────────────────────────────────────────────────────┐
│                        Application                        │
│  ScanFolder  ProcessDiscovered  AnalyzeFile  Transcode   │
│  NotifyNextCandidate  ProcessApproval  DeleteSource  ...  │
├─────────────────────┬────────────────────────────────────┤
│       Domain        │              Infra                  │
│  Entities · Ports   │  PostgreSQL · MQTT · ffmpeg · HTTP │
│  Events · Services  │  Radarr · MCP server               │
└─────────────────────┴────────────────────────────────────┘
```

---

## File Status State Machine

Every media file moves through a fixed set of statuses. Transitions are always driven by a `DomainEvent` written atomically to the `events` table.

```
                    ┌─────────────────────────────────────────────────────┐
                    │                  scan                                │
                    ▼                                                      │
             ┌────────────┐   analyze    ┌───────────┐                    │
  discover   │   active   │─────────────▶│ candidate │                    │
─────────────▶            │              └─────┬─────┘                    │
             └──────┬─────┘                    │ notify                   │
                    │ disappear                ▼                           │
                    ▼              ┌──────────────────┐                   │
             ┌────────────┐        │     notified     │                   │
             │disappeared │        └────────┬─────────┘                   │
             └────────────┘         approve │  │ reject                   │
                                            ▼  ▼                          │
                                   ┌────────┐  ┌────────┐                 │
                                   │approved│  │rejected│─────────────────┘
                                   └───┬────┘  └────────┘
                                       │ start
                                       ▼
                                ┌─────────────┐
                                │ transcoding │
                                └──────┬──────┘
                                fail   │   complete
                          ┌────────────┼──────────────────┐
                          ▼            ▼                   │
                       (active)  ┌────────────┐           │
                                 │ transcoded │           │
                                 └──────┬─────┘           │
                                        │ delete source    │
                                        ▼                  │
                                ┌──────────────┐           │
                                │source_deleted│           │
                                └──────────────┘           │
                                                           │
                    notify_next_candidate ◀────────────────┘
```

| Status | Description |
|--------|-------------|
| `active` | File discovered, awaiting metadata and analysis |
| `candidate` | Scored as worth transcoding, waiting for a notification slot |
| `notified` | Notification sent to user, awaiting approval |
| `approved` | User approved, queued for transcoding |
| `rejected` | User rejected, returned to queue for next candidate |
| `transcoding` | ffmpeg encode in progress |
| `transcoded` | Encode complete, source file still on disk |
| `source_deleted` | Source file deleted by user via HA dashboard |
| `disappeared` | File not found during last scan |

---

## Signal & Event Model

Three distinct concepts carry information through the system:

### DomainEvent — facts (write side, persisted)

Written to the `events` table inside a transaction, atomically with any status change. Represent immutable facts about what happened.

| Event | Trigger |
|-------|---------|
| `file_discovered` | New file found during scan |
| `metadata_fetched` | Radarr lookup succeeded |
| `metadata_fetch_failed` | Radarr lookup failed |
| `transcode_scored` | File analyzed and scored as candidate |
| `transcode_ineligible` | File analyzed and skipped (with reason) |
| `transcode_notified` | Candidate sent to user |
| `transcode_approved` | User approved via MQTT |
| `transcode_rejected` | User rejected via MQTT |
| `transcode_started` | ffmpeg encode started |
| `transcode_completed` | ffmpeg encode finished (gain_bytes recorded) |
| `transcode_failed` | ffmpeg encode errored |
| `source_deleted` | Original file deleted by user |

### WorkerSignal — dispatch signals (read side, ephemeral)

Emitted by the PostgreSQL NOTIFY listener when an event is inserted. Lightweight — carries only the IDs needed to trigger the next use case. Never persisted.

| Signal | Emitted on event | Triggers use case |
|--------|-----------------|-------------------|
| `FileDiscovered(id)` | `file_discovered` | `ProcessDiscoveredFileUseCase` |
| `MetadataFetched(id)` | `metadata_fetched` | `AnalyzeFileUseCase` |
| `TranscodeScored(id)` | `transcode_scored` | `NotifyNextCandidateUseCase` |
| `TranscodeApproved(id, crf)` | `transcode_approved` | `TranscodeFileUseCase` → `NotifyNextCandidateUseCase` |
| `TranscodeRejected(id)` | `transcode_rejected` | `NotifyNextCandidateUseCase` |

### ApprovalSignal — user commands (MQTT, inbound)

Received from Home Assistant via MQTT. Represent user intent, not facts — they produce `DomainEvent`s when processed.

| Signal | MQTT topic | Payload |
|--------|-----------|---------|
| `Approved { id, crf, actor }` | `rankode/approval` | `{"status":"approved","media_file_id":"…","crf":22,"actor":"…"}` |
| `Rejected { id, actor }` | `rankode/approval` | `{"status":"rejected","media_file_id":"…","actor":"…"}` |
| `DeleteSource { id }` | `rankode/delete_source` | `{"media_file_id":"…"}` |

---

## Full Workflow

### Phase 1 — Discovery & Analysis

```
cargo run -- scan /media
        │
        ├─ ffprobe each file
        │
        └─ INSERT media_files + file_discovered event
                  │
                  └─ pg_notify ──► WorkerSignal::FileDiscovered
                                         │
                                         ▼
                                  ProcessDiscoveredFileUseCase
                                  └─ Radarr lookup
                                  └─ INSERT metadata_fetched event
                                             │
                                             └─ pg_notify ──► WorkerSignal::MetadataFetched
                                                                    │
                                                                    ▼
                                                             AnalyzeFileUseCase
                                                             └─ compute bpp, CRF, gain estimate
                                                             └─ INSERT transcode_scored event
                                                                        │
                                                                        └─ pg_notify ──► WorkerSignal::TranscodeScored
                                                                                               │
                                                                                               ▼
                                                                                        NotifyNextCandidateUseCase
                                                                                        └─ pick best candidate (highest gain)
                                                                                        └─ only if no file currently notified
                                                                                        └─ MQTT publish ──► Home Assistant
```

### Phase 2 — Approval (one at a time)

The notification slot enforces that only one file is in `notified` state at a time. After each approval or rejection, `NotifyNextCandidateUseCase` picks the next best candidate.

```
Home Assistant iOS notification
        │
        ├─ Approve ──► MQTT rankode/approval {"status":"approved","crf":22,...}
        │                      │
        │                      ▼
        │               ProcessApprovalUseCase
        │               └─ status: notified → approved
        │               └─ INSERT transcode_approved event
        │                         │
        │                         └─ pg_notify ──► WorkerSignal::TranscodeApproved
        │                                                │
        │                                                ▼
        │                                         TranscodeFileUseCase
        │                                         └─ ffmpeg encode
        │                                         └─ INSERT transcode_completed event
        │                                         └─ MQTT autodiscovery ──► HA button + sensor
        │                                                │
        │                                                └─ NotifyNextCandidateUseCase (next candidate)
        │
        └─ Reject ──► MQTT rankode/approval {"status":"rejected",...}
                             │
                             └─ NotifyNextCandidateUseCase (next candidate)
```

### Phase 3 — Source Deletion (HA dashboard)

After transcoding, a button and a gain sensor appear automatically in Home Assistant via MQTT autodiscovery. The user can delete the original file from any HA dashboard.

```
Home Assistant dashboard
        │
        └─ Press "Delete source" button
                  │
                  └─ MQTT rankode/delete_source {"media_file_id":"…"}
                             │
                             ▼
                      DeleteSourceUseCase
                      └─ remove source file from disk
                      └─ status: transcoded → source_deleted
                      └─ INSERT source_deleted event
                      └─ MQTT empty payload ──► HA removes button + sensor
```

---

## Home Assistant Integration

rankode communicates with Home Assistant exclusively via MQTT.

### Outbound (rankode → HA)

| Topic | Retained | Content |
|-------|----------|---------|
| `rankode/candidate` | No | JSON notification for iOS actionable alert |
| `homeassistant/button/rankode_{id}/config` | Yes | HA autodiscovery — delete source button |
| `homeassistant/sensor/rankode_{id}_gain/config` | Yes | HA autodiscovery — gain in GB sensor |
| `rankode/transcoded/{id}/state` | Yes | Gain value (e.g. `"1.23"`) |

### Inbound (HA → rankode)

| Topic | Content |
|-------|---------|
| `rankode/approval` | Approve or reject a candidate |
| `rankode/delete_source` | Delete source of a transcoded file |

---

## Compression Analysis

Files are skipped if they don't pass the minimum thresholds. For eligible files, compression potential is:

```
compression_potential = (bits_per_pixel - 0.04) × 10 × resolution_factor
```

`resolution_factor`: **3.0** (4K ≥ 3840×2160) · **1.5** (1080p ≥ 1920×1080) · **1.0** (720p ≥ 1280×720) · **0.6** (other)

### CRF selection

CRF is tuned by IMDb rating (better films = lower CRF = higher quality) with a bpp adjustment:

| IMDb Rating | Base CRF | bpp ≥ 0.15 | bpp ≥ 0.08 | bpp ≥ 0.05 | bpp < 0.05 |
|-------------|----------|------------|------------|------------|------------|
| ≥ 7.5       | 22       | 21         | 22         | 23         | 24         |
| ≥ 6.0       | 24       | 23         | 24         | 25         | 26         |
| ≥ 4.0       | 26       | 25         | 26         | 27         | 28         |
| < 4.0       | 28       | 27         | 28         | 29         | 30         |
| None        | 24       | 23         | 24         | 25         | 26         |

### Skip reasons

| Reason | Description |
|--------|-------------|
| `ExcludedCodec` | Not H.264 — already HEVC or unsupported |
| `FileTooSmall` | Below `RANKODE_MIN_FILE_SIZE_GB` |
| `AlreadyCompressed` | bpp below `RANKODE_MIN_BITS_PER_PIXEL` |
| `InsufficientCompressionPotential` | Gain estimate below threshold |
| `AlreadyTranscoded` | File already encoded by rankode |
| `TranscodeInProgress` | File is candidate/notified/approved/transcoding |
| `FileDisappeared` | File not found on disk |

---

## Encoding

Platform-specific encoders are selected automatically at startup:

| Platform | Encoder |
|----------|---------|
| macOS (Apple Silicon) | `hevc_videotoolbox` |
| Linux + NVIDIA | `hevc_nvenc` |
| Fallback | `libx265` |

---

## CLI Reference

### `rankode migrate`
Runs SQLx migrations against the configured PostgreSQL database.

### `rankode scan [PATH]`
Recursively scans a directory (default: `.`) for video files.
- New files are inserted and analyzed
- Existing files get `last_seen_at` updated
- Missing files are marked `disappeared`
- Runs up to **8 concurrent ffprobe analyses**

### `rankode watch [--dry-run] [--scan PATH]`
Starts the event loop: PostgreSQL NOTIFY listener + MQTT approval listener + MCP server.
- `--dry-run` — analyze everything but skip actual transcoding
- `--scan PATH` — run a scan pass before entering watch mode
- Up to **8 concurrent workers**

---

## Configuration

### Database

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_SOCKET_DIR` | — | Unix socket directory (overrides TCP) |
| `DB_HOST` | `localhost` | PostgreSQL host |
| `DB_PORT` | `5433` | PostgreSQL port |
| `DB_USER` | `$USER` | Database user |
| `DB_PASSWORD` | — | Database password |
| `DB_NAME` | `rankode` | Database name |

### Radarr

| Variable | Description |
|----------|-------------|
| `RADARR_URL` | e.g. `http://localhost:7878` |
| `RADARR_API_KEY` | Your Radarr API key |

### MQTT

| Variable | Default | Description |
|----------|---------|-------------|
| `MQTT_HOST` | `localhost` | MQTT broker host |
| `MQTT_PORT` | `1883` | MQTT broker port |

### Thresholds

| Variable | Default | Description |
|----------|---------|-------------|
| `RANKODE_MIN_FILE_SIZE_GB` | `2.0` | Files below this size are skipped |
| `RANKODE_MIN_BITS_PER_PIXEL` | `0.04` | Files below this bpp are already compressed |
| `RANKODE_MIN_COMPRESSION_POTENTIAL` | `1.0` | Minimum potential to trigger transcode |

---

## Prerequisites

- **ffprobe** — `brew install ffmpeg` on macOS
- **PostgreSQL** — accessible via TCP or Unix socket
- **MQTT broker** — e.g. Mosquitto (`brew install mosquitto`)

---

## Dev Commands

| Task | Command |
|------|---------|
| Build | `cargo build` |
| Run | `cargo run -- <command> [args]` |
| Test | `cargo test` |
| Lint | `cargo clippy --all-targets` |
| Format | `cargo fmt --all` |

---

## Supported Extensions

`mp4` · `mkv` · `avi` · `mov` · `mpeg` · `mpg`

---

## License

MIT
