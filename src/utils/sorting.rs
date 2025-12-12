//! Natural sorting utilities

use std::cmp::Ordering;
use std::path::Path;

/// Sort paths using natural (human-friendly) ordering
///
/// Examples:
/// - track1.mp3 < track2.mp3 < track10.mp3 (not track1, track10, track2)
/// - Chapter 1 < Chapter 2 < Chapter 10
pub fn natural_sort<P: AsRef<Path>>(paths: &mut [P]) {
    paths.sort_by(|a, b| natural_compare(a.as_ref(), b.as_ref()));
}

/// Compare two paths using natural ordering
fn natural_compare(a: &Path, b: &Path) -> Ordering {
    let a_str = a.to_string_lossy();
    let b_str = b.to_string_lossy();

    natord::compare(&a_str, &b_str)
}

/// Sort strings using natural ordering
pub fn natural_sort_strings(strings: &mut [String]) {
    strings.sort_by(|a, b| natord::compare(a, b));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_natural_sort() {
        let mut paths = vec![
            PathBuf::from("track10.mp3"),
            PathBuf::from("track2.mp3"),
            PathBuf::from("track1.mp3"),
            PathBuf::from("track20.mp3"),
        ];

        natural_sort(&mut paths);

        assert_eq!(paths[0], PathBuf::from("track1.mp3"));
        assert_eq!(paths[1], PathBuf::from("track2.mp3"));
        assert_eq!(paths[2], PathBuf::from("track10.mp3"));
        assert_eq!(paths[3], PathBuf::from("track20.mp3"));
    }

    #[test]
    fn test_natural_sort_strings() {
        let mut strings = vec![
            "Chapter 10".to_string(),
            "Chapter 2".to_string(),
            "Chapter 1".to_string(),
        ];

        natural_sort_strings(&mut strings);

        assert_eq!(strings[0], "Chapter 1");
        assert_eq!(strings[1], "Chapter 2");
        assert_eq!(strings[2], "Chapter 10");
    }
}
