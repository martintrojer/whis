use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::config::TranscriptionProvider;
use crate::polish::Polisher;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub shortcut: String,
    #[serde(default)]
    pub provider: TranscriptionProvider,
    /// Language hint for transcription (ISO-639-1 code, e.g., "en", "de", "fr")
    /// None = auto-detect, Some("en") = English, etc.
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub openai_api_key: Option<String>,
    #[serde(default)]
    pub mistral_api_key: Option<String>,
    /// LLM provider for polishing (cleaning up) transcripts
    #[serde(default)]
    pub polisher: Polisher,
    /// Custom prompt for polishing (uses default if None)
    #[serde(default)]
    pub polish_prompt: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shortcut: "Ctrl+Shift+R".to_string(),
            provider: TranscriptionProvider::default(),
            language: None, // Auto-detect
            openai_api_key: None,
            mistral_api_key: None,
            polisher: Polisher::default(),
            polish_prompt: None,
        }
    }
}

impl Settings {
    /// Get the settings file path (~/.config/whis/settings.json)
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("whis")
            .join("settings.json")
    }

    /// Get the API key for the current provider, falling back to environment variables
    pub fn get_api_key(&self) -> Option<String> {
        match &self.provider {
            TranscriptionProvider::OpenAI => self
                .openai_api_key
                .clone()
                .or_else(|| std::env::var("OPENAI_API_KEY").ok()),
            TranscriptionProvider::Mistral => self
                .mistral_api_key
                .clone()
                .or_else(|| std::env::var("MISTRAL_API_KEY").ok()),
        }
    }

    /// Check if an API key is configured for the current provider
    pub fn has_api_key(&self) -> bool {
        self.get_api_key().is_some()
    }

    /// Get the API key for the polisher, falling back to environment variables
    pub fn get_polisher_api_key(&self) -> Option<String> {
        match &self.polisher {
            Polisher::None => None,
            Polisher::OpenAI => self
                .openai_api_key
                .clone()
                .or_else(|| std::env::var("OPENAI_API_KEY").ok()),
            Polisher::Mistral => self
                .mistral_api_key
                .clone()
                .or_else(|| std::env::var("MISTRAL_API_KEY").ok()),
        }
    }

    /// Load settings from disk
    pub fn load() -> Self {
        let path = Self::path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str(&content) {
                return settings;
            }
        }
        Self::default()
    }

    /// Save settings to disk with 0600 permissions
    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, &content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }
}
