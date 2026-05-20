# rankode 🎬

> **A deliberately over-engineered HEVC re-encoding queue processor.**
>
> This project is a learning lab — an excuse to explore **Domain-Driven Design**, idiomatic **Rust**, and **PostgreSQL as a pseudo-event bus** while building something concrete and useful. Expect hexagonal architecture where a simple script would do. That's the point.

---

## What it does

rankode scans your media library, extracts technical metadata via `ffprobe`, enriches files with movie info from Radarr, and decides whether re-encoding to HEVC is worthwhile — based on compression potential and IMDb rating.

PostgreSQL plays a dual role: **persistent storage** and **message queue** via `NOTIFY`/`LISTEN`. A trigger fires `pg_notify` on every event insert; workers pick it up and dispatch the next step.

---

## Quick Start

```bash
# 1. Run database migrations
cargo run -- migrate

# 2. Scan a media directory
cargo run -- scan /path/to/media

# 3. Watch for events (dry run — no actual transcoding)
cargo run -- watch --dry-run

# 4. Watch for events (live)
cargo run -- watch
```

---

## How it works

```
┌─────────┐     ┌──────────────┐     ┌─────────┐
│  scan   │────▶│  media_files │◀────│  watch  │
└─────────┘     └──────────────┘     └─────────┘
                       │                    │
                       ▼                    ▼
              ┌──────────────────┐  ┌─────────────────┐
              │     events       │  │   Radarr API    │
              └──────────────────┘  └─────────────────┘
                       │
                       ▼
              ┌──────────────────┐
              │  PostgresListener│  ◀── NOTIFY/LISTEN
              └──────────────────┘
```

### File lifecycle

```
active → pending      (transcode queued)
       → transcoding  (encoding in progress)
       → transcoded   (done ✓)
       → disappeared  (file gone during scan)
```

### Event flow

| Stage | Events |
|-------|--------|
| 🔍 Scan | `file_discovered`, `file_updated`, `file_disappeared` |
| 📡 Metadata | `metadata_fetched`, `metadata_fetch_failed` |
| 🎞️ Transcode | `transcode_decision_approved`, `transcode_decision_skipped`, `transcode_started`, `transcode_completed`, `transcode_failed` |

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
Listens for PostgreSQL `NOTIFY` events and dispatches workers.
- `--dry-run` — analyze everything but skip actual transcoding
- `--scan PATH` — run a scan pass before entering watch mode
- Up to **8 concurrent workers**

---

## Compression Analysis 📊

Files are skipped if they don't pass the minimum thresholds. For eligible files, compression potential is computed as:

```
compression_potential = (bits_per_pixel - 0.04) × 10 × resolution_factor
```

`resolution_factor`: **3.0** (4K), **1.5** (1080p), **1.0** (720p), **0.6** (other)

### CRF selection

CRF is tuned by IMDb rating (better films = lower CRF = higher quality) with a fine-grained bpp adjustment:

| IMDb Rating | Base CRF | bpp ≥ 0.15 | bpp ≥ 0.08 | bpp ≥ 0.05 | bpp < 0.05 |
|-------------|----------|------------|------------|------------|------------|
| ≥ 7.5       | 22       | 21         | 22         | 23         | 24         |
| ≥ 6.0       | 24       | 23         | 24         | 25         | 26         |
| ≥ 4.0       | 26       | 25         | 26         | 27         | 28         |
| < 4.0       | 28       | 27         | 28         | 29         | 30         |

### Skip reasons

`CodecNotH264` · `FileTooSmall` · `AlreadyCompressed` · `InsufficientCompressionPotential` · `AlreadyTranscoded` · `FileDisappeared` · `TranscodeInProgress`

---

## Encoding 🖥️

Platform-specific encoders are selected automatically:

| Platform | Encoder |
|----------|---------|
| macOS (Apple Silicon) | `hevc_videotoolbox` |
| Linux + NVIDIA | `hevc_nvenc` |
| Fallback | `libx265` |

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
