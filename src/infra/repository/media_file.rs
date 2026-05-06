use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{Executor, Postgres, query, query_as};
use tracing::{debug, instrument};

use crate::{
    domain::{MediaFile, MediaFileRepository, MediaFileStatus, NewMediaFile},
    infra::{
        PostgressRepository,
        repository::models::{MediaFileRow, UpsertResult},
    },
};

/// PUBLIC
#[async_trait]
impl MediaFileRepository for PostgressRepository {
    #[instrument(skip(self), err)]
    async fn find_media_file_by_id(&self, media_file_id: i64) -> Result<MediaFile> {
        sqlx::query_as!(
            MediaFileRow,
            "SELECT * FROM media_files WHERE id = $1",
            media_file_id
        )
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .context("Failed to select media file")
    }
    #[instrument(skip(self), err)]
    async fn find_files_by_library_item(&self, library_item_id: i64) -> Result<Vec<MediaFile>> {
        query_as!(
            MediaFileRow,
            "SELECT * FROM media_files WHERE library_item_id = $1",
            library_item_id
        )
        .fetch_all(&self.pool)
        .await
        .map(|v| v.into_iter().map(|r| r.into()).collect())
        .context("Can't find any files for library item")
    }
}

/// PRIVATE INNER
#[instrument(skip(executor), err)]
pub(super) async fn insert_media_file_inner<'e, E>(
    executor: E,
    media_file: &NewMediaFile,
) -> Result<UpsertResult<i64>>
where
    E: Executor<'e, Database = Postgres>,
{
    debug!("Insert new media file");

    let row = sqlx::query!(
        "INSERT INTO media_files (root_dir, file_path, file_name, size_bytes, video_codec, height, width, framerate, bitrate_kbps, status, last_seen_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
            ON CONFLICT (file_path) DO UPDATE SET last_seen_at = NOW()
            RETURNING id, (xmax = 0) as inserted",
        media_file.root_dir,
        media_file.file_path,
        media_file.file_name,
        media_file.size_bytes as i64,
        media_file.video_codec.to_string(),
        media_file.height as i32,
        media_file.width as i32,
        media_file.framerate,
        media_file.bitrate_kbps as i32,
        "active"
    )
        .fetch_one(executor)
        .await?;

    if row.inserted.unwrap_or(false) {
        Ok(UpsertResult::Inserted(row.id))
    } else {
        Ok(UpsertResult::AlreadyExists(row.id))
    }
}

#[instrument(skip(executor), err)]
pub(super) async fn link_to_library_item_inner<'e, E>(
    executor: E,
    media_file_id: i64,
    library_item_id: i64,
) -> Result<i64>
where
    E: Executor<'e, Database = Postgres>,
{
    query!(
        r#"
        UPDATE media_files
        SET library_item_id = $1
        WHERE id = $2
        RETURNING id
        "#,
        library_item_id,
        media_file_id
    )
    .fetch_one(executor)
    .await
    .context("Failed to link file to library item")
    .map(|r| r.id)
}

#[instrument(skip(executor), err)]
pub(super) async fn update_file_status_inner<'e, E>(
    executor: E,
    status: MediaFileStatus,
    media_file_id: i64,
) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    let _ = sqlx::query!(
        "UPDATE media_files SET status = $1 WHERE id = $2",
        status.as_str(),
        media_file_id
    )
    .execute(executor)
    .await?;

    Ok(())
}
