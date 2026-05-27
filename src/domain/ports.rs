use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::domain::{
    AbsoluteFilePath, DomainEvent, LibraryItem, LibraryItemId, MediaFile, MediaFileId,
    MediaFileStatus, PendingTranscodeItem, QueuedTranscode, SavingFileResult, ScannedFile,
    TranscodeOutput, UnprocessedFile, VideoProperties, WorkerSignal,
};

#[async_trait]
pub trait FetchedLibraryItemOrchestrator: Send + Sync {
    async fn attach_metadata(
        &self,
        media_file_id: &MediaFileId,
        library_item: LibraryItem,
    ) -> Result<()>;
    async fn save_fetch_failed(&self, media_file_id: MediaFileId, error: String) -> Result<()>;
}

#[async_trait]
pub trait LibraryItemProvider: Send + Sync {
    async fn search_by_filename(&self, filename: &str) -> Result<LibraryItem>;
}

#[async_trait]
pub trait EventListener: Send + Sync {
    async fn listen(&self, tx: Sender<WorkerSignal>) -> Result<()>;
}

#[async_trait]
pub trait MediaFileAnalyzer: Send + Sync {
    async fn probe(&self, file_path: &AbsoluteFilePath) -> Result<VideoProperties>;
}

#[async_trait]
pub trait FileScanner: Send + Sync {
    async fn start_scan(&self, to_scan: PathBuf) -> Receiver<ScannedFile>;
}

#[async_trait]
pub trait MediaFileRepository: Send + Sync {
    async fn find_media_file_by_id(&self, id: &MediaFileId) -> Result<MediaFile>;
    async fn find_files_by_library_item(
        &self,
        library_item_id: &LibraryItemId,
    ) -> Result<Vec<MediaFile>>;
}

#[async_trait]
pub trait LibraryItemRepository: Send + Sync {
    async fn find_library_item_by_id(&self, id: &LibraryItemId) -> Result<LibraryItem>;
}

#[async_trait]
pub trait EventRepository: Send + Sync {
    async fn save_event(&self, event: DomainEvent) -> Result<()>;
}

#[async_trait]
pub trait FileDiscoveryOrchestrator: Send + Sync {
    async fn save_discovered_file_and_event(&self, file: MediaFile) -> Result<SavingFileResult>;

    // TODO: implement mark_disappeared_files to mark files with last_seen_at < threshold as 'disappeared'
    // async fn mark_disappeared_files(&self, root_dir: &str, seen_before: Instant) -> Result<Vec<i64>>;
}

#[async_trait]
pub trait TranscodeDecisionOrchestrator: Send + Sync {
    async fn save_decision(
        &self,
        file_id: &MediaFileId,
        file_status: Option<MediaFileStatus>,
        event: DomainEvent,
    ) -> Result<()>;
}

#[async_trait]
pub trait TranscodeLifecycleOrchestrator: Send + Sync {
    async fn start(&self, media_file_id: &MediaFileId) -> Result<()>;
    async fn complete(
        &self,
        src_id: &MediaFileId,
        dst: MediaFile,
        encode_duration_secs: i32,
        gain_bytes: i64,
    ) -> Result<()>;
    async fn fail(&self, media_file_id: &MediaFileId, error: String) -> Result<()>;
}

#[async_trait]
pub trait Transcoder: Send + Sync {
    async fn transcode(&self, file_path: &AbsoluteFilePath, crf: u8) -> Result<TranscodeOutput>;
}

#[async_trait]
pub trait PendingReportRepository: Send + Sync {
    async fn list_pending(&self) -> Result<Vec<PendingTranscodeItem>>;
}

#[async_trait]
pub trait CatchUpRepository: Send + Sync {
    /// Active files with no terminal event — need metadata fetch or transcode decision.
    async fn find_unprocessed_active_files(&self) -> Result<Vec<UnprocessedFile>>;
    /// Pending or transcoding files — need transcode re-queue.
    async fn find_queued_for_transcode(&self) -> Result<Vec<QueuedTranscode>>;
}
