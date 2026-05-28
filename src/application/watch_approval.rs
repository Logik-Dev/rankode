use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc::channel;
use tracing::{error, info, instrument};

use crate::{
    application::{DeleteSourceUseCase, ProcessApprovalUseCase},
    domain::{ApprovalListener, ApprovalSignal},
    infra::MqttListener,
};

pub struct WatchApprovalUseCase {
    listener: MqttListener,
    process_approval: Arc<ProcessApprovalUseCase>,
    delete_source: Arc<DeleteSourceUseCase>,
}

impl WatchApprovalUseCase {
    pub fn new(
        listener: MqttListener,
        process_approval: Arc<ProcessApprovalUseCase>,
        delete_source: Arc<DeleteSourceUseCase>,
    ) -> Self {
        Self {
            listener,
            process_approval,
            delete_source,
        }
    }

    #[instrument(skip(self), err, name = "watch_approval")]
    pub async fn execute(self) -> Result<()> {
        info!("listening for approvals and delete commands on MQTT");
        let (tx, mut rx) = channel(32);
        let listener = self.listener;
        let process_approval = self.process_approval;
        let delete_source = self.delete_source;

        tokio::spawn(async move {
            if let Err(e) = listener.listen(tx).await {
                error!(%e, "mqtt listener failed");
            }
        });

        while let Some(signal) = rx.recv().await {
            let result = match signal {
                ApprovalSignal::Approved { .. } | ApprovalSignal::Rejected { .. } => {
                    process_approval.execute(signal).await
                }
                ApprovalSignal::DeleteSource { media_file_id } => {
                    delete_source.execute(media_file_id).await
                }
            };

            if let Err(e) = result {
                error!(%e, "command processing failed");
            }
        }

        Ok(())
    }
}
