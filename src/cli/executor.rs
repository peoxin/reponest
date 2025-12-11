use anyhow::{Context, Result};

use crate::cli::commands;
use crate::cli::{CliArgs, CliSubCommands};
use crate::config::AppConfig;

/// Execute CLI command based on the subcommand
pub async fn execute_cli_command(args: &CliArgs, config: AppConfig) -> Result<()> {
    let command = args.command.as_ref().context("No CLI command provided")?;

    match command {
        CliSubCommands::List { detail, json } => {
            commands::list_repos(config, *detail, *json, args.dirty, args.conflict)
                .await
                .context("Failed to execute list command")?;
        }
    }
    Ok(())
}
