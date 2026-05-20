use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use tracing::{debug, instrument};

use crate::{
    domain::{LibraryItem, LibraryItemProvider},
    infra::http::{error::RadarrError, models::RadarrMovie},
};

pub struct RadarrProvider {
    base_url: String,
    api_key: String,
    client: Client,
}

impl RadarrProvider {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LibraryItemProvider for RadarrProvider {
    #[instrument(skip(self), err)]
    async fn search_by_filename(&self, filename: &str) -> Result<LibraryItem> {
        debug!("Search movie on radarr");
        let movies: Vec<RadarrMovie> = self
            .client
            .get(format!("{}/api/v3/movie/lookup", self.base_url))
            .header("X-Api-Key", &self.api_key)
            .query(&[("term", filename)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        movies
            .into_iter()
            .find_map(|m| m.try_into().ok())
            .ok_or_else(|| RadarrError::NoResultWithImdbId {
                filename: filename.to_string(),
            })
            .map_err(Into::into)
    }
}
