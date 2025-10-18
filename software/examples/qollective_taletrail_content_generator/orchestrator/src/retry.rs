//! Retry logic with exponential backoff for orchestrator operations
//!
//! This module provides resilience for MCP service calls by retrying failed
//! operations with configurable exponential backoff.

use std::time::Duration;
use tokio::time::sleep;
use tracing::{warn, info, instrument};

/// Configuration for retry behavior with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (including initial attempt)
    pub max_attempts: u32,
    /// Initial delay before first retry in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
    /// Multiplier for exponential backoff (e.g., 2.0 for doubling)
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry an async operation with exponential backoff
///
/// # Arguments
/// * `operation` - Async function to retry
/// * `config` - Retry configuration parameters
/// * `operation_name` - Human-readable operation name for logging
///
/// # Returns
/// Result from the operation, or the last error if all retries exhausted
///
/// # Example
/// ```no_run
/// use orchestrator::retry::{retry_with_backoff, RetryConfig};
///
/// async fn example() {
///     let config = RetryConfig::default();
///     let result = retry_with_backoff(
///         || async { Ok::<_, std::io::Error>(42) },
///         &config,
///         "example_operation",
///     ).await;
/// }
/// ```
#[instrument(skip(operation), fields(operation_name = %operation_name, attempts = 0))]
pub async fn retry_with_backoff<F, T, E, Fut>(
    operation: F,
    config: &RetryConfig,
    operation_name: &str,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay = Duration::from_millis(config.initial_delay_ms);

    loop {
        attempt += 1;
        tracing::Span::current().record("attempts", attempt);

        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    info!(
                        operation = operation_name,
                        attempt = attempt,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(e) if attempt >= config.max_attempts => {
                warn!(
                    operation = operation_name,
                    attempt = attempt,
                    max_attempts = config.max_attempts,
                    error = %e,
                    "Operation failed after max retry attempts"
                );
                return Err(e);
            }
            Err(e) => {
                warn!(
                    operation = operation_name,
                    attempt = attempt,
                    max_attempts = config.max_attempts,
                    delay_ms = delay.as_millis(),
                    error = %e,
                    "Operation failed, will retry"
                );

                sleep(delay).await;

                // Calculate next delay with exponential backoff
                let next_delay_ms = (delay.as_millis() as f64 * config.backoff_multiplier)
                    .min(config.max_delay_ms as f64) as u64;
                delay = Duration::from_millis(next_delay_ms);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_succeeds_on_second_attempt() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let result = retry_with_backoff(
            || {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                async move {
                    if count == 0 {
                        Err("First attempt fails")
                    } else {
                        Ok("Success")
                    }
                }
            },
            &config,
            "test_operation",
        ).await;

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 2, "Should take 2 attempts");
    }

    #[tokio::test]
    async fn test_retry_fails_after_max_attempts() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let result = retry_with_backoff(
            || async { Err::<(), &str>("Always fails") },
            &config,
            "test_operation",
        ).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_exponential_backoff_timing() {
        use std::time::Instant;

        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 50,
            max_delay_ms: 500,
            backoff_multiplier: 2.0,
        };

        let start = Instant::now();

        let _ = retry_with_backoff(
            || async { Err::<(), &str>("Always fails") },
            &config,
            "test_operation",
        ).await;

        let elapsed = start.elapsed();

        // Should take at least: 50ms + 100ms = 150ms
        assert!(elapsed.as_millis() >= 150, "Should use exponential backoff delays");
    }

    #[tokio::test]
    async fn test_successful_first_attempt_no_retry() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let config = RetryConfig::default();

        let result = retry_with_backoff(
            || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                async move { Ok::<_, &str>("Success") }
            },
            &config,
            "test_operation",
        ).await;

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 1, "Should only attempt once");
    }

    #[tokio::test]
    async fn test_max_delay_caps_backoff() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 150,  // Cap at 150ms
            backoff_multiplier: 2.0,
        };

        let start = std::time::Instant::now();

        let _ = retry_with_backoff(
            || async { Err::<(), &str>("Always fails") },
            &config,
            "test_operation",
        ).await;

        let elapsed = start.elapsed();

        // Delays: 100ms, 150ms (capped), 150ms (capped), 150ms (capped) = 550ms total
        // Allow some margin for test execution overhead
        assert!(elapsed.as_millis() >= 500, "Should respect max_delay cap");
        assert!(elapsed.as_millis() < 700, "Should not exceed expected max delay");
    }
}
