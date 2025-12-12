//! Processing result model

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Result of processing a single audiobook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    /// Name of the book
    pub book_name: String,
    /// Whether processing was successful
    pub success: bool,
    /// Path to the output M4B file (if successful)
    pub output_path: Option<PathBuf>,
    /// Processing time in seconds
    pub processing_time: f64,
    /// Error message (if failed)
    pub error_message: Option<String>,
    /// Size of output file in bytes (if successful)
    pub output_size: Option<u64>,
    /// Whether copy mode was used (no re-encoding)
    pub used_copy_mode: bool,
}

impl ProcessingResult {
    /// Create a new processing result for a book
    pub fn new(book_name: String) -> Self {
        Self {
            book_name,
            success: false,
            output_path: None,
            processing_time: 0.0,
            error_message: None,
            output_size: None,
            used_copy_mode: false,
        }
    }

    /// Mark as successful with output path
    pub fn success(mut self, output_path: PathBuf, processing_time: f64, used_copy_mode: bool) -> Self {
        self.success = true;
        self.output_path = Some(output_path.clone());
        self.processing_time = processing_time;
        self.used_copy_mode = used_copy_mode;

        // Try to get file size
        if let Ok(metadata) = std::fs::metadata(&output_path) {
            self.output_size = Some(metadata.len());
        }

        self
    }

    /// Mark as failed with error message
    pub fn failure(mut self, error_message: String, processing_time: f64) -> Self {
        self.success = false;
        self.error_message = Some(error_message);
        self.processing_time = processing_time;
        self
    }

    /// Get output file size in MB
    pub fn output_size_mb(&self) -> Option<f64> {
        self.output_size.map(|size| size as f64 / (1024.0 * 1024.0))
    }
}

impl std::fmt::Display for ProcessingResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.success {
            write!(
                f,
                "✓ {} ({:.1}s, {})",
                self.book_name,
                self.processing_time,
                if self.used_copy_mode { "copy mode" } else { "transcode" }
            )?;
            if let Some(size_mb) = self.output_size_mb() {
                write!(f, " - {:.1} MB", size_mb)?;
            }
            Ok(())
        } else {
            write!(
                f,
                "✗ {} ({:.1}s) - {}",
                self.book_name,
                self.processing_time,
                self.error_message.as_deref().unwrap_or("Unknown error")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_success() {
        let result = ProcessingResult::new("Test Book".to_string())
            .success(PathBuf::from("/output/test.m4b"), 120.5, true);

        assert!(result.success);
        assert_eq!(result.processing_time, 120.5);
        assert!(result.used_copy_mode);
        assert!(result.output_path.is_some());
    }

    #[test]
    fn test_result_failure() {
        let result = ProcessingResult::new("Test Book".to_string())
            .failure("FFmpeg failed".to_string(), 45.2);

        assert!(!result.success);
        assert_eq!(result.processing_time, 45.2);
        assert_eq!(result.error_message, Some("FFmpeg failed".to_string()));
    }
}
