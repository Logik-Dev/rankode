use crate::domain::MediaFileId;

/// Data carried to the entity publisher after a successful transcode,
/// used to create a Home Assistant button + gain sensor via MQTT autodiscovery.
pub struct TranscodedNotification {
    pub media_file_id: MediaFileId,
    pub file_name: String,
    pub gain_bytes: i64,
}
