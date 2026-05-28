use crate::domain::{LibraryItemId, MediaFileId, SkipReason};

#[derive(Debug)]
pub enum DomainEvent {
    FileDiscovered {
        media_file_id: MediaFileId,
    },
    // TODO FileUpdated { media_file_id: MediaFileId },
    // TODO FileDisappeared { media_file_id: MediaFileId },
    // TODO create a ValueObject for LibraryItemId
    MetadataFetched {
        library_item_id: LibraryItemId,
    },
    MetadataFetchFailed {
        media_file_id: MediaFileId,
        error: String,
    },
    TranscodeScored {
        media_file_id: MediaFileId,
        bpp: f64,
        compression_potential: f64,
        crf: u8,
        estimated_gain_bytes: u64,
    },
    TranscodeNotified {
        media_file_id: MediaFileId,
    },
    TranscodeIneligible {
        media_file_id: MediaFileId,
        skip_reason: SkipReason,
        bpp: Option<f64>,
        compression_potential: Option<f64>,
    },
    TranscodeApproved {
        media_file_id: MediaFileId,
        approved_by: String,
        crf: u8,
    },
    TranscodeRejected {
        media_file_id: MediaFileId,
        rejected_by: String,
    },
    TranscodeStarted {
        media_file_id: MediaFileId,
    },
    TranscodeCompleted {
        media_file_id: MediaFileId,
        dst_media_file_id: MediaFileId,
        encode_duration_secs: i32,
        gain_bytes: i64,
    },
    TranscodeFailed {
        media_file_id: MediaFileId,
        error: String,
    },
    SourceDeleted {
        media_file_id: MediaFileId,
    },
}
