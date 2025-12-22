//! Single book processor

use crate::audio::{
    generate_chapters_from_files, inject_chapters_mp4box, inject_metadata_atomicparsley,
    parse_cue_file, write_mp4box_chapters, AacEncoder, FFmpeg,
};
use crate::models::{BookFolder, ProcessingResult};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

/// Processor for converting a single audiobook
pub struct Processor {
    ffmpeg: FFmpeg,
    keep_temp: bool,
    encoder: AacEncoder,
    enable_parallel_encoding: bool,
    max_concurrent_files: usize,
    quality_preset: Option<String>,
}

impl Processor {
    /// Create a new processor
    pub fn new() -> Result<Self> {
        Ok(Self {
            ffmpeg: FFmpeg::new()?,
            keep_temp: false,
            encoder: crate::audio::get_encoder(),
            enable_parallel_encoding: true,
            max_concurrent_files: 8,
            quality_preset: None,
        })
    }

    /// Create processor with options
    pub fn with_options(
        keep_temp: bool,
        encoder: AacEncoder,
        enable_parallel_encoding: bool,
        max_concurrent_files: usize,
        quality_preset: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            ffmpeg: FFmpeg::new()?,
            keep_temp,
            encoder,
            enable_parallel_encoding,
            max_concurrent_files: max_concurrent_files.clamp(1, 32),
            quality_preset,
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

        tracing::info!("=== Starting book processing: {} ===", book_folder.name);

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

        // Get quality profile (auto-detected from source)
        let mut quality = book_folder
            .get_best_quality_profile(true)
            .context("No tracks found")?
            .clone();

        // Apply quality preset override if specified
        if let Some(ref preset) = self.quality_preset {
            quality = quality.apply_preset(Some(preset.as_str()));
            tracing::info!("Applying quality preset '{}': {}", preset, quality);
        }

        if book_folder.tracks.len() == 1 {
            // Single file - just convert
            self.ffmpeg
                .convert_single_file(
                    &book_folder.tracks[0].file_path,
                    &output_path,
                    &quality,
                    use_copy,
                    self.encoder,
                )
                .await
                .context("Failed to convert audio file")?;
        } else if use_copy {
            // Copy mode - can concatenate directly
            let concat_file = temp_dir.join("concat.txt");
            let file_refs: Vec<&Path> = book_folder
                .tracks
                .iter()
                .map(|t| t.file_path.as_path())
                .collect();
            FFmpeg::create_concat_file(&file_refs, &concat_file)?;

            self.ffmpeg
                .concat_audio_files(
                    &concat_file,
                    &output_path,
                    &quality,
                    use_copy,
                    self.encoder,
                )
                .await
                .context("Failed to concatenate audio files")?;
        } else if self.enable_parallel_encoding && book_folder.tracks.len() > 1 {
            // Transcode mode - encode files in parallel with throttling
            let effective_limit = self.max_concurrent_files.min(book_folder.tracks.len());

            tracing::info!(
                "Using parallel encoding: {} files with max {} concurrent",
                book_folder.tracks.len(),
                effective_limit
            );

            // Create semaphore to limit concurrent file encodings
            let semaphore = Arc::new(Semaphore::new(effective_limit));

            // Step 1: Encode all files to AAC/M4A in parallel (with throttling)
            let mut encoded_files = Vec::new();
            let mut tasks = Vec::new();

            for (i, track) in book_folder.tracks.iter().enumerate() {
                let temp_output = temp_dir.join(format!("encoded_{:04}.m4a", i));
                encoded_files.push(temp_output.clone());

                tracing::info!(
                    "[{}/{}] Encoding: {} ({:.1} min)",
                    i + 1,
                    book_folder.tracks.len(),
                    track.file_path.file_name().unwrap().to_string_lossy(),
                    track.quality.duration / 60.0
                );

                let ffmpeg = self.ffmpeg.clone();
                let input = track.file_path.clone();
                let output = temp_output;
                let quality = quality.clone();
                let encoder = self.encoder;
                let sem = Arc::clone(&semaphore);

                // Spawn parallel encoding task with semaphore
                let task = tokio::spawn(async move {
                    // Acquire permit before encoding (blocks if limit reached)
                    let _permit = sem.acquire().await.unwrap();

                    ffmpeg
                        .convert_single_file(&input, &output, &quality, false, encoder)
                        .await
                    // Permit automatically released when _permit drops
                });

                tasks.push(task);
            }

            // Wait for all encoding tasks to complete
            for (i, task) in tasks.into_iter().enumerate() {
                match task.await {
                    Ok(Ok(())) => continue,
                    Ok(Err(e)) => {
                        return Err(e).context(format!("Track {} encoding failed", i));
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("Task {} panicked: {}", i, e));
                    }
                }
            }

            tracing::info!("All {} files encoded, now concatenating...", encoded_files.len());

            // Step 2: Concatenate the encoded files (fast, no re-encoding)
            let concat_file = temp_dir.join("concat.txt");
            let file_refs: Vec<&Path> = encoded_files.iter().map(|p| p.as_path()).collect();
            FFmpeg::create_concat_file(&file_refs, &concat_file)?;

            self.ffmpeg
                .concat_audio_files(
                    &concat_file,
                    &output_path,
                    &quality,
                    true, // use copy mode for concatenation
                    self.encoder,
                )
                .await
                .context("Failed to concatenate encoded files")?;
        } else {
            // Serial mode - concatenate and encode in one FFmpeg call (traditional method)
            tracing::info!("Using serial encoding (parallel encoding disabled in config)");

            let concat_file = temp_dir.join("concat.txt");
            let file_refs: Vec<&Path> = book_folder
                .tracks
                .iter()
                .map(|t| t.file_path.as_path())
                .collect();
            FFmpeg::create_concat_file(&file_refs, &concat_file)?;

            self.ffmpeg
                .concat_audio_files(
                    &concat_file,
                    &output_path,
                    &quality,
                    false, // transcode mode
                    self.encoder,
                )
                .await
                .context("Failed to concatenate audio files")?;
        }

        tracing::info!("Audio processing complete: {}", output_path.display());

        // Step 3: Generate and inject chapters
        let chapters = self.generate_chapters(book_folder, chapter_source)?;

        if !chapters.is_empty() {
            tracing::info!("Injecting {} chapters using MP4Box", chapters.len());

            let chapters_file = temp_dir.join("chapters.txt");
            write_mp4box_chapters(&chapters, &chapters_file)
                .context("Failed to write chapter file")?;

            inject_chapters_mp4box(&output_path, &chapters_file)
                .await
                .context("Failed to inject chapters")?;

            tracing::info!("✓ Chapter injection complete");
        }

        // Step 4: Inject metadata
        let title = book_folder.get_album_title();
        let artist = book_folder.get_album_artist();
        let year = book_folder.get_year();
        let genre = book_folder.get_genre();

        tracing::info!("Injecting metadata using AtomicParsley");
        tracing::debug!(
            "Metadata: title={:?}, artist={:?}",
            title,
            artist
        );

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

        tracing::info!("✓ Metadata injection complete");

        // Clean up temp directory
        if !self.keep_temp {
            if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
                tracing::warn!("Failed to remove temp directory: {}", e);
            }
        }

        // Calculate processing time
        let processing_time = start_time.elapsed().as_secs_f64();

        tracing::info!(
            "=== Completed: {} in {:.1}s ===",
            book_folder.name,
            processing_time
        );

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
        let processor = Processor::with_options(true, AacEncoder::AppleSilicon, true, 8, None).unwrap();
        assert!(processor.keep_temp);
        assert_eq!(processor.encoder, AacEncoder::AppleSilicon);
        assert_eq!(processor.max_concurrent_files, 8);
        assert_eq!(processor.quality_preset, None);

        let processor_with_preset = Processor::with_options(false, AacEncoder::Native, true, 4, Some("high".to_string())).unwrap();
        assert_eq!(processor_with_preset.quality_preset, Some("high".to_string()));
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
