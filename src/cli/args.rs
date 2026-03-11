//! CLI argument definitions.

use clap::Parser;
use std::path::PathBuf;

/// rustaria - A Rust download manager powered by aria2
#[derive(Parser, Debug)]
#[command(name = "rustaria")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Run in daemon mode (headless, no TUI)
    #[arg(short, long)]
    pub daemon: bool,

    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Subcommand to run
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Available subcommands.
#[derive(Parser, Debug)]
pub enum Command {
    /// Add a download
    Add(AddArgs),

    /// Pause a download
    Pause(PauseArgs),

    /// Resume a download
    Resume(ResumeArgs),

    /// Remove a download
    Remove(RemoveArgs),

    /// List downloads
    List(ListArgs),

    /// Show download status
    Status(StatusArgs),

    /// Show or edit configuration
    Config(ConfigArgs),
}

/// Arguments for the `add` command.
#[derive(Parser, Debug)]
pub struct AddArgs {
    /// URL(s) to download
    #[arg(required = true)]
    pub urls: Vec<String>,

    /// Output filename
    #[arg(short, long)]
    pub output: Option<String>,

    /// Download directory
    #[arg(short, long)]
    pub dir: Option<PathBuf>,

    /// HTTP headers (can be repeated)
    #[arg(short = 'H', long)]
    pub header: Vec<String>,

    /// Referer URL
    #[arg(long)]
    pub referer: Option<String>,

    /// User agent string
    #[arg(long)]
    pub user_agent: Option<String>,

    /// Category for the download
    #[arg(short, long)]
    pub category: Option<String>,

    /// Tags for the download
    #[arg(short, long)]
    pub tags: Vec<String>,

    /// Start download immediately (don't queue)
    #[arg(long)]
    pub start: bool,
}

/// Arguments for the `pause` command.
#[derive(Parser, Debug)]
pub struct PauseArgs {
    /// Job ID(s) to pause (or "all")
    #[arg(required = true)]
    pub ids: Vec<String>,
}

/// Arguments for the `resume` command.
#[derive(Parser, Debug)]
pub struct ResumeArgs {
    /// Job ID(s) to resume (or "all")
    #[arg(required = true)]
    pub ids: Vec<String>,
}

/// Arguments for the `remove` command.
#[derive(Parser, Debug)]
pub struct RemoveArgs {
    /// Job ID(s) to remove
    #[arg(required = true)]
    pub ids: Vec<String>,

    /// Also delete downloaded files
    #[arg(long)]
    pub delete_files: bool,

    /// Force removal without confirmation
    #[arg(short, long)]
    pub force: bool,
}

/// Arguments for the `list` command.
#[derive(Parser, Debug)]
pub struct ListArgs {
    /// Filter by status
    #[arg(short, long)]
    pub status: Option<String>,

    /// Filter by category
    #[arg(short, long)]
    pub category: Option<String>,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    pub format: String,

    /// Maximum number of results
    #[arg(short, long)]
    pub limit: Option<usize>,
}

/// Arguments for the `status` command.
#[derive(Parser, Debug)]
pub struct StatusArgs {
    /// Job ID to show status for
    #[arg(required = true)]
    pub id: String,

    /// Output format (table, json)
    #[arg(short, long, default_value = "table")]
    pub format: String,
}

/// Arguments for the `config` command.
#[derive(Parser, Debug)]
pub struct ConfigArgs {
    /// Show current configuration
    #[arg(long)]
    pub show: bool,

    /// Open configuration in editor
    #[arg(long)]
    pub edit: bool,

    /// Set a configuration value (key=value)
    #[arg(long)]
    pub set: Option<String>,

    /// Get a configuration value
    #[arg(long)]
    pub get: Option<String>,
}
