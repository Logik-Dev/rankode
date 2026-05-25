use crate::domain::{AbsoluteFilePath, FileSizeBytes, FileName};

pub struct TranscodeOutput {
    pub filename: FileName,
    pub path: AbsoluteFilePath,
    pub size_bytes: FileSizeBytes,
    pub encode_duration_secs: i32,
}
