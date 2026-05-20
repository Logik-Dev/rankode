mod application;
mod cli;
mod domain;
mod infra;

use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use tracing::instrument;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    application::{
        ProcessDiscoveredFileUseCase, ProcessFetchedLibraryItemUseCase, ScanFolderUseCase,
        WatchEventUseCase,
    },
    cli::Command,
    domain::TakeTranscodeDecisionService,
    infra::{
        Config, Ffprobe, PostgresEventListener, PostgressRepository, RadarrProvider, TokioScanner,
    },
};

#[tokio::main]
#[instrument(err)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let cmd = Command::parse();
    let cfg = Config::from_env();

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await?;

    let postgres_repo = Arc::new(PostgressRepository::new(pool.clone()));
    let postgres_listener = Arc::new(PostgresEventListener::new(pool.clone()));
    let tokio_scanner = Arc::new(TokioScanner);
    let ffprobe_analyzer = Arc::new(Ffprobe);
    let radarr_provider = Arc::new(RadarrProvider::new(&cfg.radarr_url, &cfg.radarr_api_key));

    // Scan folders to find movies
    let scan_use_case = Arc::new(ScanFolderUseCase::new(
        postgres_repo.clone(),
        tokio_scanner.clone(),
        ffprobe_analyzer.clone(),
    ));

    // Fetch movies metadata with radarr for now
    let process_discovered_use_case = Arc::new(ProcessDiscoveredFileUseCase::new(
        postgres_repo.clone(),
        radarr_provider.clone(),
        postgres_repo.clone(),
    ));

    // Take a transcode decision
    let take_decision_service = Arc::new(TakeTranscodeDecisionService::new(
        cfg.min_file_size_gb,
        cfg.min_bits_per_pixel,
        cfg.min_compression_potential,
    ));

    // When metadata fetched
    let process_fetched_use_case = Arc::new(ProcessFetchedLibraryItemUseCase::new(
        take_decision_service.clone(),
        postgres_repo.clone(),
        postgres_repo.clone(),
        postgres_repo.clone(),
    ));

    // Watch triggered notifications when an event is inserted
    let watch_use_case = WatchEventUseCase::new(
        postgres_listener,
        process_discovered_use_case.clone(),
        process_fetched_use_case.clone(),
    );

    if cmd
        .execute(postgres_repo.clone(), scan_use_case, watch_use_case)
        .await
        .is_err()
    {
        tracing::error!("Fatal error, exiting");
        std::process::exit(1);
    }
    Ok(())
}

// Logging file + stdout
fn init_tracing() {
    let stdout_layer = fmt::layer().compact().with_target(false);

    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/rankode.logs")
        .expect("Failed to open rankode.logs");

    let json_layer = fmt::layer().json().with_writer(file);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(stdout_layer)
        .with(json_layer)
        .init();
}
