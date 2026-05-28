use crate::domain::{LibraryItemId, MediaFileId};

/// Lightweight dispatch signal emitted by the PostgreSQL NOTIFY listener (read side).
/// Each variant triggers a specific use case in the event loop.
pub enum WorkerSignal {
    FileDiscovered(MediaFileId),
    MetadataFetched(LibraryItemId),
    TranscodeScored(MediaFileId),
    TranscodeApproved(MediaFileId, u8),
    TranscodeRejected(MediaFileId),
}
