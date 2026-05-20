use crate::domain::{LibraryItemId, MediaFileId};

pub enum WorkerSignal {
    FileDiscovered(MediaFileId),
    MetadataFetched(LibraryItemId),
}
