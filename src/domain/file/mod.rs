mod codec;
mod resolution;
mod status;
mod transcode;
mod value_objects;
mod video_properties;

pub use codec::*;
pub use resolution::*;
pub use status::*;
pub use transcode::*;
pub use value_objects::*;
pub use video_properties::*;

#[derive(Debug)]
pub enum SavingFileResult {
    Added,
    Skipped,
}

pub struct ScannedFile {
    pub filename: FileName,
    pub path: AbsoluteFilePath,
    // root_dir: à décider — le scanner reçoit root_dir dans start_scan() mais les TryFrom<PathBuf>
    // n'y ont pas accès. Voir comment le propager proprement.
    pub size_bytes: FileSizeBytes,
}

pub struct MediaFile {
    pub id: MediaFileId,
    pub filename: FileName,
    pub path: AbsoluteFilePath,
    // root_dir: idem, à réintégrer une fois la stratégie de propagation décidée
    pub size_bytes: FileSizeBytes,
    pub status: MediaFileStatus,
    pub video_properties: VideoProperties,
}

impl MediaFile {
    pub fn from_scan(scanned_file: ScannedFile, video_properties: VideoProperties) -> Self {
        Self {
            id: MediaFileId::default(),
            filename: scanned_file.filename,
            path: scanned_file.path,
            size_bytes: scanned_file.size_bytes,
            status: MediaFileStatus::Active,
            video_properties,
        }
    }
}
