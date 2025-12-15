//! ElevenLabs Scribe transcription provider
//!
//! ElevenLabs Scribe claims the highest accuracy in the market with ~3.3% English WER.
//! Uses multipart form upload with a different response structure.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

use super::{TranscriptionBackend, TranscriptionRequest, TranscriptionResult, DEFAULT_TIMEOUT_SECS};

const API_URL: &str = "https://api.elevenlabs.io/v1/speech-to-text";
const MODEL: &str = "scribe_v1";

#[derive(Deserialize)]
struct Response {
    text: String,
}

/// ElevenLabs Scribe transcription provider
///
/// Uses ElevenLabs' Scribe model for high-accuracy transcription.
/// Supports 99 languages with speaker diarization for up to 32 speakers.
/// Priced at $0.40/hour.
#[derive(Debug, Default, Clone)]
pub struct ElevenLabsProvider;

#[async_trait]
impl TranscriptionBackend for ElevenLabsProvider {
    fn name(&self) -> &'static str {
        "elevenlabs"
    }

    fn display_name(&self) -> &'static str {
        "ElevenLabs Scribe"
    }

    fn transcribe_sync(
        &self,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .context("Failed to create HTTP client")?;

        let mut form = reqwest::blocking::multipart::Form::new()
            .text("model_id", MODEL)
            .part(
                "file",
                reqwest::blocking::multipart::Part::bytes(request.audio_data)
                    .file_name(request.filename)
                    .mime_str("audio/mpeg")?,
            );

        if let Some(lang) = request.language {
            form = form.text("language_code", lang);
        }

        let response = client
            .post(API_URL)
            .header("xi-api-key", api_key)
            .multipart(form)
            .send()
            .context("Failed to send request to ElevenLabs API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ElevenLabs API error ({status}): {error_text}");
        }

        let text = response.text().context("Failed to get response text")?;
        let resp: Response =
            serde_json::from_str(&text).context("Failed to parse ElevenLabs API response")?;

        Ok(TranscriptionResult { text: resp.text })
    }

    async fn transcribe_async(
        &self,
        client: &reqwest::Client,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult> {
        let mut form = reqwest::multipart::Form::new()
            .text("model_id", MODEL)
            .part(
                "file",
                reqwest::multipart::Part::bytes(request.audio_data)
                    .file_name(request.filename)
                    .mime_str("audio/mpeg")?,
            );

        if let Some(lang) = request.language {
            form = form.text("language_code", lang);
        }

        let response = client
            .post(API_URL)
            .header("xi-api-key", api_key)
            .multipart(form)
            .send()
            .await
            .context("Failed to send request to ElevenLabs API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ElevenLabs API error ({status}): {error_text}");
        }

        let text = response
            .text()
            .await
            .context("Failed to get response text")?;
        let resp: Response =
            serde_json::from_str(&text).context("Failed to parse ElevenLabs API response")?;

        Ok(TranscriptionResult { text: resp.text })
    }
}
