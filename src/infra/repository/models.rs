use time::OffsetDateTime;

use crate::domain::{LibraryItem, MediaFile, VideoCodec};

pub(super) enum UpsertResult<T> {
    Inserted(T),
    AlreadyExists(T),
}

#[allow(unused)]
#[derive(sqlx::FromRow)]
pub(super) struct MediaFileRow {
    pub id: i64,
    pub library_item_id: Option<i64>,
    pub root_dir: String,
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

impl From<MediaFileRow> for MediaFile {
    fn from(row: MediaFileRow) -> Self {
        MediaFile {
            id: row.id,
            root_dir: row.root_dir,
            file_path: row.file_path,
            file_name: row.file_name,
            size_bytes: row.size_bytes as u64,
            video_codec: VideoCodec::from(row.video_codec.as_str()),
            height: row.height as u32,
            width: row.width as u32,
            framerate: row.framerate,
            bitrate_kbps: row.bitrate_kbps as u32,
            status: row.status.parse().unwrap(),
        }
    }
}

impl From<&str> for VideoCodec {
    fn from(value: &str) -> Self {
        match value {
            "h264" => VideoCodec::H264,
            "hevc" => VideoCodec::Hevc,
            "av1" => VideoCodec::Av1,
            other => VideoCodec::Unknown(other.to_string()),
        }
    }
}

#[allow(unused)]
#[derive(sqlx::FromRow)]
pub(super) struct LibraryItemRow {
    pub id: i64,
    pub title: String,
    pub year: Option<i32>,
    pub imdb_id: Option<String>,
    pub genres: Vec<String>,
    pub overview: Option<String>,
    pub imdb_rating: Option<f32>,
    pub created_at: OffsetDateTime,
}

impl From<LibraryItemRow> for LibraryItem {
    fn from(row: LibraryItemRow) -> Self {
        LibraryItem {
            id: Some(row.id),
            title: row.title,
            year: row.year,
            imdb_id: row.imdb_id,
            genres: row.genres,
            overview: row.overview,
            imdb_rating: row.imdb_rating,
        }
    }
}
