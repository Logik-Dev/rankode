use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub(super) struct NotificationPayload {
    pub event_type: String,
    pub media_file_id: Option<Uuid>,
    pub library_item_id: Option<Uuid>,
}
