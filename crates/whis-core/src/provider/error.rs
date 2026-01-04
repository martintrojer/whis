//! Provider-specific error types

use crate::TranscriptionProvider;
use std::fmt;

/// Errors that can occur during transcription provider operations
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// Provider not found in registry
    #[error("Provider not found: {0}")]
    NotFound(String),

    /// API key is missing for a provider that requires one
    #[error("API key missing for {provider}")]
    MissingApiKey { provider: String },

    /// Invalid API key format
    #[error("Invalid API key format for {provider}: {reason}")]
    InvalidApiKey { provider: String, reason: String },

    /// Transcription request failed
    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),

    /// Network/HTTP error during API call
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Invalid response from provider
    #[error("Invalid response from provider: {0}")]
    InvalidResponse(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded for provider {0}")]
    RateLimitExceeded(String),

    /// Provider-specific error
    #[error("{provider} error: {message}")]
    ProviderSpecific { provider: String, message: String },

    /// Local model not found or invalid
    #[error("Local model error: {0}")]
    LocalModelError(String),

    /// I/O error during provider operations
    #[error("Provider I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic provider error
    #[error("{0}")]
    Other(String),
}

impl ProviderError {
    /// Create a missing API key error
    pub fn missing_api_key(provider: &TranscriptionProvider) -> Self {
        Self::MissingApiKey {
            provider: provider.display_name().to_string(),
        }
    }

    /// Create an invalid API key error
    pub fn invalid_api_key(provider: &TranscriptionProvider, reason: impl fmt::Display) -> Self {
        Self::InvalidApiKey {
            provider: provider.display_name().to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a transcription failed error
    pub fn transcription_failed(msg: impl fmt::Display) -> Self {
        Self::TranscriptionFailed(msg.to_string())
    }

    /// Create a network error
    pub fn network_error(msg: impl fmt::Display) -> Self {
        Self::NetworkError(msg.to_string())
    }

    /// Create a provider-specific error
    pub fn provider_specific(provider: impl fmt::Display, message: impl fmt::Display) -> Self {
        Self::ProviderSpecific {
            provider: provider.to_string(),
            message: message.to_string(),
        }
    }
}

// Allow converting from reqwest errors
impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ProviderError::NetworkError(format!("Request timeout: {}", err))
        } else if err.is_connect() {
            ProviderError::NetworkError(format!("Connection failed: {}", err))
        } else if err.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
            ProviderError::RateLimitExceeded("API".to_string())
        } else {
            ProviderError::NetworkError(err.to_string())
        }
    }
}
