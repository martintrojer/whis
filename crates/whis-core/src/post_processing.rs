//! LLM-based transcript cleanup and post-processing.
//!
//! After transcription, raw text often contains filler words (um, uh), grammar issues,
//! and run-on sentences. This module sends transcripts to an LLM for cleanup.
//!
//! # Supported Providers
//!
//! - **OpenAI** - GPT models via chat completions API
//! - **Mistral** - Mistral models via chat completions API
//! - **Ollama** - Local LLMs (no API key required, just server URL)
//! - **None** - Pass through without processing
//!
//! # Usage
//!
//! ```ignore
//! use whis_core::post_processing::{post_process, PostProcessor};
//!
//! let cleaned = post_process(
//!     "um so like I was thinking...",
//!     &PostProcessor::OpenAI,
//!     "sk-...",
//!     "Clean up this transcript",
//!     None,
//! ).await?;
//! ```

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::http::get_http_client;

const OPENAI_CHAT_URL: &str = "https://api.openai.com/v1/chat/completions";
const MISTRAL_CHAT_URL: &str = "https://api.mistral.ai/v1/chat/completions";
const DEFAULT_TIMEOUT_SECS: u64 = 60;

pub const DEFAULT_POST_PROCESSING_PROMPT: &str = "Clean up this voice transcript. \
Remove filler words (um, uh, like, you know). \
Fix grammar and punctuation. Keep technical terms intact. \
Output only the cleaned text, no explanations.";

/// Available post-processing providers (LLM for transcript cleanup)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PostProcessor {
    None,
    OpenAI,
    Mistral,
    Ollama,
}

impl Default for PostProcessor {
    fn default() -> Self {
        crate::defaults::DEFAULT_POST_PROCESSOR
    }
}

impl fmt::Display for PostProcessor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PostProcessor::None => write!(f, "none"),
            PostProcessor::OpenAI => write!(f, "openai"),
            PostProcessor::Mistral => write!(f, "mistral"),
            PostProcessor::Ollama => write!(f, "ollama"),
        }
    }
}

impl std::str::FromStr for PostProcessor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(PostProcessor::None),
            "openai" => Ok(PostProcessor::OpenAI),
            "mistral" => Ok(PostProcessor::Mistral),
            "ollama" => Ok(PostProcessor::Ollama),
            _ => Err(format!(
                "Unknown post-processor: {}. Use 'none', 'openai', 'mistral', or 'ollama'",
                s
            )),
        }
    }
}

impl PostProcessor {
    /// Returns true if this post-processor requires an API key (cloud providers)
    pub fn requires_api_key(&self) -> bool {
        matches!(self, PostProcessor::OpenAI | PostProcessor::Mistral)
    }
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

/// Post-process (clean up) a transcript using the specified LLM provider
///
/// For cloud providers (OpenAI, Mistral), `api_key_or_url` is the API key.
/// For Ollama, `api_key_or_url` is the server URL (e.g., http://localhost:11434).
pub async fn post_process(
    text: &str,
    post_processor: &PostProcessor,
    api_key_or_url: &str,
    prompt: &str,
    model: Option<&str>,
) -> Result<String> {
    match post_processor {
        PostProcessor::None => Ok(text.to_string()),
        PostProcessor::OpenAI => post_process_openai(text, api_key_or_url, prompt, model).await,
        PostProcessor::Mistral => post_process_mistral(text, api_key_or_url, prompt, model).await,
        PostProcessor::Ollama => post_process_ollama(text, api_key_or_url, prompt, model).await,
    }
}

const DEFAULT_OPENAI_MODEL: &str = "gpt-5-nano";

async fn post_process_openai(
    text: &str,
    api_key: &str,
    system_prompt: &str,
    model: Option<&str>,
) -> Result<String> {
    let model = model.unwrap_or(DEFAULT_OPENAI_MODEL);
    let client = get_http_client()?;
    let response = client
        .post(OPENAI_CHAT_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": text}
            ]
        }))
        .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("OpenAI post-processing failed: {}", error_text));
    }

    let chat_response: ChatResponse = response.json().await?;
    chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| anyhow!("No response from OpenAI"))
}

const DEFAULT_MISTRAL_MODEL: &str = "mistral-small-latest";

async fn post_process_mistral(
    text: &str,
    api_key: &str,
    system_prompt: &str,
    model: Option<&str>,
) -> Result<String> {
    let model = model.unwrap_or(DEFAULT_MISTRAL_MODEL);
    let client = get_http_client()?;
    let response = client
        .post(MISTRAL_CHAT_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": text}
            ]
        }))
        .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Mistral post-processing failed: {}", error_text));
    }

    let chat_response: ChatResponse = response.json().await?;
    chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| anyhow!("No response from Mistral"))
}

use crate::ollama::{DEFAULT_OLLAMA_MODEL, DEFAULT_OLLAMA_URL};

/// Ollama API response structure
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    content: String,
}

async fn post_process_ollama(
    text: &str,
    server_url: &str,
    system_prompt: &str,
    model: Option<&str>,
) -> Result<String> {
    let model = model.unwrap_or(DEFAULT_OLLAMA_MODEL);
    let base_url = if server_url.is_empty() {
        DEFAULT_OLLAMA_URL
    } else {
        server_url
    };
    let url = format!("{}/api/chat", base_url.trim_end_matches('/'));

    let client = get_http_client()?;
    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": text}
            ],
            "stream": false
        }))
        .timeout(std::time::Duration::from_secs(120)) // Longer timeout for local LLM
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() {
                anyhow!(
                    "Cannot connect to Ollama at {}. Is Ollama running? Start with: ollama serve",
                    base_url
                )
            } else {
                anyhow!("Ollama request failed: {}", e)
            }
        })?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Ollama post-processing failed: {}", error_text));
    }

    let ollama_response: OllamaResponse = response.json().await?;
    Ok(ollama_response.message.content.trim().to_string())
}
