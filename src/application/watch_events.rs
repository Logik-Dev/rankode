use std::sync::Arc;

use anyhow::Result;
use futures::{StreamExt, stream::FuturesUnordered};
use tokio::sync::mpsc::channel;
use tracing::{error, instrument};

use crate::{
    application::{
        CatchUpUseCase, ProcessDiscoveredFileUseCase, ProcessFetchedLibraryItemUseCase,
        transcode_file::TranscodeFileUseCase,
    },
    domain::{EventListener, WorkerSignal},
};

pub struct WatchEventUseCase {
    listener: Arc<dyn EventListener>,
    catch_up: Arc<CatchUpUseCase>,
    discovered_file_use_case: Arc<ProcessDiscoveredFileUseCase>,
    process_fetched_use_case: Arc<ProcessFetchedLibraryItemUseCase>,
    transcode_file_use_case: Arc<TranscodeFileUseCase>,
}

impl WatchEventUseCase {
    pub fn new(
        listener: Arc<dyn EventListener>,
        catch_up: Arc<CatchUpUseCase>,
        discovered_file_use_case: Arc<ProcessDiscoveredFileUseCase>,
        process_fetched_use_case: Arc<ProcessFetchedLibraryItemUseCase>,
        transcode_file_use_case: Arc<TranscodeFileUseCase>,
    ) -> Self {
        Self {
            listener,
            catch_up,
            discovered_file_use_case,
            process_fetched_use_case,
            transcode_file_use_case,
        }
    }

    #[instrument(skip(self), err, name = "watch")]
    pub async fn execute(&self, dry_run: bool) -> Result<()> {
        const MAX_CONCURRENT_WORKERS: usize = 8;
        let (tx, mut rx) = channel(MAX_CONCURRENT_WORKERS * 4);

        self.listener.listen(tx.clone()).await?;

        // Spawned, not awaited: if we awaited, the channel could fill up before the
        // consumer loop below starts draining it, causing tx.send().await to deadlock.
        let catch_up = self.catch_up.clone();
        let tx_catch_up = tx.clone();
        tokio::spawn(async move {
            if let Err(e) = catch_up.execute(dry_run, &tx_catch_up).await {
                error!(%e, "catch-up failed");
            }
        });

        // The listener and catch-up each hold a clone. Dropping this one ensures the
        // channel closes naturally when both producers finish, triggering the `else` branch.
        drop(tx);

        let mut workers = FuturesUnordered::new();

        loop {
            tokio::select! {
                Some(notif) = rx.recv(), if workers.len() < MAX_CONCURRENT_WORKERS => {
                    let discovered_file_use_case = self.discovered_file_use_case.clone();
                    let process_fetched_use_case = self.process_fetched_use_case.clone();
                    let transcode_file_use_case = self.transcode_file_use_case.clone();

                    workers.push(dispatch_event(dry_run, notif, discovered_file_use_case, process_fetched_use_case, transcode_file_use_case));
                }
                Some(result) = workers.next() => {
                    if let Err(error) = result {
                        error!(%error, "dispatch event failed");
                    }
                }
                else => {
                    break;
                }
            }
        }

        Ok(())
    }
}

async fn dispatch_event(
    dry_run: bool,
    signal: WorkerSignal,
    discovered_file_use_case: Arc<ProcessDiscoveredFileUseCase>,
    process_fetched_use_case: Arc<ProcessFetchedLibraryItemUseCase>,
    transcode_file_use_case: Arc<TranscodeFileUseCase>,
) -> Result<()> {
    match signal {
        WorkerSignal::FileDiscovered(media_file_id) => {
            discovered_file_use_case.execute(media_file_id).await?
        }
        WorkerSignal::MetadataFetched(library_item_id) => {
            process_fetched_use_case.execute(library_item_id, dry_run).await?
        }
        WorkerSignal::TranscodeApproved(media_file_id, crf, dry_run) => {
            if !dry_run {
                transcode_file_use_case.execute(&media_file_id, crf).await?
            }
        }
    }

    Ok(())
}
