//! Core processing modules
//!
//! This module contains the main business logic for audiobook processing:
//! - Scanner: Directory scanning and book folder discovery
//! - Analyzer: Audio file analysis and quality detection
//! - Processor: Single book processing (FFmpeg, metadata, chapters)
//! - BatchProcessor: Parallel batch processing

mod analyzer;
mod batch;
mod m4b_merger;
mod organizer;
mod processor;
mod progress;
mod retry;
mod scanner;

pub use analyzer::Analyzer;
pub use batch::BatchProcessor;
pub use m4b_merger::M4bMerger;
pub use organizer::{OrganizeAction, OrganizeResult, Organizer};
pub use processor::Processor;
pub use progress::{BatchProgress, BookProgress, ProcessingStage};
pub use retry::{classify_error, retry_async, smart_retry_async, ErrorType, RetryConfig};
pub use scanner::Scanner;
