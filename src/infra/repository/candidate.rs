use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;

use crate::{
    domain::{CandidateNotification, CandidateRepository, MediaFileId},
    infra::PostgressRepository,
};

#[async_trait]
impl CandidateRepository for PostgressRepository {
    #[instrument(skip(self), err)]
    async fn find_next_candidate(&self) -> Result<Option<CandidateNotification>> {
        let row = sqlx::query!(
            r#"
            SELECT
                mf.id,
                mf.file_name,
                mf.size_bytes,
                e.gain_bytes      AS estimated_gain_bytes,
                e.compression_potential,
                e.crf,
                li.title    AS "title?",
                li.imdb_rating
            FROM media_files mf
            JOIN events e ON e.media_file_id = mf.id
                         AND e.event_type = 'transcode_scored'
            LEFT JOIN library_items li ON li.id = mf.library_item_id
            WHERE mf.status = 'candidate'
              AND NOT EXISTS (
                SELECT 1 FROM media_files WHERE status = 'notified'
              )
            ORDER BY e.gain_bytes DESC NULLS LAST
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| CandidateNotification {
            media_file_id: MediaFileId::from(r.id),
            file_name: r.file_name,
            size_bytes: r.size_bytes as u64,
            estimated_gain_bytes: r.estimated_gain_bytes.unwrap_or(0) as u64,
            compression_potential: r.compression_potential.unwrap_or(0.0),
            crf: r.crf.unwrap_or(24) as u8,
            title: r.title,
            imdb_rating: r.imdb_rating,
        }))
    }
}
