use std::sync::Arc;

use anyhow::Result;
use tracing::instrument;

use crate::domain::{
    DomainEvent, LibraryItemId, LibraryItemRepository, MediaFileRepository, MediaFileStatus,
    TakeTranscodeDecisionService, TranscodeDecision, TranscodeDecisionOrchestrator,
};

pub struct AnalyzeFileUseCase {
    library_repository: Arc<dyn LibraryItemRepository>,
    media_file_repository: Arc<dyn MediaFileRepository>,
    take_decision: Arc<TakeTranscodeDecisionService>,
    decision_orchestrator: Arc<dyn TranscodeDecisionOrchestrator>,
}

impl AnalyzeFileUseCase {
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
    pub async fn execute(&self, library_item_id: LibraryItemId) -> Result<()> {
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
                    let estimated_gain_bytes =
                        (file.size_bytes.0 as f64 * (1.0 - 0.04 / bpp)) as u64;
                    new_status = Some(MediaFileStatus::Candidate);
                    DomainEvent::TranscodeScored {
                        media_file_id: file.id,
                        bpp,
                        compression_potential,
                        crf,
                        estimated_gain_bytes,
                    }
                }
                TranscodeDecision::Skip(skip_reason) => DomainEvent::TranscodeIneligible {
                    media_file_id: file.id,
                    skip_reason,
                    compression_potential: None,
                    bpp: None,
                },
                TranscodeDecision::SkipWithAnalysis {
                    reason,
                    bpp,
                    compression_potential,
                } => DomainEvent::TranscodeIneligible {
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
