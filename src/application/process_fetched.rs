use std::sync::Arc;

use anyhow::Result;
use tracing::instrument;

use crate::domain::{
    ApprovalNotifier, CandidateNotification, DomainEvent, LibraryItemId, LibraryItemRepository,
    MediaFileRepository, MediaFileStatus, TakeTranscodeDecisionService, TranscodeDecision,
    TranscodeDecisionOrchestrator,
};

pub struct AnalyzeFileUseCase {
    library_repository: Arc<dyn LibraryItemRepository>,
    media_file_repository: Arc<dyn MediaFileRepository>,
    take_decision: Arc<TakeTranscodeDecisionService>,
    decision_orchestrator: Arc<dyn TranscodeDecisionOrchestrator>,
    approval_notifier: Arc<dyn ApprovalNotifier>,
}

impl AnalyzeFileUseCase {
    pub fn new(
        take_decision: Arc<TakeTranscodeDecisionService>,
        library_repository: Arc<dyn LibraryItemRepository>,
        media_file_repository: Arc<dyn MediaFileRepository>,
        decision_orchestrator: Arc<dyn TranscodeDecisionOrchestrator>,
        approval_notifier: Arc<dyn ApprovalNotifier>,
    ) -> Self {
        Self {
            take_decision,
            library_repository,
            media_file_repository,
            decision_orchestrator,
            approval_notifier,
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
            let mut candidate_notification: Option<CandidateNotification> = None;

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
                    candidate_notification = Some(CandidateNotification {
                        media_file_id: file.id,
                        file_name: file.filename.0.clone(),
                        size_bytes: file.size_bytes.0,
                        estimated_gain_bytes,
                        compression_potential,
                        crf,
                        title: Some(library_item.title.clone()),
                        imdb_rating: library_item.imdb_rating,
                    });
                    DomainEvent::TranscodeScored {
                        media_file_id: file.id,
                        bpp,
                        compression_potential,
                        crf,
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

            if let Some(notification) = candidate_notification {
                self.approval_notifier
                    .notify_candidate(notification)
                    .await?;
            }
        }

        Ok(())
    }
}
