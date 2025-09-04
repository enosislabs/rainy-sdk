use crate::error::{RainyError, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub api_key: Option<String>,
    pub base_url: String,
    pub timeout: Duration,
    pub user_agent: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: "https://api.enosislabs.com".to_string(),
            timeout: Duration::from_secs(30),
            user_agent: format!("rainy-sdk/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

impl AuthConfig {
    /// Creates a new AuthConfig with default values.
    /// 
    /// The base URL is automatically set to `https://api.enosislabs.com`,
    /// so you typically only need to call `.with_api_key()` or `.with_admin_key()`.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use rainy_sdk::{AuthConfig, RainyClient};
    /// 
    /// // Simple API key authentication
    /// let client = RainyClient::new(
    ///     AuthConfig::new().with_api_key("your-api-key")
    /// )?;
    /// # Ok::<(), rainy_sdk::error::RainyError>(())
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the API key for authentication.
    /// 
    /// This is the primary authentication method for regular users.
    pub fn with_api_key<S: Into<String>>(mut self, api_key: S) -> Self {
        self.api_key = Some(api_key.into());
        self
    }


    /// Overrides the default base URL.
    /// 
    /// By default, the SDK connects to `https://api.enosislabs.com`.
    /// Use this method only if you need to connect to a different endpoint.
    pub fn with_base_url<S: Into<String>>(mut self, base_url: S) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        // Set User-Agent
        headers.insert(USER_AGENT, HeaderValue::from_str(&self.user_agent)?);

        // Set Content-Type for JSON requests
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        // Set authorization header
        if let Some(api_key) = &self.api_key {
            let auth_value = format!("Bearer {api_key}");
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);
        } else {
            return Err(RainyError::Config(
                "API key is required".to_string(),
            ));
        }

        Ok(headers)
    }

    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_none() {
            return Err(RainyError::Config(
                "API key must be provided".to_string(),
            ));
        }

        if self.base_url.is_empty() {
            return Err(RainyError::Config("Base URL cannot be empty".to_string()));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct RateLimiter {
    requests_per_minute: u32,
    last_request: Instant,
    request_count: u32,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            last_request: Instant::now(),
            request_count: 0,
        }
    }

    pub async fn wait_if_needed(&mut self) -> Result<()> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_request);

        // Reset counter if a minute has passed
        if elapsed >= Duration::from_secs(60) {
            self.request_count = 0;
            self.last_request = now;
        }

        // Check if we've exceeded the rate limit
        if self.request_count >= self.requests_per_minute {
            let wait_time = Duration::from_secs(60) - elapsed;
            tokio::time::sleep(wait_time).await;
            self.request_count = 0;
            self.last_request = Instant::now();
        }

        self.request_count += 1;
        Ok(())
    }
}
