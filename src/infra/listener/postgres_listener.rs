use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgPool, postgres::PgListener};
use tokio::sync::mpsc::{Receiver, channel};
use tracing::{debug, error, instrument, warn};

use crate::{
    domain::{EventListener, MediaFileId, WorkerSignal},
    infra::listener::{error::ListenerError, models::NotificationPayload},
};

pub struct PostgresEventListener {
    pool: PgPool,
}

impl PostgresEventListener {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventListener for PostgresEventListener {
    #[instrument(skip(self), err)]
    async fn listen(&self) -> Result<Receiver<WorkerSignal>> {
        let (tx, rx) = channel(100);
        let mut listener = PgListener::connect_with(&self.pool)
            .await
            .map_err(ListenerError::ConnectionFailed)?;

        listener
            .listen("events")
            .await
            .map_err(ListenerError::ConnectionFailed)?;

        tokio::spawn(async move {
            while let Ok(notification) = listener.recv().await {
                let payload =
                    match serde_json::from_str::<NotificationPayload>(notification.payload()) {
                        Ok(p) => p,
                        Err(e) => {
                            error!(%e, payload = notification.payload(), "Failed to parse notification payload");
                            continue;
                        }
                    };

                let Some(signal) = to_worker_signal(payload) else {
                    continue;
                };

                if let Err(e) = tx.send(signal).await {
                    error!(%e, "Error sending event on channel");
                }
            }
        });

        Ok(rx)
    }
}

fn to_worker_signal(payload: NotificationPayload) -> Option<WorkerSignal> {
    match payload.event_type.as_str() {
        "file_discovered" => {
            let id = payload.media_file_id.map(MediaFileId::from).or_else(|| {
                warn!("file_discovered event missing media_file_id");
                None
            })?;
            Some(WorkerSignal::FileDiscovered(id))
        }
        "metadata_fetched" => {
            let id = payload.library_item_id.map(Into::into).or_else(|| {
                warn!("metadata_fetched event missing library_item_id");
                None
            })?;
            Some(WorkerSignal::MetadataFetched(id))
        }
        "transcode_decision_approved" => {
            let id = payload.media_file_id.map(MediaFileId::from).or_else(|| {
                warn!("transcode_decision_approved event missing media_file_id");
                None
            })?;
            let crf = payload.crf.or_else(|| {
                warn!("transcode_decision_approved event missing crf");
                None
            })?;
            Some(WorkerSignal::TranscodeApproved(id, crf))
        }
        other => {
            debug!(event_type = other, "Unhandled event type, skipping");
            None
        }
    }
}
