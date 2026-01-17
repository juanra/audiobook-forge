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

/// Sort files by their numeric part indicator
pub fn sort_by_part_number(files: &mut [std::path::PathBuf]) {
    lazy_static::lazy_static! {
        static ref NUMBER_REGEX: Regex = Regex::new(
            r"(?i)(?:part|pt\.?|disc|disk|cd)?\s*(\d+)\.m4b$"
        ).unwrap();
    }

    files.sort_by(|a, b| {
        let get_num = |p: &std::path::PathBuf| -> u32 {
            p.file_name()
                .and_then(|n| n.to_str())
                .and_then(|s| NUMBER_REGEX.captures(s))
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0)
        };
        get_num(a).cmp(&get_num(b))
    });
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
        let files = vec![
            Path::new("My Book 01.m4b"),
            Path::new("My Book 02.m4b"),
        ];
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
    fn test_sort_by_part_number() {
        let mut files = vec![
            std::path::PathBuf::from("Book Part 3.m4b"),
            std::path::PathBuf::from("Book Part 1.m4b"),
            std::path::PathBuf::from("Book Part 2.m4b"),
        ];
        sort_by_part_number(&mut files);
        assert_eq!(
            files.iter().map(|p| p.file_name().unwrap().to_str().unwrap()).collect::<Vec<_>>(),
            vec!["Book Part 1.m4b", "Book Part 2.m4b", "Book Part 3.m4b"]
        );
    }
}
