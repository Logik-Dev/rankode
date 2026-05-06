use std::str::FromStr;

use anyhow::Result;

#[derive(Debug)]
pub enum SavingFileResult {
    Added,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum MediaFileStatus {
    Active,
    Pending,
    Transcoding,
    Transcoded,
    Disappeared,
}

impl FromStr for MediaFileStatus {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(MediaFileStatus::Active),
            "pending" => Ok(MediaFileStatus::Pending),
            "transcoding" => Ok(MediaFileStatus::Transcoding),
            "transcoded" => Ok(MediaFileStatus::Transcoded),
            "disappeared" => Ok(MediaFileStatus::Disappeared),
            unknown => Err(anyhow::anyhow!("Unknown status {unknown}")),
        }
    }
}

impl MediaFileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaFileStatus::Active => "active",
            MediaFileStatus::Pending => "pending",
            MediaFileStatus::Transcoding => "transcoding",
            MediaFileStatus::Disappeared => "disappeared",
            MediaFileStatus::Transcoded => "transcoded",
        }
    }
}

#[derive(Debug, Clone)]
pub enum VideoCodec {
    H264,
    Hevc,
    Av1,
    Unknown(String),
}

impl std::fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::H264 => write!(f, "h264"),
            Self::Hevc => write!(f, "hevc"),
            Self::Av1 => write!(f, "av1"),
            Self::Unknown(other) => write!(f, "{other}"),
        }
    }
}

/// Data for creating a new media file (no ID yet assigned by the database)
#[derive(Debug, Clone)]
pub struct NewMediaFile {
    pub root_dir: String,
    pub file_path: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub video_codec: VideoCodec,
    pub height: u32,
    pub width: u32,
    pub framerate: f64,
    pub bitrate_kbps: u32,
}

impl NewMediaFile {
    pub fn bits_per_pixel(&self) -> f64 {
        self.bitrate_kbps as f64 * 1000.0
            / (self.width as f64 * self.height as f64 * self.framerate)
    }

    pub fn size_gb(&self) -> f64 {
        self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

/// Stored media file with database-assigned ID
#[derive(Debug, Clone)]
pub struct MediaFile {
    pub id: i64,
    pub root_dir: String,
    pub file_path: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub video_codec: VideoCodec,
    pub height: u32,
    pub width: u32,
    pub framerate: f64,
    pub bitrate_kbps: u32,
    pub status: MediaFileStatus,
}

impl MediaFile {
    pub fn bits_per_pixel(&self) -> f64 {
        self.bitrate_kbps as f64 * 1000.0
            / (self.width as f64 * self.height as f64 * self.framerate)
    }

    pub fn size_gb(&self) -> f64 {
        self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}
