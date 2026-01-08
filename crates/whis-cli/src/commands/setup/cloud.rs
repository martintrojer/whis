//! Cloud provider setup

use anyhow::{Result, anyhow};
use whis_core::{Settings, TranscriptionProvider};

use super::interactive;
use super::provider_helpers::{CLOUD_PROVIDERS, api_key_url, provider_description};

/// Prompt for and validate an API key
pub fn prompt_and_validate_key(provider: &TranscriptionProvider) -> Result<String> {
    // Validation loop with secure password input
    loop {
        let api_key = interactive::password(&format!("{} API key", provider.display_name()))?;

        // Validate key format
        let validation_result = match provider {
            TranscriptionProvider::OpenAI | TranscriptionProvider::OpenAIRealtime => {
                if !api_key.starts_with("sk-") {
                    Err(anyhow!("Invalid OpenAI key format. Keys start with 'sk-'"))
                } else {
                    Ok(())
                }
            }
            TranscriptionProvider::Groq => {
                if !api_key.starts_with("gsk_") {
                    Err(anyhow!("Invalid Groq key format. Keys start with 'gsk_'"))
                } else {
                    Ok(())
                }
            }
            _ => {
                if api_key.len() < 20 {
                    Err(anyhow!("API key seems too short"))
                } else {
                    Ok(())
                }
            }
        };

        match validation_result {
            Ok(_) => return Ok(api_key),
            Err(e) => interactive::error(&e.to_string()),
        }
    }
}

/// Streamlined cloud transcription setup (no post-processing config)
/// Used by the unified wizard
pub fn setup_transcription_cloud() -> Result<()> {
    let mut settings = Settings::load();

    // Build provider display items: with markers for selection, just name for confirmation
    let (items, clean_items): (Vec<String>, Vec<String>) = CLOUD_PROVIDERS
        .iter()
        .map(|provider| {
            let display = format!(
                "{:<10} - {}",
                provider.display_name(),
                provider_description(provider)
            );
            // Check if this provider or its realtime variant is configured
            let marker = if settings.transcription.api_key_for(provider).is_some()
                || (*provider == TranscriptionProvider::OpenAI
                    && settings.transcription.provider == TranscriptionProvider::OpenAIRealtime)
                || (*provider == TranscriptionProvider::Deepgram
                    && settings.transcription.provider == TranscriptionProvider::DeepgramRealtime)
            {
                " [configured]"
            } else {
                ""
            };
            (format!("{}{}", display, marker), provider.display_name().to_string())
        })
        .unzip();

    // Default to current provider if configured, otherwise first
    // Treat realtime variants same as base provider for default selection
    let default = CLOUD_PROVIDERS.iter().position(|p| {
        *p == settings.transcription.provider
            || (*p == TranscriptionProvider::OpenAI
                && settings.transcription.provider == TranscriptionProvider::OpenAIRealtime)
            || (*p == TranscriptionProvider::Deepgram
                && settings.transcription.provider == TranscriptionProvider::DeepgramRealtime)
    });

    let choice = interactive::select_clean("Which provider?", &items, &clean_items, default)?;
    let mut provider = CLOUD_PROVIDERS[choice].clone();

    // If OpenAI selected, ask for method (Standard vs Streaming)
    if provider == TranscriptionProvider::OpenAI {
        let methods = vec!["Standard - Progressive", "Streaming - Real-time"];

        // Default to current method if already using OpenAI variant
        let default_method =
            if settings.transcription.provider == TranscriptionProvider::OpenAIRealtime {
                1
            } else {
                0
            };

        let method_choice = interactive::select("Which method?", &methods, Some(default_method))?;
        if method_choice == 1 {
            provider = TranscriptionProvider::OpenAIRealtime;
        }
    }

    // If Deepgram selected, ask for method (Standard vs Streaming)
    if provider == TranscriptionProvider::Deepgram {
        let methods = vec!["Standard - Progressive", "Streaming - Real-time"];

        // Default to current method if already using Deepgram variant
        let default_method =
            if settings.transcription.provider == TranscriptionProvider::DeepgramRealtime {
                1
            } else {
                0
            };

        let method_choice = interactive::select("Which method?", &methods, Some(default_method))?;
        if method_choice == 1 {
            provider = TranscriptionProvider::DeepgramRealtime;
        }
    }

    // Check if API key already exists for this provider
    if let Some(_existing_key) = settings.transcription.api_key_for(&provider) {
        let keep = interactive::select("Keep current key?", &["Yes", "No"], Some(0))? == 0;

        if !keep {
            interactive::info(&format!(
                "Get your API key from: {}",
                api_key_url(&provider)
            ));
            let api_key = prompt_and_validate_key(&provider)?;
            settings.transcription.set_api_key(&provider, api_key);
        }
    } else {
        // No existing key - prompt for new one
        interactive::info(&format!(
            "Get your API key from: {}",
            api_key_url(&provider)
        ));
        let api_key = prompt_and_validate_key(&provider)?;
        settings.transcription.set_api_key(&provider, api_key);
    }

    settings.transcription.provider = provider;
    settings.save()?;

    Ok(())
}
