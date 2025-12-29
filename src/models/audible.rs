//! Audible metadata models and types

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use anyhow::{bail, Result};

/// Audible region with TLD mapping
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudibleRegion {
    #[serde(rename = "us")]
    US,
    #[serde(rename = "ca")]
    CA,
    #[serde(rename = "uk")]
    UK,
    #[serde(rename = "au")]
    AU,
    #[serde(rename = "fr")]
    FR,
    #[serde(rename = "de")]
    DE,
    #[serde(rename = "jp")]
    JP,
    #[serde(rename = "it")]
    IT,
    #[serde(rename = "in")]
    IN,
    #[serde(rename = "es")]
    ES,
}

impl AudibleRegion {
    /// Get the region code for Audnexus API (e.g., "us", "uk")
    pub fn tld(&self) -> &'static str {
        match self {
            Self::US => "us",
            Self::CA => "ca",
            Self::UK => "uk",
            Self::AU => "au",
            Self::FR => "fr",
            Self::DE => "de",
            Self::JP => "jp",
            Self::IT => "it",
            Self::IN => "in",
            Self::ES => "es",
        }
    }

    /// Get the TLD for Audible's API (e.g., ".com", ".co.uk")
    pub fn audible_tld(&self) -> &'static str {
        match self {
            Self::US => ".com",
            Self::CA => ".ca",
            Self::UK => ".co.uk",
            Self::AU => ".com.au",
            Self::FR => ".fr",
            Self::DE => ".de",
            Self::JP => ".co.jp",
            Self::IT => ".it",
            Self::IN => ".in",
            Self::ES => ".es",
        }
    }
}

impl FromStr for AudibleRegion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "us" => Ok(Self::US),
            "ca" => Ok(Self::CA),
            "uk" => Ok(Self::UK),
            "au" => Ok(Self::AU),
            "fr" => Ok(Self::FR),
            "de" => Ok(Self::DE),
            "jp" => Ok(Self::JP),
            "it" => Ok(Self::IT),
            "in" => Ok(Self::IN),
            "es" => Ok(Self::ES),
            _ => bail!("Invalid Audible region: {}. Valid regions: us, ca, uk, au, fr, de, jp, it, in, es", s),
        }
    }
}

impl fmt::Display for AudibleRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tld())
    }
}

impl Default for AudibleRegion {
    fn default() -> Self {
        Self::US
    }
}

/// Audible metadata from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudibleMetadata {
    pub asin: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub authors: Vec<AudibleAuthor>,
    #[serde(default)]
    pub narrators: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isbn: Option<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub series: Vec<AudibleSeries>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Runtime length in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_length_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_abridged: Option<bool>,
}

impl AudibleMetadata {
    /// Get runtime in minutes
    pub fn runtime_minutes(&self) -> Option<u32> {
        self.runtime_length_ms.map(|ms| (ms / 60_000) as u32)
    }

    /// Get primary author name
    pub fn primary_author(&self) -> Option<&str> {
        self.authors.first().map(|a| a.name.as_str())
    }

    /// Get all authors joined as a string
    pub fn authors_string(&self) -> String {
        self.authors
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Get all narrators joined as a string
    pub fn narrators_string(&self) -> String {
        self.narrators.join(", ")
    }

    /// Get primary narrator
    pub fn primary_narrator(&self) -> Option<&str> {
        self.narrators.first().map(|n| n.as_str())
    }
}

/// Audible author information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudibleAuthor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asin: Option<String>,
    pub name: String,
}

/// Audible series information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudibleSeries {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asin: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<String>,
}

/// Search result from Audible catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudibleSearchResult {
    pub asin: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub narrators: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_ms: Option<u64>,
}

impl AudibleSearchResult {
    /// Get runtime in minutes
    pub fn runtime_minutes(&self) -> Option<u32> {
        self.runtime_ms.map(|ms| (ms / 60_000) as u32)
    }
}

/// Chapter information from Audnex API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudibleChapter {
    /// Chapter title
    pub title: String,
    /// Duration in milliseconds
    #[serde(rename = "lengthMs")]
    pub length_ms: u64,
    /// Start offset in milliseconds from beginning
    #[serde(rename = "startOffsetMs")]
    pub start_offset_ms: u64,
    /// Start offset in seconds (convenience field)
    #[serde(rename = "startOffsetSec", default)]
    pub start_offset_sec: Option<u32>,
}

impl AudibleChapter {
    /// Get end time in milliseconds
    pub fn end_offset_ms(&self) -> u64 {
        self.start_offset_ms + self.length_ms
    }

    /// Convert to internal Chapter struct
    pub fn to_chapter(&self, number: u32) -> crate::audio::Chapter {
        crate::audio::Chapter::new(
            number,
            self.title.clone(),
            self.start_offset_ms,
            self.end_offset_ms(),
        )
    }
}

/// Audnex chapters API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudnexChaptersResponse {
    pub asin: String,
    #[serde(rename = "brandIntroDurationMs", default)]
    pub brand_intro_duration_ms: Option<u64>,
    #[serde(rename = "brandOutroDurationMs", default)]
    pub brand_outro_duration_ms: Option<u64>,
    pub chapters: Vec<AudibleChapter>,
    #[serde(rename = "isAccurate", default)]
    pub is_accurate: Option<bool>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(rename = "runtimeLengthMs", default)]
    pub runtime_length_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_from_str() {
        assert_eq!(AudibleRegion::from_str("us").unwrap(), AudibleRegion::US);
        assert_eq!(AudibleRegion::from_str("UK").unwrap(), AudibleRegion::UK);
        assert_eq!(AudibleRegion::from_str("Ca").unwrap(), AudibleRegion::CA);
        assert!(AudibleRegion::from_str("invalid").is_err());
    }

    #[test]
    fn test_region_tld() {
        assert_eq!(AudibleRegion::US.tld(), "us");
        assert_eq!(AudibleRegion::UK.tld(), "uk");
        assert_eq!(AudibleRegion::FR.tld(), "fr");
    }

    #[test]
    fn test_region_display() {
        assert_eq!(format!("{}", AudibleRegion::US), "us");
        assert_eq!(format!("{}", AudibleRegion::UK), "uk");
    }

    #[test]
    fn test_runtime_conversion() {
        let metadata = AudibleMetadata {
            asin: "B001".to_string(),
            title: "Test".to_string(),
            subtitle: None,
            authors: vec![],
            narrators: vec![],
            publisher: None,
            published_year: None,
            description: None,
            cover_url: None,
            isbn: None,
            genres: vec![],
            tags: vec![],
            series: vec![],
            language: None,
            runtime_length_ms: Some(3_600_000), // 1 hour in ms
            rating: None,
            is_abridged: None,
        };

        assert_eq!(metadata.runtime_minutes(), Some(60));
    }

    #[test]
    fn test_authors_string() {
        let metadata = AudibleMetadata {
            asin: "B001".to_string(),
            title: "Test".to_string(),
            subtitle: None,
            authors: vec![
                AudibleAuthor {
                    asin: None,
                    name: "Author One".to_string(),
                },
                AudibleAuthor {
                    asin: None,
                    name: "Author Two".to_string(),
                },
            ],
            narrators: vec!["Narrator One".to_string(), "Narrator Two".to_string()],
            publisher: None,
            published_year: None,
            description: None,
            cover_url: None,
            isbn: None,
            genres: vec![],
            tags: vec![],
            series: vec![],
            language: None,
            runtime_length_ms: None,
            rating: None,
            is_abridged: None,
        };

        assert_eq!(metadata.authors_string(), "Author One, Author Two");
        assert_eq!(metadata.narrators_string(), "Narrator One, Narrator Two");
        assert_eq!(metadata.primary_author(), Some("Author One"));
        assert_eq!(metadata.primary_narrator(), Some("Narrator One"));
    }

    #[test]
    fn test_audible_chapter_end_offset() {
        let chapter = AudibleChapter {
            title: "Prologue".to_string(),
            length_ms: 300_000, // 5 minutes
            start_offset_ms: 0,
            start_offset_sec: Some(0),
        };

        assert_eq!(chapter.end_offset_ms(), 300_000);
    }

    #[test]
    fn test_audible_chapter_to_chapter_conversion() {
        let audible_chapter = AudibleChapter {
            title: "Chapter 1".to_string(),
            length_ms: 600_000, // 10 minutes
            start_offset_ms: 300_000, // starts at 5 min
            start_offset_sec: Some(300),
        };

        let chapter = audible_chapter.to_chapter(1);

        assert_eq!(chapter.number, 1);
        assert_eq!(chapter.title, "Chapter 1");
        assert_eq!(chapter.start_time_ms, 300_000);
        assert_eq!(chapter.end_time_ms, 900_000);
    }
}
