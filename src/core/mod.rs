//! Core processing modules
//!
//! This module contains the main business logic for audiobook processing:
//! - Scanner: Directory scanning and book folder discovery
//! - Analyzer: Audio file analysis and quality detection
//! - Processor: Single book processing (FFmpeg, metadata, chapters)
//! - BatchProcessor: Parallel batch processing

mod scanner;
mod analyzer;
mod processor;
mod batch;
mod progress;
mod retry;
mod organizer;

pub use scanner::Scanner;
pub use analyzer::Analyzer;
pub use processor::Processor;
pub use batch::BatchProcessor;
pub use progress::{BatchProgress, BookProgress, ProcessingStage};
pub use retry::{RetryConfig, classify_error, retry_async, smart_retry_async, ErrorType};
pub use organizer::{Organizer, OrganizeResult, OrganizeAction};
