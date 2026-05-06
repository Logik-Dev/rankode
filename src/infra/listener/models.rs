use serde::Deserialize;

use crate::domain::{EventNotification, EventType};

#[derive(Debug, Deserialize)]
pub(super) struct NotificationPayload {
    pub event_type: PgEventType,
    pub media_file_id: Option<i64>,
    pub library_item_id: Option<i64>,
}

impl From<NotificationPayload> for EventNotification {
    fn from(notif: NotificationPayload) -> Self {
        let event_type: EventType = notif.event_type.into();
        let id = match event_type {
            EventType::MetadataFetched => notif.library_item_id.unwrap(),
            _ => notif.media_file_id.unwrap(),
        };

        Self { id, event_type }
    }
}

#[derive(Debug, Clone, sqlx::Type, Deserialize)]
#[sqlx(type_name = "event_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub(super) enum PgEventType {
    FileDiscovered,
    FileDisappeared,
    MetadataFetched,
    MetadataFetchFailed,
    TranscodeAnalyzed,
    TranscodeSkipped,
    TranscodeStarted,
    TranscodeCompleted,
    TranscodeFailed,
}

impl From<PgEventType> for EventType {
    fn from(event_type: PgEventType) -> Self {
        match event_type {
            PgEventType::FileDiscovered => EventType::FileDiscovered,
            PgEventType::FileDisappeared => EventType::FileDisappeared,
            PgEventType::MetadataFetched => EventType::MetadataFetched,
            PgEventType::MetadataFetchFailed => EventType::MetadataFetchFailed,
            PgEventType::TranscodeStarted => EventType::TranscodeStarted,
            PgEventType::TranscodeFailed => EventType::TranscodeFailed,
            PgEventType::TranscodeSkipped => EventType::TranscodeSkipped,
            PgEventType::TranscodeAnalyzed => EventType::TranscodeAnalyzed,
            PgEventType::TranscodeCompleted => EventType::TranscodeCompleted,
        }
    }
}
