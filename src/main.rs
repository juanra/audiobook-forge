//! Audiobook Forge CLI entry point

use anyhow::{Context, Result};
use audiobook_forge::cli::{handle_build, handle_check, handle_config, handle_organize, handle_metadata, handle_match, Cli, Commands};
use audiobook_forge::utils::ConfigManager;
use audiobook_forge::VERSION;
use clap::Parser;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Load configuration (or use defaults)
    let config = load_config()?;

    // Initialize logging (needs config for file logging settings)
    init_logging(cli.verbose, &config)?;

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

        Commands::Metadata(command) => {
            handle_metadata(command, config).await?;
        }

        Commands::Match(args) => {
            handle_match(args, config).await?;
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

/// Initialize logging based on verbosity level and config
fn init_logging(verbose: bool, config: &audiobook_forge::models::Config) -> Result<()> {
    // Determine log level
    let level_str = if verbose {
        "audiobook_forge=debug"
    } else {
        match config.logging.log_level.to_uppercase().as_str() {
            "DEBUG" => "audiobook_forge=debug",
            "WARNING" | "WARN" => "audiobook_forge=warn",
            "ERROR" => "audiobook_forge=error",
            _ => "audiobook_forge=info",
        }
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level_str));

    // Console layer (always present)
    let console_layer = fmt::layer()
        .with_target(false)
        .with_level(true)
        .with_filter(filter.clone());

    // File layer (optional)
    if config.logging.log_to_file {
        let log_file = config.logging.log_file.clone().unwrap_or_else(|| {
            let home = dirs::home_dir().expect("Cannot determine home directory");
            let log_dir = home.join(".audiobook-forge").join("logs");
            std::fs::create_dir_all(&log_dir)
                .expect("Failed to create log directory");
            log_dir.join("audiobook-forge.log")
        });

        // Create log directory
        if let Some(parent) = log_file.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create log directory")?;
        }

        // Daily rotation
        let file_appender = tracing_appender::rolling::daily(
            log_file.parent().unwrap(),
            log_file.file_name().unwrap()
        );

        let file_layer = fmt::layer()
            .with_writer(file_appender)
            .with_target(true)
            .with_level(true)
            .with_ansi(false)
            .with_filter(filter);

        // Both console and file
        tracing_subscriber::registry()
            .with(console_layer)
            .with(file_layer)
            .init();

        tracing::info!("Logging to file: {}", log_file.display());
    } else {
        // Console only
        tracing_subscriber::registry()
            .with(console_layer)
            .init();
    }

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
