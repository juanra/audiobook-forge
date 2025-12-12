//! Single book processor

use crate::audio::{
    generate_chapters_from_files, inject_chapters_mp4box, inject_metadata_atomicparsley,
    parse_cue_file, write_mp4box_chapters, FFmpeg,
};
use crate::models::{BookFolder, ProcessingResult};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Processor for converting a single audiobook
pub struct Processor {
    ffmpeg: FFmpeg,
    keep_temp: bool,
    use_apple_silicon: bool,
}

impl Processor {
    /// Create a new processor
    pub fn new() -> Result<Self> {
        Ok(Self {
            ffmpeg: FFmpeg::new()?,
            keep_temp: false,
            use_apple_silicon: false,
        })
    }

    /// Create processor with options
    pub fn with_options(keep_temp: bool, use_apple_silicon: bool) -> Result<Self> {
        Ok(Self {
            ffmpeg: FFmpeg::new()?,
            keep_temp,
            use_apple_silicon,
        })
    }

    /// Process a single book folder
    pub async fn process_book(
        &self,
        book_folder: &BookFolder,
        output_dir: &Path,
        chapter_source: &str,
    ) -> Result<ProcessingResult> {
        let start_time = Instant::now();
        let result = ProcessingResult::new(book_folder.name.clone());

        // Create output directory if it doesn't exist
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)
                .context("Failed to create output directory")?;
        }

        // Determine output file path
        let output_filename = book_folder.get_output_filename();
        let output_path = output_dir.join(&output_filename);

        // Create temp directory
        let temp_dir = self.create_temp_dir(&book_folder.name)?;

        // Check if we can use copy mode
        let use_copy = book_folder.can_use_concat_copy();

        tracing::info!(
            "Processing {} - {} tracks, copy_mode={}",
            book_folder.name,
            book_folder.tracks.len(),
            use_copy
        );

        // Step 1: Create concat file
        let concat_file = temp_dir.join("concat.txt");
        let file_refs: Vec<&Path> = book_folder
            .tracks
            .iter()
            .map(|t| t.file_path.as_path())
            .collect();
        FFmpeg::create_concat_file(&file_refs, &concat_file)?;

        // Step 2: Concatenate audio files
        let quality = book_folder
            .get_best_quality_profile(true)
            .context("No tracks found")?;

        if book_folder.tracks.len() == 1 {
            // Single file - just convert
            self.ffmpeg
                .convert_single_file(
                    &book_folder.tracks[0].file_path,
                    &output_path,
                    quality,
                    use_copy,
                    self.use_apple_silicon,
                )
                .await
                .context("Failed to convert audio file")?;
        } else {
            // Multiple files - concatenate
            self.ffmpeg
                .concat_audio_files(
                    &concat_file,
                    &output_path,
                    quality,
                    use_copy,
                    self.use_apple_silicon,
                )
                .await
                .context("Failed to concatenate audio files")?;
        }

        tracing::info!("Audio processing complete: {}", output_path.display());

        // Step 3: Generate and inject chapters
        let chapters = self.generate_chapters(book_folder, chapter_source)?;

        if !chapters.is_empty() {
            let chapters_file = temp_dir.join("chapters.txt");
            write_mp4box_chapters(&chapters, &chapters_file)
                .context("Failed to write chapter file")?;

            inject_chapters_mp4box(&output_path, &chapters_file)
                .await
                .context("Failed to inject chapters")?;

            tracing::info!("Injected {} chapters", chapters.len());
        }

        // Step 4: Inject metadata
        let title = book_folder.get_album_title();
        let artist = book_folder.get_album_artist();
        let year = book_folder.get_year();
        let genre = book_folder.get_genre();

        inject_metadata_atomicparsley(
            &output_path,
            title.as_deref(),
            artist.as_deref(),
            title.as_deref(), // Use title as album
            year,
            genre.as_deref(),
            book_folder.cover_file.as_deref(),
        )
        .await
        .context("Failed to inject metadata")?;

        tracing::info!("Metadata injection complete");

        // Clean up temp directory
        if !self.keep_temp {
            if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
                tracing::warn!("Failed to remove temp directory: {}", e);
            }
        }

        // Calculate processing time
        let processing_time = start_time.elapsed().as_secs_f64();

        // Return success result
        Ok(result.success(output_path, processing_time, use_copy))
    }

    /// Generate chapters for the book
    fn generate_chapters(
        &self,
        book_folder: &BookFolder,
        chapter_source: &str,
    ) -> Result<Vec<crate::audio::Chapter>> {
        match chapter_source {
            "cue" => {
                // Use CUE file if available
                if let Some(ref cue_file) = book_folder.cue_file {
                    tracing::info!("Using CUE file for chapters: {}", cue_file.display());
                    return parse_cue_file(cue_file);
                }
                Ok(Vec::new())
            }
            "files" | "auto" => {
                // Generate chapters from files
                if book_folder.tracks.len() > 1 {
                    let files: Vec<&Path> = book_folder
                        .tracks
                        .iter()
                        .map(|t| t.file_path.as_path())
                        .collect();
                    let durations: Vec<f64> = book_folder
                        .tracks
                        .iter()
                        .map(|t| t.quality.duration)
                        .collect();

                    tracing::info!(
                        "Generating {} chapters from files",
                        book_folder.tracks.len()
                    );
                    Ok(generate_chapters_from_files(&files, &durations))
                } else {
                    // Single file - check for CUE
                    if let Some(ref cue_file) = book_folder.cue_file {
                        tracing::info!("Using CUE file for single-file book");
                        parse_cue_file(cue_file)
                    } else {
                        Ok(Vec::new())
                    }
                }
            }
            "none" => Ok(Vec::new()),
            _ => {
                tracing::warn!("Unknown chapter source: {}, using auto", chapter_source);
                self.generate_chapters(book_folder, "auto")
            }
        }
    }

    /// Create temporary directory for processing
    fn create_temp_dir(&self, book_name: &str) -> Result<PathBuf> {
        let temp_base = std::env::temp_dir();
        let sanitized_name = sanitize_filename::sanitize(book_name);
        let temp_dir = temp_base.join(format!("audiobook-forge-{}", sanitized_name));

        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir).ok();
        }

        std::fs::create_dir_all(&temp_dir).context("Failed to create temp directory")?;

        Ok(temp_dir)
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self::new().expect("Failed to create processor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = Processor::new();
        assert!(processor.is_ok());
    }

    #[test]
    fn test_processor_with_options() {
        let processor = Processor::with_options(true, true).unwrap();
        assert!(processor.keep_temp);
        assert!(processor.use_apple_silicon);
    }

    #[test]
    fn test_create_temp_dir() {
        let processor = Processor::new().unwrap();
        let temp_dir = processor.create_temp_dir("Test Book").unwrap();

        assert!(temp_dir.exists());
        assert!(temp_dir.to_string_lossy().contains("audiobook-forge"));

        // Clean up
        std::fs::remove_dir_all(temp_dir).ok();
    }
}
