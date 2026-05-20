use crate::domain::{Bitrate, Framerate, Resolution, VideoCodec};

pub struct VideoProperties {
    pub video_codec: VideoCodec,
    pub resolution: Resolution,
    pub bitrate: Option<Bitrate>,
    pub framerate: Option<Framerate>,
}

impl VideoProperties {
    pub fn bits_per_pixel(&self) -> Option<f64> {
        let bitrate = self.bitrate.as_ref()?.as_bps();
        let pixels = self.resolution.pixel_count();
        let fps = self.framerate.as_ref()?.as_f64();

        Some(bitrate as f64 / (pixels as f64 * fps))
    }
}
