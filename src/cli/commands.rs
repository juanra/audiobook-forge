//! CLI commands and arguments

use clap::{Parser, Subcommand, Args};
use std::path::PathBuf;
use anyhow::Result;

use crate::utils::{ConfigManager, DependencyChecker};
use crate::VERSION;

/// Audiobook Forge - Convert audiobook directories to M4B format
#[derive(Parser)]
#[command(name = "audiobook-forge")]
#[command(version = VERSION)]
#[command(about = "Convert audiobook directories to M4B format with chapters and metadata")]
#[command(long_about = "
Audiobook Forge is a CLI tool that converts audiobook directories containing
MP3 files into high-quality M4B audiobook files with proper chapters and metadata.

Features:
• Automatic quality detection and preservation
• Smart chapter generation from multiple sources
• Parallel batch processing
• Metadata extraction and enhancement
• Cover art embedding
")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(global = true, short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Process audiobooks and convert to M4B
    Build(BuildArgs),

    /// Organize audiobooks into M4B and To_Convert folders
    Organize(OrganizeArgs),

    /// Manage configuration
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Fetch and manage Audible metadata
    #[command(subcommand)]
    Metadata(MetadataCommands),

    /// Interactive metadata matching for M4B files
    Match(MatchArgs),

    /// Check system dependencies
    Check,

    /// Show version information
    Version,
}

#[derive(Args)]
pub struct BuildArgs {
    /// Root directory containing audiobook folders
    #[arg(short, long)]
    pub root: Option<PathBuf>,

    /// Output directory (defaults to same as root)
    #[arg(short, long)]
    pub out: Option<PathBuf>,

    /// Number of parallel workers (1-8)
    #[arg(short = 'j', long, value_parser = clap::value_parser!(u8).range(1..=8))]
    pub parallel: Option<u8>,

    /// Skip folders with existing M4B files
    #[arg(long)]
    pub skip_existing: Option<bool>,

    /// Force reprocessing (overwrite existing)
    #[arg(long)]
    pub force: bool,

    /// Normalize existing M4B files (fix metadata)
    #[arg(long)]
    pub normalize: bool,

    /// Dry run (analyze without creating files)
    #[arg(long)]
    pub dry_run: bool,

    /// Prefer stereo over mono
    #[arg(long)]
    pub prefer_stereo: Option<bool>,

    /// Chapter source priority
    #[arg(long, value_parser = ["auto", "files", "cue", "id3", "none"])]
    pub chapter_source: Option<String>,

    /// Cover art filenames (comma-separated)
    #[arg(long)]
    pub cover_names: Option<String>,

    /// Default language for metadata
    #[arg(long)]
    pub language: Option<String>,

    /// Keep temporary files for debugging
    #[arg(long)]
    pub keep_temp: bool,

    /// Delete original files after conversion
    #[arg(long)]
    pub delete_originals: bool,

    /// Use Apple Silicon encoder (aac_at)
    #[arg(long)]
    pub use_apple_silicon_encoder: Option<bool>,

    /// Fetch metadata from Audible during build
    #[arg(long)]
    pub fetch_audible: bool,

    /// Audible region (us, uk, ca, au, fr, de, jp, it, in, es)
    #[arg(long)]
    pub audible_region: Option<String>,

    /// Auto-match books with Audible by folder name
    #[arg(long)]
    pub audible_auto_match: bool,

    /// Configuration file path
    #[arg(long)]
    pub config: Option<PathBuf>,
}

#[derive(Args)]
pub struct OrganizeArgs {
    /// Root directory to organize
    #[arg(short, long)]
    pub root: Option<PathBuf>,

    /// Dry run (show what would be done)
    #[arg(long)]
    pub dry_run: bool,

    /// Configuration file path
    #[arg(long)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Initialize config file with defaults
    Init {
        /// Overwrite existing config file
        #[arg(long)]
        force: bool,
    },

    /// Show current configuration
    Show {
        /// Configuration file path
        #[arg(long)]
        config: Option<PathBuf>,
    },

    /// Validate configuration file
    Validate {
        /// Configuration file path
        #[arg(long)]
        config: Option<PathBuf>,
    },

    /// Show config file path
    Path,

    /// Edit config file in default editor
    Edit,
}

#[derive(Subcommand)]
pub enum MetadataCommands {
    /// Fetch metadata from Audible
    Fetch {
        /// Audible ASIN (B002V5D7RU format)
        #[arg(long)]
        asin: Option<String>,

        /// Search by title
        #[arg(long)]
        title: Option<String>,

        /// Search by author
        #[arg(long)]
        author: Option<String>,

        /// Audible region (us, uk, ca, au, fr, de, jp, it, in, es)
        #[arg(long, default_value = "us")]
        region: String,

        /// Save metadata to JSON file
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Enrich M4B file with Audible metadata
    Enrich {
        /// M4B file to enrich
        #[arg(long)]
        file: PathBuf,

        /// Audible ASIN
        #[arg(long)]
        asin: Option<String>,

        /// Auto-detect ASIN from filename
        #[arg(long)]
        auto_detect: bool,

        /// Audible region
        #[arg(long, default_value = "us")]
        region: String,
    },
}

/// Arguments for the match command
#[derive(Args)]
pub struct MatchArgs {
    /// M4B file to match
    #[arg(long, short = 'f', conflicts_with = "dir")]
    pub file: Option<PathBuf>,

    /// Directory of M4B files
    #[arg(long, short = 'd', conflicts_with = "file")]
    pub dir: Option<PathBuf>,

    /// Manual title override
    #[arg(long)]
    pub title: Option<String>,

    /// Manual author override
    #[arg(long)]
    pub author: Option<String>,

    /// Auto mode (non-interactive, select best match)
    #[arg(long)]
    pub auto: bool,

    /// Audible region
    #[arg(long, default_value = "us")]
    pub region: String,

    /// Keep existing cover art instead of downloading
    #[arg(long)]
    pub keep_cover: bool,

    /// Dry run (show matches but don't apply)
    #[arg(long)]
    pub dry_run: bool,
}

/// Run the CLI application
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging based on verbosity
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level))
        )
        .init();

    // Execute command
    match cli.command {
        Commands::Build(args) => run_build(args),
        Commands::Organize(args) => run_organize(args),
        Commands::Config(cmd) => run_config(cmd),
        Commands::Metadata(cmd) => run_metadata(cmd),
        Commands::Match(args) => run_match(args),
        Commands::Check => run_check(),
        Commands::Version => run_version(),
    }
}

fn run_build(args: BuildArgs) -> Result<()> {
    println!("Build command - Phase 2-4 implementation");
    println!("Args: root={:?}, out={:?}, parallel={:?}",
        args.root, args.out, args.parallel);

    // TODO: Implement in Phase 2-4
    anyhow::bail!("Build command not yet implemented. Coming in Phase 2!");
}

fn run_organize(args: OrganizeArgs) -> Result<()> {
    println!("Organize command - Phase 5 implementation");
    println!("Args: root={:?}, dry_run={}",
        args.root, args.dry_run);

    // TODO: Implement in Phase 5
    anyhow::bail!("Organize command not yet implemented. Coming in Phase 5!");
}

fn run_config(cmd: ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Init { force } => {
            let path = ConfigManager::init(force)?;
            println!("✓ Config file created at: {}", path.display());
            println!("\nEdit the file to customize your settings:");
            println!("  audiobook-forge config edit");
            Ok(())
        }

        ConfigCommands::Show { config } => {
            let yaml = ConfigManager::show(config.as_ref())?;
            println!("{}", yaml);
            Ok(())
        }

        ConfigCommands::Validate { config } => {
            let cfg = ConfigManager::load_or_default(config.as_ref())?;
            let warnings = ConfigManager::validate(&cfg)?;

            if warnings.is_empty() {
                println!("✓ Configuration is valid");
            } else {
                println!("⚠ Configuration warnings:");
                for warning in warnings {
                    println!("  • {}", warning);
                }
            }
            Ok(())
        }

        ConfigCommands::Path => {
            let path = ConfigManager::default_config_path()?;
            println!("{}", path.display());
            Ok(())
        }

        ConfigCommands::Edit => {
            let path = ConfigManager::default_config_path()?;

            // Create config if it doesn't exist
            if !path.exists() {
                ConfigManager::init(false)?;
            }

            // Open in default editor
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
            let status = std::process::Command::new(&editor)
                .arg(&path)
                .status()?;

            if !status.success() {
                anyhow::bail!("Editor exited with non-zero status");
            }

            Ok(())
        }
    }
}

fn run_check() -> Result<()> {
    println!("Audiobook Forge v{}", VERSION);
    println!("\nChecking system dependencies...\n");

    let deps = DependencyChecker::check_all();
    let all_met = deps.iter().all(|d| d.found);

    for dep in &deps {
        println!("{}", dep);
    }

    if all_met {
        println!("\n✓ All dependencies are installed");

        // Check for Apple Silicon encoder
        if std::env::consts::OS == "macos" {
            if DependencyChecker::check_aac_at_support() {
                println!("✓ Apple Silicon encoder (aac_at) is available");
            } else {
                println!("ℹ Apple Silicon encoder (aac_at) not available");
            }
        }

        Ok(())
    } else {
        println!("\n✗ Some dependencies are missing");
        println!("\nInstallation instructions:");
        println!("  macOS:   brew install ffmpeg atomicparsley gpac");
        println!("  Ubuntu:  apt install ffmpeg atomicparsley gpac");
        println!("  Arch:    pacman -S ffmpeg atomicparsley gpac");

        anyhow::bail!("Missing required dependencies");
    }
}

fn run_metadata(cmd: MetadataCommands) -> Result<()> {
    use crate::cli::handlers::handle_metadata;
    use crate::utils::ConfigManager;

    // Load config
    let config = ConfigManager::load_or_default(None)?;

    // Run async handler
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(handle_metadata(cmd, config))
}

fn run_match(args: MatchArgs) -> Result<()> {
    use crate::cli::handlers::handle_match;
    use crate::utils::ConfigManager;

    // Load config
    let config = ConfigManager::load_or_default(None)?;

    // Run async handler
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(handle_match(args, config))
}

fn run_version() -> Result<()> {
    println!("Audiobook Forge v{}", VERSION);
    println!("Rust rewrite - High-performance audiobook processing");
    Ok(())
}
