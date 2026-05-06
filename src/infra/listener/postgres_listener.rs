use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgPool, postgres::PgListener};
use tokio::sync::mpsc::{Receiver, channel};
use tracing::{error, instrument};

use crate::{
    domain::{EventListener, EventNotification},
    infra::listener::models::NotificationPayload,
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
    async fn listen(&self) -> Result<Receiver<EventNotification>> {
        let (tx, rx) = channel(100);
        let mut listener = PgListener::connect_with(&self.pool).await?;
        listener.listen("events").await?;

        tokio::spawn(async move {
            while let Ok(notification) = listener.recv().await {
                let payload = serde_json::from_str::<NotificationPayload>(notification.payload());

                if let Ok(payload) = payload {
                    if let Err(error) = tx.send(payload.into()).await {
                        error!(%error, "Error sending event on channel");
                    }
                } else {
                    error!("Failed to parse notification payload");
                }
            }
        });

        Ok(rx)
    }
}
