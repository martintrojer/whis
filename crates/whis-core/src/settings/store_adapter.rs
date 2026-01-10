//! Store adapter for flat key-value settings conversion.
//!
//! This module provides conversion between flat key-value maps (like Tauri's plugin-store)
//! and the hierarchical Settings struct. This enables mobile apps to use the same
//! Settings struct as CLI/Desktop while storing data in platform-appropriate ways.
//!
//! # Key Mapping
//!
//! ```text
//! Flat Key              → Settings Field
//! ─────────────────────────────────────────────────
//! provider              → transcription.provider
//! language              → transcription.language
//! openai_api_key        → transcription.api_keys["openai"]
//! mistral_api_key       → transcription.api_keys["mistral"]
//! groq_api_key          → transcription.api_keys["groq"]
//! deepgram_api_key      → transcription.api_keys["deepgram"]
//! elevenlabs_api_key    → transcription.api_keys["elevenlabs"]
//! post_processor        → post_processing.processor
//! active_preset         → ui.active_preset
//! ollama_url            → services.ollama.url
//! ollama_model          → services.ollama.model
//! shortcut_key          → ui.shortcut_key
//! vad_enabled           → ui.vad.enabled
//! vad_threshold         → ui.vad.threshold
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! // In mobile app, convert store entries to Settings
//! let settings = Settings::from_store_map(&store_map);
//!
//! // After modifying settings, convert back to flat map
//! let updates = settings.to_store_map();
//! for (key, value) in updates {
//!     store.set(key, value);
//! }
//! ```

use super::Settings;
use crate::configuration::TranscriptionProvider;
use crate::transcription::PostProcessor;
use serde_json::Value;
use std::collections::HashMap;

/// API key store keys mapped to provider names.
const API_KEY_MAPPINGS: &[(&str, &str)] = &[
    ("openai_api_key", "openai"),
    ("mistral_api_key", "mistral"),
    ("groq_api_key", "groq"),
    ("deepgram_api_key", "deepgram"),
    ("elevenlabs_api_key", "elevenlabs"),
];

impl Settings {
    /// Create Settings from a flat key-value map.
    ///
    /// This is useful for loading settings from stores like Tauri's plugin-store
    /// which use flat string keys instead of nested JSON.
    ///
    /// Unknown keys are ignored. Missing keys use defaults.
    pub fn from_store_map(map: &HashMap<String, Value>) -> Self {
        let mut settings = Settings::default();

        // Transcription settings
        if let Some(Value::String(provider)) = map.get("provider")
            && let Ok(p) = provider.parse::<TranscriptionProvider>() {
                settings.transcription.provider = p;
            }

        if let Some(Value::String(lang)) = map.get("language")
            && !lang.is_empty() {
                settings.transcription.language = Some(lang.clone());
            }

        // API keys
        for (store_key, provider_name) in API_KEY_MAPPINGS {
            if let Some(Value::String(key)) = map.get(*store_key)
                && !key.is_empty() {
                    settings
                        .transcription
                        .api_keys
                        .insert(provider_name.to_string(), key.clone());
                }
        }

        // Post-processing settings
        if let Some(Value::String(processor)) = map.get("post_processor")
            && let Ok(p) = processor.parse::<PostProcessor>() {
                settings.post_processing.processor = p;
            }

        // UI settings
        if let Some(Value::String(preset)) = map.get("active_preset")
            && !preset.is_empty() {
                settings.ui.active_preset = Some(preset.clone());
            }

        if let Some(Value::String(shortcut)) = map.get("shortcut_key") {
            settings.ui.shortcut_key = shortcut.clone();
        }

        if let Some(Value::Bool(enabled)) = map.get("vad_enabled") {
            settings.ui.vad.enabled = *enabled;
        }

        if let Some(threshold) = map.get("vad_threshold")
            && let Some(t) = threshold.as_f64() {
                settings.ui.vad.threshold = t as f32;
            }

        // Services settings
        if let Some(Value::String(url)) = map.get("ollama_url")
            && !url.is_empty() {
                settings.services.ollama.url = Some(url.clone());
            }

        if let Some(Value::String(model)) = map.get("ollama_model")
            && !model.is_empty() {
                settings.services.ollama.model = Some(model.clone());
            }

        settings
    }

    /// Convert Settings to a flat key-value map.
    ///
    /// This is useful for saving settings to stores like Tauri's plugin-store.
    /// Only non-default values are included to minimize storage.
    pub fn to_store_map(&self) -> HashMap<String, Value> {
        let mut map = HashMap::new();

        // Transcription settings
        map.insert(
            "provider".to_string(),
            Value::String(self.transcription.provider.as_str().to_string()),
        );

        if let Some(ref lang) = self.transcription.language {
            map.insert("language".to_string(), Value::String(lang.clone()));
        }

        // API keys
        for (store_key, provider_name) in API_KEY_MAPPINGS {
            if let Some(key) = self.transcription.api_keys.get(*provider_name)
                && !key.is_empty() {
                    map.insert(store_key.to_string(), Value::String(key.clone()));
                }
        }

        // Post-processing settings
        map.insert(
            "post_processor".to_string(),
            Value::String(self.post_processing.processor.to_string()),
        );

        // UI settings
        if let Some(ref preset) = self.ui.active_preset {
            map.insert("active_preset".to_string(), Value::String(preset.clone()));
        }

        map.insert(
            "shortcut_key".to_string(),
            Value::String(self.ui.shortcut_key.clone()),
        );

        map.insert("vad_enabled".to_string(), Value::Bool(self.ui.vad.enabled));

        map.insert(
            "vad_threshold".to_string(),
            Value::Number(serde_json::Number::from_f64(self.ui.vad.threshold as f64).unwrap()),
        );

        // Services settings
        if let Some(ref url) = self.services.ollama.url {
            map.insert("ollama_url".to_string(), Value::String(url.clone()));
        }

        if let Some(ref model) = self.services.ollama.model {
            map.insert("ollama_model".to_string(), Value::String(model.clone()));
        }

        map
    }

    /// Get the store key for a provider's API key.
    ///
    /// Returns the flat store key name (e.g., "openai_api_key" for "openai").
    pub fn api_key_store_key(provider: &str) -> Option<&'static str> {
        // Normalize provider (openai-realtime uses openai key)
        let normalized = if provider == "openai-realtime" {
            "openai"
        } else if provider == "deepgram-realtime" {
            "deepgram"
        } else {
            provider
        };

        API_KEY_MAPPINGS
            .iter()
            .find(|(_, name)| *name == normalized)
            .map(|(key, _)| *key)
    }
}
