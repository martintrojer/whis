pub mod audio;
#[cfg(feature = "clipboard")]
pub mod clipboard;
pub mod config;
pub mod polish;
pub mod preset;
pub mod provider;
pub mod settings;
pub mod transcribe;

pub use audio::{AudioChunk, AudioRecorder, RecordingData, RecordingOutput};
#[cfg(feature = "clipboard")]
pub use clipboard::copy_to_clipboard;
pub use config::TranscriptionProvider;
pub use polish::{polish, Polisher, DEFAULT_POLISH_PROMPT};
pub use preset::{Preset, PresetSource};
pub use provider::{
    registry, TranscriptionBackend, TranscriptionRequest, TranscriptionResult,
    DEFAULT_TIMEOUT_SECS,
};
pub use settings::Settings;
pub use transcribe::{parallel_transcribe, transcribe_audio, ChunkTranscription};
