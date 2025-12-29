//! Chapter import and merge strategies

use anyhow::{Context, Result};
use std::path::Path;
use crate::audio::Chapter;

/// Source of chapter data
#[derive(Debug, Clone)]
pub enum ChapterSource {
    /// Fetch from Audnex API by ASIN
    Audnex { asin: String },
    /// Parse from text file
    TextFile { path: std::path::PathBuf },
    /// Extract from EPUB file
    Epub { path: std::path::PathBuf },
    /// Use existing chapters from M4B
    Existing,
}

/// Strategy for merging new chapters with existing ones
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChapterMergeStrategy {
    /// Keep existing timestamps, only update names
    KeepTimestamps,
    /// Replace both timestamps and names entirely
    ReplaceAll,
    /// Skip update if counts don't match
    SkipOnMismatch,
    /// Interactively ask user for each file
    Interactive,
}

impl std::fmt::Display for ChapterMergeStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KeepTimestamps => write!(f, "Keep existing timestamps, update names only"),
            Self::ReplaceAll => write!(f, "Replace all chapters (timestamps + names)"),
            Self::SkipOnMismatch => write!(f, "Skip if chapter counts don't match"),
            Self::Interactive => write!(f, "Ask for each file"),
        }
    }
}

/// Result of comparing existing vs new chapters
#[derive(Debug)]
pub struct ChapterComparison {
    pub existing_count: usize,
    pub new_count: usize,
    pub matches: bool,
}

impl ChapterComparison {
    pub fn new(existing: &[Chapter], new: &[Chapter]) -> Self {
        Self {
            existing_count: existing.len(),
            new_count: new.len(),
            matches: existing.len() == new.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chapter_comparison() {
        let existing = vec![
            Chapter::new(1, "Ch1".to_string(), 0, 1000),
            Chapter::new(2, "Ch2".to_string(), 1000, 2000),
        ];

        let new_matching = vec![
            Chapter::new(1, "Chapter One".to_string(), 0, 1000),
            Chapter::new(2, "Chapter Two".to_string(), 1000, 2000),
        ];

        let new_different = vec![
            Chapter::new(1, "Chapter One".to_string(), 0, 1000),
        ];

        let comp1 = ChapterComparison::new(&existing, &new_matching);
        assert!(comp1.matches);
        assert_eq!(comp1.existing_count, 2);

        let comp2 = ChapterComparison::new(&existing, &new_different);
        assert!(!comp2.matches);
    }

    #[test]
    fn test_merge_strategy_display() {
        assert_eq!(
            ChapterMergeStrategy::KeepTimestamps.to_string(),
            "Keep existing timestamps, update names only"
        );
    }
}
