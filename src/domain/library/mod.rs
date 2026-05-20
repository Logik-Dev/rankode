mod value_objects;

pub use value_objects::*;

pub struct LibraryItem {
    pub id: LibraryItemId,
    pub title: String,
    pub year: Option<i32>,
    pub imdb_id: String,
    pub genres: Vec<String>,
    pub overview: Option<String>,
    pub imdb_rating: Option<f32>,
}
