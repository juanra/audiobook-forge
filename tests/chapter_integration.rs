//! Integration tests for chapter functionality

use audiobook_forge::audio::{Chapter, parse_text_chapters, merge_chapters, ChapterMergeStrategy, ChapterComparison};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_parse_simple_text_file() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "Prologue").unwrap();
    writeln!(temp, "Chapter 1: The Beginning").unwrap();
    writeln!(temp, "Chapter 2: The Journey").unwrap();
    writeln!(temp, "").unwrap(); // Empty line should be ignored
    writeln!(temp, "Epilogue").unwrap();
    temp.flush().unwrap();

    let chapters = parse_text_chapters(temp.path()).unwrap();

    assert_eq!(chapters.len(), 4);
    assert_eq!(chapters[0].title, "Prologue");
    assert_eq!(chapters[1].title, "Chapter 1: The Beginning");
    assert_eq!(chapters[2].title, "Chapter 2: The Journey");
    assert_eq!(chapters[3].title, "Epilogue");
}

#[test]
fn test_parse_timestamped_text_file() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "0:00:00 Prologue").unwrap();
    writeln!(temp, "0:05:30 Chapter 1").unwrap();
    writeln!(temp, "0:15:45 Chapter 2").unwrap();
    writeln!(temp, "1:30:00 Epilogue").unwrap();
    temp.flush().unwrap();

    let chapters = parse_text_chapters(temp.path()).unwrap();

    assert_eq!(chapters.len(), 4);
    assert_eq!(chapters[0].start_time_ms, 0);
    assert_eq!(chapters[1].start_time_ms, 330_000); // 5:30
    assert_eq!(chapters[2].start_time_ms, 945_000); // 15:45
    assert_eq!(chapters[3].start_time_ms, 5_400_000); // 1:30:00

    // Check end times are set correctly
    assert_eq!(chapters[0].end_time_ms, 330_000);
    assert_eq!(chapters[1].end_time_ms, 945_000);
    assert_eq!(chapters[2].end_time_ms, 5_400_000);
}

#[test]
fn test_parse_mp4box_format() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "CHAPTER1=00:00:00.000").unwrap();
    writeln!(temp, "CHAPTER1NAME=Prologue").unwrap();
    writeln!(temp, "CHAPTER2=00:05:30.500").unwrap();
    writeln!(temp, "CHAPTER2NAME=Chapter 1: The Beginning").unwrap();
    writeln!(temp, "CHAPTER3=00:15:45.250").unwrap();
    writeln!(temp, "CHAPTER3NAME=Chapter 2: The Journey").unwrap();
    temp.flush().unwrap();

    let chapters = parse_text_chapters(temp.path()).unwrap();

    assert_eq!(chapters.len(), 3);
    assert_eq!(chapters[0].title, "Prologue");
    assert_eq!(chapters[0].start_time_ms, 0);
    assert_eq!(chapters[1].title, "Chapter 1: The Beginning");
    assert_eq!(chapters[1].start_time_ms, 330_500);
    assert_eq!(chapters[2].title, "Chapter 2: The Journey");
    assert_eq!(chapters[2].start_time_ms, 945_250);
}

#[test]
fn test_merge_keep_timestamps_strategy() {
    let existing = vec![
        Chapter::new(1, "Chapter 1".to_string(), 0, 300_000),
        Chapter::new(2, "Chapter 2".to_string(), 300_000, 600_000),
        Chapter::new(3, "Chapter 3".to_string(), 600_000, 900_000),
    ];

    let new = vec![
        Chapter::new(1, "Prologue".to_string(), 0, 0),
        Chapter::new(2, "The Adventure Begins".to_string(), 0, 0),
        Chapter::new(3, "The Journey Continues".to_string(), 0, 0),
    ];

    let merged = merge_chapters(&existing, &new, ChapterMergeStrategy::KeepTimestamps).unwrap();

    assert_eq!(merged.len(), 3);
    // Names updated
    assert_eq!(merged[0].title, "Prologue");
    assert_eq!(merged[1].title, "The Adventure Begins");
    assert_eq!(merged[2].title, "The Journey Continues");
    // Timestamps preserved
    assert_eq!(merged[0].start_time_ms, 0);
    assert_eq!(merged[0].end_time_ms, 300_000);
    assert_eq!(merged[1].start_time_ms, 300_000);
    assert_eq!(merged[1].end_time_ms, 600_000);
    assert_eq!(merged[2].start_time_ms, 600_000);
    assert_eq!(merged[2].end_time_ms, 900_000);
}

#[test]
fn test_merge_replace_all_strategy() {
    let existing = vec![
        Chapter::new(1, "Chapter 1".to_string(), 0, 300_000),
        Chapter::new(2, "Chapter 2".to_string(), 300_000, 600_000),
    ];

    let new = vec![
        Chapter::new(1, "Prologue".to_string(), 0, 200_000),
        Chapter::new(2, "Chapter 1".to_string(), 200_000, 500_000),
        Chapter::new(3, "Chapter 2".to_string(), 500_000, 800_000),
    ];

    let merged = merge_chapters(&existing, &new, ChapterMergeStrategy::ReplaceAll).unwrap();

    assert_eq!(merged.len(), 3);
    assert_eq!(merged[0].title, "Prologue");
    assert_eq!(merged[0].start_time_ms, 0);
    assert_eq!(merged[0].end_time_ms, 200_000);
    assert_eq!(merged[2].title, "Chapter 2");
}

#[test]
fn test_merge_skip_on_mismatch_strategy() {
    let existing = vec![
        Chapter::new(1, "Chapter 1".to_string(), 0, 300_000),
        Chapter::new(2, "Chapter 2".to_string(), 300_000, 600_000),
    ];

    let new = vec![
        Chapter::new(1, "Single Chapter".to_string(), 0, 600_000),
    ];

    let result = merge_chapters(&existing, &new, ChapterMergeStrategy::SkipOnMismatch);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Chapter count mismatch"));
    assert!(err_msg.contains("existing has 2"));
    assert!(err_msg.contains("new has 1"));
}

#[test]
fn test_merge_with_extra_existing_chapters() {
    let existing = vec![
        Chapter::new(1, "Chapter 1".to_string(), 0, 300_000),
        Chapter::new(2, "Chapter 2".to_string(), 300_000, 600_000),
        Chapter::new(3, "Chapter 3".to_string(), 600_000, 900_000),
        Chapter::new(4, "Chapter 4".to_string(), 900_000, 1_200_000),
    ];

    let new = vec![
        Chapter::new(1, "Prologue".to_string(), 0, 0),
        Chapter::new(2, "Part One".to_string(), 0, 0),
    ];

    let merged = merge_chapters(&existing, &new, ChapterMergeStrategy::KeepTimestamps).unwrap();

    // Should update first 2, keep last 2
    assert_eq!(merged.len(), 4);
    assert_eq!(merged[0].title, "Prologue");
    assert_eq!(merged[1].title, "Part One");
    assert_eq!(merged[2].title, "Chapter 3");
    assert_eq!(merged[3].title, "Chapter 4");
}

#[test]
fn test_chapter_comparison() {
    let chapters_a = vec![
        Chapter::new(1, "Ch1".to_string(), 0, 1000),
        Chapter::new(2, "Ch2".to_string(), 1000, 2000),
    ];

    let chapters_b = vec![
        Chapter::new(1, "Chapter One".to_string(), 0, 1000),
        Chapter::new(2, "Chapter Two".to_string(), 1000, 2000),
    ];

    let chapters_c = vec![
        Chapter::new(1, "Only One".to_string(), 0, 2000),
    ];

    let comp1 = ChapterComparison::new(&chapters_a, &chapters_b);
    assert!(comp1.matches);
    assert_eq!(comp1.existing_count, 2);
    assert_eq!(comp1.new_count, 2);

    let comp2 = ChapterComparison::new(&chapters_a, &chapters_c);
    assert!(!comp2.matches);
    assert_eq!(comp2.existing_count, 2);
    assert_eq!(comp2.new_count, 1);
}
