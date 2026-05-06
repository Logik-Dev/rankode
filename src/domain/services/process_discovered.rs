use std::sync::Arc;

use anyhow::Result;
use tracing::{debug, instrument, warn};

use crate::domain::{
    Event, EventRepository, FetchedLibraryItemOrchestrator, LibraryItemProvider,
    MediaFileRepository,
};

pub struct ProcessDiscoveredFileUseCase {
    file_repository: Arc<dyn MediaFileRepository>,
    library_provider: Arc<dyn LibraryItemProvider>,
    orchestrator: Arc<dyn FetchedLibraryItemOrchestrator>,
    event_repository: Arc<dyn EventRepository>,
}

impl ProcessDiscoveredFileUseCase {
    pub fn new(
        file_repository: Arc<dyn MediaFileRepository>,
        library_provider: Arc<dyn LibraryItemProvider>,
        orchestrator: Arc<dyn FetchedLibraryItemOrchestrator>,
        event_repository: Arc<dyn EventRepository>,
    ) -> Self {
        Self {
            file_repository,
            library_provider,
            orchestrator,
            event_repository,
        }
    }

    #[instrument(skip(self), err, name = "on_file_discovered")]
    pub async fn execute(&self, media_file_id: i64) -> Result<()> {
        let media_file = self
            .file_repository
            .find_media_file_by_id(media_file_id)
            .await?;

        let library_item = self
            .library_provider
            .search_by_filename(&media_file.file_name)
            .await;

        match library_item {
            Ok(library_item) => {
                debug!(title = %library_item.title, "Library item found");
                // library item found, save and link
                return self
                    .orchestrator
                    .attach_metadata(media_file_id, library_item)
                    .await;
            }

            Err(e) => {
                // no library item found, save metadata_fetch_failed event
                warn!("Failed to find metadata");
                return self
                    .event_repository
                    .save_event(Event::metadata_fetch_failed(media_file_id, e.to_string()))
                    .await;
            }
        }
    }
}
