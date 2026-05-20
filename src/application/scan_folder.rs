use std::{path::PathBuf, sync::Arc, time::Instant};

use anyhow::Result;
use futures::{StreamExt, stream::FuturesUnordered};
use tracing::{error, info, instrument};

use crate::domain::{
    FileDiscoveryOrchestrator, FileScanner, MediaFile, MediaFileAnalyzer, SavingFileResult,
};

#[derive(Clone)]
pub struct ScanFolderUseCase {
    repository: Arc<dyn FileDiscoveryOrchestrator>,
    scanner: Arc<dyn FileScanner>,
    analyzer: Arc<dyn MediaFileAnalyzer>,
}

impl ScanFolderUseCase {
    pub fn new(
        repository: Arc<dyn FileDiscoveryOrchestrator>,
        scanner: Arc<dyn FileScanner>,
        analyzer: Arc<dyn MediaFileAnalyzer>,
    ) -> Self {
        Self {
            repository,
            scanner,
            analyzer,
        }
    }

    #[instrument(skip(self), err, name = "scanner")]
    pub async fn execute(&self, root_dir: PathBuf) -> Result<()> {
        info!("Scan started");
        let start = Instant::now();

        let mut rx = self.scanner.start_scan(root_dir.clone()).await;
        // TODO decide what to do with it let root_dir = root_dir.to_str().context("Root directory invalid")?;

        let mut added = 0u32;
        let mut skipped = 0u32;

        const MAX_CONCURRENT_ANALYSES: usize = 8;
        let mut workers = FuturesUnordered::new();

        loop {
            tokio::select! {
                // receive file and put processing in workers vec
                Some(scanned_file) = rx.recv(), if workers.len() < MAX_CONCURRENT_ANALYSES => {
                    let analyzer = self.analyzer.clone();

                    let repository = self.repository.clone();

                    workers.push(async move {
                        let video_properties = analyzer.probe(&scanned_file.path).await?;
                        let media_file = MediaFile::from_scan(scanned_file, video_properties);
                        let added = repository.save_discovered_file_and_event(media_file).await?;

                        Ok::<SavingFileResult, anyhow::Error>(added)
                    });
                }
                // next worker
                Some(result) = workers.next() => {
                    match result {
                        Ok(SavingFileResult::Added) => added += 1,
                        Ok(SavingFileResult::Skipped) => skipped += 1,
                        Err(error) => error!(%error, "Failed to process file"),
                    }
                }
                // end
                else => {
                    break;
                }
            }
        }

        info!(added, skipped, duration = ?start.elapsed() ,"Scan finished");
        Ok(())
    }
}
