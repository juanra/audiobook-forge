//! Integration tests for M4B merge functionality

use audiobook_forge::audio::{merge_chapter_lists, Chapter};
use audiobook_forge::utils::{detect_merge_pattern, natural_sort, MergePatternType};
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
    let files: Vec<&Path> = vec![Path::new("Book One.m4b"), Path::new("Different Book.m4b")];

    let result = detect_merge_pattern(&files);

    assert!(!result.pattern_detected);
}

#[test]
fn test_sort_part_suffix_filenames() {
    let mut files = vec![
        PathBuf::from("Book Part 3.m4b"),
        PathBuf::from("Book Part 1.m4b"),
        PathBuf::from("Book Part 2.m4b"),
    ];

    natural_sort(&mut files);

    assert_eq!(
        files
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect::<Vec<_>>(),
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

    let merged = merge_chapter_lists(&[(part1_chapters, 900_000), (part2_chapters, 900_000)]);

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

    let merged = merge_chapter_lists(&[(file1, 500_000), (file2, 400_000), (file3, 600_000)]);

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
fn test_chapter_offsets_use_source_duration_not_last_chapter_end() {
    let part1 = vec![
        Chapter::new(1, "Part 1 Chapter 1".to_string(), 0, 4_000),
        Chapter::new(2, "Part 1 Chapter 2".to_string(), 4_000, 9_000),
    ];
    let part2 = vec![Chapter::new(1, "Part 2 Chapter 1".to_string(), 0, 5_000)];

    let merged = merge_chapter_lists(&[(part1, 10_000), (part2, 5_000)]);

    assert_eq!(merged[2].start_time_ms, 10_000);
    assert_eq!(merged[2].end_time_ms, 15_000);
}

#[test]
fn test_pt_pattern_variation() {
    let files: Vec<&Path> = vec![Path::new("Story Pt 1.m4b"), Path::new("Story Pt 2.m4b")];

    let result = detect_merge_pattern(&files);

    assert!(result.pattern_detected);
    assert_eq!(result.pattern_type, Some(MergePatternType::Part));
}

#[test]
fn test_disk_pattern_variation() {
    let files: Vec<&Path> = vec![Path::new("Novel Disk 1.m4b"), Path::new("Novel Disk 2.m4b")];

    let result = detect_merge_pattern(&files);

    assert!(result.pattern_detected);
    assert_eq!(result.pattern_type, Some(MergePatternType::Disc));
}

#[test]
fn test_single_file_no_pattern() {
    let files: Vec<&Path> = vec![Path::new("Single Book.m4b")];

    let result = detect_merge_pattern(&files);

    assert!(!result.pattern_detected);
}

/// Regression test for issue #31: M4B files whose track number is a *prefix*
/// (e.g. "001 Author (Year) Title.m4b") were left in raw `read_dir` order,
/// because the old suffix-anchored regex matched nothing and every file
/// collapsed to sort key 0, making the stable sort a silent no-op.
#[test]
fn test_sort_numeric_prefix_filenames() {
    // Deliberately shuffled, mirroring the scrambled order from issue #31.
    let mut files = vec![
        PathBuf::from("/books/051 Stephen Fry (2020) Troy.m4b"),
        PathBuf::from("/books/024 Stephen Fry (2020) Troy.m4b"),
        PathBuf::from("/books/002 Stephen Fry (2020) Troy.m4b"),
        PathBuf::from("/books/010 Stephen Fry (2020) Troy.m4b"),
        PathBuf::from("/books/001 Stephen Fry (2020) Troy.m4b"),
        PathBuf::from("/books/070 Stephen Fry (2020) Troy.m4b"),
    ];

    natural_sort(&mut files);

    let names: Vec<&str> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();

    assert_eq!(
        names,
        vec![
            "001 Stephen Fry (2020) Troy.m4b",
            "002 Stephen Fry (2020) Troy.m4b",
            "010 Stephen Fry (2020) Troy.m4b",
            "024 Stephen Fry (2020) Troy.m4b",
            "051 Stephen Fry (2020) Troy.m4b",
            "070 Stephen Fry (2020) Troy.m4b",
        ]
    );
}

/// Natural ordering must not regress to lexicographic for unpadded numbers.
#[test]
fn test_sort_unpadded_numeric_prefix() {
    let mut files = vec![
        PathBuf::from("10 Chapter.m4b"),
        PathBuf::from("2 Chapter.m4b"),
        PathBuf::from("1 Chapter.m4b"),
    ];

    natural_sort(&mut files);

    let names: Vec<&str> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();

    assert_eq!(
        names,
        vec!["1 Chapter.m4b", "2 Chapter.m4b", "10 Chapter.m4b"]
    );
}

/// End-to-end check for issue #31: the scanner must hand the merger files in
/// playback order regardless of the order `read_dir` returns them in.
#[test]
fn test_scanner_orders_numeric_prefix_m4b_files() {
    use audiobook_forge::core::Scanner;

    let tmp = std::env::temp_dir().join(format!("af-issue31-{}", std::process::id()));
    let book = tmp.join("03 - Troy");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&book).unwrap();

    // Created in the scrambled order seen in the issue report.
    for n in ["051", "024", "002", "010", "001", "070", "007"] {
        std::fs::write(book.join(format!("{n} Stephen Fry (2020) Troy.m4b")), b"").unwrap();
    }

    let scanner = Scanner::new();
    let found = scanner.scan_single_directory(&book).unwrap();

    let order: Vec<String> = found
        .m4b_files
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap()[..3].to_string())
        .collect();

    assert_eq!(order, vec!["001", "002", "007", "010", "024", "051", "070"]);

    let _ = std::fs::remove_dir_all(&tmp);
}
