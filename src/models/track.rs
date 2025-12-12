//! Audio track model

use super::QualityProfile;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a single audio track in an audiobook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Path to the audio file
    pub file_path: PathBuf,
    /// Quality profile of this track
    pub quality: QualityProfile,
    /// Track title (from metadata or filename)
    pub title: Option<String>,
    /// Track number
    pub track_number: Option<u32>,
    /// Album/book title
    pub album: Option<String>,
    /// Artist/author
    pub artist: Option<String>,
    /// Album artist
    pub album_artist: Option<String>,
    /// Year
    pub year: Option<u32>,
    /// Genre
    pub genre: Option<String>,
    /// Comment
    pub comment: Option<String>,
}

impl Track {
    /// Create a new track with required fields
    pub fn new(file_path: PathBuf, quality: QualityProfile) -> Self {
        Self {
            file_path,
            quality,
            title: None,
            track_number: None,
            album: None,
            artist: None,
            album_artist: None,
            year: None,
            genre: None,
            comment: None,
        }
    }

    /// Get the filename without extension
    pub fn get_filename_stem(&self) -> String {
        self.file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Get the file extension
    pub fn get_extension(&self) -> Option<String> {
        self.file_path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
    }

    /// Check if this is an MP3 file
    pub fn is_mp3(&self) -> bool {
        matches!(self.get_extension().as_deref(), Some("mp3"))
    }

    /// Check if this is an M4A/M4B file
    pub fn is_m4a(&self) -> bool {
        matches!(self.get_extension().as_deref(), Some("m4a" | "m4b"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_creation() {
        let quality = QualityProfile::new(128, 44100, 2, "mp3".to_string(), 3600.0).unwrap();
        let track = Track::new(PathBuf::from("/path/to/track.mp3"), quality);

        assert_eq!(track.get_filename_stem(), "track");
        assert_eq!(track.get_extension(), Some("mp3".to_string()));
        assert!(track.is_mp3());
        assert!(!track.is_m4a());
    }

    #[test]
    fn test_track_extensions() {
        let quality = QualityProfile::new(128, 44100, 2, "aac".to_string(), 3600.0).unwrap();
        let track_m4a = Track::new(PathBuf::from("/path/to/track.m4a"), quality.clone());
        let track_m4b = Track::new(PathBuf::from("/path/to/track.m4b"), quality);

        assert!(track_m4a.is_m4a());
        assert!(track_m4b.is_m4a());
        assert!(!track_m4a.is_mp3());
    }
}
