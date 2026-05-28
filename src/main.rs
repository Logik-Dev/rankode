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
        AnalyzeFileUseCase, CatchUpUseCase, NotifyNextCandidateUseCase, ProcessApprovalUseCase,
        ProcessDiscoveredFileUseCase, ScanFolderUseCase, WatchApprovalUseCase, WatchEventUseCase,
        transcode_file::TranscodeFileUseCase,
    },
    cli::Command,
    domain::TakeTranscodeDecisionService,
    infra::{
        Config, FfmpegTranscoder, Ffprobe, MqttListener, MqttNotifier, PostgresEventListener,
        PostgressRepository, RadarrProvider, TokioScanner,
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

    let scan_use_case = Arc::new(ScanFolderUseCase::new(
        postgres_repo.clone(),
        tokio_scanner.clone(),
        ffprobe_analyzer.clone(),
    ));

    let process_discovered_use_case = Arc::new(ProcessDiscoveredFileUseCase::new(
        postgres_repo.clone(),
        radarr_provider.clone(),
        postgres_repo.clone(),
    ));

    let take_decision_service = Arc::new(TakeTranscodeDecisionService::new(
        cfg.min_file_size_gb,
        cfg.min_bits_per_pixel,
        cfg.min_compression_potential,
    ));

    let process_fetched_use_case = Arc::new(AnalyzeFileUseCase::new(
        take_decision_service.clone(),
        postgres_repo.clone(),
        postgres_repo.clone(),
        postgres_repo.clone(),
    ));

    let mqtt_notifier = Arc::new(MqttNotifier::new(&cfg.mqtt_host, cfg.mqtt_port));

    let notify_next_candidate = Arc::new(NotifyNextCandidateUseCase::new(
        postgres_repo.clone(),
        postgres_repo.clone(),
        mqtt_notifier,
    ));

    let process_approval = Arc::new(ProcessApprovalUseCase::new(postgres_repo.clone()));
    let approval_watcher = WatchApprovalUseCase::new(
        MqttListener::new(&cfg.mqtt_host, cfg.mqtt_port),
        process_approval,
    );

    let encoder_arg = cmd.encoder_arg();
    let ffmpeg_transcoder = Arc::new(FfmpegTranscoder::build(encoder_arg).await);

    let transcode_file_use_case = Arc::new(TranscodeFileUseCase::new(
        postgres_repo.clone(),
        ffprobe_analyzer.clone(),
        postgres_repo.clone(),
        ffmpeg_transcoder,
    ));

    let catch_up_use_case = Arc::new(CatchUpUseCase::new(postgres_repo.clone()));

    let watch_use_case = WatchEventUseCase::new(
        postgres_listener,
        catch_up_use_case,
        process_discovered_use_case.clone(),
        process_fetched_use_case.clone(),
        transcode_file_use_case,
        notify_next_candidate,
    );

    if cmd
        .execute(postgres_repo.clone(), scan_use_case, watch_use_case, approval_watcher)
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
