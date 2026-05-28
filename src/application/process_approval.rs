use std::sync::Arc;

use anyhow::Result;
use tracing::{info, instrument};

use crate::domain::{
    ApprovalSignal, DomainEvent, MediaFileStatus, TranscodeDecisionOrchestrator,
};

pub struct ProcessApprovalUseCase {
    decision_orchestrator: Arc<dyn TranscodeDecisionOrchestrator>,
}

impl ProcessApprovalUseCase {
    pub fn new(decision_orchestrator: Arc<dyn TranscodeDecisionOrchestrator>) -> Self {
        Self {
            decision_orchestrator,
        }
    }

    #[instrument(skip(self), err, name = "on_approval")]
    pub async fn execute(&self, signal: ApprovalSignal) -> Result<()> {
        match signal {
            ApprovalSignal::Approved {
                media_file_id,
                crf,
                approved_by,
            } => {
                info!(%media_file_id, crf, actor = %approved_by, "transcode approved");
                self.decision_orchestrator
                    .save_decision(
                        &media_file_id,
                        Some(MediaFileStatus::Approved),
                        DomainEvent::TranscodeApproved {
                            media_file_id,
                            approved_by,
                            crf,
                        },
                    )
                    .await
            }
            ApprovalSignal::Rejected {
                media_file_id,
                rejected_by,
            } => {
                info!(%media_file_id, actor = %rejected_by, "transcode rejected");
                self.decision_orchestrator
                    .save_decision(
                        &media_file_id,
                        Some(MediaFileStatus::Rejected),
                        DomainEvent::TranscodeRejected {
                            media_file_id,
                            rejected_by,
                        },
                    )
                    .await
            }
            // Routed away in WatchApprovalUseCase before reaching here.
            ApprovalSignal::DeleteSource { .. } => unreachable!(),
        }
    }
}
