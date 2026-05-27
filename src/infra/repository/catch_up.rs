use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;

use crate::{
    domain::{CatchUpRepository, LibraryItemId, MediaFileId, QueuedTranscode, UnprocessedFile},
    infra::PostgressRepository,
};

#[async_trait]
impl CatchUpRepository for PostgressRepository {
    #[instrument(skip(self), err)]
    async fn find_unprocessed_active_files(&self) -> Result<Vec<UnprocessedFile>> {
        // transcode_decision_approved is intentionally absent from the exclusion list:
        // a dry_run decision doesn't set status=pending, so the file stays active and
        // must be re-evaluated on the next real run.
        let rows = sqlx::query!(
            r#"
            SELECT mf.id, mf.library_item_id
            FROM media_files mf
            WHERE mf.status = 'active'
              AND NOT EXISTS (
                SELECT 1 FROM events e
                WHERE e.media_file_id = mf.id
                  AND e.event_type IN (
                    'metadata_fetch_failed',
                    'transcode_decision_skipped',
                    'transcode_completed',
                    'transcode_failed'
                  )
              )
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| UnprocessedFile {
                media_file_id: MediaFileId::from(r.id),
                library_item_id: r.library_item_id.map(LibraryItemId::from),
            })
            .collect())
    }

    #[instrument(skip(self), err)]
    async fn find_queued_for_transcode(&self) -> Result<Vec<QueuedTranscode>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                mf.id,
                mf.status,
                e.crf,
                e.dry_run
            FROM media_files mf
            JOIN LATERAL (
                SELECT crf, dry_run
                FROM events
                WHERE media_file_id = mf.id AND event_type = 'transcode_decision_approved'
                ORDER BY occurred_at DESC
                LIMIT 1
            ) e ON true
            WHERE mf.status IN ('pending', 'transcoding')
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|r| {
                // SQLx infers LATERAL columns as nullable even with an INNER JOIN.
                // A null crf indicates a corrupted decision event — skip the file.
                let crf = r.crf? as u8;
                Some(QueuedTranscode {
                    media_file_id: MediaFileId::from(r.id),
                    is_crashed: r.status == "transcoding",
                    crf,
                    dry_run: r.dry_run,
                })
            })
            .collect())
    }
}
