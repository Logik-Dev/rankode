use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{Executor, Postgres};
use tracing::{debug, instrument};

use crate::{
    domain::{Event, EventRepository},
    infra::PostgressRepository,
};

#[async_trait]
impl EventRepository for PostgressRepository {
    #[instrument(skip(self), err)]
    async fn save_event(&self, event: Event) -> Result<()> {
        insert_event_inner(&self.pool, event).await?;
        Ok(())
    }
}

#[instrument(skip(executor), err)]
pub(super) async fn insert_event_inner<'e, E>(executor: E, event: Event) -> Result<i64>
where
    E: Executor<'e, Database = Postgres>,
{
    debug!("Insert new event");

    sqlx::query!(
         "INSERT INTO events (event_type, media_file_id, library_item_id, compression_potential, bits_per_pixel, crf, skip_reason, dst_media_file_id, encode_duration_secs, gain_bytes, error_message, dry_run)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
          RETURNING id",
        event.event_type.as_str(),
        event.media_file_id,
        event.library_item_id,
        event.compression_potential,
        event.bits_per_pixel,
        event.crf,
        event.skip_reason.map(|s| s.as_str()),
        event.dst_media_file_id,
        event.encode_duration_secs,
        event.gain_bytes,
        event.error_message.as_deref(),
        event.dry_run
    )
        .fetch_one(executor)
        .await
        .map(|r| r.id)
        .context("Failed to insert event")
}
