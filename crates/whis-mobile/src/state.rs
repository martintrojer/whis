//! Application state management for whis-mobile.
//!
//! This module manages the shared state across Tauri commands, including
//! recording state, audio channels, and transcription configuration.

use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
pub use whis_core::RecordingState;
use whis_core::config::TranscriptionProvider;

/// Cached transcription configuration loaded from Tauri store.
#[derive(Clone)]
pub struct TranscriptionConfig {
    pub provider: TranscriptionProvider,
    pub api_key: String,
    pub language: Option<String>,
}

/// Application state shared across Tauri commands.
///
/// This struct mirrors the pattern used in whis-desktop for consistency.
/// State is wrapped in Arc<Mutex> for thread-safe access from async contexts.
#[derive(Clone)]
pub struct AppState {
    /// Current recording state (Idle, Recording, Transcribing)
    pub recording_state: Arc<Mutex<RecordingState>>,

    /// Channel for progressive transcription audio samples (unbounded for chunker)
    pub audio_tx: Arc<Mutex<Option<mpsc::UnboundedSender<Vec<f32>>>>>,

    /// Receiver for progressive transcription result
    pub transcription_rx: Arc<Mutex<Option<oneshot::Receiver<Result<String, String>>>>>,

    /// Cached transcription config (provider, API key, language)
    pub transcription_config: Arc<Mutex<Option<TranscriptionConfig>>>,

    /// Channel for OpenAI Realtime streaming (bounded, separate from progressive)
    pub realtime_audio_tx: Arc<Mutex<Option<mpsc::Sender<Vec<f32>>>>>,
}

impl AppState {
    /// Create new application state with default values.
    pub fn new() -> Self {
        Self {
            recording_state: Arc::new(Mutex::new(RecordingState::Idle)),
            audio_tx: Arc::new(Mutex::new(None)),
            transcription_rx: Arc::new(Mutex::new(None)),
            transcription_config: Arc::new(Mutex::new(None)),
            realtime_audio_tx: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
