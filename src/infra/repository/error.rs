#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("media file not found")]
    MediaFileNotFound,
    #[error("library item not found")]
    LibraryItemNotFound,
    #[error("database error: {0}")]
    Database(sqlx::Error),
    #[error("migration failed: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
}

impl RepositoryError {
    pub(super) fn from_sqlx(e: sqlx::Error, if_not_found: Self) -> Self {
        match e {
            sqlx::Error::RowNotFound => if_not_found,
            e => Self::Database(e),
        }
    }
}

impl From<sqlx::Error> for RepositoryError {
    fn from(e: sqlx::Error) -> Self {
        Self::Database(e)
    }
}
