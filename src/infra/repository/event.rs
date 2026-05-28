use anyhow::Result;
use async_trait::async_trait;
use sqlx::{Executor, Postgres};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::{
    domain::{DomainEvent, EventRepository},
    infra::{PostgressRepository, repository::error::RepositoryError},
};

#[derive(Default)]
struct EventRecord {
    event_type: &'static str,
    media_file_id: Option<Uuid>,
    library_item_id: Option<Uuid>,
    bits_per_pixel: Option<f64>,
    compression_potential: Option<f64>,
    crf: Option<i16>,
    skip_reason: Option<&'static str>,
    dst_media_file_id: Option<Uuid>,
    encode_duration_secs: Option<i32>,
    gain_bytes: Option<i64>,
    error_message: Option<String>,
    actor: Option<String>,
}

#[async_trait]
impl EventRepository for PostgressRepository {
    #[instrument(skip(self), err)]
    async fn save_event(&self, event: DomainEvent) -> Result<()> {
        insert_event_inner(&self.pool, event).await?;
        Ok(())
    }
}

#[instrument(skip(executor), err)]
pub(super) async fn insert_event_inner<'e, E>(
    executor: E,
    event: DomainEvent,
) -> Result<(), RepositoryError>
where
    E: Executor<'e, Database = Postgres>,
{
    debug!("Insert new event");

    let record = EventRecord::from(event);

    sqlx::query!(
         "INSERT INTO events (event_type, media_file_id, library_item_id, compression_potential, bits_per_pixel, crf, skip_reason, dst_media_file_id, encode_duration_secs, gain_bytes, error_message, actor)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
          RETURNING id",
          record.event_type,
          record.media_file_id,
          record.library_item_id,
          record.compression_potential,
          record.bits_per_pixel,
          record.crf,
          record.skip_reason,
          record.dst_media_file_id,
          record.encode_duration_secs,
          record.gain_bytes,
          record.error_message,
          record.actor
    )
        .fetch_one(executor)
        .await
        .map(|_| ())
        .map_err(RepositoryError::Database)
}

impl From<DomainEvent> for EventRecord {
    fn from(event: DomainEvent) -> Self {
        match event {
            DomainEvent::FileDiscovered { media_file_id } => EventRecord {
                event_type: "file_discovered",
                media_file_id: Some(media_file_id.as_uuid()),
                ..Default::default()
            },
            DomainEvent::MetadataFetched { library_item_id } => EventRecord {
                event_type: "metadata_fetched",
                library_item_id: Some(library_item_id.as_uuid()),
                ..Default::default()
            },
            DomainEvent::MetadataFetchFailed {
                media_file_id,
                error,
            } => EventRecord {
                event_type: "metadata_fetch_failed",
                media_file_id: Some(media_file_id.as_uuid()),
                error_message: Some(error),
                ..Default::default()
            },
            DomainEvent::TranscodeScored {
                media_file_id,
                bpp,
                compression_potential,
                crf,
                estimated_gain_bytes,
            } => EventRecord {
                event_type: "transcode_scored",
                media_file_id: Some(media_file_id.as_uuid()),
                bits_per_pixel: Some(bpp),
                compression_potential: Some(compression_potential),
                crf: Some(crf.into()),
                gain_bytes: Some(estimated_gain_bytes as i64),
                ..Default::default()
            },
            DomainEvent::TranscodeNotified { media_file_id } => EventRecord {
                event_type: "transcode_notified",
                media_file_id: Some(media_file_id.as_uuid()),
                ..Default::default()
            },
            DomainEvent::TranscodeIneligible {
                media_file_id,
                skip_reason,
                bpp,
                compression_potential,
            } => EventRecord {
                event_type: "transcode_ineligible",
                media_file_id: Some(media_file_id.as_uuid()),
                skip_reason: Some(skip_reason.as_str()),
                bits_per_pixel: bpp,
                compression_potential,
                ..Default::default()
            },
            DomainEvent::TranscodeApproved {
                media_file_id,
                approved_by,
                crf,
            } => EventRecord {
                event_type: "transcode_approved",
                media_file_id: Some(media_file_id.as_uuid()),
                crf: Some(crf.into()),
                actor: Some(approved_by),
                ..Default::default()
            },
            DomainEvent::TranscodeRejected {
                media_file_id,
                rejected_by,
            } => EventRecord {
                event_type: "transcode_rejected",
                media_file_id: Some(media_file_id.as_uuid()),
                actor: Some(rejected_by),
                ..Default::default()
            },
            DomainEvent::TranscodeStarted { media_file_id } => EventRecord {
                event_type: "transcode_started",
                media_file_id: Some(media_file_id.as_uuid()),
                ..Default::default()
            },
            DomainEvent::TranscodeFailed {
                media_file_id,
                error,
            } => EventRecord {
                event_type: "transcode_failed",
                media_file_id: Some(media_file_id.as_uuid()),
                error_message: Some(error),
                ..Default::default()
            },
            DomainEvent::TranscodeCompleted {
                media_file_id,
                dst_media_file_id,
                encode_duration_secs,
                gain_bytes,
            } => EventRecord {
                event_type: "transcode_completed",
                media_file_id: Some(media_file_id.as_uuid()),
                dst_media_file_id: Some(dst_media_file_id.as_uuid()),
                encode_duration_secs: Some(encode_duration_secs),
                gain_bytes: Some(gain_bytes),
                ..Default::default()
            },
        }
    }
}
