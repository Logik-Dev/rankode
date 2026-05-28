use crate::domain::{LibraryItemId, MediaFileId};

pub enum WorkerSignal {
    FileDiscovered(MediaFileId),
    MetadataFetched(LibraryItemId),
    TranscodeScored(MediaFileId),
    TranscodeApproved(MediaFileId, u8),
    TranscodeRejected(MediaFileId),
}
