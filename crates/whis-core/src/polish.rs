use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

const OPENAI_CHAT_URL: &str = "https://api.openai.com/v1/chat/completions";
const MISTRAL_CHAT_URL: &str = "https://api.mistral.ai/v1/chat/completions";
const DEFAULT_TIMEOUT_SECS: u64 = 60;

pub const DEFAULT_POLISH_PROMPT: &str = "Clean up this voice transcript. Fix grammar and punctuation. \
Remove filler words (um, uh, like, you know). \
If the speaker corrects themselves, keep only the correction. \
Preserve technical terms and proper nouns. Output only the cleaned text.";

/// Available polishing providers (LLM for transcript cleanup)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Polisher {
    #[default]
    None,
    OpenAI,
    Mistral,
}

impl fmt::Display for Polisher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Polisher::None => write!(f, "none"),
            Polisher::OpenAI => write!(f, "openai"),
            Polisher::Mistral => write!(f, "mistral"),
        }
    }
}

impl std::str::FromStr for Polisher {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Polisher::None),
            "openai" => Ok(Polisher::OpenAI),
            "mistral" => Ok(Polisher::Mistral),
            _ => Err(format!(
                "Unknown polisher: {}. Use 'none', 'openai', or 'mistral'",
                s
            )),
        }
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

/// Polish (clean up) a transcript using the specified LLM provider
pub async fn polish(
    text: &str,
    polisher: &Polisher,
    api_key: &str,
    prompt: &str,
) -> Result<String> {
    match polisher {
        Polisher::None => Ok(text.to_string()),
        Polisher::OpenAI => polish_openai(text, api_key, prompt).await,
        Polisher::Mistral => polish_mistral(text, api_key, prompt).await,
    }
}

async fn polish_openai(text: &str, api_key: &str, system_prompt: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let response = client
        .post(OPENAI_CHAT_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": "gpt-5-nano-2025-08-07",
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
        return Err(anyhow!("OpenAI polish failed: {}", error_text));
    }

    let chat_response: ChatResponse = response.json().await?;
    chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| anyhow!("No response from OpenAI"))
}

async fn polish_mistral(text: &str, api_key: &str, system_prompt: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let response = client
        .post(MISTRAL_CHAT_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": "mistral-small-latest",
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
        return Err(anyhow!("Mistral polish failed: {}", error_text));
    }

    let chat_response: ChatResponse = response.json().await?;
    chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| anyhow!("No response from Mistral"))
}
