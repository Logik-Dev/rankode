use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;

use crate::domain::{Event, EventNotification, LibraryItem, MediaFile, MediaFileStatus, NewMediaFile, SavingFileResult};

#[async_trait]
pub trait FetchedLibraryItemOrchestrator: Send + Sync {
    async fn attach_metadata(&self, media_file_id: i64, library_item: LibraryItem) -> Result<()>;
}

#[async_trait]
pub trait LibraryItemProvider: Send + Sync {
    async fn search_by_filename(&self, filename: &str) -> Result<LibraryItem>;
}

#[async_trait]
pub trait EventListener: Send + Sync {
    async fn listen(&self) -> Result<Receiver<EventNotification>>;
}

#[async_trait]
pub trait MediaFileAnalyzer: Send + Sync {
    async fn probe(&self, file_path: PathBuf, root_dir: &str) -> Result<NewMediaFile>;
}

#[async_trait]
pub trait FileScanner: Send + Sync {
    async fn start_scan(&self, to_scan: PathBuf) -> Receiver<PathBuf>;
}

#[async_trait]
pub trait MediaFileRepository: Send + Sync {
    async fn find_media_file_by_id(&self, id: i64) -> Result<MediaFile>;
    async fn find_files_by_library_item(&self, library_item_id: i64) -> Result<Vec<MediaFile>>;
}

#[async_trait]
pub trait LibraryItemRepository: Send + Sync {
    async fn find_library_item_by_id(&self, id: i64) -> Result<LibraryItem>;
}

#[async_trait]
pub trait EventRepository: Send + Sync {
    async fn save_event(&self, event: Event) -> Result<()>;
}

#[async_trait]
pub trait FileDiscoveryOrchestrator: Send + Sync {
    async fn save_discovered_file_and_event(&self, file: NewMediaFile) -> Result<SavingFileResult>;

    // TODO: implement mark_disappeared_files to mark files with last_seen_at < threshold as 'disappeared'
    // async fn mark_disappeared_files(&self, root_dir: &str, seen_before: Instant) -> Result<Vec<i64>>;
}

#[async_trait]
pub trait TranscodeDecisionOrchestrator: Send + Sync {
    async fn save_decision_and_events(
        &self,
        file_id: Option<i64>,
        file_status: Option<MediaFileStatus>,
        events_to_save: Vec<Event>,
    ) -> Result<()>;
}
