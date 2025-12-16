//! Mistral Voxtral transcription provider

use anyhow::Result;
use async_trait::async_trait;

use super::{
    TranscriptionBackend, TranscriptionRequest, TranscriptionResult,
    openai_compatible_transcribe_async, openai_compatible_transcribe_sync,
};

const API_URL: &str = "https://api.mistral.ai/v1/audio/transcriptions";
const MODEL: &str = "voxtral-mini-latest";

/// Mistral Voxtral transcription provider
#[derive(Debug, Default, Clone)]
pub struct MistralProvider;

#[async_trait]
impl TranscriptionBackend for MistralProvider {
    fn name(&self) -> &'static str {
        "mistral"
    }

    fn display_name(&self) -> &'static str {
        "Mistral Voxtral"
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
