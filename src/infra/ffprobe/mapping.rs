use std::str::FromStr;

use tracing::instrument;

use crate::{
    domain::{Bitrate, Framerate, VideoProperties, Resolution, VideoCodec},
    infra::ffprobe::{
        error::FfprobeError,
        output::{FfprobeOutput, StreamType},
    },
};

impl TryFrom<FfprobeOutput> for VideoProperties {
    type Error = FfprobeError;
    #[instrument(skip(output), err, name = "video_properties_from_ffprobe")]
    fn try_from(output: FfprobeOutput) -> Result<Self, Self::Error> {
        let video_stream = output
            .streams
            .iter()
            .find(|stream| matches!(stream.codec_type, StreamType::Video))
            .ok_or(FfprobeError::NoVideoStream)?;

        let resolution = Resolution::new(
            video_stream.height.ok_or(FfprobeError::MissingResolution)?,
            video_stream.width.ok_or(FfprobeError::MissingResolution)?,
        )?;

        let bitrate = video_stream
            .bit_rate
            .as_deref()
            .or(output.format.bit_rate.as_deref())
            .and_then(|s| s.parse::<u64>().ok())
            .and_then(|bitrate| Bitrate::new(bitrate).ok());

        let framerate = video_stream
            .avg_frame_rate
            .as_deref()
            .and_then(|s| s.parse::<Framerate>().ok());

        let video_codec = VideoCodec::from_str(&video_stream.codec_name).unwrap();

        Ok(Self {
            video_codec,
            resolution,
            bitrate,
            framerate,
        })
    }
}
