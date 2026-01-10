//! Transcription pipeline and related utilities.
//!
//! This module contains:
//! - Transcription functions (batch and progressive)
//! - Ollama integration for local LLM
//! - Post-processing with LLM cleanup
//! - Connection warmup utilities

mod ollama;
mod ollama_manager;
mod post_processing;
mod transcribe;
mod warmup;

pub use ollama::{
    DEFAULT_OLLAMA_MODEL, DEFAULT_OLLAMA_URL, OLLAMA_MODEL_OPTIONS, OllamaModel,
    ensure_ollama_ready, ensure_ollama_running, has_model, is_ollama_installed, is_ollama_running,
    list_models, pull_model, pull_model_with_progress,
};
pub use ollama_manager::{clear_warmup_cache, preload_ollama};
pub use post_processing::{DEFAULT_POST_PROCESSING_PROMPT, PostProcessor, post_process};
pub use transcribe::{
    ChunkTranscription, batch_transcribe, progressive_transcribe_cloud, transcribe_audio,
    transcribe_audio_async, transcribe_audio_with_format, transcribe_audio_with_progress,
};
#[cfg(feature = "local-transcription")]
pub use transcribe::{LocalAudioChunk, progressive_transcribe_local};
pub use warmup::{WarmupConfig, warmup_configured};
