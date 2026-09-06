//! Filesystem cache for Audible metadata

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::models::AudibleMetadata;

/// Filesystem cache for Audible metadata
pub struct AudibleCache {
    cache_dir: PathBuf,
    ttl: Duration,
}

impl AudibleCache {
    /// Create a new cache with default settings (7 days TTL)
    pub fn new() -> Result<Self> {
        Self::with_ttl(Duration::from_secs(7 * 24 * 3600))
    }

    /// Create a new cache with custom TTL, in the default per-user cache directory
    pub fn with_ttl(ttl: Duration) -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .context("No cache directory found")?
            .join("audiobook-forge")
            .join("audible");

        Self::with_cache_dir(cache_dir, ttl)
    }

    /// Create a cache rooted at an explicit directory.
    ///
    /// Lets a caller keep a cache outside the per-user location, and lets tests
    /// run against an isolated directory instead of sharing (and mutating) the
    /// developer's real cache.
    pub fn with_cache_dir(cache_dir: std::path::PathBuf, ttl: Duration) -> Result<Self> {
        // Create cache directory if it doesn't exist
        std::fs::create_dir_all(&cache_dir)
            .context("Failed to create cache directory")?;

        Ok(Self { cache_dir, ttl })
    }

    /// Create a new cache with TTL from config (in hours)
    pub fn with_ttl_hours(hours: u64) -> Result<Self> {
        if hours == 0 {
            // No caching - use 0 duration
            Self::with_ttl(Duration::from_secs(0))
        } else {
            Self::with_ttl(Duration::from_secs(hours * 3600))
        }
    }

    /// Get cached metadata for an ASIN
    pub async fn get(&self, asin: &str) -> Option<AudibleMetadata> {
        // If TTL is 0, caching is disabled
        if self.ttl.as_secs() == 0 {
            return None;
        }

        let cache_path = self.cache_path(asin);

        if !cache_path.exists() {
            tracing::debug!("Cache miss for ASIN: {}", asin);
            return None;
        }

        // Check if cache is expired
        if let Ok(metadata) = std::fs::metadata(&cache_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    if elapsed > self.ttl {
                        tracing::debug!("Cache expired for ASIN: {} (age: {:?})", asin, elapsed);
                        // Clean up expired cache file
                        let _ = std::fs::remove_file(&cache_path);
                        return None;
                    }
                }
            }
        }

        // Read and deserialize cache file
        match tokio::fs::read_to_string(&cache_path).await {
            Ok(content) => match serde_json::from_str::<AudibleMetadata>(&content) {
                Ok(metadata) => {
                    tracing::debug!("Cache hit for ASIN: {}", asin);
                    Some(metadata)
                }
                Err(e) => {
                    tracing::warn!("Failed to parse cache file for {}: {}", asin, e);
                    // Clean up corrupted cache file
                    let _ = std::fs::remove_file(&cache_path);
                    None
                }
            },
            Err(e) => {
                tracing::debug!("Failed to read cache file for {}: {}", asin, e);
                None
            }
        }
    }

    /// Store metadata in cache for an ASIN
    pub async fn set(&self, asin: &str, metadata: &AudibleMetadata) -> Result<()> {
        // If TTL is 0, caching is disabled
        if self.ttl.as_secs() == 0 {
            return Ok(());
        }

        let cache_path = self.cache_path(asin);

        let json = serde_json::to_string_pretty(metadata)
            .context("Failed to serialize metadata")?;

        tokio::fs::write(&cache_path, json)
            .await
            .context("Failed to write cache file")?;

        tracing::debug!("Cached metadata for ASIN: {} at {}", asin, cache_path.display());

        Ok(())
    }

    /// Clear cache for a specific ASIN
    pub fn clear(&self, asin: &str) -> Result<()> {
        let cache_path = self.cache_path(asin);

        if cache_path.exists() {
            std::fs::remove_file(&cache_path)
                .context("Failed to remove cache file")?;
            tracing::debug!("Cleared cache for ASIN: {}", asin);
        }

        Ok(())
    }

    /// Clear all cached metadata
    pub fn clear_all(&self) -> Result<()> {
        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir)? {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
            tracing::debug!("Cleared all Audible cache");
        }

        Ok(())
    }

    /// Get the cache file path for an ASIN
    fn cache_path(&self, asin: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", asin))
    }

    /// Get cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get cache statistics (number of files, total size)
    pub fn stats(&self) -> Result<CacheStats> {
        let mut count = 0;
        let mut total_size = 0u64;

        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir)? {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                        count += 1;
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            total_size += metadata.len();
                        }
                    }
                }
            }
        }

        Ok(CacheStats {
            file_count: count,
            total_size_bytes: total_size,
        })
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub file_count: usize,
    pub total_size_bytes: u64,
}

impl CacheStats {
    /// Get total size in megabytes
    pub fn size_mb(&self) -> f64 {
        self.total_size_bytes as f64 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AudibleAuthor, AudibleSeries};

    fn create_test_metadata() -> AudibleMetadata {
        AudibleMetadata {
            asin: "B001".to_string(),
            title: "Test Book".to_string(),
            subtitle: None,
            authors: vec![AudibleAuthor {
                asin: None,
                name: "Test Author".to_string(),
            }],
            narrators: vec!["Test Narrator".to_string()],
            publisher: Some("Test Publisher".to_string()),
            published_year: Some(2020),
            description: Some("Test description".to_string()),
            cover_url: None,
            isbn: None,
            genres: vec!["Fiction".to_string()],
            tags: vec![],
            series: vec![],
            language: Some("English".to_string()),
            runtime_length_ms: Some(3600000),
            rating: Some(4.5),
            is_abridged: Some(false),
        }
    }

    /// An isolated cache directory, so tests never read or mutate the developer's
    /// real cache and cannot race each other over shared ASIN keys.
    fn test_cache(label: &str, ttl: Duration) -> (AudibleCache, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "audiobook-forge-cache-test-{}-{}",
            std::process::id(),
            label
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let cache = AudibleCache::with_cache_dir(dir.clone(), ttl).unwrap();
        (cache, dir)
    }

    const TEST_TTL: Duration = Duration::from_secs(7 * 24 * 3600);

    #[tokio::test]
    async fn test_cache_set_and_get() {
        let (cache, dir) = test_cache("set-and-get", TEST_TTL);
        let metadata = create_test_metadata();

        cache.set("B001", &metadata).await.unwrap();

        let cached = cache.get("B001").await;
        assert!(cached.is_some());

        let cached = cached.unwrap();
        assert_eq!(cached.asin, "B001");
        assert_eq!(cached.title, "Test Book");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let (cache, dir) = test_cache("miss", TEST_TTL);

        let cached = cache.get("NONEXISTENT").await;
        assert!(cached.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_cache_disabled() {
        let (cache, dir) = test_cache("disabled", Duration::from_secs(0));
        let metadata = create_test_metadata();

        // Set cache (should be no-op)
        cache.set("B001", &metadata).await.unwrap();

        // Get cache (should return None since caching is disabled)
        let cached = cache.get("B001").await;
        assert!(cached.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_cache_stats_counts_written_entries() {
        // `file_count` and `total_size_bytes` are unsigned, so the previous
        // `>= 0` assertions were always true and proved nothing. Assert instead
        // that stats actually track what was written.
        let (cache, dir) = test_cache("stats", TEST_TTL);

        let before = cache.stats().unwrap();
        assert_eq!(before.file_count, 0);
        assert_eq!(before.total_size_bytes, 0);

        cache.set("B001", &create_test_metadata()).await.unwrap();

        let after = cache.stats().unwrap();
        assert_eq!(after.file_count, 1, "one cached entry should be counted");
        assert!(
            after.total_size_bytes > 0,
            "a written entry should contribute bytes"
        );
        assert!(after.size_mb() > 0.0);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
