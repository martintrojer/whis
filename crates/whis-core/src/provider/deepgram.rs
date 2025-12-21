//! Deepgram Nova transcription provider
//!
//! Deepgram uses a different API format than OpenAI-style providers:
//! - Raw audio bytes in request body (not multipart form)
//! - Options passed as query parameters
//! - Different response JSON structure

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

use super::{
    DEFAULT_TIMEOUT_SECS, TranscriptionBackend, TranscriptionRequest, TranscriptionResult,
};

const API_URL: &str = "https://api.deepgram.com/v1/listen";
const MODEL: &str = "nova-2";

#[derive(Deserialize)]
struct Response {
    results: Results,
}

#[derive(Deserialize)]
struct Results {
    channels: Vec<Channel>,
}

#[derive(Deserialize)]
struct Channel {
    alternatives: Vec<Alternative>,
}

#[derive(Deserialize)]
struct Alternative {
    transcript: String,
}

/// Deepgram Nova transcription provider
///
/// Uses Deepgram's REST API with Nova-2 model.
/// Offers fast transcription at $0.26/hour with good accuracy.
#[derive(Debug, Default, Clone)]
pub struct DeepgramProvider;

#[async_trait]
impl TranscriptionBackend for DeepgramProvider {
    fn name(&self) -> &'static str {
        "deepgram"
    }

    fn display_name(&self) -> &'static str {
        "Deepgram Nova"
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

        let mut url = reqwest::Url::parse(API_URL).context("Failed to parse Deepgram URL")?;
        url.query_pairs_mut()
            .append_pair("model", MODEL)
            .append_pair("smart_format", "true");

        if let Some(lang) = &request.language {
            url.query_pairs_mut().append_pair("language", lang);
        }

        let response = client
            .post(url)
            .header("Authorization", format!("Token {api_key}"))
            .header("Content-Type", &request.mime_type)
            .body(request.audio_data)
            .send()
            .context("Failed to send request to Deepgram API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Deepgram API error ({status}): {error_text}");
        }

        let text = response.text().context("Failed to get response text")?;
        let resp: Response =
            serde_json::from_str(&text).context("Failed to parse Deepgram API response")?;

        let transcript = resp
            .results
            .channels
            .first()
            .and_then(|c| c.alternatives.first())
            .map(|a| a.transcript.clone())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Deepgram API returned unexpected response format: no transcript found"
                )
            })?;

        Ok(TranscriptionResult { text: transcript })
    }

    async fn transcribe_async(
        &self,
        client: &reqwest::Client,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult> {
        let mut url = reqwest::Url::parse(API_URL).context("Failed to parse Deepgram URL")?;
        url.query_pairs_mut()
            .append_pair("model", MODEL)
            .append_pair("smart_format", "true");

        if let Some(lang) = &request.language {
            url.query_pairs_mut().append_pair("language", lang);
        }

        let response = client
            .post(url)
            .header("Authorization", format!("Token {api_key}"))
            .header("Content-Type", &request.mime_type)
            .body(request.audio_data)
            .send()
            .await
            .context("Failed to send request to Deepgram API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Deepgram API error ({status}): {error_text}");
        }

        let text = response
            .text()
            .await
            .context("Failed to get response text")?;
        let resp: Response =
            serde_json::from_str(&text).context("Failed to parse Deepgram API response")?;

        let transcript = resp
            .results
            .channels
            .first()
            .and_then(|c| c.alternatives.first())
            .map(|a| a.transcript.clone())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Deepgram API returned unexpected response format: no transcript found"
                )
            })?;

        Ok(TranscriptionResult { text: transcript })
    }
}
