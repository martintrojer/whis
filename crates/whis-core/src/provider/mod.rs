//! Transcription provider trait and registry
//!
//! This module provides an extensible architecture for adding new transcription providers.
//! Each provider implements the `TranscriptionBackend` trait.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

/// Stages of the transcription workflow for progress reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptionStage {
    /// Recording audio from microphone
    Recording,
    /// Encoding audio to MP3
    Encoding,
    /// Uploading audio to cloud provider
    Uploading,
    /// Transcribing audio (cloud processing or local inference)
    Transcribing,
    /// Post-processing transcript with LLM
    PostProcessing,
    /// Transcription complete
    Complete,
}

impl TranscriptionStage {
    /// Get a human-readable status message for this stage
    pub fn message(&self) -> &'static str {
        match self {
            Self::Recording => "Recording...",
            Self::Encoding => "Encoding...",
            Self::Uploading => "Uploading...",
            Self::Transcribing => "Transcribing...",
            Self::PostProcessing => "Post-processing...",
            Self::Complete => "Done!",
        }
    }
}

/// Progress callback type for reporting transcription stages
pub type ProgressCallback = Arc<dyn Fn(TranscriptionStage) + Send + Sync>;

mod deepgram;
mod elevenlabs;
mod groq;
#[cfg(feature = "local-transcription")]
mod local_parakeet;
#[cfg(feature = "local-transcription")]
mod local_whisper;
mod mistral;
mod openai;
#[cfg(feature = "realtime")]
mod openai_realtime;

/// Default timeout for API requests (5 minutes)
pub const DEFAULT_TIMEOUT_SECS: u64 = 300;

pub use deepgram::DeepgramProvider;
pub use elevenlabs::ElevenLabsProvider;
pub use groq::GroqProvider;
#[cfg(feature = "local-transcription")]
pub use local_parakeet::LocalParakeetProvider;
#[cfg(feature = "local-transcription")]
pub use local_parakeet::transcribe_raw as transcribe_raw_parakeet;
#[cfg(feature = "local-transcription")]
pub use local_whisper::LocalWhisperProvider;
#[cfg(feature = "local-transcription")]
pub use local_whisper::transcribe_raw;
pub use mistral::MistralProvider;
pub use openai::OpenAIProvider;
#[cfg(feature = "realtime")]
pub use openai_realtime::OpenAIRealtimeProvider;

use crate::config::TranscriptionProvider;

/// Request data for transcription
#[derive(Clone)]
pub struct TranscriptionRequest {
    pub audio_data: Vec<u8>,
    pub language: Option<String>,
    pub filename: String,
    pub mime_type: String,
    /// Optional progress callback for status updates
    pub progress: Option<ProgressCallback>,
}

impl TranscriptionRequest {
    /// Create a new request without progress callback
    pub fn new(audio_data: Vec<u8>, language: Option<String>) -> Self {
        Self {
            audio_data,
            language,
            filename: "audio.mp3".to_string(),
            mime_type: "audio/mpeg".to_string(),
            progress: None,
        }
    }

    /// Set the progress callback
    pub fn with_progress(mut self, callback: ProgressCallback) -> Self {
        self.progress = Some(callback);
        self
    }

    /// Report progress if callback is set
    pub fn report(&self, stage: TranscriptionStage) {
        if let Some(cb) = &self.progress {
            cb(stage);
        }
    }
}

/// Result of a transcription
pub struct TranscriptionResult {
    pub text: String,
}

/// Response structure for OpenAI-compatible APIs (OpenAI, Groq, Mistral)
#[derive(Deserialize)]
struct OpenAICompatibleResponse {
    text: String,
}

/// Helper for OpenAI-compatible transcription APIs (sync version)
///
/// Used by OpenAI, Groq, and Mistral which share the same API format.
pub(crate) fn openai_compatible_transcribe_sync(
    api_url: &str,
    model: &str,
    api_key: &str,
    request: TranscriptionRequest,
) -> Result<TranscriptionResult> {
    // Report uploading stage
    request.report(TranscriptionStage::Uploading);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .build()
        .context("Failed to create HTTP client")?;

    let mut form = reqwest::blocking::multipart::Form::new()
        .text("model", model.to_string())
        .part(
            "file",
            reqwest::blocking::multipart::Part::bytes(request.audio_data.clone())
                .file_name(request.filename.clone())
                .mime_str(&request.mime_type)?,
        );

    if let Some(lang) = request.language.clone() {
        form = form.text("language", lang);
    }

    // Report transcribing stage (request sent, waiting for response)
    request.report(TranscriptionStage::Transcribing);

    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {api_key}"))
        .multipart(form)
        .send()
        .context("Failed to send request")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("API error ({status}): {error_text}");
    }

    let text = response.text().context("Failed to get response text")?;
    let resp: OpenAICompatibleResponse =
        serde_json::from_str(&text).context("Failed to parse API response")?;

    Ok(TranscriptionResult { text: resp.text })
}

/// Helper for OpenAI-compatible transcription APIs (async version)
///
/// Used by OpenAI, Groq, and Mistral which share the same API format.
pub(crate) async fn openai_compatible_transcribe_async(
    client: &reqwest::Client,
    api_url: &str,
    model: &str,
    api_key: &str,
    request: TranscriptionRequest,
) -> Result<TranscriptionResult> {
    // Report uploading stage
    request.report(TranscriptionStage::Uploading);

    let mut form = reqwest::multipart::Form::new()
        .text("model", model.to_string())
        .part(
            "file",
            reqwest::multipart::Part::bytes(request.audio_data.clone())
                .file_name(request.filename.clone())
                .mime_str(&request.mime_type)?,
        );

    if let Some(lang) = request.language.clone() {
        form = form.text("language", lang);
    }

    // Report transcribing stage
    request.report(TranscriptionStage::Transcribing);

    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {api_key}"))
        .multipart(form)
        .send()
        .await
        .context("Failed to send request")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("API error ({status}): {error_text}");
    }

    let text = response
        .text()
        .await
        .context("Failed to get response text")?;
    let resp: OpenAICompatibleResponse =
        serde_json::from_str(&text).context("Failed to parse API response")?;

    Ok(TranscriptionResult { text: resp.text })
}

/// Trait for transcription providers
///
/// Implement this trait to add a new transcription provider.
#[async_trait]
pub trait TranscriptionBackend: Send + Sync {
    /// Unique identifier for this provider (e.g., "openai", "deepgram")
    fn name(&self) -> &'static str;

    /// Display name for UI (e.g., "OpenAI Whisper", "Deepgram Nova")
    fn display_name(&self) -> &'static str;

    /// Synchronous transcription (for simple single-file case)
    fn transcribe_sync(
        &self,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult>;

    /// Async transcription for parallel chunk processing
    async fn transcribe_async(
        &self,
        client: &reqwest::Client,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult>;
}

/// Registry of all available transcription providers
pub struct ProviderRegistry {
    providers: HashMap<&'static str, Arc<dyn TranscriptionBackend>>,
}

impl ProviderRegistry {
    /// Create registry with all built-in providers
    pub fn new() -> Self {
        let mut providers: HashMap<&'static str, Arc<dyn TranscriptionBackend>> = HashMap::new();

        providers.insert("openai", Arc::new(OpenAIProvider));
        #[cfg(feature = "realtime")]
        providers.insert("openai-realtime", Arc::new(OpenAIRealtimeProvider));
        providers.insert("mistral", Arc::new(MistralProvider));
        providers.insert("groq", Arc::new(GroqProvider));
        providers.insert("deepgram", Arc::new(DeepgramProvider));
        providers.insert("elevenlabs", Arc::new(ElevenLabsProvider));
        #[cfg(feature = "local-transcription")]
        providers.insert("local-whisper", Arc::new(LocalWhisperProvider));
        #[cfg(feature = "local-transcription")]
        providers.insert("local-parakeet", Arc::new(LocalParakeetProvider));

        Self { providers }
    }

    /// Get a provider by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn TranscriptionBackend>> {
        self.providers.get(name).cloned()
    }

    /// List all provider names
    pub fn list(&self) -> Vec<&'static str> {
        self.providers.keys().copied().collect()
    }

    /// Get provider for a TranscriptionProvider enum value
    ///
    /// Returns an error if the provider is not registered (should never happen
    /// if all enum variants have corresponding providers in the registry).
    pub fn get_by_kind(
        &self,
        kind: &TranscriptionProvider,
    ) -> Result<Arc<dyn TranscriptionBackend>> {
        self.get(kind.as_str()).ok_or_else(|| {
            anyhow::anyhow!(
                "Provider '{}' not found in registry. This is a bug - \
                 all TranscriptionProvider variants must have registered providers.",
                kind.as_str()
            )
        })
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the global provider registry
pub fn registry() -> &'static ProviderRegistry {
    static REGISTRY: OnceLock<ProviderRegistry> = OnceLock::new();
    REGISTRY.get_or_init(ProviderRegistry::new)
}
