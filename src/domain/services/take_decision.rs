use tracing::{debug, instrument};

use crate::domain::{MediaFile, MediaFileStatus, SkipReason};

#[derive(Debug)]
pub struct TakeTranscodeDecisionUseCase {
    min_compression_potential: f32,
    min_size_gb: f64,
    min_bpp: f32,
}
#[derive(Debug)]
pub struct TranscodeDecision {
    pub outcome: DecisionOutcome,
    pub criteria: DecisionCriteria,
}

impl TranscodeDecision {
    fn with_outcome(outcome: DecisionOutcome, criteria: Option<DecisionCriteria>) -> Self {
        let criteria = criteria.unwrap_or(DecisionCriteria {
            compression_potential: 0.0,
            crf: None,
        });
        Self { outcome, criteria }
    }

    fn with_skipped_reason(reason: SkipReason, criteria: Option<DecisionCriteria>) -> Self {
        Self::with_outcome(DecisionOutcome::Skipped(reason), criteria)
    }
}

#[derive(Debug)]
pub enum DecisionOutcome {
    Skipped(SkipReason),
    ShouldEncode,
}

#[derive(Debug)]
pub struct DecisionCriteria {
    pub compression_potential: f32,
    pub crf: Option<u8>,
}

impl TakeTranscodeDecisionUseCase {
    pub fn new(min_size_gb: f64, min_bpp: f32, min_compression_potential: f32) -> Self {
        Self {
            min_size_gb,
            min_bpp,
            min_compression_potential,
        }
    }
    #[instrument(skip(file), name = "decision", fields(size = file.size_gb(), bpp = file.bits_per_pixel()))]
    pub fn execute(&self, file: &MediaFile, imdb_rating: Option<f32>) -> TranscodeDecision {
        debug!("Taking transcode decision");

        if file.size_gb() < self.min_size_gb {
            return TranscodeDecision::with_skipped_reason(SkipReason::FileTooSmall, None);
        }
        if file.status == MediaFileStatus::Transcoded {
            return TranscodeDecision::with_skipped_reason(SkipReason::AlreadyTranscoded, None);
        }
        if file.status == MediaFileStatus::Disappeared {
            return TranscodeDecision::with_skipped_reason(SkipReason::FileDisappeared, None);
        }

        if file.bits_per_pixel() < self.min_bpp as f64 {
            return TranscodeDecision::with_skipped_reason(SkipReason::AlreadyCompressed, None);
        }

        let resolution_factor = resolution_factor(file.height, file.width);
        let compression_potential =
            (file.bits_per_pixel() as f32 - self.min_bpp) * 10.0_f32 * resolution_factor;

        if compression_potential <= self.min_compression_potential {
            return TranscodeDecision::with_skipped_reason(
                SkipReason::InsufficientCompressionPotential,
                Some(DecisionCriteria {
                    compression_potential,
                    crf: None,
                }),
            );
        }

        let crf = crf_from_rating_and_bpp(file.bits_per_pixel(), imdb_rating);

        TranscodeDecision::with_outcome(
            DecisionOutcome::ShouldEncode,
            Some(DecisionCriteria {
                compression_potential,
                crf: Some(crf).or(Some(24)),
            }),
        )
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
