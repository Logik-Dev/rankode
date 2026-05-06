use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::{debug, error, instrument};

use crate::domain::{
    Event, FetchedLibraryItemOrchestrator, FileDiscoveryOrchestrator, LibraryItem,
    MediaFileStatus, NewMediaFile, SavingFileResult, TranscodeDecisionOrchestrator,
};
use crate::infra::repository::event::insert_event_inner;
use crate::infra::repository::library_item::insert_library_item_inner;
use crate::infra::repository::media_file::{link_to_library_item_inner, update_file_status_inner};
use crate::infra::repository::{media_file::insert_media_file_inner, models::UpsertResult};

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
            .context("Postgres migration failed")
    }
}

#[async_trait]
impl FileDiscoveryOrchestrator for PostgressRepository {
    #[instrument(skip(self), err)]
    async fn save_discovered_file_and_event(
        &self,
        media_file: NewMediaFile,
    ) -> Result<SavingFileResult> {
        let mut result = SavingFileResult::Skipped;
        let mut tx = self.pool.begin().await?;

        match insert_media_file_inner(&mut *tx, &media_file).await {
            Ok(UpsertResult::AlreadyExists(_)) => {
                debug!(filename = %media_file.file_name, "File already exists, skipping");
            }
            Ok(UpsertResult::Inserted(media_file_id)) => {
                debug!(filename = %media_file.file_name, "New file inserted");
                let event = Event::file_discovered(media_file_id);
                insert_event_inner(&mut *tx, event).await?;
                result = SavingFileResult::Added;
            }
            Err(_) => {
                error!(filename = %media_file.file_name, "Failed to insert file");
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
    #[instrument(skip(self), err)]
    async fn attach_metadata(&self, media_file_id: i64, library_item: LibraryItem) -> Result<()> {
        debug!("Save library item, link it and save metadata_fetched_event");

        let mut tx = self.pool.begin().await?;

        let library_item_id = insert_library_item_inner(&mut *tx, library_item).await?;
        link_to_library_item_inner(&mut *tx, media_file_id, library_item_id).await?;

        let event = Event::metadata_fetched(library_item_id);
        insert_event_inner(&mut *tx, event).await?;

        tx.commit().await.context("Failed to commit transaction")
    }
}

#[async_trait]
impl TranscodeDecisionOrchestrator for PostgressRepository {
    #[instrument(skip(self), err)]
    async fn save_decision_and_events(
        &self,
        file_id: Option<i64>,
        file_status: Option<MediaFileStatus>,
        events: Vec<Event>,
    ) -> Result<()> {
        debug!("Insert events and update file status if needed");

        let mut tx = self.pool.begin().await?;

        if let (Some(id), Some(status)) = (file_id, file_status) {
            update_file_status_inner(&mut *tx, status, id).await?;
        }

        for event in events {
            insert_event_inner(&mut *tx, event).await?;
        }

        tx.commit().await.context("Failed to commit transaction")
    }
}
