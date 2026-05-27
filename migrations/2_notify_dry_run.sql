CREATE OR REPLACE FUNCTION notify_event()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(
        'events',
        json_build_object(
            'event_type', NEW.event_type,
            'media_file_id', NEW.media_file_id,
            'library_item_id', NEW.library_item_id,
            'crf', NEW.crf,
            'dry_run', NEW.dry_run
        )::text
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
