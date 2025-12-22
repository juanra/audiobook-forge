//! Directory scanner for discovering audiobook folders

use crate::models::{BookFolder, BookCase, Config};
use anyhow::{Context, Result};
use std::path::Path;
use walkdir::WalkDir;

/// Scanner for discovering audiobook folders in a directory tree
pub struct Scanner {
    /// Cover art filenames to search for
    cover_filenames: Vec<String>,
    /// Auto-extract embedded cover art
    auto_extract_cover: bool,
}

impl Scanner {
    /// Create a new scanner with default cover filenames
    pub fn new() -> Self {
        Self {
            cover_filenames: vec![
                "cover.jpg".to_string(),
                "folder.jpg".to_string(),
                "cover.png".to_string(),
                "folder.png".to_string(),
            ],
            auto_extract_cover: true,
        }
    }

    /// Create scanner with custom cover filenames
    pub fn with_cover_filenames(cover_filenames: Vec<String>) -> Self {
        Self {
            cover_filenames,
            auto_extract_cover: true,
        }
    }

    /// Create scanner from config
    pub fn from_config(config: &Config) -> Self {
        Self {
            cover_filenames: config.metadata.cover_filenames.clone(),
            auto_extract_cover: config.metadata.auto_extract_cover,
        }
    }

    /// Scan a directory for audiobook folders
    pub fn scan_directory(&self, root: &Path) -> Result<Vec<BookFolder>> {
        if !root.exists() {
            anyhow::bail!("Directory does not exist: {}", root.display());
        }

        if !root.is_dir() {
            anyhow::bail!("Path is not a directory: {}", root.display());
        }

        let mut book_folders = Vec::new();

        // Walk through directory tree, but only go 2 levels deep
        // (root → book folders → files)
        for entry in WalkDir::new(root)
            .max_depth(2)
            .min_depth(1)
            .into_iter()
            .filter_entry(|e| e.file_type().is_dir())
        {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            // Skip hidden directories
            if self.is_hidden(path) {
                continue;
            }

            // Check if this is a valid audiobook folder
            if let Some(book) = self.scan_folder(path)? {
                book_folders.push(book);
            }
        }

        Ok(book_folders)
    }

    /// Scan a single directory as an audiobook folder (for auto-detect mode)
    pub fn scan_single_directory(&self, path: &Path) -> Result<BookFolder> {
        if !path.exists() {
            anyhow::bail!("Directory does not exist: {}", path.display());
        }

        if !path.is_dir() {
            anyhow::bail!("Path is not a directory: {}", path.display());
        }

        // Scan the folder
        if let Some(book) = self.scan_folder(path)? {
            Ok(book)
        } else {
            anyhow::bail!("Current directory does not contain valid audiobook files");
        }
    }

    /// Scan a single folder and determine if it's an audiobook
    fn scan_folder(&self, path: &Path) -> Result<Option<BookFolder>> {
        let mut book = BookFolder::new(path.to_path_buf());

        // Find audio files
        for entry in std::fs::read_dir(path).context("Failed to read directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let file_path = entry.path();

            if !file_path.is_file() {
                continue;
            }

            let extension = file_path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase());

            match extension.as_deref() {
                Some("mp3") => {
                    book.mp3_files.push(file_path);
                }
                Some("m4b") => {
                    book.m4b_files.push(file_path);
                }
                Some("m4a") => {
                    // M4A files are treated like MP3s (can be converted)
                    book.mp3_files.push(file_path);
                }
                Some("cue") => {
                    book.cue_file = Some(file_path);
                }
                Some("jpg") | Some("png") | Some("jpeg") => {
                    // Check if this is a cover art file
                    if book.cover_file.is_none() {
                        if self.is_cover_art(&file_path) {
                            book.cover_file = Some(file_path);
                        }
                    }
                }
                _ => {}
            }
        }

        // Classify the book
        book.classify();

        // Only return if it's a valid audiobook folder (Cases A, B, or C)
        if matches!(book.case, BookCase::A | BookCase::B | BookCase::C) {
            // Sort MP3 files naturally
            crate::utils::natural_sort(&mut book.mp3_files);

            // Auto-extract embedded cover art if enabled and no standalone cover found
            if self.auto_extract_cover
                && book.cover_file.is_none()
                && !book.mp3_files.is_empty()
            {
                // Try extracting from first audio file (mp3_files includes both MP3 and M4A)
                let first_audio = book.mp3_files.first();

                if let Some(audio_file) = first_audio {
                    // Create temp file for extracted cover
                    let extracted_cover = path.join(".extracted_cover.jpg");

                    match crate::audio::extract_embedded_cover(audio_file, &extracted_cover) {
                        Ok(true) => {
                            tracing::info!(
                                "Extracted embedded cover from: {}",
                                audio_file.file_name().unwrap_or_default().to_string_lossy()
                            );
                            book.cover_file = Some(extracted_cover);
                        }
                        Ok(false) => {
                            tracing::debug!("No embedded cover found in first audio file");
                        }
                        Err(e) => {
                            tracing::warn!("Failed to extract embedded cover: {}", e);
                        }
                    }
                }
            }

            Ok(Some(book))
        } else {
            Ok(None)
        }
    }

    /// Check if a path is hidden (starts with .)
    fn is_hidden(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    }

    /// Check if a file is cover art based on filename
    fn is_cover_art(&self, path: &Path) -> bool {
        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            let filename_lower = filename.to_lowercase();
            self.cover_filenames
                .iter()
                .any(|cover| cover.to_lowercase() == filename_lower)
        } else {
            false
        }
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_scanner_creation() {
        let scanner = Scanner::new();
        assert_eq!(scanner.cover_filenames.len(), 4);
    }

    #[test]
    fn test_scanner_with_custom_covers() {
        let scanner = Scanner::with_cover_filenames(vec!["custom.jpg".to_string()]);
        assert_eq!(scanner.cover_filenames.len(), 1);
        assert_eq!(scanner.cover_filenames[0], "custom.jpg");
    }

    #[test]
    fn test_scan_empty_directory() {
        let dir = tempdir().unwrap();
        let scanner = Scanner::new();
        let books = scanner.scan_directory(dir.path()).unwrap();
        assert_eq!(books.len(), 0);
    }

    #[test]
    fn test_scan_directory_with_audiobook() {
        let dir = tempdir().unwrap();
        let book_dir = dir.path().join("Test Book");
        fs::create_dir(&book_dir).unwrap();

        // Create some MP3 files
        fs::write(book_dir.join("01.mp3"), b"fake mp3 data").unwrap();
        fs::write(book_dir.join("02.mp3"), b"fake mp3 data").unwrap();
        fs::write(book_dir.join("cover.jpg"), b"fake image data").unwrap();

        let scanner = Scanner::new();
        let books = scanner.scan_directory(dir.path()).unwrap();

        assert_eq!(books.len(), 1);
        assert_eq!(books[0].name, "Test Book");
        assert_eq!(books[0].case, BookCase::A); // Multiple MP3s
        assert_eq!(books[0].mp3_files.len(), 2);
        assert!(books[0].cover_file.is_some());
    }

    #[test]
    fn test_hidden_directory_skipped() {
        let dir = tempdir().unwrap();
        let hidden_dir = dir.path().join(".hidden");
        fs::create_dir(&hidden_dir).unwrap();
        fs::write(hidden_dir.join("01.mp3"), b"fake mp3 data").unwrap();

        let scanner = Scanner::new();
        let books = scanner.scan_directory(dir.path()).unwrap();

        // Hidden directory should be skipped
        assert_eq!(books.len(), 0);
    }
}
