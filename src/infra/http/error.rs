#[derive(Debug, thiserror::Error)]
pub enum RadarrError {
    #[error("request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("no Radarr result with IMDB ID for '{filename}'")]
    NoResultWithImdbId { filename: String },
}
