//! Microphone recording configuration

use std::time::Duration;
use whis_core::TranscriptionProvider;

/// Microphone recording configuration
#[derive(Debug, Clone)]
pub struct MicrophoneConfig {
    /// Recording duration (None = interactive)
    pub duration: Option<Duration>,
    /// Disable VAD
    pub no_vad: bool,
    /// Provider (for preloading)
    pub provider: TranscriptionProvider,
    /// Whether post-processing will be used (for preloading)
    pub will_post_process: bool,
}

// Note: MicrophoneMode has been removed as microphone recording now exclusively
// uses the progressive transcription path (see progressive_record_and_transcribe
// in commands/record/mod.rs). The batch recording path only handles file/stdin.
