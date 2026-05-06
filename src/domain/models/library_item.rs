#[derive(Debug, Clone)]
pub struct LibraryItem {
    pub id: Option<i64>,
    pub title: String,
    pub year: Option<i32>,
    pub imdb_id: Option<String>,
    pub genres: Vec<String>,
    pub overview: Option<String>,
    pub imdb_rating: Option<f32>,
}
