use tracing::{debug, instrument};

use crate::domain::{
    MediaFile, MediaFileStatus,
    models::{SkipReason, TranscodeDecision},
};

#[derive(Debug)]
pub struct TakeTranscodeDecisionService {
    min_compression_potential: f64,
    min_size_gb: f64,
    min_bpp: f64,
}

impl TakeTranscodeDecisionService {
    pub fn new(min_size_gb: f64, min_bpp: f64, min_compression_potential: f64) -> Self {
        Self {
            min_size_gb,
            min_bpp,
            min_compression_potential,
        }
    }
    #[instrument(skip(file), name = "decision", fields(size = file.size_bytes.as_gb(), bpp = file.video_properties.bits_per_pixel()))]
    pub fn execute(&self, file: &MediaFile, imdb_rating: Option<f32>) -> TranscodeDecision {
        debug!("Taking transcode decision");

        // Candidate/Notified = awaiting user approval; Approved = approval recorded but transcode
        // not yet started; Transcoding = actively encoding. All mean "don't re-queue this file".
        if matches!(
            file.status,
            MediaFileStatus::Candidate
                | MediaFileStatus::Notified
                | MediaFileStatus::Approved
                | MediaFileStatus::Transcoding
        ) {
            return TranscodeDecision::Skip(SkipReason::TranscodeInProgress);
        }

        if file.status == MediaFileStatus::Transcoded {
            return TranscodeDecision::Skip(SkipReason::AlreadyTranscoded);
        }

        if file.status == MediaFileStatus::Disappeared {
            return TranscodeDecision::Skip(SkipReason::FileDisappeared);
        }

        if !file.video_properties.video_codec.needs_transcoding() {
            return TranscodeDecision::Skip(SkipReason::ExcludedCodec);
        }

        if file.size_bytes.as_gb() < self.min_size_gb {
            return TranscodeDecision::Skip(SkipReason::FileTooSmall);
        }

        let bpp = file.video_properties.bits_per_pixel().unwrap_or(0.0);

        if bpp < self.min_bpp {
            return TranscodeDecision::Skip(SkipReason::AlreadyCompressed);
        }

        let resolution = &file.video_properties.resolution;
        let resolution_factor = resolution_factor(resolution.height(), resolution.width());
        let compression_potential = (bpp - self.min_bpp) * 10.0 * resolution_factor as f64;

        if compression_potential <= self.min_compression_potential {
            return TranscodeDecision::SkipWithAnalysis {
                reason: SkipReason::InsufficientCompressionPotential,
                bpp,
                compression_potential,
            };
        }

        let crf = crf_from_rating_and_bpp(bpp, imdb_rating);

        TranscodeDecision::Encode {
            bpp,
            compression_potential,
            crf,
        }
    }
}

pub fn crf_from_rating_and_bpp(bpp: f64, rating: Option<f32>) -> u8 {
    let base_crf = match rating {
        Some(r) if r >= 7.5 => 22,
        Some(r) if r >= 6.0 => 24,
        Some(r) if r >= 4.0 => 26,
        Some(_) => 28,
        None => 24,
    };

    let adjustment: i8 = match bpp {
        b if b >= 0.15 => -1,
        b if b >= 0.08 => 0,
        b if b >= 0.05 => 1,
        _ => 2,
    };

    (base_crf as i8 + adjustment).clamp(20, 30) as u8
}

pub fn resolution_factor(height: u32, width: u32) -> f32 {
    match height.saturating_mul(width) {
        p if p >= 3840 * 2160 => 3.0,
        p if p >= 1920 * 1080 => 1.5,
        p if p >= 1280 * 720 => 1.0,
        _ => 0.6,
    }
}
