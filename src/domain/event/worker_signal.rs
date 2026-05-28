use crate::domain::{LibraryItemId, MediaFileId};

pub enum WorkerSignal {
    FileDiscovered(MediaFileId),
    MetadataFetched(LibraryItemId),
    TranscodeApproved(MediaFileId, u8),
    TranscodeRejected(MediaFileId),
}
