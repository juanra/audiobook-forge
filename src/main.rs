//! Audiobook Forge CLI entry point

use anyhow::Result;
use audiobook_forge::cli::{handle_build, handle_check, handle_config, handle_organize, Cli, Commands};
use audiobook_forge::utils::ConfigManager;
use audiobook_forge::VERSION;
use clap::Parser;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose)?;

    // Load configuration (or use defaults)
    let config = load_config()?;

    // Execute command
    match cli.command {
        Commands::Build(args) => {
            handle_build(args, config).await?;
        }

        Commands::Organize(args) => {
            handle_organize(args, config)?;
        }

        Commands::Config(command) => {
            handle_config(command)?;
        }

        Commands::Check => {
            handle_check()?;
        }

        Commands::Version => {
            println!("audiobook-forge {}", VERSION);
            println!("A Rust-powered CLI tool for converting audiobooks to M4B format");
        }
    }

    Ok(())
}

/// Initialize logging based on verbosity level
fn init_logging(verbose: bool) -> Result<()> {
    let filter = if verbose {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("audiobook_forge=debug"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("audiobook_forge=info"))
    };

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_level(true)
        .init();

    Ok(())
}

/// Load configuration file (or use defaults if not found)
fn load_config() -> Result<audiobook_forge::models::Config> {
    let config_path = ConfigManager::default_config_path()?;

    if config_path.exists() {
        ConfigManager::load(&config_path)
    } else {
        // Config file not found - use defaults silently
        tracing::debug!("No config file found, using defaults");
        Ok(audiobook_forge::models::Config::default())
    }
}
