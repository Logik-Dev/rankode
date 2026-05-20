use std::sync::Arc;

use anyhow::Result;
use tracing::{debug, instrument, warn};

use crate::domain::{
    FetchedLibraryItemOrchestrator, LibraryItemProvider, MediaFileId,
    MediaFileRepository,
};

pub struct ProcessDiscoveredFileUseCase {
    file_repository: Arc<dyn MediaFileRepository>,
    library_provider: Arc<dyn LibraryItemProvider>,
    orchestrator: Arc<dyn FetchedLibraryItemOrchestrator>,
}

impl ProcessDiscoveredFileUseCase {
    pub fn new(
        file_repository: Arc<dyn MediaFileRepository>,
        library_provider: Arc<dyn LibraryItemProvider>,
        orchestrator: Arc<dyn FetchedLibraryItemOrchestrator>,
    ) -> Self {
        Self {
            file_repository,
            library_provider,
            orchestrator,
        }
    }

    #[instrument(skip(self), err, name = "on_file_discovered")]
    pub async fn execute(&self, media_file_id: MediaFileId) -> Result<()> {
        let media_file = self
            .file_repository
            .find_media_file_by_id(&media_file_id)
            .await?;

        let library_item = self
            .library_provider
            .search_by_filename(&media_file.filename.0)
            .await;

        match library_item {
            Ok(library_item) => {
                debug!(title = %library_item.title, "Library item found");
                // library item found, save and link
                return self
                    .orchestrator
                    .attach_metadata(&media_file_id, library_item)
                    .await;
            }

            Err(e) => {
                // no library item found, save metadata_fetch_failed event
                warn!("Failed to find metadata");
                return self
                    .orchestrator
                    .save_fetch_failed(media_file.id, e.to_string())
                    .await;
            }
        }
    }
}
