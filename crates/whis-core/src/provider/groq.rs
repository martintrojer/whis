//! Groq Whisper transcription provider
//!
//! Groq offers an OpenAI-compatible API running Whisper models on their custom LPU hardware.
//! It's significantly faster and cheaper than OpenAI's hosted Whisper.

use anyhow::Result;
use async_trait::async_trait;

use super::{
    TranscriptionBackend, TranscriptionRequest, TranscriptionResult,
    openai_compatible_transcribe_async, openai_compatible_transcribe_sync,
};

const API_URL: &str = "https://api.groq.com/openai/v1/audio/transcriptions";
const MODEL: &str = "whisper-large-v3-turbo";

/// Groq Whisper transcription provider
///
/// Uses Groq's OpenAI-compatible API with Whisper models running on LPU hardware.
/// Offers ~240x real-time transcription speed at $0.04/hour.
#[derive(Debug, Default, Clone)]
pub struct GroqProvider;

#[async_trait]
impl TranscriptionBackend for GroqProvider {
    fn name(&self) -> &'static str {
        "groq"
    }

    fn display_name(&self) -> &'static str {
        "Groq Whisper"
    }

    fn transcribe_sync(
        &self,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult> {
        openai_compatible_transcribe_sync(API_URL, MODEL, api_key, request)
    }

    async fn transcribe_async(
        &self,
        client: &reqwest::Client,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult> {
        openai_compatible_transcribe_async(client, API_URL, MODEL, api_key, request).await
    }
}
