use crate::{
    application::{ScanFolderUseCase, WatchEventUseCase},
    infra::{PostgressRepository, mcp::server::run_mcp_server},
};
use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::{path::PathBuf, sync::Arc};

/// HEVC encoder to use for transcoding.
#[derive(Debug, Clone, ValueEnum, Default)]
pub enum EncoderArg {
    /// Auto-detect the best available encoder.
    #[default]
    Auto,
    /// Apple VideoToolbox (macOS / Apple Silicon).
    Videotoolbox,
    /// NVIDIA NVENC (Linux + NVIDIA GPU).
    Nvenc,
    /// Software encoder, available everywhere.
    Libx265,
}

/// Scan media files, analyze them and fetch metadatas.
/// Take decision to know if they should be transcoded.
#[derive(Debug, Parser)]
#[command(name = "rankode")]
pub enum Command {
    /// Do postgresql schema migration.
    Migrate,
    /// Scan a given folder to find new media files and ffprobe them.
    Scan {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Watch for new events and execute associated actions.
    Watch {
        /// If true do not transcode pending files.
        #[arg(long, short, default_value = "false")]
        dry_run: bool,

        /// Do a scan of the given folder before watching.
        #[arg(long, short, default_value = None)]
        scan: Option<PathBuf>,

        /// HEVC encoder to use for transcoding.
        #[arg(long, default_value = "auto")]
        encoder: EncoderArg,

        /// Port for the MCP HTTP server.
        #[arg(long, default_value = "3333")]
        mcp_port: u16,
    },
    // TODO Process,
}

impl Command {
    /// Extracts the encoder arg before `execute` consumes `self`.
    pub fn encoder_arg(&self) -> EncoderArg {
        match self {
            Command::Watch { encoder, .. } => encoder.clone(),
            _ => EncoderArg::Auto,
        }
    }

    pub async fn execute(
        self,
        repository: Arc<PostgressRepository>,
        scanner: Arc<ScanFolderUseCase>,
        watcher: WatchEventUseCase,
    ) -> Result<()> {
        match self {
            Command::Migrate => repository.migrate().await,
            Command::Scan { path } => scanner.execute(path).await,
            Command::Watch {
                dry_run,
                scan,
                mcp_port,
                ..
            } => {
                if let Some(path) = scan {
                    tokio::spawn(async move {
                        let _ = scanner.execute(path).await;
                    });
                }

                tokio::select! {
                    _watch = watcher.execute(dry_run) => { Ok(()) },
                    _mcp = run_mcp_server(repository.clone(), mcp_port) => { Ok(()) },
                    _ = tokio::signal::ctrl_c() => { Ok(()) },
                }
            }
        }
    }
}
