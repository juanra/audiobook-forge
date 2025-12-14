//! Match models for interactive metadata matching

use super::AudibleMetadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Confidence level for match quality (similar to BEETS Recommendation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchConfidence {
    /// Strong match (>96% - distance < 0.04)
    Strong,
    /// Medium match (88-96% - distance < 0.12)
    Medium,
    /// Low match (80-88% - distance < 0.20)
    Low,
    /// Weak or no clear match (<80%)
    None,
}

/// Distance/penalty calculation for metadata comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataDistance {
    /// Individual penalties by field name
    penalties: HashMap<String, f64>,
    /// Weighted total distance (0.0 = perfect, 1.0 = worst)
    total: f64,
}

impl MetadataDistance {
    /// Create a new empty distance
    pub fn new() -> Self {
        Self {
            penalties: HashMap::new(),
            total: 0.0,
        }
    }

    /// Add a weighted penalty for a specific field
    pub fn add_penalty(&mut self, field: &str, distance: f64, weight: f64) {
        let weighted = distance * weight;
        self.penalties.insert(field.to_string(), distance);
        self.total += weighted;
    }

    /// Get total weighted distance
    pub fn total_distance(&self) -> f64 {
        self.total
    }

    /// Get penalty for a specific field
    pub fn get_penalty(&self, field: &str) -> Option<f64> {
        self.penalties.get(field).copied()
    }

    /// Get all field penalties
    pub fn penalties(&self) -> &HashMap<String, f64> {
        &self.penalties
    }
}

impl Default for MetadataDistance {
    fn default() -> Self {
        Self::new()
    }
}

/// Match candidate with distance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchCandidate {
    /// Distance/penalty score
    pub distance: MetadataDistance,
    /// Audible metadata
    pub metadata: AudibleMetadata,
    /// Confidence level
    pub confidence: MatchConfidence,
}

/// Source of metadata extraction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetadataSource {
    /// Extracted from embedded M4B tags
    Embedded,
    /// Parsed from filename
    Filename,
    /// Manually provided by user
    Manual,
}

/// Current metadata extracted from M4B file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentMetadata {
    /// Book title
    pub title: Option<String>,
    /// Author name (from artist/album_artist)
    pub author: Option<String>,
    /// Publication year
    pub year: Option<u32>,
    /// Duration in seconds
    pub duration: Option<f64>,
    /// Source of this metadata
    pub source: MetadataSource,
}

impl CurrentMetadata {
    /// Check if metadata is sufficient for searching
    pub fn is_sufficient(&self) -> bool {
        self.title.is_some() || self.author.is_some()
    }

    /// Merge with another CurrentMetadata (self takes priority)
    pub fn merge_with(self, other: CurrentMetadata) -> CurrentMetadata {
        CurrentMetadata {
            title: self.title.or(other.title),
            author: self.author.or(other.author),
            year: self.year.or(other.year),
            duration: self.duration.or(other.duration),
            source: self.source,  // Keep original source
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_distance() {
        let mut distance = MetadataDistance::new();
        distance.add_penalty("title", 0.1, 0.4);  // 0.04 weighted
        distance.add_penalty("author", 0.2, 0.3); // 0.06 weighted

        assert!((distance.total_distance() - 0.10).abs() < 0.001);
        assert_eq!(distance.get_penalty("title"), Some(0.1));
        assert_eq!(distance.get_penalty("author"), Some(0.2));
    }

    #[test]
    fn test_current_metadata_is_sufficient() {
        let metadata = CurrentMetadata {
            title: Some("Test".to_string()),
            author: None,
            year: None,
            duration: None,
            source: MetadataSource::Embedded,
        };
        assert!(metadata.is_sufficient());

        let empty = CurrentMetadata {
            title: None,
            author: None,
            year: None,
            duration: None,
            source: MetadataSource::Embedded,
        };
        assert!(!empty.is_sufficient());
    }

    #[test]
    fn test_current_metadata_merge() {
        let embedded = CurrentMetadata {
            title: Some("Title from tags".to_string()),
            author: None,
            year: Some(2020),
            duration: None,
            source: MetadataSource::Embedded,
        };

        let filename = CurrentMetadata {
            title: Some("Title from filename".to_string()),
            author: Some("Author from filename".to_string()),
            year: None,
            duration: None,
            source: MetadataSource::Filename,
        };

        let merged = embedded.merge_with(filename);

        // Embedded takes priority for title
        assert_eq!(merged.title, Some("Title from tags".to_string()));
        // Filename provides author
        assert_eq!(merged.author, Some("Author from filename".to_string()));
        // Year from embedded
        assert_eq!(merged.year, Some(2020));
        // Source from embedded (first)
        assert_eq!(merged.source, MetadataSource::Embedded);
    }
}
