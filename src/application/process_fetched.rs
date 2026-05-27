use std::sync::Arc;

use anyhow::Result;
use tracing::instrument;

use crate::domain::{
    DomainEvent, LibraryItemId, LibraryItemRepository, MediaFileRepository, MediaFileStatus,
    TakeTranscodeDecisionService, TranscodeDecision, TranscodeDecisionOrchestrator,
};

pub struct ProcessFetchedLibraryItemUseCase {
    library_repository: Arc<dyn LibraryItemRepository>,
    media_file_repository: Arc<dyn MediaFileRepository>,
    take_decision: Arc<TakeTranscodeDecisionService>,
    decision_orchestrator: Arc<dyn TranscodeDecisionOrchestrator>,
}

impl ProcessFetchedLibraryItemUseCase {
    pub fn new(
        take_decision: Arc<TakeTranscodeDecisionService>,
        library_repository: Arc<dyn LibraryItemRepository>,
        media_file_repository: Arc<dyn MediaFileRepository>,
        decision_orchestrator: Arc<dyn TranscodeDecisionOrchestrator>,
    ) -> Self {
        Self {
            take_decision,
            library_repository,
            media_file_repository,
            decision_orchestrator,
        }
    }
    #[instrument(skip(self), err, name = "on_metadata_fetched")]
    pub async fn execute(&self, library_item_id: LibraryItemId, dry_run: bool) -> Result<()> {
        let library_item = self
            .library_repository
            .find_library_item_by_id(&library_item_id)
            .await?;

        let media_files = self
            .media_file_repository
            .find_files_by_library_item(&library_item_id)
            .await?;

        for file in &media_files {
            let mut new_status: Option<MediaFileStatus> = None;

            let decision = self.take_decision.execute(file, library_item.imdb_rating);

            let event = match decision {
                TranscodeDecision::Encode {
                    bpp,
                    compression_potential,
                    crf,
                } => {
                    // Invariant: status=Pending ↔ a real transcode is queued.
                    // Catch-up relies on this to identify files that need recovery.
                    if !dry_run {
                        new_status = Some(MediaFileStatus::Pending);
                    }
                    DomainEvent::TranscodeDecisionApproved {
                        media_file_id: file.id,
                        bpp,
                        compression_potential,
                        crf,
                        dry_run,
                    }
                }
                TranscodeDecision::Skip(skip_reason) => DomainEvent::TranscodeDecisionSkipped {
                    media_file_id: file.id,
                    skip_reason,
                    compression_potential: None,
                    bpp: None,
                },
                TranscodeDecision::SkipWithAnalysis {
                    reason,
                    bpp,
                    compression_potential,
                } => DomainEvent::TranscodeDecisionSkipped {
                    media_file_id: file.id,
                    skip_reason: reason,
                    bpp: Some(bpp),
                    compression_potential: Some(compression_potential),
                },
            };

            self.decision_orchestrator
                .save_decision(&file.id, new_status, event)
                .await?;
        }

        Ok(())
    }
}
