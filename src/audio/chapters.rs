//! Chapter generation and management

use anyhow::{Context, Result};
use regex::Regex;
use std::path::Path;

/// Represents a chapter in an audiobook
#[derive(Debug, Clone)]
pub struct Chapter {
    /// Chapter number (1-based)
    pub number: u32,
    /// Chapter title
    pub title: String,
    /// Start time in milliseconds
    pub start_time_ms: u64,
    /// End time in milliseconds
    pub end_time_ms: u64,
}

impl Chapter {
    /// Create a new chapter
    pub fn new(number: u32, title: String, start_time_ms: u64, end_time_ms: u64) -> Self {
        Self {
            number,
            title,
            start_time_ms,
            end_time_ms,
        }
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> u64 {
        self.end_time_ms - self.start_time_ms
    }

    /// Format as MP4Box chapter format
    pub fn to_mp4box_format(&self) -> String {
        let start_time = format_time_ms(self.start_time_ms);
        format!("CHAPTER{}={}\nCHAPTER{}NAME={}\n",
            self.number, start_time, self.number, self.title)
    }
}

/// Format milliseconds as HH:MM:SS.mmm
fn format_time_ms(ms: u64) -> String {
    let total_seconds = ms / 1000;
    let milliseconds = ms % 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds)
}

/// Generate chapters from file list (one file = one chapter)
pub fn generate_chapters_from_files(
    files: &[&Path],
    durations: &[f64], // Duration in seconds for each file
) -> Vec<Chapter> {
    tracing::debug!("Generating chapters from {} files", files.len());

    let mut chapters = Vec::new();
    let mut current_time_ms: u64 = 0;

    for (i, (file, &duration_secs)) in files.iter().zip(durations.iter()).enumerate() {
        let duration_ms = (duration_secs * 1000.0) as u64;
        let title = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&format!("Chapter {}", i + 1))
            .to_string();

        let chapter = Chapter::new(
            (i + 1) as u32,
            title,
            current_time_ms,
            current_time_ms + duration_ms,
        );

        chapters.push(chapter);
        current_time_ms += duration_ms;
    }

    tracing::debug!("Generated {} chapters", chapters.len());

    chapters
}

/// Parse CUE file and extract chapters
pub fn parse_cue_file(cue_path: &Path) -> Result<Vec<Chapter>> {
    let content = std::fs::read_to_string(cue_path)
        .context("Failed to read CUE file")?;

    let mut chapters = Vec::new();
    let mut current_chapter = 1u32;

    // Regex patterns for CUE file parsing
    let track_regex = Regex::new(r"^\s*TRACK\s+(\d+)\s+AUDIO").unwrap();
    let title_regex = Regex::new(r#"^\s*TITLE\s+"(.+)""#).unwrap();
    let index_regex = Regex::new(r"^\s*INDEX\s+01\s+(\d+):(\d+):(\d+)").unwrap();

    let mut current_title: Option<String> = None;
    let mut current_time_ms: Option<u64> = None;
    let mut last_chapter_start: u64 = 0;

    for line in content.lines() {
        // Check for TRACK line
        if track_regex.is_match(line) {
            // Save previous chapter if we have one
            if let (Some(title), Some(time_ms)) = (current_title.take(), current_time_ms.take()) {
                // We don't know the end time yet, will be filled when we see the next chapter
                chapters.push(Chapter::new(
                    current_chapter - 1,
                    title,
                    last_chapter_start,
                    time_ms,
                ));
                last_chapter_start = time_ms;
            }
        }

        // Check for TITLE line
        if let Some(caps) = title_regex.captures(line) {
            current_title = Some(caps[1].to_string());
        }

        // Check for INDEX line
        if let Some(caps) = index_regex.captures(line) {
            let minutes: u64 = caps[1].parse().unwrap_or(0);
            let seconds: u64 = caps[2].parse().unwrap_or(0);
            let frames: u64 = caps[3].parse().unwrap_or(0);

            // CUE uses frames (75 frames per second)
            let time_ms = (minutes * 60 * 1000) + (seconds * 1000) + ((frames * 1000) / 75);
            current_time_ms = Some(time_ms);
            current_chapter += 1;
        }
    }

    // Add the last chapter (end time will be set later based on total duration)
    if let (Some(title), Some(time_ms)) = (current_title, current_time_ms) {
        chapters.push(Chapter::new(
            current_chapter - 1,
            title,
            last_chapter_start,
            time_ms,
        ));
    }

    Ok(chapters)
}

/// Write chapters to MP4Box chapter file format
pub fn write_mp4box_chapters(chapters: &[Chapter], output_path: &Path) -> Result<()> {
    let mut content = String::new();

    for chapter in chapters {
        content.push_str(&chapter.to_mp4box_format());
    }

    std::fs::write(output_path, content)
        .context("Failed to write chapter file")?;

    Ok(())
}

/// Inject chapters into M4B file using MP4Box
pub async fn inject_chapters_mp4box(
    m4b_file: &Path,
    chapters_file: &Path,
) -> Result<()> {
    let output = tokio::process::Command::new("MP4Box")
        .args(&["-chap", &chapters_file.display().to_string()])
        .arg(m4b_file)
        .output()
        .await
        .context("Failed to execute MP4Box")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("MP4Box failed: {}", stderr);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_time_ms() {
        assert_eq!(format_time_ms(0), "00:00:00.000");
        assert_eq!(format_time_ms(1000), "00:00:01.000");
        assert_eq!(format_time_ms(60000), "00:01:00.000");
        assert_eq!(format_time_ms(3661500), "01:01:01.500");
    }

    #[test]
    fn test_chapter_creation() {
        let chapter = Chapter::new(1, "Introduction".to_string(), 0, 60000);
        assert_eq!(chapter.number, 1);
        assert_eq!(chapter.title, "Introduction");
        assert_eq!(chapter.duration_ms(), 60000);
    }

    #[test]
    fn test_generate_chapters_from_files() {
        let files = vec![
            Path::new("chapter01.mp3"),
            Path::new("chapter02.mp3"),
            Path::new("chapter03.mp3"),
        ];
        let durations = vec![120.5, 180.3, 95.7]; // seconds

        let chapters = generate_chapters_from_files(&files, &durations);

        assert_eq!(chapters.len(), 3);
        assert_eq!(chapters[0].title, "chapter01");
        assert_eq!(chapters[0].start_time_ms, 0);
        assert_eq!(chapters[1].start_time_ms, 120500);
        assert_eq!(chapters[2].start_time_ms, 300800); // 120.5 + 180.3 = 300.8s
    }

    #[test]
    fn test_chapter_mp4box_format() {
        let chapter = Chapter::new(1, "Test Chapter".to_string(), 0, 60000);
        let formatted = chapter.to_mp4box_format();

        assert!(formatted.contains("CHAPTER1=00:00:00.000"));
        assert!(formatted.contains("CHAPTER1NAME=Test Chapter"));
    }
}
