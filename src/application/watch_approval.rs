use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc::channel;
use tracing::{error, info, instrument};

use crate::{
    application::ProcessApprovalUseCase,
    domain::ApprovalListener,
    infra::MqttListener,
};

pub struct WatchApprovalUseCase {
    listener: MqttListener,
    process_approval: Arc<ProcessApprovalUseCase>,
}

impl WatchApprovalUseCase {
    pub fn new(listener: MqttListener, process_approval: Arc<ProcessApprovalUseCase>) -> Self {
        Self {
            listener,
            process_approval,
        }
    }

    #[instrument(skip(self), err, name = "watch_approval")]
    pub async fn execute(self) -> Result<()> {
        info!("listening for approvals on MQTT");
        let (tx, mut rx) = channel(32);
        let listener = self.listener;
        let process_approval = self.process_approval;

        tokio::spawn(async move {
            if let Err(e) = listener.listen(tx).await {
                error!(%e, "mqtt approval listener failed");
            }
        });

        while let Some(signal) = rx.recv().await {
            if let Err(e) = process_approval.execute(signal).await {
                error!(%e, "process approval failed");
            }
        }

        Ok(())
    }
}
