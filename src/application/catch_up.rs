use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc::Sender;
use tracing::{info, instrument, warn};

use crate::domain::{CatchUpRepository, WorkerSignal};

pub struct CatchUpUseCase {
    repository: Arc<dyn CatchUpRepository>,
}

impl CatchUpUseCase {
    pub fn new(repository: Arc<dyn CatchUpRepository>) -> Self {
        Self { repository }
    }

    #[instrument(skip(self, tx), err, name = "catch_up")]
    pub async fn execute(&self, dry_run: bool, tx: &Sender<WorkerSignal>) -> Result<()> {
        let mut count = 0usize;

        for item in self.repository.find_queued_for_transcode().await? {
            if item.is_crashed {
                // Crash recovery mutates state (re-queues a real transcode).
                // In dry_run we only preview — no side effects.
                if dry_run {
                    continue;
                }
                warn!(media_file_id = ?item.media_file_id, "crash recovery: re-queuing interrupted transcode");
            }
            // Preserve the original decision's dry_run flag: if the stored decision
            // was already a dry_run, keep it even if the current run is not.
            let effective_dry_run = item.dry_run || dry_run;
            tx.send(WorkerSignal::TranscodeApproved(item.media_file_id, item.crf, effective_dry_run)).await?;
            count += 1;
        }

        for item in self.repository.find_unprocessed_active_files().await? {
            let signal = match item.library_item_id {
                None => WorkerSignal::FileDiscovered(item.media_file_id),
                Some(lib_id) => WorkerSignal::MetadataFetched(lib_id),
            };
            tx.send(signal).await?;
            count += 1;
        }

        if count > 0 {
            info!(count, "catch-up: re-queued interrupted work");
        }

        Ok(())
    }
}
