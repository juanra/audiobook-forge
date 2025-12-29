//! CLI commands and arguments

use clap::{Parser, Subcommand, Args};
use std::path::PathBuf;

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

    /// Quality preset for output audio
    #[arg(long, value_parser = ["low", "medium", "high", "ultra", "maximum", "source"])]
    pub quality: Option<String>,

    /// AAC encoder to use (auto, aac_at, libfdk_aac, aac)
    #[arg(long)]
    pub aac_encoder: Option<String>,

    /// DEPRECATED: Use --aac-encoder instead
    #[arg(long, hide = true)]
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

        /// Update chapters from file (text/EPUB)
        #[arg(long)]
        chapters: Option<PathBuf>,

        /// Fetch chapters from Audnex API by ASIN
        #[arg(long, conflicts_with = "chapters")]
        chapters_asin: Option<String>,

        /// Only update chapters, skip metadata enrichment
        #[arg(long)]
        update_chapters_only: bool,

        /// Chapter merge strategy (keep-timestamps, replace-all, skip-on-mismatch, interactive)
        #[arg(long, default_value = "interactive")]
        merge_strategy: String,
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
