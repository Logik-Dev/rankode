use std::time::Instant;

use anyhow::Result;
use async_trait::async_trait;
use tokio::process::Command;

use crate::{
    cli::EncoderArg,
    domain::{AbsoluteFilePath, FileSizeBytes, FileName, TranscodeOutput, Transcoder},
    infra::ffmpeg::{encoder::resolve_encoder, error::TranscoderError},
};

pub struct FfmpegTranscoder {
    encoder: &'static str,
}

impl FfmpegTranscoder {
    pub async fn build(arg: EncoderArg) -> Self {
        let encoder = resolve_encoder(arg).await;
        Self { encoder }
    }

    /// Returns the right quality flag and value for the selected encoder.
    fn quality_args(&self, crf: u8) -> (&'static str, String) {
        let flag = match self.encoder {
            "hevc_videotoolbox" => "-q:v",
            "hevc_nvenc" => "-cq",
            _ => "-crf",
        };
        (flag, crf.to_string())
    }
}

#[async_trait]
impl Transcoder for FfmpegTranscoder {
    async fn transcode(&self, file_path: &AbsoluteFilePath, crf: u8) -> Result<TranscodeOutput> {
        let input = &file_path.0;

        let stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or(TranscoderError::InvalidPath)?;

        let parent = input.parent().ok_or(TranscoderError::InvalidPath)?;
        let output_name = format!("{stem}.hevc.mkv");
        let output = parent.join(&output_name);

        let (quality_flag, quality_value) = self.quality_args(crf);

        let started_at = Instant::now();

        let status = Command::new("ffmpeg")
            .args(["-loglevel", "quiet", "-i"])
            .arg(input)
            .args(["-c:v", self.encoder, quality_flag, &quality_value, "-c:a", "copy", "-y"])
            .arg(&output)
            .status()
            .await
            .map_err(TranscoderError::SpawnFailed)?;

        if !status.success() {
            return Err(TranscoderError::ProcessFailed(status.code()).into());
        }

        let encode_duration_secs = started_at.elapsed().as_secs() as i32;
        let size_bytes = std::fs::metadata(&output)
            .map(|m| m.len())
            .map_err(TranscoderError::SpawnFailed)?;

        Ok(TranscodeOutput {
            filename: FileName(output_name),
            path: AbsoluteFilePath(output),
            size_bytes: FileSizeBytes(size_bytes),
            encode_duration_secs,
        })
    }
}
