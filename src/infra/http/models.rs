use serde::Deserialize;

use crate::domain::{LibraryItem, LibraryItemId};

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

impl TryFrom<RadarrMovie> for LibraryItem {
    type Error = ();

    fn try_from(m: RadarrMovie) -> Result<Self, Self::Error> {
        let imdb_id = m.imdb_id.ok_or(())?;
        Ok(LibraryItem {
            id: LibraryItemId::default(),
            title: m.title,
            year: m.year,
            imdb_id,
            imdb_rating: m.ratings.and_then(|r| r.imdb).and_then(|r| r.value),
            genres: m.genres,
            overview: m.overview,
        })
    }
}
