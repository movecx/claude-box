use anyhow::{anyhow, Result};
use clap::Parser;
use claude_box::cli::{Cli, Command};
use claude_box::config::{Config, Paths};
use claude_box::terminal::TerminalWrapper;
use claude_box::tui::run_config_tui;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommands first
    if let Some(Command::Config) = cli.command {
        return run_config_tui();
    }

    // Otherwise, run an environment
    let config = Config::load()?;
    let paths = Paths::new()?;

    // Determine which environment to run
    let env_key = match cli.environment {
        Some(name) => name,
        None => {
            // Use default environment or prompt to create one
            match config.get_default_environment() {
                Some(name) => name.to_string(),
                None => {
                    if config.environments.is_empty() {
                        eprintln!("No environments configured.");
                        eprintln!("Run 'claude-box config' to create one.");
                        return Err(anyhow!("No environments available"));
                    } else {
                        // List available environments
                        eprintln!("No default environment set. Available environments:");
                        for key in config.list_environments() {
                            let name = config
                                .get_environment(key)
                                .map(|c| c.name.as_str())
                                .unwrap_or(key);
                            eprintln!("  - {} ({})", name, key);
                        }
                        eprintln!();
                        eprintln!("Use 'claude-box <name>' or set a default with 'claude-box config'");
                        return Err(anyhow!("No default environment"));
                    }
                }
            }
        }
    };

    // Get environment config
    let env_config = config
        .get_environment(&env_key)
        .ok_or_else(|| anyhow!("Environment '{}' not found", env_key))?;

    // Ensure environment directory exists
    let claude_data_dir = paths.claude_data_dir(&env_key);
    if !claude_data_dir.exists() {
        claude_box::environment::create_claude_data_structure(&claude_data_dir)?;
    }

    // Get current working directory
    let working_dir = std::env::current_dir()?;

    // Run the terminal wrapper
    let wrapper = TerminalWrapper::new(env_config, &claude_data_dir, &working_dir);
    wrapper.run()
}
