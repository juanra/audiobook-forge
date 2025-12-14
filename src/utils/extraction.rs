//! Metadata extraction from M4B files and filenames

use crate::models::{CurrentMetadata, MetadataSource};
use anyhow::{Result, Context};
use std::path::Path;

/// Extract metadata from M4B file (embedded tags first, filename fallback)
pub fn extract_current_metadata(file_path: &Path) -> Result<CurrentMetadata> {
    // Try embedded metadata first
    let embedded = extract_from_embedded_tags(file_path)?;

    if embedded.is_sufficient() {
        Ok(embedded)
    } else {
        // Fallback to filename parsing
        let from_filename = extract_from_filename(file_path)?;

        // Merge: prefer embedded values if present, use filename as fallback
        Ok(embedded.merge_with(from_filename))
    }
}

/// Extract from embedded M4B tags
fn extract_from_embedded_tags(file_path: &Path) -> Result<CurrentMetadata> {
    let tag = mp4ameta::Tag::read_from_path(file_path)
        .context("Failed to read M4B metadata")?;

    Ok(CurrentMetadata {
        title: tag.title().map(|s| s.to_string()),
        author: tag.artist().map(|s| s.to_string())
            .or_else(|| tag.album_artist().map(|s| s.to_string())),
        year: tag.year().and_then(|s| s.parse::<u32>().ok()),
        duration: None, // TODO: get from FFprobe if needed
        source: MetadataSource::Embedded,
    })
}

/// Extract from filename using pattern matching
fn extract_from_filename(file_path: &Path) -> Result<CurrentMetadata> {
    let filename = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Pattern: "Author - Title"
    if let Some((author, title)) = parse_author_title_pattern(filename) {
        return Ok(CurrentMetadata {
            title: Some(title),
            author: Some(author),
            year: None,
            duration: None,
            source: MetadataSource::Filename,
        });
    }

    // Fallback: use entire filename as title
    Ok(CurrentMetadata {
        title: Some(filename.to_string()),
        author: None,
        year: None,
        duration: None,
        source: MetadataSource::Filename,
    })
}

/// Parse "Author - Title" pattern
fn parse_author_title_pattern(filename: &str) -> Option<(String, String)> {
    // Split on " - " (note the spaces)
    let parts: Vec<&str> = filename.split(" - ").collect();

    if parts.len() >= 2 {
        let author = parts[0].trim().to_string();
        let title = parts[1..].join(" - ").trim().to_string();
        Some((author, title))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_author_title_pattern() {
        let (author, title) = parse_author_title_pattern("Andy Weir - Project Hail Mary").unwrap();
        assert_eq!(author, "Andy Weir");
        assert_eq!(title, "Project Hail Mary");

        // Multiple hyphens
        let (author, title) = parse_author_title_pattern("Isaac Asimov - I, Robot - Complete Edition").unwrap();
        assert_eq!(author, "Isaac Asimov");
        assert_eq!(title, "I, Robot - Complete Edition");

        // No match
        assert_eq!(parse_author_title_pattern("JustATitle"), None);
    }
}
