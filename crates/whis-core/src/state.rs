//! Shared application state types

/// Recording state for UI applications
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum RecordingState {
    Idle,
    Recording,
    Transcribing,
}
