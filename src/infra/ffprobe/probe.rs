use anyhow::Result;
use async_trait::async_trait;
use tokio::process::Command;
use tracing::instrument;

use crate::{
    domain::{AbsoluteFilePath, MediaFileAnalyzer, VideoProperties},
    infra::ffprobe::{error::FfprobeError, output::FfprobeOutput},
};

pub struct Ffprobe;

#[async_trait]
impl MediaFileAnalyzer for Ffprobe {
    #[instrument(skip(self, file_path), err, fields(path = ?file_path.as_ref()))]
    async fn probe(&self, file_path: &AbsoluteFilePath) -> Result<VideoProperties> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
            ])
            .arg(file_path.as_ref())
            .output()
            .await
            .map_err(FfprobeError::SpawnFailed)?;

        if !output.status.success() {
            return Err(FfprobeError::ProcessFailed(output.status.code()).into());
        }

        let ffprobe_output: FfprobeOutput =
            serde_json::from_slice(&output.stdout).map_err(FfprobeError::InvalidOutput)?;

        ffprobe_output.try_into().map_err(Into::into)
    }
}
