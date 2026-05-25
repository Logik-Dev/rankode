use std::sync::Arc;

use anyhow::Result;

use crate::domain::{
    MediaFile, MediaFileAnalyzer, MediaFileId, MediaFileRepository, MediaFileStatus,
    TranscodeLifecycleOrchestrator, Transcoder,
};

pub struct TranscodeFileUseCase {
    pub media_file_repo: Arc<dyn MediaFileRepository>,
    pub analyzer: Arc<dyn MediaFileAnalyzer>,
    pub orchestrator: Arc<dyn TranscodeLifecycleOrchestrator>,
    pub transcoder: Arc<dyn Transcoder>,
}

impl TranscodeFileUseCase {
    pub fn new(
        media_file_repo: Arc<dyn MediaFileRepository>,
        analyzer: Arc<dyn MediaFileAnalyzer>,
        orchestrator: Arc<dyn TranscodeLifecycleOrchestrator>,
        transcoder: Arc<dyn Transcoder>,
    ) -> Self {
        Self { media_file_repo, analyzer, orchestrator, transcoder }
    }

    pub async fn execute(&self, media_file_id: &MediaFileId, crf: u8) -> Result<()> {
        let src = self.media_file_repo.find_media_file_by_id(media_file_id).await?;

        self.orchestrator.start(media_file_id).await?;

        let output = match self.transcoder.transcode(&src.path, crf).await {
            Ok(o) => o,
            Err(e) => return self.orchestrator.fail(media_file_id, e.to_string()).await,
        };

        let video_properties = match self.analyzer.probe(&output.path).await {
            Ok(p) => p,
            Err(e) => return self.orchestrator.fail(media_file_id, e.to_string()).await,
        };

        let gain_bytes = src.size_bytes.0 as i64 - output.size_bytes.0 as i64;
        let encode_duration_secs = output.encode_duration_secs;

        let dst = MediaFile {
            id: MediaFileId::default(),
            filename: output.filename,
            path: output.path,
            size_bytes: output.size_bytes,
            status: MediaFileStatus::Transcoded,
            video_properties,
        };

        self.orchestrator.complete(media_file_id, dst, encode_duration_secs, gain_bytes).await
    }
}
