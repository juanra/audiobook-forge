//! Audiobook Forge - CLI tool for converting audiobook directories to M4B format
//!
//! This library provides the core functionality for audiobook processing, including:
//! - Audio file analysis and quality detection
//! - FFmpeg-based transcoding and concatenation
//! - Metadata extraction and injection
//! - Chapter generation and management
//! - Parallel batch processing

pub mod audio;
pub mod cli;
pub mod core;
pub mod models;
pub mod ui;
pub mod utils;

// Re-export commonly used types
pub use core::{Analyzer, BatchProcessor, Processor, Scanner};
pub use models::{BookCase, BookFolder, ProcessingResult, QualityProfile, Track};
pub use utils::Config;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
