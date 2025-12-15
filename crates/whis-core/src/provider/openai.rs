//! OpenAI Whisper transcription provider

use anyhow::Result;
use async_trait::async_trait;

use super::{
    openai_compatible_transcribe_async, openai_compatible_transcribe_sync, TranscriptionBackend,
    TranscriptionRequest, TranscriptionResult,
};

const API_URL: &str = "https://api.openai.com/v1/audio/transcriptions";
const MODEL: &str = "whisper-1";

/// OpenAI Whisper transcription provider
#[derive(Debug, Default, Clone)]
pub struct OpenAIProvider;

#[async_trait]
impl TranscriptionBackend for OpenAIProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn display_name(&self) -> &'static str {
        "OpenAI Whisper"
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
