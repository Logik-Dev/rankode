use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{Executor, Postgres, query, query_as};
use tracing::{debug, instrument};

use crate::{
    domain::{LibraryItem, LibraryItemRepository},
    infra::{PostgressRepository, repository::models::LibraryItemRow},
};

#[async_trait]
impl LibraryItemRepository for PostgressRepository {
    #[instrument(skip(self), err)]
    async fn find_library_item_by_id(&self, id: i64) -> Result<LibraryItem> {
        debug!("Select library item by id");

        query_as!(
            LibraryItemRow,
            "SELECT * FROM library_items WHERE id = $1",
            id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to select library item found")
        .map(Into::into)
    }
}

#[instrument(skip(executor), err)]
pub(super) async fn insert_library_item_inner<'e, E>(executor: E, item: LibraryItem) -> Result<i64>
where
    E: Executor<'e, Database = Postgres>,
{
    debug!("Insert new library item");

    query!(
        r#"
            INSERT INTO library_items (title, year, imdb_id, genres, overview, imdb_rating)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT(imdb_id) DO UPDATE SET imdb_id = EXCLUDED.imdb_id
            RETURNING id
        "#,
        item.title,
        item.year,
        item.imdb_id,
        &item.genres,
        item.overview,
        item.imdb_rating
    )
    .fetch_one(executor)
    .await
    .context("Failed to insert library item")
    .map(|r| r.id)
}
