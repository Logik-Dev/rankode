use std::sync::Arc;

use anyhow::Result;
use tracing::instrument;

use crate::domain::{
    Event, LibraryItemRepository, MediaFileRepository, MediaFileStatus,
    TakeTranscodeDecisionUseCase, TranscodeDecisionOrchestrator,
    services::take_decision::DecisionOutcome,
};

pub struct ProcessFetchedLibraryItemUseCase {
    library_repository: Arc<dyn LibraryItemRepository>,
    media_file_repository: Arc<dyn MediaFileRepository>,
    take_decision: Arc<TakeTranscodeDecisionUseCase>,
    decision_orchestrator: Arc<dyn TranscodeDecisionOrchestrator>,
}

impl ProcessFetchedLibraryItemUseCase {
    pub fn new(
        take_decision: Arc<TakeTranscodeDecisionUseCase>,
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
    pub async fn execute(&self, library_item_id: i64, dry_run: bool) -> Result<()> {
        let library_item = self
            .library_repository
            .find_library_item_by_id(library_item_id)
            .await?;

        let media_files = self
            .media_file_repository
            .find_files_by_library_item(library_item_id)
            .await?;

        for file in &media_files {
            let mut events_to_save = Vec::with_capacity(2);
            let mut file_to_update: Option<(i64, MediaFileStatus)> = None;

            let decision = self.take_decision.execute(file, library_item.imdb_rating);

            // transcode_analyzed event
            events_to_save.push(Event::transcode_analyzed(
                file.id,
                file.bits_per_pixel() as f32,
                decision.criteria.crf,
                decision.criteria.compression_potential,
                dry_run,
            ));

            match decision.outcome {
                DecisionOutcome::ShouldEncode => {
                    file_to_update = Some((file.id, MediaFileStatus::Pending));
                }
                DecisionOutcome::Skipped(skip_reason) => {
                    // transcode_skipped event
                    events_to_save.push(Event::transcode_skipped(
                        file.id,
                        file.bits_per_pixel() as f32,
                        decision.criteria.crf,
                        decision.criteria.compression_potential,
                        skip_reason,
                    ));
                }
            }

            let (file_id, file_status) = file_to_update.unzip();
            self.decision_orchestrator
                .save_decision_and_events(file_id, file_status, events_to_save)
                .await?;
        }

        Ok(())
    }
}
