use anyhow::Result;
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::{debug, error, instrument};

use crate::infra::repository::error::RepositoryError;
use crate::domain::{
    DomainEvent, FetchedLibraryItemOrchestrator, FileDiscoveryOrchestrator, LibraryItem, MediaFile,
    MediaFileId, MediaFileStatus, SavingFileResult, TranscodeDecisionOrchestrator,
    TranscodeLifecycleOrchestrator,
};
use crate::infra::repository::event::insert_event_inner;
use crate::infra::repository::library_item::insert_library_item_inner;
use crate::infra::repository::media_file::{
    insert_media_file_inner, link_to_library_item_inner, update_file_status_inner,
};
use crate::infra::repository::models::UpsertResult;

pub struct PostgressRepository {
    pub(super) pool: PgPool,
}

impl PostgressRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self), err)]
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!()
            .run(&self.pool)
            .await
            .map_err(RepositoryError::Migration)
            .map_err(Into::into)
    }
}

#[async_trait]
impl FileDiscoveryOrchestrator for PostgressRepository {
    #[instrument(skip(self, media_file), err, fields(filename = %media_file.filename.0))]
    async fn save_discovered_file_and_event(
        &self,
        media_file: MediaFile,
    ) -> Result<SavingFileResult> {
        let mut result = SavingFileResult::Skipped;
        let mut tx = self.pool.begin().await?;

        match insert_media_file_inner(&mut *tx, &media_file).await {
            Ok(UpsertResult::AlreadyExists(_)) => {
                debug!("File already exists, skipping");
            }
            Ok(UpsertResult::Inserted(_)) => {
                debug!("New file inserted");
                let event = DomainEvent::FileDiscovered {
                    media_file_id: media_file.id,
                };
                insert_event_inner(&mut *tx, event).await?;
                result = SavingFileResult::Added;
            }
            Err(_) => {
                error!("Failed to insert file");
                tx.rollback().await?;
                return Ok(result);
            }
        };

        tx.commit().await?;
        Ok(result)
    }
}

#[async_trait]
impl FetchedLibraryItemOrchestrator for PostgressRepository {
    #[instrument(skip(self, library_item),err, fields(media_file_id, library_item_id = %library_item.id.as_uuid()))]
    async fn attach_metadata(
        &self,
        media_file_id: &MediaFileId,
        library_item: LibraryItem,
    ) -> Result<()> {
        debug!("Save library item, link it and save metadata_fetched_event");

        let mut tx = self.pool.begin().await?;

        let library_item_id = insert_library_item_inner(&mut *tx, library_item).await?;
        link_to_library_item_inner(&mut *tx, media_file_id, &library_item_id).await?;

        let event = DomainEvent::MetadataFetched { library_item_id };
        insert_event_inner(&mut *tx, event).await?;

        tx.commit().await.map_err(RepositoryError::Database).map_err(Into::into)
    }

    async fn save_fetch_failed(&self, media_file_id: MediaFileId, error: String) -> Result<()> {
        let event = DomainEvent::MetadataFetchFailed {
            media_file_id,
            error,
        };

        insert_event_inner(&self.pool, event).await.map_err(Into::into)
    }
}

#[async_trait]
impl TranscodeDecisionOrchestrator for PostgressRepository {
    #[instrument(skip(self), err)]
    async fn save_decision(
        &self,
        file_id: &MediaFileId,
        file_status: Option<MediaFileStatus>,
        event: DomainEvent,
    ) -> Result<()> {
        debug!("Insert events and update file status if needed");

        let mut tx = self.pool.begin().await?;

        if let Some(status) = file_status {
            update_file_status_inner(&mut *tx, status, file_id).await?;
        }

        insert_event_inner(&mut *tx, event).await?;

        tx.commit().await.map_err(RepositoryError::Database).map_err(Into::into)
    }
}

#[async_trait]
impl TranscodeLifecycleOrchestrator for PostgressRepository {
    #[instrument(skip(self), err)]
    async fn start(&self, media_file_id: &MediaFileId) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        update_file_status_inner(&mut *tx, MediaFileStatus::Transcoding, media_file_id).await?;
        insert_event_inner(&mut *tx, DomainEvent::TranscodeStarted { media_file_id: *media_file_id }).await?;
        tx.commit().await.map_err(RepositoryError::Database).map_err(Into::into)
    }

    #[instrument(skip(self, dst), err)]
    async fn complete(
        &self,
        src_id: &MediaFileId,
        dst: MediaFile,
        encode_duration_secs: i32,
        gain_bytes: i64,
    ) -> Result<()> {
        let dst_id = dst.id;
        let mut tx = self.pool.begin().await?;
        insert_media_file_inner(&mut *tx, &dst).await?;
        update_file_status_inner(&mut *tx, MediaFileStatus::Transcoded, src_id).await?;
        insert_event_inner(&mut *tx, DomainEvent::TranscodeCompleted {
            media_file_id: *src_id,
            dst_media_file_id: dst_id,
            encode_duration_secs,
            gain_bytes,
        }).await?;
        tx.commit().await.map_err(RepositoryError::Database).map_err(Into::into)
    }

    #[instrument(skip(self), err)]
    async fn fail(&self, media_file_id: &MediaFileId, error: String) -> Result<()> {
        insert_event_inner(&self.pool, DomainEvent::TranscodeFailed {
            media_file_id: *media_file_id,
            error,
        }).await.map_err(Into::into)
    }
}
