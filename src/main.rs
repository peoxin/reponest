mod cli;
pub mod config;
pub mod core;
mod tui;

use anyhow::{Context, Result};
use clap::Parser;
use cli::CliArgs;
use config::AppConfig;
use tracing_subscriber::EnvFilter;

/// Set up logging based on RUST_LOG environment variable
pub fn setup_logging() {
    if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
            )
            .with_target(true)
            .with_writer(std::io::stderr)
            .init();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_logging();

    let cli_args = CliArgs::parse();
    let app_config = AppConfig::from_layers(&cli_args);

    if cli_args.print_config {
        app_config.print();
        return Ok(());
    }

    match &cli_args.command {
        Some(_) => {
            cli::execute_cli_command(&cli_args, app_config)
                .await
                .context("Failed to execute CLI command")?;
        }
        None => {
            tui::run_tui_app(app_config)
                .await
                .context("Failed to run TUI application")?;
        }
    }

    Ok(())
}
