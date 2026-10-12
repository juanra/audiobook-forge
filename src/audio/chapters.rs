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
    let content = std::fs::read_to_string(cue_path).context("Failed to read CUE file")?;

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

/// Escape a value for ffmpeg's `;FFMETADATA1` format.
///
/// The format is line-oriented with `=` as separator and `;`/`#` as comment
/// markers; all of them, plus the backslash itself, are backslash-escaped.
fn escape_ffmetadata(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '=' | ';' | '#' | '\\' => {
                escaped.push('\\');
                escaped.push(c);
            }
            '\r' | '\n' => escaped.push(' '),
            _ => escaped.push(c),
        }
    }
    escaped
}

/// Write chapters in ffmpeg's `;FFMETADATA1` format (millisecond timebase).
///
/// Adapted from the chapter export in PR #19 by AlexIllinois2.
pub fn write_ffmetadata(chapters: &[Chapter], path: &Path) -> Result<()> {
    let mut content = String::from(";FFMETADATA1\n");
    for chapter in chapters {
        content.push_str(&format!(
            "[CHAPTER]\nTIMEBASE=1/1000\nSTART={}\nEND={}\ntitle={}\n",
            chapter.start_time_ms,
            chapter.end_time_ms.max(chapter.start_time_ms),
            escape_ffmetadata(&chapter.title)
        ));
    }

    std::fs::write(path, content)
        .with_context(|| format!("Failed to write chapter file: {}", path.display()))
}

/// Replace the chapters of an M4B file, then check they all arrived.
///
/// Uses ffmpeg rather than MP4Box: `MP4Box -chap` writes only the Nero `chpl`
/// atom, whose one-byte count wrapped past 255 and silently dropped chapters
/// (issue #35). ffmpeg writes a QuickTime chapter track with every chapter,
/// plus a `chpl` capped at 255 for players that read only that. See
/// docs/adr/0001-chapter-injection-with-ffmpeg.md.
pub async fn inject_chapters(m4b_file: &Path, chapters: &[Chapter]) -> Result<()> {
    let directory = m4b_file
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let metadata_file = tempfile::Builder::new()
        .suffix(".ffmeta")
        .tempfile()
        .context("Failed to create chapter metadata file")?;
    write_ffmetadata(chapters, metadata_file.path())?;

    // Same directory as the target so the final rename cannot cross filesystems.
    let remuxed = tempfile::Builder::new()
        .prefix(".audiobook-forge-chapters-")
        .suffix(".m4b")
        .tempfile_in(directory)
        .context("Failed to create temporary M4B file")?;

    let output = tokio::process::Command::new("ffmpeg")
        .args(["-v", "error", "-y", "-i"])
        .arg(m4b_file)
        .args(["-f", "ffmetadata", "-i"])
        .arg(metadata_file.path())
        .args([
            "-map",
            "0",
            "-map_metadata",
            "0",
            "-map_chapters",
            "1",
            "-c",
            "copy",
            "-f",
            "mp4",
        ])
        .arg(remuxed.path())
        .output()
        .await
        .context("Failed to execute ffmpeg")?;

    if !output.status.success() {
        anyhow::bail!(
            "ffmpeg failed to inject chapters: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    verify_chapter_count(remuxed.path(), chapters.len()).await?;

    remuxed
        .persist(m4b_file)
        .with_context(|| format!("Failed to replace {}", m4b_file.display()))?;

    Ok(())
}

/// Fail when a finished file does not carry the chapters it was given.
///
/// A container write can drop chapters while every tool exits 0 (issue #35),
/// so the count is read back rather than trusted.
pub async fn verify_chapter_count(m4b_file: &Path, expected: usize) -> Result<()> {
    let actual = crate::audio::read_m4b_chapters(m4b_file)
        .await
        .context("Failed to read back chapters")?
        .len();

    if actual != expected {
        anyhow::bail!(
            "{} has {} chapters but {} were written",
            m4b_file.display(),
            actual,
            expected
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_write_ffmetadata_escapes_titles() {
        let chapters = vec![
            Chapter::new(1, "Intro".to_string(), 0, 1000),
            Chapter::new(2, "A=B; #1 \\ line\nbreak".to_string(), 1000, 2000),
        ];
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("chapters.ffmeta");

        write_ffmetadata(&chapters, &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();

        assert!(content.starts_with(";FFMETADATA1\n"));
        assert_eq!(content.matches("[CHAPTER]").count(), 2);
        assert!(content.contains("START=1000\nEND=2000\n"));
        assert!(content.contains("title=A\\=B\\; \\#1 \\\\ line break\n"));
    }

    #[tokio::test]
    async fn test_inject_chapters_keeps_more_than_255() {
        // Issue #35: MP4Box wrapped the chapter count at 256 (300 -> 44).
        let dir = tempfile::tempdir().unwrap();
        let m4b = dir.path().join("book.m4b");
        let status = std::process::Command::new("ffmpeg")
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "anullsrc=r=44100:cl=mono",
                "-t",
                "30",
            ])
            .args(["-c:a", "aac", "-b:a", "32k"])
            .arg(&m4b)
            .status()
            .expect("ffmpeg must be installed to run this test");
        assert!(status.success());

        let chapters: Vec<Chapter> = (0..300u64)
            .map(|i| {
                Chapter::new(
                    i as u32 + 1,
                    format!("Chapter {}", i + 1),
                    i * 100,
                    (i + 1) * 100,
                )
            })
            .collect();

        inject_chapters(&m4b, &chapters).await.unwrap();

        let written = crate::audio::read_m4b_chapters(&m4b).await.unwrap();
        assert_eq!(written.len(), 300);
        assert_eq!(written[299].title, "Chapter 300");
    }
}
