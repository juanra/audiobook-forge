//! Integration tests for Audible metadata functionality

use audiobook_forge::audio::{AudibleClient, detect_asin, clean_sequence};
use audiobook_forge::models::AudibleRegion;
use audiobook_forge::utils::AudibleCache;

#[test]
fn test_asin_detection_patterns() {
    // Test various ASIN patterns in folder names
    assert_eq!(
        detect_asin("Project Hail Mary [B00G3L6JMS]"),
        Some("B00G3L6JMS".to_string())
    );

    assert_eq!(
        detect_asin("B002V5D7RU - The Martian"),
        Some("B002V5D7RU".to_string())
    );

    assert_eq!(
        detect_asin("The Hobbit - B00DBXFBHA - J.R.R. Tolkien"),
        Some("B00DBXFBHA".to_string())
    );

    assert_eq!(
        detect_asin("Book Title [B00G3L6JMS].m4b"),
        Some("B00G3L6JMS".to_string())
    );

    // Test non-matching patterns
    assert_eq!(detect_asin("No ASIN Here"), None);
    assert_eq!(detect_asin("Invalid B12345"), None); // Too short
    assert_eq!(detect_asin("B12345678"), None); // Only 8 chars after B
}

#[test]
fn test_sequence_cleaning() {
    // Test various sequence formats
    assert_eq!(clean_sequence("Book 1"), "1");
    assert_eq!(clean_sequence("1.5"), "1.5");
    assert_eq!(clean_sequence("2, Dramatized Adaptation"), "2");
    assert_eq!(clean_sequence("Book 0.5"), "0.5");
    assert_eq!(clean_sequence("3.14"), "3.14");

    // Test non-numeric sequences
    assert_eq!(clean_sequence("no numbers"), "no numbers");
    assert_eq!(clean_sequence(""), "");
}

#[test]
fn test_region_parsing() {
    use std::str::FromStr;

    // Test region parsing
    assert_eq!(AudibleRegion::from_str("us").unwrap(), AudibleRegion::US);
    assert_eq!(AudibleRegion::from_str("UK").unwrap(), AudibleRegion::UK);
    assert_eq!(AudibleRegion::from_str("Ca").unwrap(), AudibleRegion::CA);
    assert_eq!(AudibleRegion::from_str("jp").unwrap(), AudibleRegion::JP);

    // Test TLD mapping
    assert_eq!(AudibleRegion::US.tld(), "us");
    assert_eq!(AudibleRegion::UK.tld(), "uk");
    assert_eq!(AudibleRegion::JP.tld(), "jp");

    // Test invalid region
    assert!(AudibleRegion::from_str("invalid").is_err());
}

#[test]
fn test_client_creation() {
    // Test basic client creation
    let client = AudibleClient::new(AudibleRegion::US);
    assert!(client.is_ok());

    let client = client.unwrap();
    assert_eq!(client.region(), AudibleRegion::US);

    // Test client with custom rate limit
    let client = AudibleClient::with_rate_limit(AudibleRegion::UK, 50);
    assert!(client.is_ok());
}

#[test]
fn test_cache_creation() {
    // Test cache creation
    let cache = AudibleCache::new();
    assert!(cache.is_ok());

    // Test cache with custom TTL
    let cache = AudibleCache::with_ttl_hours(24);
    assert!(cache.is_ok());

    // Test disabled cache (TTL = 0)
    let cache = AudibleCache::with_ttl_hours(0);
    assert!(cache.is_ok());
}

// Integration tests that require network access
// These are ignored by default - run with: cargo test -- --ignored

#[tokio::test]
#[ignore]
async fn test_real_asin_lookup() {
    // Test real API call to fetch metadata
    // This requires internet connection and may be rate limited

    let client = AudibleClient::new(AudibleRegion::US).unwrap();

    // Use a well-known ASIN (The Martian by Andy Weir)
    let result = client.fetch_by_asin("B00B5HZGUG").await;

    match result {
        Ok(metadata) => {
            assert_eq!(metadata.asin, "B00B5HZGUG");
            assert!(!metadata.title.is_empty());
            assert!(!metadata.authors.is_empty());
            println!("Fetched: {} by {}", metadata.title, metadata.authors_string());
        }
        Err(e) => {
            // API might be down or rate limited
            println!("API call failed (may be rate limited): {}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_real_search() {
    // Test real search API call
    let client = AudibleClient::new(AudibleRegion::US).unwrap();

    let result = client.search(Some("The Martian"), Some("Andy Weir")).await;

    match result {
        Ok(results) => {
            assert!(!results.is_empty());
            println!("Found {} results", results.len());
            for result in results.iter().take(3) {
                println!("  - {} by {}", result.title, result.authors.join(", "));
            }
        }
        Err(e) => {
            println!("Search failed (may be rate limited): {}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_cache_workflow() {
    // Test full cache workflow with real API
    let client = AudibleClient::new(AudibleRegion::US).unwrap();
    let cache = AudibleCache::with_ttl_hours(1).unwrap();

    let asin = "B00B5HZGUG";

    // Clear any existing cache
    let _ = cache.clear(asin);

    // First fetch - should hit API
    let cached = cache.get(asin).await;
    assert!(cached.is_none(), "Cache should be empty initially");

    if let Ok(metadata) = client.fetch_by_asin(asin).await {
        // Store in cache
        cache.set(asin, &metadata).await.unwrap();

        // Second fetch - should hit cache
        let cached = cache.get(asin).await;
        assert!(cached.is_some(), "Cache should have metadata now");

        let cached_metadata = cached.unwrap();
        assert_eq!(cached_metadata.asin, metadata.asin);
        assert_eq!(cached_metadata.title, metadata.title);

        // Clean up
        let _ = cache.clear(asin);
    }
}
