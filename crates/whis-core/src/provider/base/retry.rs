//! Retry logic with exponential backoff for cloud transcription providers.
//!
//! This module provides retry functionality for transient errors like:
//! - 408 Request Timeout (SLOW_UPLOAD)
//! - 429 Rate Limited
//! - 5xx Server Errors
//! - Network/connection errors

use std::time::Duration;

use reqwest::StatusCode;

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Base delay in milliseconds (doubles with each attempt)
    pub base_delay_ms: u64,
    /// Maximum delay cap in milliseconds
    pub max_delay_ms: u64,
    /// Multiplier for rate limit errors (429)
    pub rate_limit_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000, // 1 second
            max_delay_ms: 16000, // 16 seconds
            rate_limit_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Calculate the delay for a given attempt number
    pub fn delay_for_attempt(&self, attempt: u32, is_rate_limited: bool) -> Duration {
        let base_delay = self.base_delay_ms * 2u64.pow(attempt);
        let delay_ms = base_delay.min(self.max_delay_ms);

        if is_rate_limited {
            Duration::from_millis((delay_ms as f64 * self.rate_limit_multiplier) as u64)
        } else {
            Duration::from_millis(delay_ms)
        }
    }
}

/// Check if an HTTP status code is retryable
pub fn is_retryable_status(status: StatusCode) -> bool {
    matches!(status.as_u16(), 408 | 429 | 500 | 502 | 503 | 504)
}

/// Check if a status code indicates rate limiting
pub fn is_rate_limited(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS
}

/// Check if a reqwest error is retryable
pub fn is_retryable_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect() || err.is_request()
}
