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

/// Supported text file formats for chapter import
#[derive(Debug, Clone, Copy)]
pub enum TextFormat {
    /// One title per line
    Simple,
    /// Timestamps + titles (e.g., "00:00:00 Prologue")
    Timestamped,
    /// MP4Box format (CHAPTER1=00:00:00\nCHAPTER1NAME=Title)
    Mp4Box,
}

/// Parse chapters from text file
pub fn parse_text_chapters(path: &Path) -> Result<Vec<Chapter>> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read chapter file")?;

    // Auto-detect format
    let format = detect_text_format(&content);

    match format {
        TextFormat::Simple => parse_simple_format(&content),
        TextFormat::Timestamped => parse_timestamped_format(&content),
        TextFormat::Mp4Box => parse_mp4box_format(&content),
    }
}

/// Detect text file format
fn detect_text_format(content: &str) -> TextFormat {
    use regex::Regex;

    lazy_static::lazy_static! {
        static ref MP4BOX_REGEX: Regex = Regex::new(r"CHAPTER\d+=\d{2}:\d{2}:\d{2}").unwrap();
        static ref TIMESTAMP_REGEX: Regex = Regex::new(r"^\d{1,2}:\d{2}:\d{2}").unwrap();
    }

    // Check for MP4Box format
    if MP4BOX_REGEX.is_match(content) {
        return TextFormat::Mp4Box;
    }

    // Check for timestamped format (first line)
    if let Some(first_line) = content.lines().next() {
        if TIMESTAMP_REGEX.is_match(first_line.trim()) {
            return TextFormat::Timestamped;
        }
    }

    // Default to simple
    TextFormat::Simple
}

/// Parse simple format (one title per line)
fn parse_simple_format(content: &str) -> Result<Vec<Chapter>> {
    let chapters: Vec<Chapter> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
        .map(|(i, line)| {
            Chapter::new(
                (i + 1) as u32,
                line.trim().to_string(),
                0, // No timestamps in simple format
                0,
            )
        })
        .collect();

    if chapters.is_empty() {
        anyhow::bail!("No chapters found in file");
    }

    Ok(chapters)
}

/// Parse timestamped format (HH:MM:SS Title)
fn parse_timestamped_format(content: &str) -> Result<Vec<Chapter>> {
    use regex::Regex;

    lazy_static::lazy_static! {
        static ref TIMESTAMP_REGEX: Regex =
            Regex::new(r"^(\d{1,2}):(\d{2}):(\d{2})\s*[-:]?\s*(.+)$").unwrap();
    }

    let mut chapters: Vec<Chapter> = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(caps) = TIMESTAMP_REGEX.captures(line) {
            let hours: u64 = caps[1].parse().context("Invalid hour")?;
            let minutes: u64 = caps[2].parse().context("Invalid minute")?;
            let seconds: u64 = caps[3].parse().context("Invalid second")?;
            let title = caps[4].trim().to_string();

            let start_ms = (hours * 3600 + minutes * 60 + seconds) * 1000;

            // Set end time for previous chapter
            if !chapters.is_empty() {
                let prev_idx = chapters.len() - 1;
                chapters[prev_idx].end_time_ms = start_ms;
            }

            chapters.push(Chapter::new(
                (i + 1) as u32,
                title,
                start_ms,
                0, // Will be set by next chapter or total duration
            ));
        } else {
            tracing::warn!("Skipping malformed line {}: {}", i + 1, line);
        }
    }

    if chapters.is_empty() {
        anyhow::bail!("No valid timestamped chapters found");
    }

    Ok(chapters)
}

/// Parse MP4Box format
fn parse_mp4box_format(content: &str) -> Result<Vec<Chapter>> {
    use regex::Regex;

    lazy_static::lazy_static! {
        static ref CHAPTER_REGEX: Regex =
            Regex::new(r"CHAPTER(\d+)=(\d{2}):(\d{2}):(\d{2})\.(\d{3})").unwrap();
        static ref NAME_REGEX: Regex =
            Regex::new(r"CHAPTER(\d+)NAME=(.+)").unwrap();
    }

    let mut chapter_times: std::collections::HashMap<u32, u64> = std::collections::HashMap::new();
    let mut chapter_names: std::collections::HashMap<u32, String> = std::collections::HashMap::new();

    for line in content.lines() {
        if let Some(caps) = CHAPTER_REGEX.captures(line) {
            let num: u32 = caps[1].parse().context("Invalid chapter number")?;
            let hours: u64 = caps[2].parse().context("Invalid hour")?;
            let minutes: u64 = caps[3].parse().context("Invalid minute")?;
            let seconds: u64 = caps[4].parse().context("Invalid second")?;
            let millis: u64 = caps[5].parse().context("Invalid millisecond")?;

            let start_ms = (hours * 3600 + minutes * 60 + seconds) * 1000 + millis;
            chapter_times.insert(num, start_ms);
        }

        if let Some(caps) = NAME_REGEX.captures(line) {
            let num: u32 = caps[1].parse().context("Invalid chapter number")?;
            let name = caps[2].trim().to_string();
            chapter_names.insert(num, name);
        }
    }

    if chapter_times.is_empty() {
        anyhow::bail!("No chapters found in MP4Box format");
    }

    // Build chapters
    let mut chapters = Vec::new();
    let mut numbers: Vec<u32> = chapter_times.keys().copied().collect();
    numbers.sort();

    for (i, &num) in numbers.iter().enumerate() {
        let start_ms = *chapter_times.get(&num).unwrap();
        let title = chapter_names
            .get(&num)
            .cloned()
            .unwrap_or_else(|| format!("Chapter {}", num));

        let end_ms = if i + 1 < numbers.len() {
            *chapter_times.get(&numbers[i + 1]).unwrap()
        } else {
            0 // Will be set later
        };

        chapters.push(Chapter::new(num, title, start_ms, end_ms));
    }

    Ok(chapters)
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

    #[test]
    fn test_detect_simple_format() {
        let content = "Prologue\nChapter 1\nChapter 2";
        assert!(matches!(detect_text_format(content), TextFormat::Simple));
    }

    #[test]
    fn test_detect_timestamped_format() {
        let content = "00:00:00 Prologue\n00:05:30 Chapter 1";
        assert!(matches!(detect_text_format(content), TextFormat::Timestamped));
    }

    #[test]
    fn test_detect_mp4box_format() {
        let content = "CHAPTER1=00:00:00.000\nCHAPTER1NAME=Prologue";
        assert!(matches!(detect_text_format(content), TextFormat::Mp4Box));
    }

    #[test]
    fn test_parse_simple_format() {
        let content = "Prologue\nChapter 1: The Beginning\nChapter 2: The Journey";
        let chapters = parse_simple_format(content).unwrap();

        assert_eq!(chapters.len(), 3);
        assert_eq!(chapters[0].title, "Prologue");
        assert_eq!(chapters[1].title, "Chapter 1: The Beginning");
        assert_eq!(chapters[2].title, "Chapter 2: The Journey");
    }

    #[test]
    fn test_parse_timestamped_format() {
        let content = "0:00:00 Prologue\n0:05:30 Chapter 1\n0:15:45 Chapter 2";
        let chapters = parse_timestamped_format(content).unwrap();

        assert_eq!(chapters.len(), 3);
        assert_eq!(chapters[0].start_time_ms, 0);
        assert_eq!(chapters[1].start_time_ms, 330_000); // 5:30
        assert_eq!(chapters[2].start_time_ms, 945_000); // 15:45
    }

    #[test]
    fn test_parse_mp4box_format() {
        let content = "CHAPTER1=00:00:00.000\nCHAPTER1NAME=Prologue\nCHAPTER2=00:05:30.500\nCHAPTER2NAME=Chapter 1";
        let chapters = parse_mp4box_format(content).unwrap();

        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "Prologue");
        assert_eq!(chapters[0].start_time_ms, 0);
        assert_eq!(chapters[1].title, "Chapter 1");
        assert_eq!(chapters[1].start_time_ms, 330_500);
    }
}
