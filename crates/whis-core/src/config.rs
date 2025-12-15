use serde::{Deserialize, Serialize};
use std::fmt;

/// Available transcription providers
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TranscriptionProvider {
    #[default]
    OpenAI,
    Mistral,
    Groq,
    Deepgram,
    ElevenLabs,
}

impl TranscriptionProvider {
    /// Get the string identifier for this provider
    pub fn as_str(&self) -> &'static str {
        match self {
            TranscriptionProvider::OpenAI => "openai",
            TranscriptionProvider::Mistral => "mistral",
            TranscriptionProvider::Groq => "groq",
            TranscriptionProvider::Deepgram => "deepgram",
            TranscriptionProvider::ElevenLabs => "elevenlabs",
        }
    }

    /// Get the environment variable name for this provider's API key
    pub fn api_key_env_var(&self) -> &'static str {
        match self {
            TranscriptionProvider::OpenAI => "OPENAI_API_KEY",
            TranscriptionProvider::Mistral => "MISTRAL_API_KEY",
            TranscriptionProvider::Groq => "GROQ_API_KEY",
            TranscriptionProvider::Deepgram => "DEEPGRAM_API_KEY",
            TranscriptionProvider::ElevenLabs => "ELEVENLABS_API_KEY",
        }
    }

    /// List all available providers
    pub fn all() -> &'static [TranscriptionProvider] {
        &[
            TranscriptionProvider::OpenAI,
            TranscriptionProvider::Mistral,
            TranscriptionProvider::Groq,
            TranscriptionProvider::Deepgram,
            TranscriptionProvider::ElevenLabs,
        ]
    }

    /// Human-readable display name for this provider
    pub fn display_name(&self) -> &'static str {
        match self {
            TranscriptionProvider::OpenAI => "OpenAI",
            TranscriptionProvider::Mistral => "Mistral",
            TranscriptionProvider::Groq => "Groq",
            TranscriptionProvider::Deepgram => "Deepgram",
            TranscriptionProvider::ElevenLabs => "ElevenLabs",
        }
    }
}

impl fmt::Display for TranscriptionProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for TranscriptionProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(TranscriptionProvider::OpenAI),
            "mistral" => Ok(TranscriptionProvider::Mistral),
            "groq" => Ok(TranscriptionProvider::Groq),
            "deepgram" => Ok(TranscriptionProvider::Deepgram),
            "elevenlabs" => Ok(TranscriptionProvider::ElevenLabs),
            _ => Err(format!(
                "Unknown provider: {}. Available: openai, mistral, groq, deepgram, elevenlabs",
                s
            )),
        }
    }
}
