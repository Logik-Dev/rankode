use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;

use crate::{
    domain::{PendingReportRepository, PendingTranscodeItem},
    infra::PostgressRepository,
};

#[async_trait]
impl PendingReportRepository for PostgressRepository {
    #[instrument(skip(self), err)]
    async fn list_pending(&self) -> Result<Vec<PendingTranscodeItem>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                mf.file_name,
                mf.size_bytes,
                mf.height,
                mf.width,
                e.bits_per_pixel,
                e.compression_potential,
                e.crf
            FROM media_files mf
            JOIN events e ON e.media_file_id = mf.id
                AND e.event_type = 'transcode_decision_approved'
            WHERE mf.status = 'pending'
            ORDER BY e.compression_potential DESC NULLS LAST
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .filter_map(|r| {
                Some(PendingTranscodeItem {
                    file_name: r.file_name,
                    size_bytes: r.size_bytes as u64,
                    height: r.height as u32,
                    width: r.width as u32,
                    bits_per_pixel: r.bits_per_pixel?,
                    compression_potential: r.compression_potential?,
                    crf: r.crf? as u8,
                })
            })
            .collect();

        Ok(items)
    }
}
