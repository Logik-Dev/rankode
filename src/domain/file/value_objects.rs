use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use uuid::Uuid;

use crate::domain::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MediaFileId(Uuid);

impl MediaFileId {
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for MediaFileId {
    fn default() -> Self {
        Self(Uuid::now_v7())
    }
}

impl From<Uuid> for MediaFileId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
pub struct FileName(pub String);
pub struct AbsoluteFilePath(pub PathBuf);
pub struct RootDirectory(pub PathBuf);
pub struct FileSizeBytes(pub u64);
pub struct Bitrate(u64);
pub struct Framerate {
    numerator: u32,
    denominator: u32,
}

impl FileSizeBytes {
    pub fn as_gb(&self) -> f64 {
        return self.0 as f64 / 1024.0 / 1024.0 / 1024.0;
    }
}

impl AsRef<Path> for AbsoluteFilePath {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl AsRef<Path> for RootDirectory {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl Bitrate {
    pub fn new(value: u64) -> Result<Self, DomainError> {
        if value == 0 {
            return Err(DomainError::InvalidBitrate);
        }
        Ok(Self(value))
    }

    pub fn as_bps(&self) -> u64 {
        self.0
    }

    pub fn as_mbs(&self) -> f64 {
        self.0 as f64 / 1_000_000.0
    }
}

impl FromStr for Framerate {
    type Err = DomainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (num, den) = s.split_once("/").ok_or(DomainError::InvalidFramerate)?;

        let numerator = num.parse().map_err(|_| DomainError::InvalidFramerate)?;
        let denominator = den.parse().map_err(|_| DomainError::InvalidFramerate)?;

        Self::new(numerator, denominator)
    }
}

impl Framerate {
    pub fn new(numerator: u32, denominator: u32) -> Result<Self, DomainError> {
        if denominator == 0 {
            return Err(DomainError::InvalidFramerate);
        }

        Ok(Self {
            numerator,
            denominator,
        })
    }

    pub fn as_f64(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
}
