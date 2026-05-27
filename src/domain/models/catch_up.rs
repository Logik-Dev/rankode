use crate::domain::{LibraryItemId, MediaFileId};

pub struct UnprocessedFile {
    pub media_file_id: MediaFileId,
    /// None  → metadata not yet fetched  → re-emit FileDiscovered
    /// Some  → metadata fetched but no decision → re-emit MetadataFetched
    pub library_item_id: Option<LibraryItemId>,
}

pub struct QueuedTranscode {
    pub media_file_id: MediaFileId,
    /// true when the file was in 'transcoding' state — the process crashed mid-encode
    pub is_crashed: bool,
    pub crf: u8,
    pub dry_run: bool,
}
