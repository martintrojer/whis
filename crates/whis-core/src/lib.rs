pub mod audio;
#[cfg(feature = "clipboard")]
pub mod clipboard;
pub mod config;
pub mod model;
pub mod ollama;
pub mod post_processing;
pub mod preset;
pub mod provider;
pub mod resample;
pub mod settings;
pub mod state;
pub mod transcribe;
pub mod verbose;

pub use audio::{
    AudioChunk, AudioDeviceInfo, AudioRecorder, RecordingData, RecordingOutput, list_audio_devices,
    load_audio_file, load_audio_stdin,
};
#[cfg(feature = "clipboard")]
pub use clipboard::{ClipboardMethod, copy_to_clipboard};
pub use config::TranscriptionProvider;
pub use post_processing::{DEFAULT_POST_PROCESSING_PROMPT, PostProcessor, post_process};
pub use preset::{Preset, PresetSource};
pub use provider::{
    DEFAULT_TIMEOUT_SECS, TranscriptionBackend, TranscriptionRequest, TranscriptionResult, registry,
};
pub use settings::Settings;
pub use state::RecordingState;
pub use transcribe::{
    ChunkTranscription, parallel_transcribe, transcribe_audio, transcribe_audio_with_format,
};
pub use verbose::set_verbose;
