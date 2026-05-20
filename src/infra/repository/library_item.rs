use anyhow::Result;
use async_trait::async_trait;
use sqlx::{Executor, Postgres, query, query_as};
use tracing::{debug, instrument};

use crate::{
    domain::{LibraryItem, LibraryItemId, LibraryItemRepository},
    infra::{PostgressRepository, repository::{error::RepositoryError, models::LibraryItemRow}},
};

#[async_trait]
impl LibraryItemRepository for PostgressRepository {
    #[instrument(skip(self, id), err, fields(library_item_id = % id.as_uuid()))]
    async fn find_library_item_by_id(&self, id: &LibraryItemId) -> Result<LibraryItem> {
        debug!("Select library item by id");

        let row = query_as!(
            LibraryItemRow,
            "SELECT * FROM library_items WHERE id = $1",
            id.as_uuid()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::from_sqlx(e, RepositoryError::LibraryItemNotFound))?;

        Ok(row.into())
    }
}

#[instrument(skip(executor, item), err, fields(title = %item.title))]
pub(super) async fn insert_library_item_inner<'e, E>(
    executor: E,
    item: LibraryItem,
) -> Result<LibraryItemId, RepositoryError>
where
    E: Executor<'e, Database = Postgres>,
{
    debug!("Insert new library item");

    query!(
        r#"
            INSERT INTO library_items (id, title, year, imdb_id, genres, overview, imdb_rating)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT(imdb_id) DO UPDATE SET imdb_id = EXCLUDED.imdb_id
            RETURNING id
        "#,
        item.id.as_uuid(),
        item.title,
        item.year,
        item.imdb_id,
        &item.genres,
        item.overview,
        item.imdb_rating
    )
    .fetch_one(executor)
    .await
    .map_err(RepositoryError::Database)
    .map(|r| r.id.into())
}
