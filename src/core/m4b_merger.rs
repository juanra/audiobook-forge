//! M4B file merger for lossless concatenation

use crate::audio::{read_m4b_chapters, merge_chapter_lists, Chapter, FFmpeg};
use crate::audio::{write_mp4box_chapters, inject_chapters_mp4box, inject_metadata_atomicparsley};
use crate::models::BookFolder;
use crate::utils::sort_by_part_number;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Merger for combining multiple M4B files
pub struct M4bMerger {
    ffmpeg: FFmpeg,
    keep_temp: bool,
}

impl M4bMerger {
    /// Create a new M4B merger
    pub fn new() -> Result<Self> {
        Ok(Self {
            ffmpeg: FFmpeg::new()?,
            keep_temp: false,
        })
    }

    /// Create merger with options
    pub fn with_options(keep_temp: bool) -> Result<Self> {
        Ok(Self {
            ffmpeg: FFmpeg::new()?,
            keep_temp,
        })
    }

    /// Merge multiple M4B files into one
    pub async fn merge_m4b_files(
        &self,
        book_folder: &BookFolder,
        output_dir: &Path,
    ) -> Result<PathBuf> {
        let mut m4b_files = book_folder.m4b_files.clone();

        // Sort files by part number
        sort_by_part_number(&mut m4b_files);

        tracing::info!(
            "Merging {} M4B files for: {}",
            m4b_files.len(),
            book_folder.name
        );

        // Create temp directory
        let temp_dir = self.create_temp_dir(&book_folder.name)?;

        // Step 1: Extract chapters from all files
        tracing::info!("Extracting chapters from source files...");
        let mut all_chapters: Vec<Vec<Chapter>> = Vec::new();

        for m4b_file in &m4b_files {
            match read_m4b_chapters(m4b_file).await {
                Ok(chapters) => {
                    tracing::debug!(
                        "  {} chapters from: {}",
                        chapters.len(),
                        m4b_file.file_name().unwrap_or_default().to_string_lossy()
                    );
                    all_chapters.push(chapters);
                }
                Err(e) => {
                    tracing::warn!(
                        "Could not read chapters from {}: {}",
                        m4b_file.display(),
                        e
                    );
                    all_chapters.push(Vec::new());
                }
            }
        }

        // Merge chapter lists with adjusted timestamps
        let merged_chapters = merge_chapter_lists(&all_chapters);
        tracing::info!("Total merged chapters: {}", merged_chapters.len());

        // Step 2: Create concat file for FFmpeg
        let concat_file = temp_dir.join("concat.txt");
        let file_refs: Vec<&Path> = m4b_files.iter().map(|p| p.as_path()).collect();
        FFmpeg::create_concat_file(&file_refs, &concat_file)?;

        // Step 3: Concatenate audio losslessly
        let output_filename = book_folder.get_output_filename();
        let output_path = output_dir.join(&output_filename);

        tracing::info!("Concatenating audio (lossless copy mode)...");

        self.ffmpeg
            .concat_m4b_files(&concat_file, &output_path)
            .await
            .context("Failed to concatenate M4B files")?;

        // Step 4: Inject merged chapters
        if !merged_chapters.is_empty() {
            tracing::info!("Injecting {} merged chapters...", merged_chapters.len());

            let chapters_file = temp_dir.join("chapters.txt");
            write_mp4box_chapters(&merged_chapters, &chapters_file)?;

            inject_chapters_mp4box(&output_path, &chapters_file)
                .await
                .context("Failed to inject chapters")?;
        }

        // Step 5: Copy metadata from first file
        tracing::info!("Copying metadata from first source file...");
        self.copy_metadata_from_first(&m4b_files[0], &output_path, book_folder).await?;

        // Clean up
        if !self.keep_temp {
            if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
                tracing::warn!("Failed to remove temp directory: {}", e);
            }
        }

        tracing::info!("M4B merge complete: {}", output_path.display());

        Ok(output_path)
    }

    /// Copy metadata from first source file to output
    async fn copy_metadata_from_first(
        &self,
        source: &Path,
        output: &Path,
        book_folder: &BookFolder,
    ) -> Result<()> {
        // Extract metadata from first file using ffprobe
        let metadata = self.ffmpeg.probe_metadata(source).await?;

        // Use folder name as title if not in metadata
        let title = metadata.title.or_else(|| Some(book_folder.name.clone()));
        let artist = metadata.artist;
        let album = metadata.album.or_else(|| title.clone());
        let album_artist = metadata.album_artist;
        let year = metadata.year;
        let genre = metadata.genre;
        let composer = metadata.composer;
        let comment = metadata.comment;

        inject_metadata_atomicparsley(
            output,
            title.as_deref(),
            artist.as_deref(),
            album.as_deref(),
            album_artist.as_deref(),
            year,
            genre.as_deref(),
            composer.as_deref(),
            comment.as_deref(),
            book_folder.cover_file.as_deref(),
        )
        .await
        .context("Failed to inject metadata")?;

        Ok(())
    }

    /// Create temporary directory
    fn create_temp_dir(&self, book_name: &str) -> Result<PathBuf> {
        let temp_base = std::env::temp_dir();
        let sanitized_name = sanitize_filename::sanitize(book_name);
        let temp_dir = temp_base.join(format!("audiobook-forge-merge-{}", sanitized_name));

        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir).ok();
        }

        std::fs::create_dir_all(&temp_dir).context("Failed to create temp directory")?;

        Ok(temp_dir)
    }
}

impl Default for M4bMerger {
    fn default() -> Self {
        Self::new().expect("Failed to create M4B merger")
    }
}
