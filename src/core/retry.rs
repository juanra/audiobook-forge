//! Error recovery and retry logic

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier (exponential backoff)
    pub backoff_multiplier: f64,
}

impl RetryConfig {
    /// Create a new retry config with sensible defaults
    pub fn new() -> Self {
        Self {
            max_retries: 2,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }

    /// Create a retry config with custom settings
    pub fn with_settings(
        max_retries: usize,
        initial_delay: Duration,
        max_delay: Duration,
        backoff_multiplier: f64,
    ) -> Self {
        Self {
            max_retries,
            initial_delay,
            max_delay,
            backoff_multiplier,
        }
    }

    /// No retries
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            initial_delay: Duration::from_secs(0),
            max_delay: Duration::from_secs(0),
            backoff_multiplier: 1.0,
        }
    }

    /// Calculate delay for retry attempt
    fn calculate_delay(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            return self.initial_delay;
        }

        let delay_secs = self.initial_delay.as_secs_f64()
            * self.backoff_multiplier.powi(attempt as i32);

        let delay = Duration::from_secs_f64(delay_secs);

        // Clamp to max_delay
        if delay > self.max_delay {
            self.max_delay
        } else {
            delay
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute a function with retry logic
pub async fn retry_async<F, Fut, T>(config: &RetryConfig, mut f: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match f().await {
            Ok(result) => {
                if attempt > 0 {
                    tracing::info!("Retry successful after {} attempt(s)", attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                last_error = Some(e);

                if attempt < config.max_retries {
                    let delay = config.calculate_delay(attempt);
                    tracing::warn!(
                        "Attempt {} failed, retrying in {:?}...",
                        attempt + 1,
                        delay
                    );
                    sleep(delay).await;
                } else {
                    tracing::error!("All {} retry attempts failed", config.max_retries + 1);
                }
            }
        }
    }

    // If we get here, all retries failed
    Err(last_error.unwrap())
}

/// Error classification for smart retry logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    /// Transient errors (worth retrying)
    Transient,
    /// Permanent errors (no point retrying)
    Permanent,
}

/// Classify an error to determine if retry is worthwhile
pub fn classify_error(error: &anyhow::Error) -> ErrorType {
    let error_msg = error.to_string().to_lowercase();

    // Transient errors (worth retrying)
    if error_msg.contains("timeout")
        || error_msg.contains("connection")
        || error_msg.contains("temporarily unavailable")
        || error_msg.contains("too many open files")
        || error_msg.contains("resource temporarily unavailable")
    {
        return ErrorType::Transient;
    }

    // Permanent errors (no point retrying)
    if error_msg.contains("file not found")
        || error_msg.contains("permission denied")
        || error_msg.contains("invalid")
        || error_msg.contains("unsupported")
        || error_msg.contains("corrupted")
    {
        return ErrorType::Permanent;
    }

    // Default to transient (conservative approach)
    ErrorType::Transient
}

/// Execute with smart retry (only retry transient errors)
pub async fn smart_retry_async<F, Fut, T>(config: &RetryConfig, mut f: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match f().await {
            Ok(result) => {
                if attempt > 0 {
                    tracing::info!("Smart retry successful after {} attempt(s)", attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                let error_type = classify_error(&e);

                if error_type == ErrorType::Permanent {
                    tracing::error!("Permanent error detected, not retrying: {}", e);
                    return Err(e);
                }

                last_error = Some(e);

                if attempt < config.max_retries {
                    let delay = config.calculate_delay(attempt);
                    tracing::warn!(
                        "Transient error (attempt {}), retrying in {:?}...",
                        attempt + 1,
                        delay
                    );
                    sleep(delay).await;
                } else {
                    tracing::error!(
                        "All {} retry attempts failed for transient error",
                        config.max_retries + 1
                    );
                }
            }
        }
    }

    Err(last_error.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_config_creation() {
        let config = RetryConfig::new();
        assert_eq!(config.max_retries, 2);
        assert_eq!(config.initial_delay, Duration::from_secs(1));
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_retry_config_no_retry() {
        let config = RetryConfig::no_retry();
        assert_eq!(config.max_retries, 0);
    }

    #[test]
    fn test_calculate_delay() {
        let config = RetryConfig::new();

        assert_eq!(config.calculate_delay(0), Duration::from_secs(1));
        assert_eq!(config.calculate_delay(1), Duration::from_secs(2));
        assert_eq!(config.calculate_delay(2), Duration::from_secs(4));
        assert_eq!(config.calculate_delay(3), Duration::from_secs(8));

        // Test max delay clamping
        let config = RetryConfig::with_settings(
            5,
            Duration::from_secs(1),
            Duration::from_secs(5),
            2.0,
        );
        assert_eq!(config.calculate_delay(10), Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_retry_async_success_first_try() {
        let config = RetryConfig::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let result: Result<i32> = retry_async(&config, || {
            let counter = Arc::clone(&counter);
            async move {
                counter.fetch_add(1, Ordering::Relaxed);
                Ok::<i32, anyhow::Error>(42)
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_retry_async_success_after_retries() {
        let config = RetryConfig::with_settings(
            3,
            Duration::from_millis(10),
            Duration::from_millis(100),
            2.0,
        );
        let counter = Arc::new(AtomicUsize::new(0));

        let result = retry_async(&config, || {
            let counter = Arc::clone(&counter);
            async move {
                let count = counter.fetch_add(1, Ordering::Relaxed);
                if count < 2 {
                    anyhow::bail!("Transient error");
                }
                Ok::<i32, anyhow::Error>(42)
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn test_retry_async_all_fail() {
        let config = RetryConfig::with_settings(
            2,
            Duration::from_millis(10),
            Duration::from_millis(100),
            2.0,
        );
        let counter = Arc::new(AtomicUsize::new(0));

        let result: Result<i32> = retry_async(&config, || {
            let counter = Arc::clone(&counter);
            async move {
                counter.fetch_add(1, Ordering::Relaxed);
                anyhow::bail!("Always fails")
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::Relaxed), 3); // Initial + 2 retries
    }

    #[test]
    fn test_classify_error() {
        let transient = anyhow::anyhow!("Connection timeout");
        assert_eq!(classify_error(&transient), ErrorType::Transient);

        let permanent = anyhow::anyhow!("File not found");
        assert_eq!(classify_error(&permanent), ErrorType::Permanent);

        let unknown = anyhow::anyhow!("Some random error");
        assert_eq!(classify_error(&unknown), ErrorType::Transient);
    }

    #[tokio::test]
    async fn test_smart_retry_permanent_error() {
        let config = RetryConfig::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let result: Result<i32> = smart_retry_async(&config, || {
            let counter = Arc::clone(&counter);
            async move {
                counter.fetch_add(1, Ordering::Relaxed);
                anyhow::bail!("File not found")
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::Relaxed), 1); // No retries for permanent error
    }

    #[tokio::test]
    async fn test_smart_retry_transient_error() {
        let config = RetryConfig::with_settings(
            2,
            Duration::from_millis(10),
            Duration::from_millis(100),
            2.0,
        );
        let counter = Arc::new(AtomicUsize::new(0));

        let result = smart_retry_async(&config, || {
            let counter = Arc::clone(&counter);
            async move {
                let count = counter.fetch_add(1, Ordering::Relaxed);
                if count < 2 {
                    anyhow::bail!("Connection timeout");
                }
                Ok::<i32, anyhow::Error>(42)
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::Relaxed), 3);
    }
}
