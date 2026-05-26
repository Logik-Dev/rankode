use std::sync::Arc;

use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{CallToolResult, Content, Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};

use crate::domain::PendingReportRepository;

#[derive(Clone)]
pub struct RankodeHandler {
    repo: Arc<dyn PendingReportRepository>,
    tool_router: ToolRouter<RankodeHandler>,
}

#[tool_router]
impl RankodeHandler {
    pub fn new(repo: Arc<dyn PendingReportRepository>) -> Self {
        Self {
            repo,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "List all files pending HEVC transcoding, sorted by estimated gain descending"
    )]
    async fn list_pending(&self) -> Result<CallToolResult, McpError> {
        let items = self
            .repo
            .list_pending()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        if items.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No files pending transcoding.",
            )]));
        }

        let total_size_gb: f64 = items.iter().map(|i| i.size_bytes as f64 / 1e9).sum();
        let total_gain_gb: f64 = items
            .iter()
            .map(|i| i.estimated_gain_bytes() as f64 / 1e9)
            .sum();

        let mut lines = vec![
            format!(
                "{} files pending — {:.1} GB total",
                items.len(),
                total_size_gb
            ),
            format!(
                "Estimated gain: ~{:.1} GB ({:.0}%)\n",
                total_gain_gb,
                total_gain_gb / total_size_gb * 100.0
            ),
            format!(
                "{:<50} {:>9}  {:>9}  {:>8}  CRF",
                "File", "Size", "Estimated", "Gain"
            ),
            "-".repeat(85),
        ];

        for item in &items {
            let size_gb = item.size_bytes as f64 / 1e9;
            let estimated_gb = item.estimated_output_bytes() as f64 / 1e9;
            let gain_gb = item.estimated_gain_bytes() as f64 / 1e9;
            let resolution = format!("{}p", item.height);

            lines.push(format!(
                "{:<50} {:>7.1} GB  {:>7.1} GB  {:>+6.1} GB  {}",
                format!("{} ({})", truncate(&item.file_name, 40), resolution),
                size_gb,
                estimated_gb,
                -gain_gb,
                item.crf,
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(
            lines.join("\n"),
        )]))
    }
}

#[tool_handler]
impl ServerHandler for RankodeHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::from_build_env())
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
