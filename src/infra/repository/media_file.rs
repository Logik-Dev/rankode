use anyhow::Result;
use async_trait::async_trait;
use sqlx::{Executor, Postgres, query, query_as};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::{
    domain::{LibraryItemId, MediaFile, MediaFileId, MediaFileRepository, MediaFileStatus},
    infra::{
        PostgressRepository,
        repository::{error::RepositoryError, models::{MediaFileRow, UpsertResult}},
    },
};

/// PUBLIC
#[async_trait]
impl MediaFileRepository for PostgressRepository {
    #[instrument(skip(self), err)]
    async fn find_media_file_by_id(&self, media_file_id: &MediaFileId) -> Result<MediaFile> {
        let row: MediaFileRow = sqlx::query_as!(
            MediaFileRow,
            "SELECT * FROM media_files WHERE id = $1",
            media_file_id.as_uuid()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::from_sqlx(e, RepositoryError::MediaFileNotFound))?;

        row.try_into().map_err(Into::into)
    }
    #[instrument(skip(self), err)]
    async fn find_files_by_library_item(
        &self,
        library_item_id: &LibraryItemId,
    ) -> Result<Vec<MediaFile>> {
        let rows: Vec<MediaFileRow> = query_as!(
            MediaFileRow,
            "SELECT * FROM media_files WHERE library_item_id = $1",
            library_item_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        rows.into_iter()
            .map(|r| r.try_into().map_err(Into::into))
            .collect()
    }
}

/// PRIVATE INNER
#[instrument(skip(executor, media_file), err)]
pub(super) async fn insert_media_file_inner<'e, E>(
    executor: E,
    media_file: &MediaFile,
) -> Result<UpsertResult<Uuid>, RepositoryError>
where
    E: Executor<'e, Database = Postgres>,
{
    debug!("Insert new media file");

    let id = media_file.id.as_uuid();
    let codec = media_file.video_properties.video_codec.to_string();
    let height = media_file.video_properties.resolution.height() as i32;
    let width = media_file.video_properties.resolution.width() as i32;
    let bitrate_kbps = media_file
        .video_properties
        .bitrate
        .as_ref()
        .map(|b| (b.as_bps() / 1000) as i32)
        .unwrap_or(0);
    let framerate = media_file
        .video_properties
        .framerate
        .as_ref()
        .map(|f| f.as_f64())
        .unwrap_or(0.0);

    let row = sqlx::query!(
        "INSERT INTO media_files (id, file_path, file_name, size_bytes, video_codec, height, width, framerate, bitrate_kbps, status, last_seen_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'active', NOW())
            ON CONFLICT (file_path) DO UPDATE SET last_seen_at = NOW()
            RETURNING (xmax = 0) as inserted",
        id,
        media_file.path.0.to_str().unwrap_or_default(),
        media_file.filename.0,
        media_file.size_bytes.0 as i64,
        codec,
        height,
        width,
        framerate,
        bitrate_kbps,
    )
    .fetch_one(executor)
    .await?;

    if row.inserted.unwrap_or(false) {
        Ok(UpsertResult::Inserted(id))
    } else {
        Ok(UpsertResult::AlreadyExists(id))
    }
}

#[instrument(skip(executor), err)]
pub(super) async fn link_to_library_item_inner<'e, E>(
    executor: E,
    media_file_id: &MediaFileId,
    library_item_id: &LibraryItemId,
) -> Result<Uuid, RepositoryError>
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
        library_item_id.as_uuid(),
        media_file_id.as_uuid()
    )
    .fetch_one(executor)
    .await
    .map_err(RepositoryError::Database)
    .map(|r| r.id)
}

#[instrument(skip(executor), err)]
pub(super) async fn update_file_status_inner<'e, E>(
    executor: E,
    status: MediaFileStatus,
    media_file_id: &MediaFileId,
) -> Result<(), RepositoryError>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query!(
        "UPDATE media_files SET status = $1 WHERE id = $2",
        status.as_str(),
        media_file_id.as_uuid()
    )
    .execute(executor)
    .await
    .map(|_| ())
    .map_err(RepositoryError::Database)
}
