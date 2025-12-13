//! Audio track analyzer

use crate::audio::{extract_metadata, FFmpeg};
use crate::models::{BookFolder, Track};
use anyhow::Result;
use futures::stream::{self, StreamExt};

/// Analyzer for audio tracks
pub struct Analyzer {
    ffmpeg: FFmpeg,
    parallel_workers: usize,
}

impl Analyzer {
    /// Create a new analyzer
    pub fn new() -> Result<Self> {
        Ok(Self {
            ffmpeg: FFmpeg::new()?,
            parallel_workers: 8,
        })
    }

    /// Create analyzer with custom parallel workers
    pub fn with_workers(workers: usize) -> Result<Self> {
        Ok(Self {
            ffmpeg: FFmpeg::new()?,
            parallel_workers: workers.clamp(1, 16),
        })
    }

    /// Analyze all MP3 files in a book folder
    pub async fn analyze_book_folder(&self, book_folder: &mut BookFolder) -> Result<()> {
        // Analyze all MP3 files in parallel
        let results = stream::iter(&book_folder.mp3_files)
            .map(|mp3_file| async {
                // Probe audio file
                let quality = self.ffmpeg.probe_audio_file(mp3_file).await?;

                // Create track
                let mut track = Track::new(mp3_file.clone(), quality);

                // Extract metadata
                if let Err(e) = extract_metadata(&mut track) {
                    tracing::warn!(
                        "Failed to extract metadata from {}: {}",
                        mp3_file.display(),
                        e
                    );
                }

                Ok::<Track, anyhow::Error>(track)
            })
            .buffer_unordered(self.parallel_workers)
            .collect::<Vec<_>>()
            .await;

        // Collect successful tracks
        let mut tracks = Vec::new();
        for result in results {
            match result {
                Ok(track) => tracks.push(track),
                Err(e) => {
                    tracing::error!("Failed to analyze track: {}", e);
                    return Err(e);
                }
            }
        }

        // Sort tracks by filename (they should already be sorted from scanner)
        // This is just to ensure consistency
        tracks.sort_by(|a, b| a.file_path.cmp(&b.file_path));

        book_folder.tracks = tracks;

        Ok(())
    }

    /// Get total duration of book folder in seconds
    pub fn get_total_duration(&self, book_folder: &BookFolder) -> f64 {
        book_folder.get_total_duration()
    }

    /// Check if book can use copy mode (all tracks compatible)
    pub fn can_use_copy_mode(&self, book_folder: &BookFolder) -> bool {
        book_folder.can_use_concat_copy()
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new().expect("Failed to create analyzer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::QualityProfile;
    use std::path::PathBuf;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = Analyzer::new();
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_analyzer_with_workers() {
        let analyzer = Analyzer::with_workers(4).unwrap();
        assert_eq!(analyzer.parallel_workers, 4);

        // Test clamping
        let analyzer = Analyzer::with_workers(20).unwrap();
        assert_eq!(analyzer.parallel_workers, 16);
    }

    #[test]
    fn test_can_use_copy_mode() {
        let analyzer = Analyzer::new().unwrap();
        let mut book = BookFolder::new(PathBuf::from("/test"));

        // Test with AAC/M4A files (can use copy mode)
        let aac_quality = QualityProfile::new(128, 44100, 2, "aac".to_string(), 3600.0).unwrap();
        book.tracks = vec![
            Track::new(PathBuf::from("1.m4a"), aac_quality.clone()),
            Track::new(PathBuf::from("2.m4a"), aac_quality),
        ];

        assert!(analyzer.can_use_copy_mode(&book));

        // Test with MP3 files (cannot use copy mode - must transcode)
        let mp3_quality = QualityProfile::new(128, 44100, 2, "mp3".to_string(), 3600.0).unwrap();
        book.tracks = vec![
            Track::new(PathBuf::from("1.mp3"), mp3_quality.clone()),
            Track::new(PathBuf::from("2.mp3"), mp3_quality),
        ];

        assert!(!analyzer.can_use_copy_mode(&book));
    }
}
