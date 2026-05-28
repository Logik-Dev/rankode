use crate::domain::MediaFileId;

/// Data carried from the repository to the notifier when a candidate is ready to be sent to the user.
pub struct CandidateNotification {
    pub media_file_id: MediaFileId,
    pub file_name: String,
    pub size_bytes: u64,
    pub estimated_gain_bytes: u64,
    pub compression_potential: f64,
    pub crf: u8,
    pub title: Option<String>,
    pub imdb_rating: Option<f32>,
}
