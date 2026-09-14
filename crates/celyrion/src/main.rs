//! `celyrion` command-line entry point.

#![forbid(unsafe_code)]

use std::path::PathBuf;

use anyhow::Context;
use celyrion_runtime::Runtime;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

/// Celyrion AI inference engine.
#[derive(Debug, Parser)]
#[command(name = "celyrion", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Serve the model described by a `model.yaml` file.
    Serve {
        /// Path to the model configuration.
        model: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    match Cli::parse().command {
        Command::Serve { model } => {
            let runtime =
                Runtime::load(&model).with_context(|| format!("loading {}", model.display()))?;
            println!("celyrion: loaded model `{}`", runtime.config().name);
            runtime.serve().await.context("serving model")?;
        }
    }
    Ok(())
}
