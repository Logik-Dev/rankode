use serde::Deserialize;

use crate::domain::LibraryItem;

#[derive(Deserialize)]
pub(super) struct RadarrMovie {
    #[serde(rename = "imdbId")]
    imdb_id: Option<String>,
    title: String,
    year: Option<i32>,
    overview: Option<String>,
    #[serde(default)]
    genres: Vec<String>,
    ratings: Option<RadarrRatings>,
}

#[derive(Deserialize)]
struct RadarrRatings {
    imdb: Option<RadarrRatingValue>,
}

#[derive(Deserialize)]
struct RadarrRatingValue {
    value: Option<f32>,
}

impl From<RadarrMovie> for LibraryItem {
    fn from(m: RadarrMovie) -> Self {
        LibraryItem {
            id: None,
            title: m.title,
            year: m.year,
            imdb_id: m.imdb_id,
            imdb_rating: m.ratings.and_then(|r| r.imdb).and_then(|r| r.value),
            genres: m.genres,
            overview: m.overview,
        }
    }
}
