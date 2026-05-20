#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("resolution is not valid")]
    InvalidResolution,
    #[error("bitrate is not valid")]
    InvalidBitrate,
    #[error("framerate is not valid")]
    InvalidFramerate,
    #[error("unknown media file status: {0}")]
    UnknownStatus(String),
}
