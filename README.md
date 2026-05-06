# rankode

HEVC re-encoding queue processor. Scans media directories, extracts technical metadata via ffprobe, enriches with movie metadata via Radarr, and decides whether to re-encode files to HEVC based on compression potential.

## Quick Start

```bash
# Run database migrations
cargo run -- migrate

# Scan a directory
cargo run -- scan /path/to/media

# Start watching for new files (dry run)
cargo run -- watch --dry-run

# Start watching for new files (live)
cargo run -- watch
```

## Configuration

Set environment variables before running:

```bash
# Database
export DB_SOCKET_DIR=/tmp    # or use DB_HOST/DB_PORT for TCP
export DB_NAME=rankode

# Radarr (required)
export RADARR_URL=http://localhost:7878
export RADARR_API_KEY=your_api_key

# Transcoding thresholds (optional)
export RANKODE_MIN_FILE_SIZE_GB=2.0
export RANKODE_MIN_BITS_PER_PIXEL=0.04
export RANKODE_MIN_COMPRESSION_POTENTIAL=1.0
```

## Architecture

PostgreSQL is used as both persistent storage and message queue via `NOTIFY`/`LISTEN`.

```
┌─────────┐     ┌──────────────┐     ┌─────────┐
│  scan   │────▶│  media_files │◀────│  watch  │
└─────────┘     └──────────────┘     └─────────┘
                       │                    │
                       ▼                    ▼
              ┌──────────────────┐  ┌─────────────────┐
              │     events       │  │   Radarr API    │
              └──────────────────┘  └─────────────────┘
                       ▲
                       │
              ┌──────────────────┐
              │ PostgresListener │
              └──────────────────┘
```

PostgreSQL triggers emit `pg_notify` on INSERT to `events` table, workers listen via `PostgresListener`.

## Project Structure

```
src/
├── main.rs                    # Entry point, dependency wiring
├── cli.rs                     # Command enum with clap derive
├── domain/
│   ├── ports.rs               # Traits: FetchedLibraryItemOrchestrator, LibraryItemProvider,
│   │                          #   EventListener, MediaFileAnalyzer, FileScanner, repositories
│   ├── models/
│   │   ├── mod.rs
│   │   ├── media_file.rs      # MediaFile, MediaFileStatus, VideoCodec
│   │   ├── library_item.rs    # LibraryItem
│   │   └── event.rs          # Event, EventType, EventNotification, SkipReason
│   └── services/
│       ├── mod.rs
│       ├── scan_folder.rs           # ScanFolderUseCase (8 concurrent analyses)
│       ├── watch_events.rs          # WatchEventUseCase (event dispatcher)
│       ├── process_discovered.rs    # ProcessDiscoveredFileUseCase → Radarr lookup
│       ├── process_fetched.rs       # ProcessFetchedLibraryItemUseCase → decision maker
│       └── take_decision.rs        # TakeTranscodeDecisionUseCase, crf_from_rating_and_bpp()
└── infra/
    ├── config.rs              # Config::from_env() - all environment variables
    ├── ffmpeg/
    │   ├── mod.rs
    │   ├── models.rs          # FfprobeOutput, FfprobeStream, FfprobeFormat
    │   └── probe.rs           # Ffprobe, MediaFileAnalyzer impl
    ├── http/
    │   ├── mod.rs
    │   ├── radarr.rs          # RadarrProvider, LibraryItemProvider impl
    │   └── models.rs          # RadarrMovie, RadarrRatings
    ├── listener/
    │   ├── mod.rs
    │   ├── postgres_listener.rs  # PostgresEventListener, EventListener impl
    │   └── models.rs             # NotificationPayload, PgEventType
    ├── repository/
    │   ├── mod.rs
    │   ├── models.rs          # MediaFileRow, LibraryItemRow, UpsertResult<T>
    │   ├── media_file.rs      # insert_media_file_inner, link_to_library_item_inner
    │   ├── library_item.rs    # insert_library_item_inner, LibraryItemRepository
    │   ├── event.rs           # insert_event_inner, EventRepository
    │   └── orchestrator.rs    # PostgresRepository (all trait impls)
    └── scanner/
        ├── mod.rs
        └── tokio.rs           # TokioScanner (FileScanner impl), VIDEO_EXTENSIONS

migrations/
└── 1_init.sql                 # PostgreSQL schema
```

## File Status Flow

```
active → pending    (transcode decision made, queued for encoding)
     → transcoding  (encoding in progress)
     → transcoded   (successful HEVC encode)
     → disappeared  (file missing during scan)
```

## Events (events table)

**Scan**: `file_discovered`, `file_updated`, `file_disappeared`
**Watch/Metadata**: `metadata_fetched`, `metadata_fetch_failed`
**Watch/Transcode**: `transcode_analyzed`, `transcode_skipped`, `transcode_started`, `transcode_completed`, `transcode_failed`

## CLI Commands

### `rankode migrate`
Runs SQLx migrations against the PostgreSQL database.

### `rankode scan [PATH]`
Scans directory recursively (default: `.`), inserts new files and updates `last_seen_at` on existing ones.
- Files not found in scan but present in DB → marked `disappeared`
- Emits `file_discovered` event for new files via PostgreSQL notify trigger
- Runs up to **8 concurrent analyses**

### `rankode watch [--dry-run] [--scan PATH]`
- `--dry-run`: If true, do not transcode pending files
- `--scan`: Optionally run a scan first before watching
- Listens to PostgreSQL NOTIFY events and dispatches to workers (up to **8 concurrent**)
- Handles `file_discovered` and `metadata_fetched` events

## Video Extensions Supported

`mp4`, `mkv`, `avi`, `mov`, `mpeg`, `mpg`

## Compression Analysis

Files are analyzed based on `bits_per_pixel` (bitrate / pixels_per_second) and IMDb rating:

```
compression_potential = (bits_per_pixel - 0.04) * 10 * resolution_factor
```

Where `resolution_factor` is 3.0 (4K ≥ 3840×2160), 1.5 (1080p ≥ 1920×1080), 1.0 (720p ≥ 1280×720), 0.6 (other).

CRF is determined by IMDb rating (better movies = lower CRF) adjusted by bits_per_pixel.

| IMDb Rating | Base CRF | bpp ≥ 0.15 | bpp ≥ 0.08 | bpp ≥ 0.05 | bpp < 0.05 |
|-------------|----------|------------|------------|------------|------------|
| ≥ 7.5       | 22       | 21         | 22         | 23         | 24         |
| ≥ 6.0       | 24       | 23         | 24         | 25         | 26         |
| ≥ 4.0       | 26       | 25         | 26         | 27         | 28         |
| < 4.0       | 28       | 27         | 28         | 29         | 30         |

### SkipReason enum
`CodecNotH264`, `FileTooSmall`, `AlreadyCompressed`, `InsufficientCompressionPotential`, `AlreadyTranscoded`, `FileDisappeared`, `TranscodeInProgress`

## Configuration Thresholds

| Variable | Default | Description |
|----------|---------|-------------|
| `RANKODE_MIN_FILE_SIZE_GB` | 2.0 | Minimum file size to consider |
| `RANKODE_MIN_BITS_PER_PIXEL` | 0.04 | Files below this are considered already compressed |
| `RANKODE_MIN_COMPRESSION_POTENTIAL` | 1.0 | Minimum compression potential to transcode |

## Encoding

### Platform-specific Encoders
- **macOS**: `hevc_videotoolbox` (Apple Silicon)
- **Linux NVIDIA**: `hevc_nvenc`
- **Fallback**: `libx265`

## Prerequisites

- `ffprobe` (part of ffmpeg) — `brew install ffmpeg` on macOS
- PostgreSQL database accessible

## Common Development Tasks

| Task | Command |
|------|---------|
| Build | `cargo build` |
| Run | `cargo run -- <command> [args]` |
| Test | `cargo test` |
| Lint | `cargo clippy --all-targets` |
| Format | `cargo fmt --all` |

## License

MIT