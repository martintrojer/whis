//! Unified error types for whis-core
//!
//! This module provides a hierarchical error type system that replaces
//! scattered use of `anyhow::Error` with domain-specific error types.
//!
//! # Error Hierarchy
//!
//! ```text
//! WhisError
//!   ├── Audio(AudioError)       - Recording, encoding, device errors
//!   ├── Provider(ProviderError) - Transcription provider errors
//!   ├── Config(String)          - Configuration errors
//!   ├── Model(ModelError)       - Model download/verification errors
//!   └── Io(std::io::Error)      - Generic I/O errors
//! ```
//!
//! # Migration Strategy
//!
//! This error system is designed for gradual migration from `anyhow::Result`:
//!
//! 1. **Phase 1** (current): Error types defined, re-exported at crate root
//! 2. **Phase 2**: Migrate audio and provider modules to use typed errors
//! 3. **Phase 3**: Update public APIs to return `Result<T, WhisError>`
//! 4. **Phase 4**: Remove `anyhow` dependency from whis-core
//!
//! # Backward Compatibility
//!
//! During migration, both error systems coexist:
//! - New code uses `WhisError`
//! - Existing code continues using `anyhow::Error`
//! - Conversions provided via `From` implementations

// Re-export domain-specific error types for convenience
pub use crate::audio::AudioError;
pub use crate::provider::ProviderError;

/// Top-level error type for whis-core operations
#[derive(Debug, thiserror::Error)]
pub enum WhisError {
    /// Audio-related errors (recording, encoding, devices)
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),

    /// Provider-related errors (transcription, API keys)
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Model download or verification errors
    #[error("Model error: {0}")]
    Model(String),

    /// Settings-related errors
    #[error("Settings error: {0}")]
    Settings(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error (for gradual migration from anyhow)
    #[error("{0}")]
    Other(String),
}

impl WhisError {
    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a model error
    pub fn model(msg: impl Into<String>) -> Self {
        Self::Model(msg.into())
    }

    /// Create a settings error
    pub fn settings(msg: impl Into<String>) -> Self {
        Self::Settings(msg.into())
    }

    /// Create a generic error
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

/// Convenience result type using WhisError
pub type Result<T> = std::result::Result<T, WhisError>;

// Allow converting from anyhow::Error during migration
impl From<anyhow::Error> for WhisError {
    fn from(err: anyhow::Error) -> Self {
        WhisError::Other(err.to_string())
    }
}

// Note: WhisError automatically implements Into<anyhow::Error>
// because it implements std::error::Error via thiserror
