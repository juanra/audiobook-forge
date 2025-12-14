//! Metadata extraction from M4B files and filenames

use crate::models::{CurrentMetadata, MetadataSource};
use anyhow::{Result, Context};
use std::path::Path;

/// Extract metadata from M4B file (embedded tags first, filename fallback)
pub fn extract_current_metadata(file_path: &Path) -> Result<CurrentMetadata> {
    // Try embedded metadata first
    let embedded = extract_from_embedded_tags(file_path)?;

    // Always try filename parsing to fill in any gaps
    let from_filename = extract_from_filename(file_path)?;

    // Merge: prefer embedded values if present, use filename as fallback
    // This ensures we get author from filename even if title is embedded
    Ok(embedded.merge_with(from_filename))
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
    // Try different separators: " - ", "_-_", " -_ ", etc.
    let separators = [" - ", "_-_", " -_ ", "_ -_", "_- "];

    for separator in separators {
        let parts: Vec<&str> = filename.split(separator).collect();

        if parts.len() >= 2 {
            // Clean up underscores from author/title and convert to spaces
            let author = parts[0].replace('_', " ").trim().to_string();
            let title = parts[1..].join(separator).replace('_', " ").trim().to_string();

            // Only return if both author and title are non-empty
            if !author.is_empty() && !title.is_empty() {
                return Some((author, title));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_author_title_pattern() {
        // Standard space-dash-space pattern
        let (author, title) = parse_author_title_pattern("Andy Weir - Project Hail Mary").unwrap();
        assert_eq!(author, "Andy Weir");
        assert_eq!(title, "Project Hail Mary");

        // Multiple hyphens
        let (author, title) = parse_author_title_pattern("Isaac Asimov - I, Robot - Complete Edition").unwrap();
        assert_eq!(author, "Isaac Asimov");
        assert_eq!(title, "I, Robot - Complete Edition");

        // Underscore patterns (common in downloaded audiobooks)
        let (author, title) = parse_author_title_pattern("Adam_Phillips_-_On_Giving_Up").unwrap();
        assert_eq!(author, "Adam Phillips");
        assert_eq!(title, "On Giving Up");

        let (author, title) = parse_author_title_pattern("Morgan_Housel_-_The_Art_of_Spending_Money").unwrap();
        assert_eq!(author, "Morgan Housel");
        assert_eq!(title, "The Art of Spending Money");

        // Mixed underscores and spaces
        let (author, title) = parse_author_title_pattern("Neil_deGrasse_Tyson - Just Visiting This Planet").unwrap();
        assert_eq!(author, "Neil deGrasse Tyson");
        assert_eq!(title, "Just Visiting This Planet");

        // No match
        assert_eq!(parse_author_title_pattern("JustATitle"), None);
    }
}
