//! Progress tracking for batch processing

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Stage of book processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessingStage {
    /// Scanning directories
    Scanning,
    /// Analyzing tracks
    Analyzing,
    /// Processing audio
    Processing,
    /// Injecting chapters
    Chapters,
    /// Injecting metadata
    Metadata,
    /// Complete
    Complete,
}

impl ProcessingStage {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Scanning => "Scanning",
            Self::Analyzing => "Analyzing",
            Self::Processing => "Processing",
            Self::Chapters => "Chapters",
            Self::Metadata => "Metadata",
            Self::Complete => "Complete",
        }
    }
}

/// Progress information for a single book
#[derive(Debug, Clone)]
pub struct BookProgress {
    /// Book name
    pub name: String,
    /// Current processing stage
    pub stage: ProcessingStage,
    /// Progress percentage (0-100)
    pub progress: f32,
    /// Start time
    pub start_time: Instant,
    /// Estimated time remaining in seconds (None if unknown)
    pub eta_seconds: Option<f64>,
}

impl BookProgress {
    /// Create a new book progress tracker
    pub fn new(name: String) -> Self {
        Self {
            name,
            stage: ProcessingStage::Scanning,
            progress: 0.0,
            start_time: Instant::now(),
            eta_seconds: None,
        }
    }

    /// Update stage
    pub fn set_stage(&mut self, stage: ProcessingStage) {
        self.stage = stage;
    }

    /// Update progress
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 100.0);
    }

    /// Calculate ETA based on current progress
    pub fn update_eta(&mut self) {
        if self.progress > 0.0 {
            let elapsed = self.start_time.elapsed().as_secs_f64();
            let total_estimated = elapsed / (self.progress as f64 / 100.0);
            self.eta_seconds = Some(total_estimated - elapsed);
        }
    }

    /// Get elapsed time in seconds
    pub fn elapsed_seconds(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
}

/// Batch progress tracker
#[derive(Debug, Clone)]
pub struct BatchProgress {
    /// Total number of books
    total_books: usize,
    /// Number of completed books
    completed: Arc<AtomicUsize>,
    /// Number of failed books
    failed: Arc<AtomicUsize>,
    /// Total bytes processed
    bytes_processed: Arc<AtomicU64>,
    /// Start time
    start_time: Instant,
}

impl BatchProgress {
    /// Create a new batch progress tracker
    pub fn new(total_books: usize) -> Self {
        Self {
            total_books,
            completed: Arc::new(AtomicUsize::new(0)),
            failed: Arc::new(AtomicUsize::new(0)),
            bytes_processed: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
        }
    }

    /// Mark a book as completed
    pub fn mark_completed(&self) {
        self.completed.fetch_add(1, Ordering::Relaxed);
    }

    /// Mark a book as failed
    pub fn mark_failed(&self) {
        self.failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Add processed bytes
    pub fn add_bytes(&self, bytes: u64) {
        self.bytes_processed.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Get number of completed books
    pub fn completed_count(&self) -> usize {
        self.completed.load(Ordering::Relaxed)
    }

    /// Get number of failed books
    pub fn failed_count(&self) -> usize {
        self.failed.load(Ordering::Relaxed)
    }

    /// Get total processed bytes
    pub fn total_bytes(&self) -> u64 {
        self.bytes_processed.load(Ordering::Relaxed)
    }

    /// Get total books
    pub fn total_books(&self) -> usize {
        self.total_books
    }

    /// Get overall progress percentage (0-100)
    pub fn overall_progress(&self) -> f32 {
        if self.total_books == 0 {
            return 0.0;
        }

        let processed = self.completed_count() + self.failed_count();
        (processed as f32 / self.total_books as f32) * 100.0
    }

    /// Calculate ETA for remaining books
    pub fn eta_seconds(&self) -> Option<f64> {
        let completed = self.completed_count();
        if completed == 0 {
            return None;
        }

        let elapsed = self.start_time.elapsed().as_secs_f64();
        let avg_time_per_book = elapsed / completed as f64;
        let remaining_books = self.total_books - completed - self.failed_count();

        if remaining_books > 0 {
            Some(avg_time_per_book * remaining_books as f64)
        } else {
            Some(0.0)
        }
    }

    /// Get elapsed time in seconds
    pub fn elapsed_seconds(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Format ETA as human-readable string
    pub fn format_eta(&self) -> String {
        match self.eta_seconds() {
            Some(seconds) if seconds > 0.0 => {
                let hours = (seconds / 3600.0) as u64;
                let minutes = ((seconds % 3600.0) / 60.0) as u64;
                let secs = (seconds % 60.0) as u64;

                if hours > 0 {
                    format!("{}h {:02}m {:02}s", hours, minutes, secs)
                } else if minutes > 0 {
                    format!("{}m {:02}s", minutes, secs)
                } else {
                    format!("{}s", secs)
                }
            }
            _ => "calculating...".to_string(),
        }
    }

    /// Format elapsed time as human-readable string
    pub fn format_elapsed(&self) -> String {
        let seconds = self.elapsed_seconds();
        let hours = (seconds / 3600.0) as u64;
        let minutes = ((seconds % 3600.0) / 60.0) as u64;
        let secs = (seconds % 60.0) as u64;

        if hours > 0 {
            format!("{}h {:02}m {:02}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {:02}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }

    /// Check if batch is complete
    pub fn is_complete(&self) -> bool {
        self.completed_count() + self.failed_count() >= self.total_books
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_processing_stage_name() {
        assert_eq!(ProcessingStage::Scanning.name(), "Scanning");
        assert_eq!(ProcessingStage::Analyzing.name(), "Analyzing");
        assert_eq!(ProcessingStage::Complete.name(), "Complete");
    }

    #[test]
    fn test_book_progress() {
        let mut progress = BookProgress::new("Test Book".to_string());
        assert_eq!(progress.name, "Test Book");
        assert_eq!(progress.stage, ProcessingStage::Scanning);
        assert_eq!(progress.progress, 0.0);

        progress.set_stage(ProcessingStage::Processing);
        assert_eq!(progress.stage, ProcessingStage::Processing);

        progress.set_progress(50.0);
        assert_eq!(progress.progress, 50.0);

        // Test clamping
        progress.set_progress(150.0);
        assert_eq!(progress.progress, 100.0);
    }

    #[test]
    fn test_batch_progress() {
        let progress = BatchProgress::new(10);
        assert_eq!(progress.total_books(), 10);
        assert_eq!(progress.completed_count(), 0);
        assert_eq!(progress.failed_count(), 0);
        assert_eq!(progress.overall_progress(), 0.0);

        progress.mark_completed();
        assert_eq!(progress.completed_count(), 1);
        assert_eq!(progress.overall_progress(), 10.0);

        progress.mark_failed();
        assert_eq!(progress.failed_count(), 1);
        assert_eq!(progress.overall_progress(), 20.0);
    }

    #[test]
    fn test_batch_progress_bytes() {
        let progress = BatchProgress::new(5);
        assert_eq!(progress.total_bytes(), 0);

        progress.add_bytes(1024);
        assert_eq!(progress.total_bytes(), 1024);

        progress.add_bytes(2048);
        assert_eq!(progress.total_bytes(), 3072);
    }

    #[test]
    fn test_batch_progress_eta() {
        let progress = BatchProgress::new(10);

        // No ETA with 0 completed
        assert!(progress.eta_seconds().is_none());

        // Sleep a bit to get some elapsed time
        thread::sleep(Duration::from_millis(100));

        // Mark one complete
        progress.mark_completed();

        // Should have an ETA now
        assert!(progress.eta_seconds().is_some());
    }

    #[test]
    fn test_batch_progress_is_complete() {
        let progress = BatchProgress::new(3);
        assert!(!progress.is_complete());

        progress.mark_completed();
        progress.mark_completed();
        assert!(!progress.is_complete());

        progress.mark_completed();
        assert!(progress.is_complete());
    }

    #[test]
    fn test_format_eta() {
        let progress = BatchProgress::new(1);
        let eta = progress.format_eta();
        assert_eq!(eta, "calculating...");
    }

    #[test]
    fn test_format_elapsed() {
        let progress = BatchProgress::new(1);
        thread::sleep(Duration::from_millis(100));
        let elapsed = progress.format_elapsed();
        assert!(elapsed.ends_with('s'));
    }
}
