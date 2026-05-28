use std::sync::Arc;

use anyhow::Result;
use tracing::instrument;

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
            return Ok(());
        };

        let media_file_id = notification.media_file_id;

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
