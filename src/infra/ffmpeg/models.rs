use std::str::FromStr;

use serde::Deserialize;

use crate::domain::VideoCodec;

impl FromStr for VideoCodec {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "h264" | "avc" => Ok(Self::H264),
            "hevc" => Ok(Self::Hevc),
            "av1" | "av01" => Ok(Self::Av1),
            other => Ok(Self::Unknown(other.to_string())),
        }
    }
}

#[derive(Deserialize)]
pub(super) struct FfprobeOutput {
    pub streams: Vec<FfprobeStream>,
    pub format: FfprobeFormat,
}

#[derive(Deserialize)]
pub(super) struct FfprobeStream {
    pub codec_type: Option<String>,
    pub codec_name: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub r_frame_rate: Option<String>,
    pub bit_rate: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct FfprobeFormat {
    //pub duration: Option<String>,
    pub bit_rate: Option<String>,
    pub size: Option<String>,
    //pub filename: Option<String>,
}
