//! Audible API client for metadata fetching

use anyhow::{Context, Result};
use governor::{Quota, RateLimiter, state::{InMemoryState, direct::NotKeyed as DirectNotKeyed}, clock::DefaultClock};
use reqwest::Client;
use serde::Deserialize;
use std::num::NonZeroU32;
use std::path::Path;
use std::time::Duration;

use crate::models::{AudibleMetadata, AudibleRegion, AudibleAuthor, AudibleSeries};

const AUDNEXUS_BASE_URL: &str = "https://api.audnex.us";
const DEFAULT_TIMEOUT_SECS: u64 = 10;

/// Audible API client with rate limiting
pub struct AudibleClient {
    client: Client,
    rate_limiter: RateLimiter<DirectNotKeyed, InMemoryState, DefaultClock>,
    region: AudibleRegion,
}

impl AudibleClient {
    /// Create a new Audible client for the specified region
    pub fn new(region: AudibleRegion) -> Result<Self> {
        Self::with_rate_limit(region, 100)
    }

    /// Create a new Audible client with custom rate limit
    pub fn with_rate_limit(region: AudibleRegion, requests_per_minute: u32) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .user_agent("audiobook-forge")
            .build()
            .context("Failed to create HTTP client")?;

        // Rate limiter: requests_per_minute (e.g., 100 req/min = ~150ms between requests)
        let quota = Quota::per_minute(
            NonZeroU32::new(requests_per_minute)
                .unwrap_or(NonZeroU32::new(100).unwrap())
        );
        let rate_limiter = RateLimiter::direct(quota);

        Ok(Self {
            client,
            rate_limiter,
            region,
        })
    }

    /// Fetch metadata by ASIN
    pub async fn fetch_by_asin(&self, asin: &str) -> Result<AudibleMetadata> {
        // Wait for rate limiter
        self.rate_limiter.until_ready().await;

        let url = format!("{}/books/{}?region={}",
            AUDNEXUS_BASE_URL, asin, self.region.tld());

        tracing::debug!("Fetching Audible metadata: {}", url);

        let response = self.client.get(&url)
            .send()
            .await
            .context("Failed to fetch from Audnexus API")?;

        if !response.status().is_success() {
            anyhow::bail!("API returned status: {}", response.status());
        }

        // Parse the response
        let api_response: AudnexusBookResponse = response.json()
            .await
            .context("Failed to parse Audible metadata")?;

        // Convert to our metadata structure
        Ok(convert_audnexus_to_metadata(api_response))
    }

    /// Search by title and/or author using Audible's API
    /// Returns full metadata for each result by fetching from Audnexus
    pub async fn search(&self, title: Option<&str>, author: Option<&str>) -> Result<Vec<AudibleMetadata>> {
        if title.is_none() && author.is_none() {
            anyhow::bail!("Must provide at least title or author for search");
        }

        // Wait for rate limiter
        self.rate_limiter.until_ready().await;

        // Build query parameters for Audible's search API
        let mut query_params = vec![
            ("num_results", "10"),
            ("products_sort_by", "Relevance"),
        ];

        if let Some(t) = title {
            query_params.push(("title", t));
        }
        if let Some(a) = author {
            query_params.push(("author", a));
        }

        // Use Audible's direct API with region-specific TLD
        let audible_tld = self.region.audible_tld();
        let url = format!("https://api.audible{}/1.0/catalog/products", audible_tld);

        tracing::debug!("Searching Audible: title={:?}, author={:?}", title, author);

        let response = self.client.get(&url)
            .query(&query_params)
            .send()
            .await
            .context("Failed to search Audible API")?;

        if !response.status().is_success() {
            anyhow::bail!("Search API returned status: {}", response.status());
        }

        // Parse Audible's search response (just ASINs)
        let search_response: AudibleSearchResponse = response.json()
            .await
            .context("Failed to parse Audible search results")?;

        if search_response.products.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch full metadata from Audnexus for each ASIN
        let mut metadata_results = Vec::new();
        for product in search_response.products.iter().take(10) {
            match self.fetch_by_asin(&product.asin).await {
                Ok(metadata) => metadata_results.push(metadata),
                Err(e) => {
                    tracing::warn!("Failed to fetch metadata for ASIN {}: {}", product.asin, e);
                }
            }
        }

        Ok(metadata_results)
    }

    /// Download cover image
    pub async fn download_cover(&self, cover_url: &str, dest_path: &Path) -> Result<()> {
        // Wait for rate limiter
        self.rate_limiter.until_ready().await;

        tracing::debug!("Downloading cover from: {}", cover_url);

        let response = self.client.get(cover_url)
            .send()
            .await
            .context("Failed to download cover")?;

        if !response.status().is_success() {
            anyhow::bail!("Cover download failed: {}", response.status());
        }

        let bytes = response.bytes()
            .await
            .context("Failed to read cover bytes")?;

        std::fs::write(dest_path, bytes)
            .context("Failed to write cover file")?;

        tracing::debug!("Cover saved to: {}", dest_path.display());

        Ok(())
    }

    /// Change the region for this client
    pub fn set_region(&mut self, region: AudibleRegion) {
        self.region = region;
    }

    /// Get the current region
    pub fn region(&self) -> AudibleRegion {
        self.region
    }
}

// API response structures

// Audible's search API response (from api.audible.com)
#[derive(Debug, Deserialize)]
struct AudibleSearchResponse {
    products: Vec<AudibleProduct>,
    #[serde(default)]
    #[allow(dead_code)]  // Part of API response but not used
    total_results: u32,
}

#[derive(Debug, Deserialize)]
struct AudibleProduct {
    asin: String,
}

// Audnexus API response structures
// These match the API response format from https://api.audnex.us

#[derive(Debug, Deserialize)]
struct AudnexusBookResponse {
    asin: String,
    title: String,
    subtitle: Option<String>,
    authors: Option<Vec<AudnexusAuthor>>,
    narrators: Option<Vec<AudnexusNarrator>>,
    #[serde(rename = "publisherName")]
    publisher_name: Option<String>,
    #[serde(rename = "releaseDate")]
    release_date: Option<String>,
    summary: Option<String>,
    image: Option<String>,
    isbn: Option<String>,
    genres: Option<Vec<AudnexusGenre>>,
    #[serde(rename = "seriesPrimary")]
    series_primary: Option<AudnexusSeries>,
    #[serde(rename = "seriesSecondary")]
    series_secondary: Option<AudnexusSeries>,
    language: Option<String>,
    #[serde(rename = "runtimeLengthMin")]
    runtime_length_min: Option<u64>,
    #[serde(rename = "formatType")]
    format_type: Option<String>,
    rating: Option<String>,  // API returns string, we'll parse to f32
}

#[derive(Debug, Deserialize)]
struct AudnexusAuthor {
    asin: Option<String>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct AudnexusNarrator {
    name: String,
}

#[derive(Debug, Deserialize)]
struct AudnexusGenre {
    #[allow(dead_code)]  // Part of API response but not used
    asin: Option<String>,
    name: String,
    #[serde(rename = "type")]
    genre_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AudnexusSeries {
    asin: Option<String>,
    name: String,
    position: Option<String>,
}

/// Convert Audnexus API response to our metadata structure
fn convert_audnexus_to_metadata(api: AudnexusBookResponse) -> AudibleMetadata {
    // Extract year from release_date (format: "YYYY-MM-DD")
    let published_year = api.release_date
        .as_ref()
        .and_then(|date| date.split('-').next())
        .and_then(|year_str| year_str.parse::<u32>().ok());

    // Convert authors
    let authors = api.authors
        .unwrap_or_default()
        .into_iter()
        .map(|a| AudibleAuthor {
            asin: a.asin,
            name: a.name,
        })
        .collect();

    // Convert narrators
    let narrators = api.narrators
        .unwrap_or_default()
        .into_iter()
        .map(|n| n.name)
        .collect();

    // Extract genres and tags
    let genres_data = api.genres.unwrap_or_default();
    let genres: Vec<String> = genres_data
        .iter()
        .filter(|g| g.genre_type.as_deref() == Some("genre"))
        .map(|g| g.name.clone())
        .collect();

    let tags: Vec<String> = genres_data
        .iter()
        .filter(|g| g.genre_type.as_deref() == Some("tag"))
        .map(|g| g.name.clone())
        .collect();

    // Build series list
    let mut series = Vec::new();
    if let Some(primary) = api.series_primary {
        series.push(AudibleSeries {
            asin: primary.asin,
            name: primary.name,
            sequence: primary.position.map(|p| clean_sequence(&p)),
        });
    }
    if let Some(secondary) = api.series_secondary {
        series.push(AudibleSeries {
            asin: secondary.asin,
            name: secondary.name,
            sequence: secondary.position.map(|p| clean_sequence(&p)),
        });
    }

    // Determine if abridged
    let is_abridged = api.format_type
        .as_ref()
        .map(|ft| ft.to_lowercase() == "abridged");

    // Convert runtime from minutes to milliseconds
    let runtime_length_ms = api.runtime_length_min.map(|min| min * 60_000);

    // Parse rating from string to f32
    let rating = api.rating
        .and_then(|r| r.parse::<f32>().ok());

    AudibleMetadata {
        asin: api.asin,
        title: api.title,
        subtitle: api.subtitle,
        authors,
        narrators,
        publisher: api.publisher_name,
        published_year,
        description: api.summary,
        cover_url: api.image,
        isbn: api.isbn,
        genres,
        tags,
        series,
        language: api.language,
        runtime_length_ms,
        rating,
        is_abridged,
    }
}

/// Detect ASIN from folder name or string
/// ASIN pattern: B followed by 9 alphanumeric characters (e.g., B002V5D7RU)
pub fn detect_asin(text: &str) -> Option<String> {
    use regex::Regex;

    lazy_static::lazy_static! {
        static ref ASIN_REGEX: Regex = Regex::new(r"\b(B[0-9A-Z]{9})\b").unwrap();
    }

    ASIN_REGEX.captures(text)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

/// Clean series sequence from Audible format
/// Extracts numbers from strings like "Book 1", "1.5", ".5", etc.
pub fn clean_sequence(sequence: &str) -> String {
    use regex::Regex;

    lazy_static::lazy_static! {
        static ref SEQ_REGEX: Regex = Regex::new(r"(\d+(?:\.\d+)?)").unwrap();
    }

    SEQ_REGEX.captures(sequence)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| sequence.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asin_detection() {
        assert_eq!(detect_asin("Book Title [B002V5D7RU]"), Some("B002V5D7RU".to_string()));
        assert_eq!(detect_asin("B002V5D7RU - Book Title"), Some("B002V5D7RU".to_string()));
        assert_eq!(detect_asin("Project Hail Mary [B00G3L6JMS].m4b"), Some("B00G3L6JMS".to_string()));
        assert_eq!(detect_asin("No ASIN Here"), None);
        assert_eq!(detect_asin("Invalid B12345"), None); // Too short
    }

    #[test]
    fn test_clean_sequence() {
        assert_eq!(clean_sequence("Book 1"), "1");
        assert_eq!(clean_sequence("1.5"), "1.5");
        assert_eq!(clean_sequence("Book 0.5"), "0.5");
        assert_eq!(clean_sequence("2, Dramatized Adaptation"), "2");
        assert_eq!(clean_sequence("no numbers"), "no numbers");
    }

    #[test]
    fn test_client_creation() {
        let client = AudibleClient::new(AudibleRegion::US).unwrap();
        assert_eq!(client.region(), AudibleRegion::US);
    }

    #[test]
    fn test_region_change() {
        let mut client = AudibleClient::new(AudibleRegion::US).unwrap();
        assert_eq!(client.region(), AudibleRegion::US);

        client.set_region(AudibleRegion::UK);
        assert_eq!(client.region(), AudibleRegion::UK);
    }
}
