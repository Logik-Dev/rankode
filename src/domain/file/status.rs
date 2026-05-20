use std::str::FromStr;

use crate::domain::DomainError;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum MediaFileStatus {
    Active,
    Pending,
    Transcoding,
    Transcoded,
    Disappeared,
}

impl FromStr for MediaFileStatus {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(MediaFileStatus::Active),
            "pending" => Ok(MediaFileStatus::Pending),
            "transcoding" => Ok(MediaFileStatus::Transcoding),
            "transcoded" => Ok(MediaFileStatus::Transcoded),
            "disappeared" => Ok(MediaFileStatus::Disappeared),
            unknown => Err(DomainError::UnknownStatus(unknown.to_string())),
        }
    }
}

impl MediaFileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaFileStatus::Active => "active",
            MediaFileStatus::Pending => "pending",
            MediaFileStatus::Transcoding => "transcoding",
            MediaFileStatus::Disappeared => "disappeared",
            MediaFileStatus::Transcoded => "transcoded",
        }
    }
}
