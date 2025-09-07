use crate::{RainyError, Result};
use std::time::Duration;
use tokio::time::sleep;

/// Configuration for retry logic with exponential backoff.
///
/// `RetryConfig` defines the parameters for retrying failed operations,
/// such as the maximum number of retries and the delay between attempts.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// The maximum number of retry attempts to make.
    pub max_retries: u32,

    /// The base delay between retries, in milliseconds. This is the starting point
    /// for the exponential backoff calculation.
    pub base_delay_ms: u64,

    /// The maximum possible delay between retries, in milliseconds.
    pub max_delay_ms: u64,

    /// The multiplier for the exponential backoff. Each subsequent delay is
    /// multiplied by this factor.
    pub backoff_multiplier: f64,

    /// A flag indicating whether to add a random jitter to the delay time.
    /// Jitter helps to prevent a "thundering herd" problem in distributed systems.
    pub jitter: bool,
}

impl Default for RetryConfig {
    /// Creates a default `RetryConfig`.
    ///
    /// The default settings are:
    /// - `max_retries`: 3
    /// - `base_delay_ms`: 1000 (1 second)
    /// - `max_delay_ms`: 30000 (30 seconds)
    /// - `backoff_multiplier`: 2.0
    /// - `jitter`: true
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Creates a new `RetryConfig` with a specified maximum number of retries
    /// and default values for other settings.
    ///
    /// # Arguments
    ///
    /// * `max_retries` - The maximum number of times to retry an operation.
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    /// Calculates the delay duration for a specific retry attempt.
    ///
    /// The delay is calculated using exponential backoff, and optionally includes jitter.
    ///
    /// # Arguments
    ///
    /// * `attempt` - The current retry attempt number (starting from 0).
    ///
    /// # Returns
    ///
    /// A `Duration` to wait before the next attempt.
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_delay = self.base_delay_ms as f64;
        let multiplier = self.backoff_multiplier.powi(attempt as i32);
        let mut delay = base_delay * multiplier;

        // Add jitter if enabled (±25%)
        if self.jitter && attempt > 0 {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let jitter_factor = rng.gen_range(0.75..=1.25);
            delay *= jitter_factor;
        }

        // Cap at maximum delay
        delay = delay.min(self.max_delay_ms as f64);

        Duration::from_millis(delay as u64)
    }
}

/// Executes an asynchronous operation with retry logic based on the provided `RetryConfig`.
///
/// This function will repeatedly call the `operation` closure until it succeeds,
/// or until the maximum number of retries is reached.
///
/// # Type Parameters
///
/// * `F` - The type of the operation, which must be a closure that returns a future.
/// * `Fut` - The type of the future returned by the closure.
/// * `T` - The success type of the `Result` returned by the future.
///
/// # Arguments
///
/// * `config` - The `RetryConfig` to use for the retry logic.
/// * `operation` - The asynchronous operation to execute.
///
/// # Returns
///
/// A `Result` containing the success value `T` if the operation succeeds,
/// or the last `RainyError` if all retry attempts fail.
pub async fn retry_with_backoff<F, Fut, T>(config: &RetryConfig, operation: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                // Check if error is retryable
                if !error.is_retryable() || attempt == config.max_retries {
                    return Err(error);
                }

                // Calculate delay for next attempt
                let delay = config.delay_for_attempt(attempt);

                #[cfg(feature = "tracing")]
                tracing::warn!(
                    "Request failed (attempt {}/{}), retrying in {:?}: {}",
                    attempt + 1,
                    config.max_retries + 1,
                    delay,
                    error
                );

                last_error = Some(error);

                // Wait before retrying
                if attempt < config.max_retries {
                    sleep(delay).await;
                }
            }
        }
    }

    // This should never be reached, but just in case
    Err(last_error.unwrap_or_else(|| RainyError::Network {
        message: "All retry attempts failed".to_string(),
        retryable: false,
        source_error: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_calculation() {
        let config = RetryConfig::default();

        // Test delay progression
        let delay0 = config.delay_for_attempt(0);
        let delay1 = config.delay_for_attempt(1);
        let delay2 = config.delay_for_attempt(2);

        assert!(delay0.as_millis() >= 1000);
        assert!(delay1.as_millis() >= delay0.as_millis());
        assert!(delay2.as_millis() >= delay1.as_millis());
        assert!(delay2.as_millis() <= 30000);
    }
}
