use std::sync::Arc;

use anyhow::Result;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService,
    session::local::LocalSessionManager,
};
use tracing::info;

use crate::{domain::PendingReportRepository, infra::mcp::handler::RankodeHandler};

pub async fn run_mcp_server(repo: Arc<dyn PendingReportRepository>, port: u16) -> Result<()> {
    if let Err(e) = start_mcp_server(repo, port).await {
        tracing::error!(%e, port, "MCP server failed to start");
    }
    std::future::pending().await
}

async fn start_mcp_server(repo: Arc<dyn PendingReportRepository>, port: u16) -> Result<()> {
    let service = StreamableHttpService::new(
        move || Ok(RankodeHandler::new(repo.clone())),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default(),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let addr = format!("0.0.0.0:{port}");
    let tcp_listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("MCP server listening on http://{addr}/mcp");

    axum::serve(tcp_listener, router).await?;
    Ok(())
}
