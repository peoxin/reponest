use clap::{Parser, Subcommand, builder::Styles};

/// Styles for clap output
const STYLES: Styles = Styles::styled()
    .header(clap::builder::styling::AnsiColor::Green.on_default().bold())
    .usage(clap::builder::styling::AnsiColor::Green.on_default().bold())
    .literal(clap::builder::styling::AnsiColor::Cyan.on_default().bold())
    .placeholder(clap::builder::styling::AnsiColor::Yellow.on_default());

/// Command line arguments
#[derive(Parser, Debug)]
#[command(name = "reponest")]
#[command(author, version, about)]
#[command(styles = STYLES)]
#[command(
    long_about = "A TUI/CLI tool for managing multiple git repositories written in Rust.\n\n\
    By default (without subcommands), launches an interactive TUI.\n\
    Use specified subcommands for non-interactive CLI output."
)]
#[command(after_long_help = "Examples:\n  \
    reponest [PATH]                   # Launch interactive TUI\n  \
    reponest --dirty [PATH]           # Launch TUI, show only dirty repos\n  \
    reponest list [PATH]              # List all repos (CLI)\n  \
    reponest list --detail [PATH]     # List all repos with details (CLI)")]
pub struct CliArgs {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<CliSubCommands>,

    /// Path to scan for repos (default: home directory)
    #[arg(global = true, value_name = "PATH")]
    pub path: Option<String>,

    /// Maximum scan depth (0 = unlimited)
    #[arg(global = true, long, value_name = "DEPTH")]
    pub max_depth: Option<usize>,

    /// Show only repos with uncommitted changes
    #[arg(global = true, long)]
    pub dirty: bool,

    /// Show only repos with conflicts
    #[arg(global = true, long)]
    pub conflict: bool,

    /// Configuration file to load
    #[arg(
        global = true,
        short,
        long,
        value_name = "FILE",
        help_heading = "Configuration"
    )]
    pub config: Option<String>,

    /// Theme to use for TUI
    #[arg(
        global = true,
        long,
        value_name = "THEME",
        help_heading = "Configuration"
    )]
    pub theme: Option<String>,

    /// Print current configuration and exit
    #[arg(global = true, long, help_heading = "Configuration")]
    pub print_config: bool,

    /// Write the cwd on exit to FILE
    #[arg(global = true, long, value_name = "FILE")]
    pub cwd_file: Option<String>,
}

/// Subcommands and their arguments
#[derive(Subcommand, Debug)]
pub enum CliSubCommands {
    /// List repositories (non-interactive output)
    #[command(visible_alias = "ls")]
    List {
        /// Show detailed information
        #[arg(long)]
        detail: bool,

        /// Output as JSON format
        #[arg(long)]
        json: bool,
    },
}
