//! Integration tests for M4B merge functionality

use audiobook_forge::utils::{detect_merge_pattern, sort_by_part_number, MergePatternType};
use audiobook_forge::audio::{merge_chapter_lists, Chapter};
use std::path::{Path, PathBuf};

#[test]
fn test_part_pattern_detection() {
    let files: Vec<&Path> = vec![
        Path::new("The Great Book Part 1.m4b"),
        Path::new("The Great Book Part 2.m4b"),
        Path::new("The Great Book Part 3.m4b"),
    ];

    let result = detect_merge_pattern(&files);

    assert!(result.pattern_detected);
    assert_eq!(result.pattern_type, Some(MergePatternType::Part));
    assert_eq!(result.base_name, Some("The Great Book".to_string()));
}

#[test]
fn test_disc_pattern_detection() {
    let files: Vec<&Path> = vec![
        Path::new("Audiobook CD1.m4b"),
        Path::new("Audiobook CD2.m4b"),
    ];

    let result = detect_merge_pattern(&files);

    assert!(result.pattern_detected);
    assert_eq!(result.pattern_type, Some(MergePatternType::Disc));
}

#[test]
fn test_numeric_suffix_detection() {
    let files: Vec<&Path> = vec![
        Path::new("My Book 01.m4b"),
        Path::new("My Book 02.m4b"),
        Path::new("My Book 03.m4b"),
    ];

    let result = detect_merge_pattern(&files);

    assert!(result.pattern_detected);
    assert_eq!(result.pattern_type, Some(MergePatternType::NumericSuffix));
}

#[test]
fn test_unrelated_files_no_pattern() {
    let files: Vec<&Path> = vec![
        Path::new("Book One.m4b"),
        Path::new("Different Book.m4b"),
    ];

    let result = detect_merge_pattern(&files);

    assert!(!result.pattern_detected);
}

#[test]
fn test_sort_by_part_number() {
    let mut files = vec![
        PathBuf::from("Book Part 3.m4b"),
        PathBuf::from("Book Part 1.m4b"),
        PathBuf::from("Book Part 2.m4b"),
    ];

    sort_by_part_number(&mut files);

    assert_eq!(
        files.iter().map(|p| p.file_name().unwrap().to_str().unwrap()).collect::<Vec<_>>(),
        vec!["Book Part 1.m4b", "Book Part 2.m4b", "Book Part 3.m4b"]
    );
}

#[test]
fn test_chapter_merge_with_offsets() {
    let part1_chapters = vec![
        Chapter::new(1, "Prologue".to_string(), 0, 300_000),
        Chapter::new(2, "Chapter 1".to_string(), 300_000, 900_000),
    ];

    let part2_chapters = vec![
        Chapter::new(1, "Chapter 2".to_string(), 0, 600_000),
        Chapter::new(2, "Epilogue".to_string(), 600_000, 900_000),
    ];

    let merged = merge_chapter_lists(&[part1_chapters, part2_chapters]);

    assert_eq!(merged.len(), 4);

    // Part 1 chapters unchanged
    assert_eq!(merged[0].title, "Prologue");
    assert_eq!(merged[0].start_time_ms, 0);
    assert_eq!(merged[1].title, "Chapter 1");
    assert_eq!(merged[1].start_time_ms, 300_000);

    // Part 2 chapters offset by part 1 duration (900_000)
    assert_eq!(merged[2].title, "Chapter 2");
    assert_eq!(merged[2].start_time_ms, 900_000);
    assert_eq!(merged[3].title, "Epilogue");
    assert_eq!(merged[3].start_time_ms, 1_500_000);
}

#[test]
fn test_merge_synthesized_one_chapter_per_file() {
    // Simulates issue #15: incremental M4B files with no internal chapters. Each
    // file contributes a single synthesized chapter spanning its full duration.
    // Merging must yield one sequential chapter per file with cumulative offsets.
    let file1 = vec![Chapter::new(1, "001 Troy".to_string(), 0, 500_000)];
    let file2 = vec![Chapter::new(1, "002 Troy".to_string(), 0, 400_000)];
    let file3 = vec![Chapter::new(1, "003 Troy".to_string(), 0, 600_000)];

    let merged = merge_chapter_lists(&[file1, file2, file3]);

    assert_eq!(merged.len(), 3);

    // Sequential numbering across files.
    assert_eq!(merged[0].number, 1);
    assert_eq!(merged[1].number, 2);
    assert_eq!(merged[2].number, 3);

    // Titles preserved from each source file.
    assert_eq!(merged[0].title, "001 Troy");
    assert_eq!(merged[1].title, "002 Troy");
    assert_eq!(merged[2].title, "003 Troy");

    // Cumulative offsets: each chapter starts where the previous file ended.
    assert_eq!(merged[0].start_time_ms, 0);
    assert_eq!(merged[0].end_time_ms, 500_000);
    assert_eq!(merged[1].start_time_ms, 500_000);
    assert_eq!(merged[1].end_time_ms, 900_000);
    assert_eq!(merged[2].start_time_ms, 900_000);
    assert_eq!(merged[2].end_time_ms, 1_500_000);
}

#[test]
fn test_pt_pattern_variation() {
    let files: Vec<&Path> = vec![
        Path::new("Story Pt 1.m4b"),
        Path::new("Story Pt 2.m4b"),
    ];

    let result = detect_merge_pattern(&files);

    assert!(result.pattern_detected);
    assert_eq!(result.pattern_type, Some(MergePatternType::Part));
}

#[test]
fn test_disk_pattern_variation() {
    let files: Vec<&Path> = vec![
        Path::new("Novel Disk 1.m4b"),
        Path::new("Novel Disk 2.m4b"),
    ];

    let result = detect_merge_pattern(&files);

    assert!(result.pattern_detected);
    assert_eq!(result.pattern_type, Some(MergePatternType::Disc));
}

#[test]
fn test_single_file_no_pattern() {
    let files: Vec<&Path> = vec![
        Path::new("Single Book.m4b"),
    ];

    let result = detect_merge_pattern(&files);

    assert!(!result.pattern_detected);
}
