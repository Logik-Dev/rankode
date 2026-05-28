use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet};
use tokio::sync::mpsc::Sender;

use crate::{
    domain::{ApprovalListener, ApprovalSignal},
    infra::mqtt::mapping::ApprovalPayload,
};

pub struct MqttListener {
    client: AsyncClient,
    eventloop: rumqttc::EventLoop,
}

impl MqttListener {
    pub fn new(host: &str, port: u16) -> Self {
        let mut options = MqttOptions::new("rankode-listener", host, port);
        options.set_keep_alive(Duration::from_secs(5));

        let (client, eventloop) = AsyncClient::new(options, 10);

        Self { client, eventloop }
    }
}

#[async_trait]
impl ApprovalListener for MqttListener {
    async fn listen(mut self, tx: Sender<ApprovalSignal>) -> Result<()> {
        self.client
            .subscribe("rankode/approval", rumqttc::QoS::ExactlyOnce)
            .await?;

        loop {
            match self.eventloop.poll().await {
                Ok(Event::Incoming(Packet::Publish(p))) => {
                    match serde_json::from_slice::<ApprovalPayload>(&p.payload)
                        .ok()
                        .and_then(|payload| ApprovalSignal::try_from(payload).ok())
                    {
                        Some(signal) => {
                            if tx.send(signal).await.is_err() {
                                return Ok(());
                            }
                        }
                        None => tracing::warn!("received invalid approval payload, skipping"),
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("mqtt listener event loop error: {e}");
                    return Err(e.into());
                }
            }
        }
    }
}
