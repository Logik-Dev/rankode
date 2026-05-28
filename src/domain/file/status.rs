use std::str::FromStr;

use crate::domain::DomainError;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum MediaFileStatus {
    Active,
    Candidate,
    Notified,
    Approved,
    Rejected,
    Pending,
    Transcoding,
    Transcoded,
    SourceDeleted,
    Disappeared,
}

impl FromStr for MediaFileStatus {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(MediaFileStatus::Active),
            "candidate" => Ok(MediaFileStatus::Candidate),
            "notified" => Ok(MediaFileStatus::Notified),
            "approved" => Ok(MediaFileStatus::Approved),
            "rejected" => Ok(MediaFileStatus::Rejected),
            "pending" => Ok(MediaFileStatus::Pending),
            "transcoding" => Ok(MediaFileStatus::Transcoding),
            "transcoded" => Ok(MediaFileStatus::Transcoded),
            "source_deleted" => Ok(MediaFileStatus::SourceDeleted),
            "disappeared" => Ok(MediaFileStatus::Disappeared),
            unknown => Err(DomainError::UnknownStatus(unknown.to_string())),
        }
    }
}

impl MediaFileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaFileStatus::Active => "active",
            MediaFileStatus::Candidate => "candidate",
            MediaFileStatus::Notified => "notified",
            MediaFileStatus::Approved => "approved",
            MediaFileStatus::Rejected => "rejected",
            MediaFileStatus::Pending => "pending",
            MediaFileStatus::Transcoding => "transcoding",
            MediaFileStatus::Disappeared => "disappeared",
            MediaFileStatus::Transcoded => "transcoded",
            MediaFileStatus::SourceDeleted => "source_deleted",
        }
    }
}
