use crate::{RainyError, Result};
use std::time::Duration;
use tokio::time::sleep;

/// Configures the behavior of the automatic retry mechanism.
///
/// This struct defines the parameters for exponential backoff with optional jitter,
/// allowing fine-grained control over how the client handles retryable errors.
///
/// # Examples
///
/// ```
/// # use rainy_sdk::retry::RetryConfig;
/// // A configuration with up to 5 retries.
/// let config = RetryConfig::new(5);
///
/// // A more customized configuration.
/// let custom_config = RetryConfig {
///     max_retries: 4,
///     base_delay_ms: 500, // Start with a 500ms delay
///     max_delay_ms: 60_000, // Cap delays at 1 minute
///     backoff_multiplier: 1.5,
///     jitter: true,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// The maximum number of times to retry a failed request.
    /// A value of 0 means no retries will be attempted.
    pub max_retries: u32,

    /// The initial delay in milliseconds before the first retry.
    pub base_delay_ms: u64,

    /// The absolute maximum delay in milliseconds between retries.
    /// The calculated delay will be capped at this value.
    pub max_delay_ms: u64,

    /// The multiplier for increasing the delay between retries.
    /// A value of 2.0 means the delay doubles with each attempt.
    pub backoff_multiplier: f64,

    /// If `true`, a random jitter (±25%) is added to the delay to prevent
    /// thundering herd problems.
    pub jitter: bool,
}

impl Default for RetryConfig {
    /// Creates a default `RetryConfig`.
    ///
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
    /// and default values for other parameters.
    ///
    /// # Arguments
    ///
    /// * `max_retries` - The maximum number of retries to attempt.
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    /// Calculates the delay duration for a given retry attempt.
    ///
    /// This method implements exponential backoff logic, applying jitter and the
    /// maximum delay cap as configured.
    ///
    /// # Arguments
    ///
    /// * `attempt` - The current retry attempt number (0-indexed).
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

/// Executes an asynchronous operation with retry logic based on the provided configuration.
///
/// This function wraps an async operation (like an API call), automatically retrying it
/// if it fails with a retryable `RainyError`.
///
/// # Type Parameters
///
/// * `F`: The type of the closure that produces the future.
/// * `Fut`: The type of the future returned by the closure.
/// * `T`: The success type of the operation's `Result`.
///
/// # Arguments
///
/// * `config` - The `RetryConfig` to use for the operation.
/// * `operation` - A closure that returns a future representing the async operation.
///
/// # Returns
///
/// A `Result` which is `Ok(T)` if the operation succeeds, or the last `RainyError`
/// if all retry attempts fail.
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

    // This path should ideally not be reached if max_retries >= 0.
    // It returns the last recorded error, or a generic failure message if no error was recorded.
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
