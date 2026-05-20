use std::str::FromStr;

use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{
    Bitrate, DomainError, FileSizeBytes, Framerate, LibraryItem, LibraryItemId, MediaFile,
    MediaFileId, Resolution, VideoCodec, VideoProperties,
};

pub(super) enum UpsertResult<T> {
    Inserted(T),
    AlreadyExists(T),
}

#[allow(unused)]
#[derive(sqlx::FromRow)]
pub(super) struct MediaFileRow {
    pub id: Uuid,
    pub library_item_id: Option<Uuid>,
    pub root_dir: Option<String>,
    pub file_path: String,
    pub file_name: String,
    pub size_bytes: i64,
    pub video_codec: String,
    pub height: i32,
    pub width: i32,
    pub bitrate_kbps: i32,
    pub framerate: f64,
    pub status: String,
    pub last_seen_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl TryFrom<MediaFileRow> for MediaFile {
    type Error = DomainError;

    fn try_from(row: MediaFileRow) -> Result<Self, Self::Error> {
        let resolution = Resolution::new(row.height as u32, row.width as u32)?;
        let bitrate = Bitrate::new(row.bitrate_kbps as u64 * 1000).ok();
        let framerate = Framerate::new(row.framerate as u32, 1).ok();
        let video_codec = VideoCodec::from_str(&row.video_codec).unwrap();

        Ok(MediaFile {
            id: MediaFileId::from(row.id),
            filename: crate::domain::FileName(row.file_name),
            path: crate::domain::AbsoluteFilePath(row.file_path.into()),
            size_bytes: FileSizeBytes(row.size_bytes as u64),
            status: row.status.parse()?,
            video_properties: VideoProperties {
                video_codec,
                resolution,
                bitrate,
                framerate,
            },
        })
    }
}

#[allow(unused)]
#[derive(sqlx::FromRow)]
pub(super) struct LibraryItemRow {
    pub id: Uuid,
    pub title: String,
    pub year: Option<i32>,
    pub imdb_id: String,
    pub genres: Vec<String>,
    pub overview: Option<String>,
    pub imdb_rating: Option<f32>,
    pub created_at: OffsetDateTime,
}

impl From<LibraryItemRow> for LibraryItem {
    fn from(row: LibraryItemRow) -> Self {
        LibraryItem {
            id: LibraryItemId::from(row.id),
            title: row.title,
            year: row.year,
            imdb_id: row.imdb_id,
            genres: row.genres,
            overview: row.overview,
            imdb_rating: row.imdb_rating,
        }
    }
}
