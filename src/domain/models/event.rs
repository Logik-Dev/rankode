#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum EventType {
    FileDiscovered,
    FileUpdated,
    FileDisappeared,
    MetadataFetched,
    MetadataFetchFailed,
    TranscodeAnalyzed,
    TranscodeSkipped,
    TranscodeStarted,
    TranscodeCompleted,
    TranscodeFailed,
}
impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::FileDiscovered => "file_discovered",
            EventType::FileUpdated => "file_updated",
            EventType::FileDisappeared => "file_disappeared",
            EventType::MetadataFetched => "metadata_fetched",
            EventType::MetadataFetchFailed => "metadata_fetch_failed",
            EventType::TranscodeAnalyzed => "transcode_analyzed",
            EventType::TranscodeSkipped => "transcode_skipped",
            EventType::TranscodeStarted => "transcode_started",
            EventType::TranscodeCompleted => "transcode_completed",
            EventType::TranscodeFailed => "transcode_failed",
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventNotification {
    pub event_type: EventType,
    pub id: i64,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SkipReason {
    CodecNotH264,
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
            Self::CodecNotH264 => "codec_not_h264",
            Self::FileTooSmall => "file_too_small",
            Self::AlreadyCompressed => "already_compressed",
            Self::AlreadyTranscoded => "already_transcoded",
            Self::FileDisappeared => "file_disappeared",
            Self::TranscodeInProgress => "transcode_in_progress",
            Self::InsufficientCompressionPotential => "insufficient_compression_potential",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub media_file_id: Option<i64>,
    pub library_item_id: Option<i64>,
    pub compression_potential: Option<f32>,
    pub bits_per_pixel: Option<f32>,
    pub crf: Option<i32>,
    pub skip_reason: Option<SkipReason>,
    pub dst_media_file_id: Option<i64>,
    pub encode_duration_secs: Option<i32>,
    pub gain_bytes: Option<i64>,
    pub error_message: Option<String>,
    pub dry_run: bool,
}

impl Event {
    pub fn file_discovered(media_file_id: i64) -> Self {
        let mut event = Self::from_type(EventType::FileDiscovered);
        event.media_file_id = Some(media_file_id);
        event
    }

    pub fn transcode_analyzed(
        media_file_id: i64,
        bits_per_pixel: f32,
        crf: Option<u8>,
        compression_potential: f32,
        dry_run: bool,
    ) -> Self {
        let mut event = Self::from_type(EventType::TranscodeAnalyzed);
        event.media_file_id = Some(media_file_id);
        event.bits_per_pixel = Some(bits_per_pixel);
        event.crf = crf.map(|c| c as i32);
        event.compression_potential = Some(compression_potential);
        event.dry_run = dry_run;
        event
    }

    pub fn transcode_skipped(
        media_file_id: i64,
        bits_per_pixel: f32,
        crf: Option<u8>,
        compression_potential: f32,
        skip_reason: SkipReason,
    ) -> Self {
        let mut event = Self::from_type(EventType::TranscodeSkipped);
        event.media_file_id = Some(media_file_id);
        event.bits_per_pixel = Some(bits_per_pixel);
        event.crf = crf.map(|c| c as i32);
        event.compression_potential = Some(compression_potential);
        event.skip_reason = Some(skip_reason);
        event
    }

    #[allow(unused)]
    pub fn transcode_started(media_file_id: i64, bits_per_pixel: f32, crf: Option<u8>) -> Self {
        let mut event = Self::from_type(EventType::TranscodeStarted);
        event.media_file_id = Some(media_file_id);
        event.bits_per_pixel = Some(bits_per_pixel);
        event.crf = crf.map(|c| c as i32);
        event
    }

    #[allow(unused)]
    pub fn transcode_completed(
        media_file_id: i64,
        crf: Option<u8>,
        duration_secs: i32,
        gain_bytes: i64,
    ) -> Self {
        let mut event = Self::from_type(EventType::TranscodeCompleted);
        event.media_file_id = Some(media_file_id);
        event.crf = crf.map(|c| c as i32);
        event.encode_duration_secs = Some(duration_secs);
        event.gain_bytes = Some(gain_bytes);
        event
    }

    #[allow(unused)]
    pub fn transcode_failed(media_file_id: i64, duration_secs: i32, error: String) -> Self {
        let mut event = Self::from_type(EventType::TranscodeFailed);
        event.media_file_id = Some(media_file_id);
        event.encode_duration_secs = Some(duration_secs);
        event.error_message = Some(error);
        event
    }

    pub fn metadata_fetched(library_item_id: i64) -> Self {
        let mut event = Self::from_type(EventType::MetadataFetched);
        event.library_item_id = Some(library_item_id);
        event
    }

    pub fn metadata_fetch_failed(media_file_id: i64, error: String) -> Self {
        let mut event = Self::from_type(EventType::MetadataFetchFailed);
        event.media_file_id = Some(media_file_id);
        event.error_message = Some(error);
        event
    }

    fn from_type(event_type: EventType) -> Self {
        Self {
            event_type,
            media_file_id: None,
            library_item_id: None,
            compression_potential: None,
            bits_per_pixel: None,
            crf: None,
            skip_reason: None,
            dst_media_file_id: None,
            encode_duration_secs: None,
            gain_bytes: None,
            error_message: None,
            dry_run: false,
        }
    }
}
