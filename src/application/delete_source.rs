use std::sync::Arc;

use anyhow::Result;
use tracing::{info, instrument};

use crate::domain::{
    DomainEvent, MediaFileId, MediaFileRepository, MediaFileStatus, TranscodeDecisionOrchestrator,
    TranscodeEntityPublisher,
};

pub struct DeleteSourceUseCase {
    media_file_repo: Arc<dyn MediaFileRepository>,
    decision_orchestrator: Arc<dyn TranscodeDecisionOrchestrator>,
    entity_publisher: Arc<dyn TranscodeEntityPublisher>,
}

impl DeleteSourceUseCase {
    pub fn new(
        media_file_repo: Arc<dyn MediaFileRepository>,
        decision_orchestrator: Arc<dyn TranscodeDecisionOrchestrator>,
        entity_publisher: Arc<dyn TranscodeEntityPublisher>,
    ) -> Self {
        Self {
            media_file_repo,
            decision_orchestrator,
            entity_publisher,
        }
    }

    #[instrument(skip(self), err, name = "delete_source", fields(file_id = %media_file_id))]
    pub async fn execute(&self, media_file_id: MediaFileId) -> Result<()> {
        let file = self.media_file_repo.find_media_file_by_id(&media_file_id).await?;

        info!(file = %file.filename.0, "deleting source file");

        // TODO: extract into a FileDeleter port (infra layer) to respect hexagonal architecture.
        // Direct use of tokio::fs here is a pragmatic shortcut — file I/O has no alternative
        // implementation today, but a port would allow mocking in tests and keep side effects
        // out of the application layer.
        tokio::fs::remove_file(&file.path).await?;

        self.decision_orchestrator
            .save_decision(
                &media_file_id,
                Some(MediaFileStatus::SourceDeleted),
                DomainEvent::SourceDeleted { media_file_id },
            )
            .await?;

        self.entity_publisher.unpublish_transcoded(&media_file_id).await
    }
}
