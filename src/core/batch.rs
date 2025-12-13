//! Batch processor for parallel audiobook processing

use crate::core::{Processor, RetryConfig, smart_retry_async};
use crate::models::{BookFolder, ProcessingResult};
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};

/// Batch processor for converting multiple audiobooks in parallel
pub struct BatchProcessor {
    /// Number of parallel workers
    workers: usize,
    /// Keep temporary files for debugging
    keep_temp: bool,
    /// Use Apple Silicon hardware encoder
    use_apple_silicon: bool,
    /// Enable parallel file encoding
    enable_parallel_encoding: bool,
    /// Maximum concurrent encoding operations (to limit CPU usage)
    max_concurrent_encodes: usize,
    /// Retry configuration
    retry_config: RetryConfig,
}

impl BatchProcessor {
    /// Create a new batch processor with default settings
    pub fn new(workers: usize) -> Self {
        Self {
            workers: workers.clamp(1, 16),
            keep_temp: false,
            use_apple_silicon: false,
            enable_parallel_encoding: true,
            max_concurrent_encodes: 2, // Default: 2 concurrent encodes
            retry_config: RetryConfig::new(),
        }
    }

    /// Create batch processor with custom options
    pub fn with_options(
        workers: usize,
        keep_temp: bool,
        use_apple_silicon: bool,
        enable_parallel_encoding: bool,
        max_concurrent_encodes: usize,
        retry_config: RetryConfig,
    ) -> Self {
        Self {
            workers: workers.clamp(1, 16),
            keep_temp,
            use_apple_silicon,
            enable_parallel_encoding,
            max_concurrent_encodes: max_concurrent_encodes.clamp(1, 16),
            retry_config,
        }
    }

    /// Process multiple books in parallel
    pub async fn process_batch(
        &self,
        books: Vec<BookFolder>,
        output_dir: &Path,
        chapter_source: &str,
    ) -> Vec<ProcessingResult> {
        let total_books = books.len();

        if total_books == 0 {
            return Vec::new();
        }

        tracing::info!(
            "Starting batch processing: {} books with {} workers (max {} concurrent encodes)",
            total_books,
            self.workers,
            self.max_concurrent_encodes
        );

        // Create a semaphore to limit concurrent encoding operations
        let encode_semaphore = Arc::new(Semaphore::new(self.max_concurrent_encodes));

        // Create channel for collecting results
        let (result_tx, mut result_rx) = mpsc::channel(total_books);

        // Spawn tasks for each book
        let mut handles = Vec::new();

        for (index, book) in books.into_iter().enumerate() {
            let result_tx = result_tx.clone();
            let output_dir = output_dir.to_path_buf();
            let chapter_source = chapter_source.to_string();
            let keep_temp = self.keep_temp;
            let use_apple_silicon = self.use_apple_silicon;
            let enable_parallel_encoding = self.enable_parallel_encoding;
            let encode_semaphore = Arc::clone(&encode_semaphore);
            let retry_config = self.retry_config.clone();

            let handle = tokio::spawn(async move {
                // Acquire semaphore permit before encoding (limits concurrent encodes)
                let _permit = encode_semaphore.acquire().await.unwrap();

                tracing::info!(
                    "[{}/{}] Processing: {}",
                    index + 1,
                    total_books,
                    book.name
                );

                // Process with retry logic
                let result = smart_retry_async(&retry_config, || {
                    Self::process_single_book(
                        &book,
                        &output_dir,
                        &chapter_source,
                        keep_temp,
                        use_apple_silicon,
                        enable_parallel_encoding,
                    )
                })
                .await
                .unwrap_or_else(|e| {
                    // If all retries fail, return a failure result
                    tracing::error!("✗ {}: {}", book.name, e);
                    ProcessingResult::new(book.name.clone())
                        .failure(format!("All retries failed: {}", e), 0.0)
                });

                // Send result through channel
                let _ = result_tx.send(result).await;
            });

            handles.push(handle);
        }

        // Drop the original sender so the receiver knows when all tasks are done
        drop(result_tx);

        // Collect all results
        let mut results = Vec::new();
        while let Some(result) = result_rx.recv().await {
            results.push(result);
        }

        // Wait for all tasks to complete
        for handle in handles {
            let _ = handle.await;
        }

        tracing::info!(
            "Batch processing complete: {}/{} successful",
            results.iter().filter(|r| r.success).count(),
            results.len()
        );

        results
    }

    /// Process a single book (internal helper)
    async fn process_single_book(
        book: &BookFolder,
        output_dir: &Path,
        chapter_source: &str,
        keep_temp: bool,
        use_apple_silicon: bool,
        enable_parallel_encoding: bool,
    ) -> Result<ProcessingResult> {
        let processor = Processor::with_options(keep_temp, use_apple_silicon, enable_parallel_encoding)?;

        let result = processor
            .process_book(book, output_dir, chapter_source)
            .await?;

        tracing::info!("✓ {}: {:.1}s", book.name, result.processing_time);
        Ok(result)
    }

    /// Get recommended worker count based on system
    pub fn recommended_workers() -> usize {
        let cpu_count = num_cpus::get();

        // Use 50% of CPU cores for parallel processing
        // (reserves cores for FFmpeg itself which is multi-threaded)
        (cpu_count / 2).max(1).min(8)
    }
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new(Self::recommended_workers())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_processor_creation() {
        let processor = BatchProcessor::new(4);
        assert_eq!(processor.workers, 4);
        assert_eq!(processor.max_concurrent_encodes, 2);
        assert!(!processor.keep_temp);
        assert!(!processor.use_apple_silicon);
    }

    #[test]
    fn test_batch_processor_with_options() {
        let processor = BatchProcessor::with_options(8, true, true, true, 4, RetryConfig::new());
        assert_eq!(processor.workers, 8);
        assert_eq!(processor.max_concurrent_encodes, 4);
        assert!(processor.keep_temp);
        assert!(processor.use_apple_silicon);
    }

    #[test]
    fn test_worker_clamping() {
        // Test lower bound
        let processor = BatchProcessor::new(0);
        assert_eq!(processor.workers, 1);

        // Test upper bound
        let processor = BatchProcessor::new(100);
        assert_eq!(processor.workers, 16);
    }

    #[test]
    fn test_concurrent_encode_clamping() {
        let processor = BatchProcessor::with_options(4, false, false, true, 0, RetryConfig::new());
        assert_eq!(processor.max_concurrent_encodes, 1);

        let processor = BatchProcessor::with_options(4, false, false, true, 100, RetryConfig::new());
        assert_eq!(processor.max_concurrent_encodes, 16);
    }

    #[test]
    fn test_recommended_workers() {
        let workers = BatchProcessor::recommended_workers();
        assert!(workers >= 1);
        assert!(workers <= 8);
    }

    #[tokio::test]
    async fn test_empty_batch() {
        let processor = BatchProcessor::new(4);
        let results = processor
            .process_batch(Vec::new(), Path::new("/tmp"), "auto")
            .await;
        assert_eq!(results.len(), 0);
    }
}
