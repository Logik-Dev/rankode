pub enum TranscodeDecision {
    Skip(SkipReason),
    SkipWithAnalysis {
        reason: SkipReason,
        bpp: f64,
        compression_potential: f64,
    },
    Encode {
        bpp: f64,
        compression_potential: f64,
        crf: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SkipReason {
    ExcludedCodec,
    FileTooSmall,
    AlreadyCompressed,
    InsufficientCompressionPotential,
    AlreadyTranscoded,
    FileDisappeared,
    TranscodeInProgress,
}

impl SkipReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExcludedCodec => "excluded_codec",
            Self::FileTooSmall => "file_too_small",
            Self::AlreadyCompressed => "already_compressed",
            Self::AlreadyTranscoded => "already_transcoded",
            Self::FileDisappeared => "file_disappeared",
            Self::TranscodeInProgress => "transcode_in_progress",
            Self::InsufficientCompressionPotential => "insufficient_compression_potential",
        }
    }
}
