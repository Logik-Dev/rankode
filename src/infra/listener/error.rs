#[derive(Debug, thiserror::Error)]
pub enum ListenerError {
    #[error("failed to connect to PostgreSQL listener: {0}")]
    ConnectionFailed(#[from] sqlx::Error),
}
