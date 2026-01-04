//! No-op VAD processor implementation (used when VAD feature is disabled)
//!
//! Provides the same API as the real VAD processor, but acts as a passthrough
//! that doesn't perform any voice activity detection.

use anyhow::Result;

/// VAD chunk size constant (for API compatibility)
pub const VAD_CHUNK_SIZE: usize = 512;

/// VAD state information for external queries (no-op version)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VadState {
    /// Whether speech is currently being detected (always false for no-op)
    pub is_speaking: bool,
    /// Whether in hangover period (always false for no-op)
    pub in_hangover: bool,
}

impl VadState {
    /// Returns true if VAD is in complete silence (always true for no-op)
    pub fn is_silence(&self) -> bool {
        true
    }
}

/// No-op Voice Activity Detection processor
///
/// This implementation is used when the "vad" feature is disabled.
/// It provides the same API as the real VadProcessor but acts as a passthrough.
#[derive(Debug, Clone)]
pub struct VadProcessor;

impl VadProcessor {
    /// Create a new no-op VAD processor
    pub fn new(_enabled: bool, _threshold: f32) -> Result<Self> {
        Ok(Self)
    }

    /// Create a disabled VAD processor (same as new for no-op)
    pub fn disabled() -> Result<Self> {
        Ok(Self)
    }

    /// Check if VAD is enabled (always false for no-op)
    pub fn is_enabled(&self) -> bool {
        false
    }

    /// Check if VAD is currently detecting complete silence (always true for no-op)
    pub fn is_silence(&self) -> bool {
        true
    }

    /// Get current VAD state (always returns silence for no-op)
    pub fn state(&self) -> VadState {
        VadState {
            is_speaking: false,
            in_hangover: false,
        }
    }

    /// Process audio samples (passthrough - returns all samples)
    pub fn process(&mut self, samples: &[f32]) -> Vec<f32> {
        samples.to_vec()
    }

    /// Reset the VAD state (no-op)
    pub fn reset(&mut self) {
        // No-op
    }

    /// Flush remaining buffered samples (no-op - returns empty vec)
    pub fn flush(&mut self) -> Vec<f32> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_vad_passthrough() {
        let mut vad = VadProcessor::new(true, 0.5).unwrap();
        let samples = vec![0.1, 0.2, 0.3];
        let output = vad.process(&samples);
        assert_eq!(output, samples);
    }

    #[test]
    fn test_noop_vad_disabled() {
        let vad = VadProcessor::disabled().unwrap();
        assert!(!vad.is_enabled());
    }
}
