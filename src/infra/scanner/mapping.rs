use std::path::{Path, PathBuf};

use tracing::instrument;

use crate::{
    domain::{AbsoluteFilePath, FileName, FileSizeBytes, ScannedFile},
    infra::scanner::error::ScannerError,
};

impl TryFrom<&Path> for AbsoluteFilePath {
    type Error = ScannerError;

    #[instrument(err, name = "absolute_file_path_from_path")]
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        std::fs::canonicalize(value)
            .map(|path| Self(path))
            .map_err(|_| ScannerError::AbsolutePath)
    }
}

impl TryFrom<&Path> for FileName {
    type Error = ScannerError;

    #[instrument(err, name = "filename_try_from_path")]
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        value
            .file_name()
            .and_then(|os| os.to_str())
            .map(|s| s.to_string())
            .map(|name| Self(name))
            .ok_or(ScannerError::FilenameNotFound)
    }
}

impl TryFrom<&Path> for FileSizeBytes {
    type Error = ScannerError;

    #[instrument(err, name = "file_size_try_from_path")]
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        value
            .metadata()
            .map(|metadata| Self(metadata.len()))
            .map_err(|_| ScannerError::FileSizeNotFound)
    }
}
impl TryFrom<PathBuf> for ScannedFile {
    type Error = ScannerError;

    #[instrument(err, name = "scanned_file_from_path")]
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let path = value.as_path();

        Ok(Self {
            size_bytes: path.try_into()?,
            path: path.try_into()?,
            filename: path.try_into()?,
        })
    }
}
