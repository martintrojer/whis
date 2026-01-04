//! Audio-specific error types

use std::fmt;

/// Errors that can occur during audio operations
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    /// Device not found or unavailable
    #[error("Audio device not found: {0}")]
    DeviceNotFound(String),

    /// Failed to initialize or start recording
    #[error("Recording failed: {0}")]
    RecordingFailed(String),

    /// Failed to encode audio data
    #[error("Encoding failed: {0}")]
    EncodingFailed(String),

    /// Failed to load audio from source
    #[error("Failed to load audio: {0}")]
    LoadFailed(String),

    /// Invalid audio stream configuration
    #[error("Invalid stream configuration: {0}")]
    InvalidConfig(String),

    /// Resampling error
    #[error("Resampling error: {0}")]
    ResamplingError(String),

    /// VAD processing error
    #[error("VAD processing error: {0}")]
    VadError(String),

    /// I/O error during audio operations
    #[error("Audio I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic audio error
    #[error("{0}")]
    Other(String),
}

impl AudioError {
    /// Create a device not found error
    pub fn device_not_found(device: impl fmt::Display) -> Self {
        Self::DeviceNotFound(device.to_string())
    }

    /// Create a recording failed error
    pub fn recording_failed(msg: impl fmt::Display) -> Self {
        Self::RecordingFailed(msg.to_string())
    }

    /// Create an encoding failed error
    pub fn encoding_failed(msg: impl fmt::Display) -> Self {
        Self::EncodingFailed(msg.to_string())
    }
}

// Allow converting from cpal errors
impl From<cpal::BuildStreamError> for AudioError {
    fn from(err: cpal::BuildStreamError) -> Self {
        AudioError::RecordingFailed(err.to_string())
    }
}

impl From<cpal::PlayStreamError> for AudioError {
    fn from(err: cpal::PlayStreamError) -> Self {
        AudioError::RecordingFailed(err.to_string())
    }
}

impl From<cpal::DevicesError> for AudioError {
    fn from(err: cpal::DevicesError) -> Self {
        AudioError::DeviceNotFound(err.to_string())
    }
}
