use std::{
    fs::FileType,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use async_trait::async_trait;
use tokio::{
    fs::read_dir,
    sync::{
        Semaphore,
        mpsc::{Receiver, Sender, channel},
    },
    task::JoinSet,
};
use tracing::{debug, error, instrument};

use crate::domain::FileScanner;

const CHANNEL_CAPACITY: usize = 1_024;
const MAX_CONCURRENT_DIRS: usize = 64;
const VIDEO_EXTENSIONS: [&str; 6] = ["mp4", "mkv", "avi", "mov", "mpeg", "mpg"];

pub struct TokioScanner;

#[async_trait]
impl FileScanner for TokioScanner {
    #[instrument(skip(self))]
    async fn start_scan(&self, to_scan: PathBuf) -> Receiver<PathBuf> {
        let (tx, rx) = channel::<PathBuf>(CHANNEL_CAPACITY);
        let sem = Arc::new(Semaphore::new(MAX_CONCURRENT_DIRS));
        tokio::spawn(Self::handle_dirs(tx, to_scan, sem));
        rx
    }
}

impl TokioScanner {
    #[instrument(skip(tx, sem))]
    async fn handle_dirs(tx: Sender<PathBuf>, root_dir: PathBuf, sem: Arc<Semaphore>) {
        let mut set = JoinSet::new();

        // scan root dir first
        set.spawn(Self::scan_dir(tx.clone(), root_dir, sem.clone()));

        while let Some(result) = set.join_next().await {
            match result {
                Ok(Ok(subdirs)) => {
                    for dir in subdirs {
                        // then one task per subdir
                        set.spawn(Self::scan_dir(tx.clone(), dir, sem.clone()));
                    }
                }
                Ok(Err(error)) => debug!(%error, "Failed to scan directory"),
                Err(error) => error!(%error, "Task failed"),
            }
        }
    }

    async fn scan_dir(
        tx: Sender<PathBuf>,
        dir: PathBuf,
        sem: Arc<Semaphore>,
    ) -> Result<Vec<PathBuf>> {
        let _permit = sem.acquire().await?;
        let mut subdirs = Vec::with_capacity(16);
        let mut entries = read_dir(&dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_type = entry.file_type().await?;

            if file_type.is_dir() {
                subdirs.push(path);
            } else if is_video_file(file_type, &path) {
                tx.send(path).await?;
            }
        }

        Ok(subdirs)
    }
}

fn is_video_file(file_type: FileType, path: &Path) -> bool {
    file_type.is_file()
        && path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| VIDEO_EXTENSIONS.contains(&ext))
}
