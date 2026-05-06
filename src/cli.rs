use crate::{
    domain::{ScanFolderUseCase, WatchEventUseCase},
    infra::PostgressRepository,
};
use anyhow::Result;
use clap::Parser;
use std::{path::PathBuf, sync::Arc};

/// Scan media files, analyze them and fetch metadatas.
/// Take decision to know if they should be transcoded.
#[derive(Debug, Parser)]
#[command(name = "rankode")]
pub enum Command {
    /// Do postgresql schema migration.
    Migrate,
    /// Scan a given folder to find new media files and ffprobe them.
    Scan {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Watch for new events and execute associated actions.
    Watch {
        /// If true do not transcode pending files.
        #[arg(long, short, default_value = "false")]
        dry_run: bool,

        /// Do a scan of the given folder.
        #[arg(long, short, default_value = None)]
        scan: Option<PathBuf>,
    },
    // TODO Process,
}

impl Command {
    pub async fn execute(
        self,
        repository: Arc<PostgressRepository>,
        scanner: Arc<ScanFolderUseCase>,
        watcher: WatchEventUseCase,
    ) -> Result<()> {
        match self {
            Command::Migrate => repository.migrate().await,
            Command::Scan { path } => scanner.execute(path).await,
            Command::Watch { dry_run, scan } => {
                if let Some(path) = scan {
                    tokio::spawn(async move {
                        let _ = scanner.execute(path).await;
                    });
                }

                tokio::select! {
                    _watch_res = watcher.execute(dry_run) => {Ok(())},
                    _ = tokio::signal::ctrl_c() => {Ok(())},
                }
            }
        }
    }
}
