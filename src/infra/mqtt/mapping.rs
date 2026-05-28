use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::{ApprovalSignal, CandidateNotification},
    infra::mqtt::error::MqttError,
};

#[derive(Serialize)]
pub(super) struct CandidatePayload {
    pub media_file_id: Uuid,
    pub file_name: String,
    pub size_bytes: u64,
    pub estimated_gain_bytes: u64,
    pub compression_potential: f64,
    pub crf: u8,
    pub title: Option<String>,
    pub imdb_rating: Option<f32>,
}

impl From<CandidateNotification> for CandidatePayload {
    fn from(value: CandidateNotification) -> Self {
        Self {
            media_file_id: value.media_file_id.as_uuid(),
            file_name: value.file_name,
            size_bytes: value.size_bytes,
            estimated_gain_bytes: value.estimated_gain_bytes,
            compression_potential: value.compression_potential,
            crf: value.crf,
            title: value.title,
            imdb_rating: value.imdb_rating,
        }
    }
}

#[derive(Deserialize)]
pub(super) struct ApprovalPayload {
    pub status: ApprovalPayloadStatus,
    pub media_file_id: Uuid,
    pub crf: Option<u8>,
    pub actor: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum ApprovalPayloadStatus {
    Approved,
    Rejected,
}

impl TryFrom<ApprovalPayload> for ApprovalSignal {
    type Error = MqttError;

    fn try_from(value: ApprovalPayload) -> Result<Self, Self::Error> {
        let signal = match value.status {
            ApprovalPayloadStatus::Approved => Self::Approved {
                media_file_id: value.media_file_id.into(),
                crf: value.crf.ok_or_else(|| MqttError::InvalidApprovalSignal)?,
                approved_by: value.actor,
            },
            ApprovalPayloadStatus::Rejected => Self::Rejected {
                media_file_id: value.media_file_id.into(),
                rejected_by: value.actor,
            },
        };

        Ok(signal)
    }
}
