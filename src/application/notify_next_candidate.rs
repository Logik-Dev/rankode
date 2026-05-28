use std::sync::Arc;

use anyhow::Result;
use tracing::{debug, info, instrument};

use crate::domain::{
    ApprovalNotifier, CandidateRepository, DomainEvent, MediaFileStatus,
    TranscodeDecisionOrchestrator,
};

pub struct NotifyNextCandidateUseCase {
    candidate_repository: Arc<dyn CandidateRepository>,
    decision_orchestrator: Arc<dyn TranscodeDecisionOrchestrator>,
    approval_notifier: Arc<dyn ApprovalNotifier>,
}

impl NotifyNextCandidateUseCase {
    pub fn new(
        candidate_repository: Arc<dyn CandidateRepository>,
        decision_orchestrator: Arc<dyn TranscodeDecisionOrchestrator>,
        approval_notifier: Arc<dyn ApprovalNotifier>,
    ) -> Self {
        Self {
            candidate_repository,
            decision_orchestrator,
            approval_notifier,
        }
    }

    #[instrument(skip(self), err, name = "notify_next_candidate")]
    pub async fn execute(&self) -> Result<()> {
        let Some(notification) = self.candidate_repository.find_next_candidate().await? else {
            debug!("no candidate available (slot busy or queue empty)");
            return Ok(());
        };

        let media_file_id = notification.media_file_id;

        info!(
            file = %notification.file_name,
            gain_gb = format!("{:.2}", notification.estimated_gain_bytes as f64 / 1e9),
            crf = notification.crf,
            "notifying candidate"
        );

        self.decision_orchestrator
            .save_decision(
                &media_file_id,
                Some(MediaFileStatus::Notified),
                DomainEvent::TranscodeNotified { media_file_id },
            )
            .await?;

        self.approval_notifier.notify_candidate(notification).await
    }
}
