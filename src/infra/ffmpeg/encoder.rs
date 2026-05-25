use tokio::process::Command;
use tracing::{info, warn};

use crate::cli::EncoderArg;

const PREFERRED: &[&str] = &["hevc_videotoolbox", "hevc_nvenc", "libx265"];

pub async fn resolve_encoder(arg: EncoderArg) -> &'static str {
    match arg {
        EncoderArg::Videotoolbox => "hevc_videotoolbox",
        EncoderArg::Nvenc => "hevc_nvenc",
        EncoderArg::Libx265 => "libx265",
        EncoderArg::Auto => auto_detect().await,
    }
}

async fn auto_detect() -> &'static str {
    let output = Command::new("ffmpeg")
        .args(["-encoders", "-v", "quiet"])
        .output()
        .await;

    let Ok(output) = output else {
        warn!("Failed to run ffmpeg -encoders, falling back to libx265");
        return "libx265";
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    for &encoder in PREFERRED {
        if stdout.contains(encoder) {
            info!(encoder, "Auto-detected HEVC encoder");
            return encoder;
        }
    }

    warn!("No preferred HEVC encoder found, falling back to libx265");
    "libx265"
}
