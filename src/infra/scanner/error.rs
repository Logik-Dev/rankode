use crate::domain::ScannedFile;

#[derive(Debug, thiserror::Error)]
pub enum ScannerError {
    #[error("file path is not absolute")]
    AbsolutePath,
    #[error("filename not found")]
    FilenameNotFound,
    #[error("file size not found")]
    FileSizeNotFound,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("semaphore closed")]
    SemaphoreClosed(#[from] tokio::sync::AcquireError),
    #[error("channel closed")]
    ChannelClosed,
}

impl From<tokio::sync::mpsc::error::SendError<ScannedFile>> for ScannerError {
    fn from(_: tokio::sync::mpsc::error::SendError<ScannedFile>) -> Self {
        Self::ChannelClosed
    }
}
