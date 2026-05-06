use std::path::{Path, PathBuf};

use crate::{
    domain::{MediaFileAnalyzer, NewMediaFile, VideoCodec},
    infra::ffmpeg::models::FfprobeOutput,
};
use anyhow::{Context, Ok, Result};
use async_trait::async_trait;
use tokio::process::Command;

pub struct Ffprobe;

impl TryFrom<FfprobeOutput> for NewMediaFile {
    type Error = anyhow::Error;

    fn try_from(ffprobe: FfprobeOutput) -> Result<Self> {
        let video = ffprobe
            .streams
            .iter()
            .find(|s| s.codec_type.as_deref() == Some("video"))
            .context("No video stream found")?;
        Ok(NewMediaFile {
            // Filesystem paths will be added by the handler
            root_dir: String::new(),
            file_path: String::new(),
            file_name: String::new(),
            size_bytes: ffprobe
                .format
                .size
                // .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            video_codec: video
                .codec_name
                .as_deref()
                .and_then(|c| c.parse().ok())
                .unwrap_or(VideoCodec::Unknown("unknown".into())),
            height: video.height.unwrap_or(0),
            width: video.width.unwrap_or(0),
            framerate: parse_fraction(video.r_frame_rate.as_deref().unwrap_or("25/1")),
            bitrate_kbps: video
                .bit_rate
                .as_deref()
                .or(ffprobe.format.bit_rate.as_deref())
                .and_then(|b| b.parse::<u64>().ok())
                .map(|bps| (bps / 1000) as u32)
                .unwrap_or(0),
        })
    }
}

#[async_trait]
impl MediaFileAnalyzer for Ffprobe {
    async fn probe(&self, file_path: PathBuf, root_dir: &str) -> Result<NewMediaFile> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
                file_path.to_str().unwrap(),
            ])
            .output()
            .await?;

        let ffprobe_output: FfprobeOutput = serde_json::from_slice(&output.stdout)?;
        let mut media_file = NewMediaFile::try_from(ffprobe_output)?;

        media_file.file_path = file_path.to_string_lossy().to_string();
        media_file.root_dir = root_dir.to_string();
        media_file.file_name = extract_filename(&file_path);

        Ok(media_file)
    }
}

fn extract_filename(path: &Path) -> String {
    path.file_name()
        .and_then(|os| os.to_str())
        .map(str::to_owned)
        .unwrap_or_default()
}

fn parse_fraction(fraction: &str) -> f64 {
    let parts: Vec<&str> = fraction.split("/").collect();
    if parts.len() == 2 {
        let num: f64 = parts[0].parse().ok().unwrap_or(0.0);
        let den: f64 = parts[1].parse().ok().unwrap_or(0.0);
        if den > 0.0 {
            return num / den;
        }
    }
    0.0
}
