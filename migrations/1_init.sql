CREATE TABLE library_items (
    id                  UUID          NOT NULL PRIMARY KEY,
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
    id                  UUID         NOT NULL PRIMARY KEY,
    library_item_id     UUID       REFERENCES library_items(id),
    root_dir            TEXT,
    file_path           TEXT         NOT NULL UNIQUE,
    file_name           TEXT         NOT NULL,
    size_bytes          BIGINT       NOT NULL,
    video_codec         TEXT         NOT NULL,
    height              INTEGER      NOT NULL,
    width               INTEGER      NOT NULL,
    bitrate_kbps        INTEGER      NOT NULL,
    framerate           DOUBLE PRECISION         NOT NULL,
    status              TEXT         NOT NULL DEFAULT 'active',
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_seen_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_media_files_status ON media_files(status);
CREATE INDEX idx_media_files_library_item ON media_files(library_item_id);
CREATE INDEX idx_media_files_root_dir ON media_files(root_dir);
CREATE INDEX idx_media_files_root_dir_status ON media_files(root_dir, status);

CREATE TABLE events (
    id                  BIGSERIAL PRIMARY KEY,
    occurred_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    event_type          TEXT NOT NULL,
    media_file_id       UUID REFERENCES media_files(id),
    library_item_id     UUID REFERENCES library_items(id),
    compression_potential  DOUBLE PRECISION,
    bits_per_pixel         DOUBLE PRECISION,
    crf                    SMALLINT,
    skip_reason            TEXT,
    dst_media_file_id      UUID REFERENCES media_files(id),
    encode_duration_secs   INT,
    gain_bytes             BIGINT,
    error_message          TEXT,
    actor                  TEXT
);

CREATE OR REPLACE FUNCTION notify_event()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(
        'events',
        json_build_object(
            'event_type', NEW.event_type,
            'media_file_id', NEW.media_file_id,
            'library_item_id', NEW.library_item_id,
            'crf', NEW.crf
        )::text
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER events_notify
    AFTER INSERT ON events
    FOR EACH ROW EXECUTE FUNCTION notify_event();

CREATE INDEX idx_media_events_file ON events(media_file_id);
CREATE INDEX idx_media_events_item ON events(library_item_id);
CREATE INDEX idx_media_events_type ON events(event_type);
