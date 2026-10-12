//! M4B merge pattern detection for identifying related audiobook parts

use regex::Regex;
use std::path::Path;

/// Result of merge pattern analysis
#[derive(Debug, Clone)]
pub struct MergePatternResult {
    /// Whether a merge pattern was detected
    pub pattern_detected: bool,
    /// The base name (without part/disc indicators)
    pub base_name: Option<String>,
    /// The detected pattern type
    pub pattern_type: Option<MergePatternType>,
}

/// Types of merge patterns we recognize
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergePatternType {
    /// Part 1, Part 2, Pt 1, Pt. 1, Part1, Part2
    Part,
    /// Disc 1, Disc1, CD1, CD 1, Disk 1, Disk1
    Disc,
    /// Simple numeric suffix: Title 01.m4b, Title 02.m4b
    NumericSuffix,
}

/// Detect if a list of M4B files follow a merge pattern
pub fn detect_merge_pattern(files: &[&Path]) -> MergePatternResult {
    if files.len() < 2 {
        return MergePatternResult {
            pattern_detected: false,
            base_name: None,
            pattern_type: None,
        };
    }

    // Try each pattern type
    if let Some((base, pattern_type)) = try_detect_pattern(files) {
        return MergePatternResult {
            pattern_detected: true,
            base_name: Some(base),
            pattern_type: Some(pattern_type),
        };
    }

    MergePatternResult {
        pattern_detected: false,
        base_name: None,
        pattern_type: None,
    }
}

/// Try to detect a specific pattern type
fn try_detect_pattern(files: &[&Path]) -> Option<(String, MergePatternType)> {
    // Define patterns in order of specificity
    lazy_static::lazy_static! {
        // Part patterns: Part 1, Part1, Pt 1, Pt. 1
        static ref PART_REGEX: Regex = Regex::new(
            r"(?i)^(.+?)\s*(?:part|pt\.?)\s*(\d+)\.m4b$"
        ).unwrap();

        // Disc patterns: Disc 1, Disc1, CD1, CD 1, Disk 1
        static ref DISC_REGEX: Regex = Regex::new(
            r"(?i)^(.+?)\s*(?:disc|disk|cd)\s*(\d+)\.m4b$"
        ).unwrap();

        // Numeric suffix: Title 01.m4b, Title 1.m4b (must be at least 2 files with sequential numbers)
        static ref NUMERIC_REGEX: Regex = Regex::new(
            r"(?i)^(.+?)\s+(\d{1,2})\.m4b$"
        ).unwrap();
    }

    // Try Part pattern
    if let Some(base) = check_pattern_match(files, &PART_REGEX) {
        return Some((base, MergePatternType::Part));
    }

    // Try Disc pattern
    if let Some(base) = check_pattern_match(files, &DISC_REGEX) {
        return Some((base, MergePatternType::Disc));
    }

    // Try Numeric suffix pattern
    if let Some(base) = check_pattern_match(files, &NUMERIC_REGEX) {
        return Some((base, MergePatternType::NumericSuffix));
    }

    None
}

/// Check if all files match a pattern and have the same base name
fn check_pattern_match(files: &[&Path], regex: &Regex) -> Option<String> {
    let mut base_names: Vec<String> = Vec::new();
    let mut numbers: Vec<u32> = Vec::new();

    for file in files {
        let filename = file.file_name()?.to_str()?;
        let caps = regex.captures(filename)?;

        let base = caps.get(1)?.as_str().trim().to_string();
        let num: u32 = caps.get(2)?.as_str().parse().ok()?;

        base_names.push(base);
        numbers.push(num);
    }

    // All base names must match
    if base_names.is_empty() {
        return None;
    }

    let first_base = &base_names[0];
    if !base_names.iter().all(|b| b == first_base) {
        return None;
    }

    // Numbers should be sequential starting from 1 (or 01)
    numbers.sort();
    let expected: Vec<u32> = (1..=(numbers.len() as u32)).collect();
    if numbers != expected {
        // Also try 0-indexed
        let expected_zero: Vec<u32> = (0..(numbers.len() as u32)).collect();
        if numbers != expected_zero {
            return None;
        }
    }

    Some(first_base.clone())
}

/// Check whether every directory is a sequential disc/part directory.
///
/// Recognized names include `CD1`, `Disc 2`, `Disk3`, `Part 4` and `Pt. 5`, the
/// same wording behind a prefix shared by every sibling (`Title - Disc 01`,
/// `Book X CD 2`), and bare numbers (`01`, `02`). Requiring a complete `1..n`
/// set avoids folding ordinary nested library layouts such as
/// `Author/Book One` and `Author/Book Two` into one book, and requiring the
/// disc wording behind a prefix keeps `Series 1`/`Series 2` as separate books.
pub fn are_sequential_part_directories(directories: &[std::path::PathBuf]) -> bool {
    if directories.len() < 2 {
        return false;
    }

    let mut keys = Vec::with_capacity(directories.len());
    let mut numbers = Vec::with_capacity(directories.len());
    for directory in directories {
        let Some((key, number)) = directory
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(parse_part_directory_name)
        else {
            return false;
        };
        keys.push(key);
        numbers.push(number);
    }

    // Siblings must share one shape: all bare numbers, or all the same prefix.
    if keys.windows(2).any(|pair| pair[0] != pair[1]) {
        return false;
    }

    numbers.sort_unstable();
    numbers == (1..=directories.len() as u32).collect::<Vec<_>>()
}

/// Split a disc/part directory name into a comparison key and its number.
///
/// The key is the normalized prefix before the disc wording (empty for `CD 1`),
/// or a sentinel for bare numbers so they never match a prefixed sibling.
fn parse_part_directory_name(name: &str) -> Option<(String, u32)> {
    lazy_static::lazy_static! {
        static ref PART_DIRECTORY_REGEX: Regex = Regex::new(
            r"(?i)^(.*?)\b(?:part|pt\.?|disc|disk|cd)\s*(\d+)$"
        ).unwrap();
        static ref NUMBER_DIRECTORY_REGEX: Regex = Regex::new(r"^(\d+)$").unwrap();
    }

    if let Some(captures) = NUMBER_DIRECTORY_REGEX.captures(name) {
        let number = captures[1].parse().ok()?;
        return Some(("#".to_string(), number));
    }

    let captures = PART_DIRECTORY_REGEX.captures(name)?;
    let prefix = captures[1]
        .trim_end_matches(|c: char| c.is_whitespace() || matches!(c, '-' | '_' | '.' | ','))
        .to_lowercase();
    let number = captures[2].parse().ok()?;
    Some((prefix, number))
}

/// Whether a directory name is a generic audio container rather than a title.
///
/// Used when a book folder holds no audio itself and its tracks sit in a single
/// subfolder: `Book One/MP3` is named after `Book One`, while `Author/Title`
/// keeps `Title` (issue #37). A lone disc folder (`CD 1`) counts as generic too.
pub fn is_generic_container_directory(name: &str) -> bool {
    lazy_static::lazy_static! {
        static ref GENERIC_DIRECTORY_REGEX: Regex = Regex::new(
            r"(?i)^(?:mp3|m4a|m4b|aac|flac|ogg|audio|audiobook|files|tracks)$"
        ).unwrap();
    }

    GENERIC_DIRECTORY_REGEX.is_match(name.trim())
        || parse_part_directory_name(name.trim()).is_some_and(|(key, _)| key.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_part_pattern() {
        let files = vec![
            Path::new("Book Name Part 1.m4b"),
            Path::new("Book Name Part 2.m4b"),
        ];
        let result = detect_merge_pattern(&files);
        assert!(result.pattern_detected);
        assert_eq!(result.pattern_type, Some(MergePatternType::Part));
        assert_eq!(result.base_name, Some("Book Name".to_string()));
    }

    #[test]
    fn test_detect_disc_pattern() {
        let files = vec![
            Path::new("Audiobook CD1.m4b"),
            Path::new("Audiobook CD2.m4b"),
        ];
        let result = detect_merge_pattern(&files);
        assert!(result.pattern_detected);
        assert_eq!(result.pattern_type, Some(MergePatternType::Disc));
    }

    #[test]
    fn test_detect_numeric_suffix() {
        let files = vec![Path::new("My Book 01.m4b"), Path::new("My Book 02.m4b")];
        let result = detect_merge_pattern(&files);
        assert!(result.pattern_detected);
        assert_eq!(result.pattern_type, Some(MergePatternType::NumericSuffix));
    }

    #[test]
    fn test_no_pattern_detected() {
        let files = vec![
            Path::new("Different Book.m4b"),
            Path::new("Another Book.m4b"),
        ];
        let result = detect_merge_pattern(&files);
        assert!(!result.pattern_detected);
    }

    #[test]
    fn test_single_file_no_pattern() {
        let files = vec![Path::new("Single Book.m4b")];
        let result = detect_merge_pattern(&files);
        assert!(!result.pattern_detected);
    }

    #[test]
    fn test_sequential_part_directories() {
        let disc_directories = vec![
            std::path::PathBuf::from("CD1"),
            std::path::PathBuf::from("CD2"),
        ];
        assert!(are_sequential_part_directories(&disc_directories));

        let book_directories = vec![
            std::path::PathBuf::from("Book One"),
            std::path::PathBuf::from("Book Two"),
        ];
        assert!(!are_sequential_part_directories(&book_directories));
    }

    fn dirs(names: &[&str]) -> Vec<std::path::PathBuf> {
        names.iter().map(std::path::PathBuf::from).collect()
    }

    #[test]
    fn test_sequential_part_directories_with_shared_prefix() {
        assert!(are_sequential_part_directories(&dirs(&[
            "Book X CD 1",
            "Book X CD 2"
        ])));
        assert!(are_sequential_part_directories(&dirs(&[
            "Title - Disc 01",
            "Title - Disc 02"
        ])));
        // Different prefixes are different books.
        assert!(!are_sequential_part_directories(&dirs(&[
            "Book X CD 1",
            "Book Y CD 2"
        ])));
        // A number without disc wording is a series, not a disc set.
        assert!(!are_sequential_part_directories(&dirs(&[
            "Series 1", "Series 2"
        ])));
        // The keyword must be a whole word.
        assert!(!are_sequential_part_directories(&dirs(&[
            "Abcd 1", "Abcd 2"
        ])));
    }

    #[test]
    fn test_sequential_part_directories_bare_numbers() {
        assert!(are_sequential_part_directories(&dirs(&["01", "02", "03"])));
        assert!(!are_sequential_part_directories(&dirs(&["01", "03"])));
        assert!(!are_sequential_part_directories(&dirs(&["01", "CD 2"])));
    }

    #[test]
    fn test_generic_container_directory() {
        for name in ["MP3", "Audio", "flac", "CD 1", "Disc1"] {
            assert!(is_generic_container_directory(name), "{name}");
        }
        for name in ["Elantris", "Edición 2005", "Book X CD 1"] {
            assert!(!is_generic_container_directory(name), "{name}");
        }
    }
}
