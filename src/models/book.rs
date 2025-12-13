//! Audiobook folder model

use super::{QualityProfile, Track, AudibleMetadata};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Classification of audiobook folders based on their contents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookCase {
    /// Case A: Folder with multiple MP3 files (needs processing)
    A,
    /// Case B: Folder with single MP3 file (needs processing)
    B,
    /// Case C: Folder with existing M4B file (may skip or normalize)
    C,
    /// Case D: Unknown or invalid folder structure
    D,
}

impl BookCase {
    /// Convert case to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            BookCase::A => "A",
            BookCase::B => "B",
            BookCase::C => "C",
            BookCase::D => "D",
        }
    }
}

impl std::fmt::Display for BookCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Case {}", self.as_str())
    }
}

/// Represents an audiobook folder with its contents and metadata
#[derive(Debug, Clone)]
pub struct BookFolder {
    /// Path to the folder
    pub folder_path: PathBuf,
    /// Folder name (used as book title)
    pub name: String,
    /// Classification case
    pub case: BookCase,
    /// List of audio tracks found
    pub tracks: Vec<Track>,
    /// MP3 files found (before analysis)
    pub mp3_files: Vec<PathBuf>,
    /// M4B files found
    pub m4b_files: Vec<PathBuf>,
    /// Cover art file path
    pub cover_file: Option<PathBuf>,
    /// CUE file path (if present)
    pub cue_file: Option<PathBuf>,
    /// Audible metadata (if fetched)
    pub audible_metadata: Option<AudibleMetadata>,
    /// Detected ASIN from folder name or metadata
    pub detected_asin: Option<String>,
}

impl BookFolder {
    /// Create a new BookFolder from a path
    pub fn new(folder_path: PathBuf) -> Self {
        let name = folder_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Self {
            folder_path,
            name,
            case: BookCase::D,
            tracks: Vec::new(),
            mp3_files: Vec::new(),
            m4b_files: Vec::new(),
            cover_file: None,
            cue_file: None,
            audible_metadata: None,
            detected_asin: None,
        }
    }

    /// Classify the folder based on its contents
    pub fn classify(&mut self) {
        let mp3_count = self.mp3_files.len();
        let m4b_count = self.m4b_files.len();

        self.case = if m4b_count > 0 {
            BookCase::C
        } else if mp3_count > 1 {
            BookCase::A
        } else if mp3_count == 1 {
            BookCase::B
        } else {
            BookCase::D
        };
    }

    /// Get total duration of all tracks in seconds
    pub fn get_total_duration(&self) -> f64 {
        self.tracks.iter().map(|t| t.quality.duration).sum()
    }

    /// Get the best quality profile among all tracks
    pub fn get_best_quality_profile(&self, prefer_stereo: bool) -> Option<&QualityProfile> {
        if self.tracks.is_empty() {
            return None;
        }

        let mut best = &self.tracks[0].quality;
        for track in &self.tracks[1..] {
            if track.quality.is_better_than(best, prefer_stereo) {
                best = &track.quality;
            }
        }
        Some(best)
    }

    /// Check if all tracks can be concatenated without re-encoding (copy mode)
    pub fn can_use_concat_copy(&self) -> bool {
        if self.tracks.is_empty() {
            return false;
        }

        // MP3 codec cannot be copied into M4B container - must transcode to AAC
        let first_codec = self.tracks[0].quality.codec.to_lowercase();
        if first_codec == "mp3" || first_codec == "mp3float" {
            return false;
        }

        if self.tracks.len() <= 1 {
            return true;
        }

        let first = &self.tracks[0].quality;
        self.tracks[1..]
            .iter()
            .all(|t| first.is_compatible_for_concat(&t.quality))
    }

    /// Get output filename for the M4B file
    pub fn get_output_filename(&self) -> String {
        format!("{}.m4b", self.name)
    }

    /// Get estimated file size in bytes (rough estimate)
    pub fn estimate_output_size(&self) -> u64 {
        let duration = self.get_total_duration();
        if let Some(quality) = self.get_best_quality_profile(true) {
            // bitrate (kbps) * duration (s) * 1000 / 8 = bytes
            ((quality.bitrate as f64 * duration * 1000.0) / 8.0) as u64
        } else {
            0
        }
    }

    /// Check if folder is processable (Case A or B)
    pub fn is_processable(&self) -> bool {
        matches!(self.case, BookCase::A | BookCase::B)
    }

    /// Get album artist from tracks (first non-None value)
    pub fn get_album_artist(&self) -> Option<String> {
        self.tracks
            .iter()
            .find_map(|t| t.album_artist.clone().or_else(|| t.artist.clone()))
    }

    /// Get album title from tracks (first non-None value)
    pub fn get_album_title(&self) -> Option<String> {
        self.tracks
            .iter()
            .find_map(|t| t.album.clone())
            .or_else(|| Some(self.name.clone()))
    }

    /// Get year from tracks (first non-None value)
    pub fn get_year(&self) -> Option<u32> {
        self.tracks.iter().find_map(|t| t.year)
    }

    /// Get genre from tracks (first non-None value)
    pub fn get_genre(&self) -> Option<String> {
        self.tracks.iter().find_map(|t| t.genre.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_case_display() {
        assert_eq!(BookCase::A.to_string(), "Case A");
        assert_eq!(BookCase::B.to_string(), "Case B");
        assert_eq!(BookCase::C.to_string(), "Case C");
        assert_eq!(BookCase::D.to_string(), "Case D");
    }

    #[test]
    fn test_book_folder_creation() {
        let book = BookFolder::new(PathBuf::from("/path/to/My Book"));
        assert_eq!(book.name, "My Book");
        assert_eq!(book.case, BookCase::D);
    }

    #[test]
    fn test_book_folder_classification() {
        let mut book = BookFolder::new(PathBuf::from("/path/to/book"));

        // Case A: multiple MP3s
        book.mp3_files = vec![
            PathBuf::from("1.mp3"),
            PathBuf::from("2.mp3"),
        ];
        book.classify();
        assert_eq!(book.case, BookCase::A);

        // Case B: single MP3
        book.mp3_files = vec![PathBuf::from("1.mp3")];
        book.classify();
        assert_eq!(book.case, BookCase::B);

        // Case C: M4B present
        book.m4b_files = vec![PathBuf::from("book.m4b")];
        book.classify();
        assert_eq!(book.case, BookCase::C);

        // Case D: no audio files
        book.mp3_files.clear();
        book.m4b_files.clear();
        book.classify();
        assert_eq!(book.case, BookCase::D);
    }

    #[test]
    fn test_can_use_concat_copy() {
        let mut book = BookFolder::new(PathBuf::from("/path/to/book"));

        // Test with AAC/M4A files (can use concat copy)
        let quality1 = QualityProfile::new(128, 44100, 2, "aac".to_string(), 3600.0).unwrap();
        let quality2 = QualityProfile::new(128, 44100, 2, "aac".to_string(), 1800.0).unwrap();

        book.tracks = vec![
            Track::new(PathBuf::from("1.m4a"), quality1),
            Track::new(PathBuf::from("2.m4a"), quality2),
        ];

        assert!(book.can_use_concat_copy());

        // Test with MP3 files (cannot use concat copy - must transcode)
        let mp3_quality1 = QualityProfile::new(128, 44100, 2, "mp3".to_string(), 3600.0).unwrap();
        let mp3_quality2 = QualityProfile::new(128, 44100, 2, "mp3".to_string(), 1800.0).unwrap();

        book.tracks = vec![
            Track::new(PathBuf::from("1.mp3"), mp3_quality1),
            Track::new(PathBuf::from("2.mp3"), mp3_quality2),
        ];

        assert!(!book.can_use_concat_copy());
    }
}
