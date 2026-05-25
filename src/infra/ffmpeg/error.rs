#[derive(Debug, thiserror::Error)]
pub enum TranscoderError {
    #[error("failed to spawn ffmpeg: {0}")]
    SpawnFailed(#[from] std::io::Error),
    #[error("ffmpeg exited with code {0:?}")]
    ProcessFailed(Option<i32>),
    #[error("invalid input file path")]
    InvalidPath,
}
