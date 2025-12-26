use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "claude-box")]
#[command(about = "A multi-platform Claude Code sandbox manager")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Environment name to open (uses default if not specified)
    #[arg(value_name = "ENVIRONMENT")]
    pub environment: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Open the configuration TUI to manage environments
    Config,
}
