-- 001_init.sql
-- Creates media_files and library_items tables
-- One library_item can have multiple media_files (original + reencoded)

-- Create library_items table
CREATE TABLE library_items (
    id                  BIGSERIAL     PRIMARY KEY,
    title               TEXT          NOT NULL,
    year                INTEGER,
    imdb_id             TEXT          NOT NULL UNIQUE,
    genres              TEXT[]        NOT NULL,
    overview            TEXT,
    imdb_rating         REAL,
    created_at          TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_library_items_imdb_id ON library_items(imdb_id);

CREATE TABLE media_files (
    id                  BIGSERIAL    PRIMARY KEY,
    library_item_id     BIGINT       REFERENCES library_items(id),
    root_dir            TEXT         NOT NULL,
    file_path           TEXT         NOT NULL UNIQUE,
    file_name           TEXT         NOT NULL,
    size_bytes          BIGINT       NOT NULL,
    -- Technical metadata (from ffprobe)
    video_codec         TEXT         NOT NULL,
    height              INTEGER      NOT NULL,
    width               INTEGER      NOT NULL,
    bitrate_kbps        INTEGER      NOT NULL,
    framerate           DOUBLE PRECISION         NOT NULL,
    -- Status: active, transcoded, disappeared
    status              TEXT         NOT NULL DEFAULT 'active',
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_seen_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Indexes for media_files
CREATE INDEX idx_media_files_status ON media_files(status);
-- Search for files by status (e.g. find all 'disappeared' files for cleanup)

CREATE INDEX idx_media_files_library_item ON media_files(library_item_id);
-- Join media_files to library_items to get metadata (title, year, imdb_id...)

CREATE INDEX idx_media_files_root_dir ON media_files(root_dir);
-- Filter files by root_dir when scanning a directory to detect disappeared files

CREATE INDEX idx_media_files_root_dir_status ON media_files(root_dir, status);
-- Filter files by root_dir AND status to find disappeared files in a specific directory

-- Media events table
CREATE TABLE events (
    id                  BIGSERIAL PRIMARY KEY,
    occurred_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    event_type          TEXT NOT NULL,

    -- anchoring: file OR library item, never both
    media_file_id       BIGINT REFERENCES media_files(id),
    library_item_id     BIGINT REFERENCES library_items(id),

    -- metrics
    compression_potential  REAL,
    bits_per_pixel         REAL,
    crf                    INT,
    skip_reason            TEXT,

    -- transcode result
    dst_media_file_id      BIGINT REFERENCES media_files(id),
    encode_duration_secs   INT,
    gain_bytes             BIGINT,

    -- error
    error_message          TEXT,

    -- dry run flag
    dry_run                BOOLEAN NOT NULL DEFAULT false
);

-- Trigger to notify on media_events insert
CREATE OR REPLACE FUNCTION notify_event()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(
        'events',
        json_build_object(
            'event_type', NEW.event_type,
            'media_file_id', NEW.media_file_id,
            'library_item_id', NEW.library_item_id
        )::text
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER events_notify
    AFTER INSERT ON events
    FOR EACH ROW EXECUTE FUNCTION notify_event();

-- Indexes for media_events
CREATE INDEX idx_media_events_file ON events(media_file_id);
-- Join events to media_files to get file details

CREATE INDEX idx_media_events_item ON events(library_item_id);
-- Join events to library_items to get movie/series info

CREATE INDEX idx_media_events_type ON events(event_type);
-- Filter events by type to find all events of a specific type (e.g. transcode_completed)
