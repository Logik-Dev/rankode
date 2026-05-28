use std::sync::Arc;

use anyhow::Result;
use tracing::instrument;

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
        }
    }
}
