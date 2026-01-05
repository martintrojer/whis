//! Cloud provider setup

use anyhow::{Result, anyhow};
use whis_core::{PostProcessor, Settings, TranscriptionProvider};

use super::interactive;
use super::post_processing::configure_post_processing_options;
use super::provider_helpers::{
    CLOUD_PROVIDERS, api_key_url, get_provider_status, provider_description,
};
use crate::ui::mask_key;

/// Type alias for menu action callbacks
type MenuAction = Box<dyn FnOnce(&mut Settings) -> Result<()>>;

/// Setup for cloud providers (full interactive menu)
pub fn setup_cloud() -> Result<()> {
    let mut settings = Settings::load();
    let (configured, unconfigured) = get_provider_status(&settings);

    // Show current status
    interactive::info("Provider status:");
    for provider in CLOUD_PROVIDERS {
        let is_active = settings.transcription.provider == *provider;
        let active_marker = if is_active { " [active]" } else { "" };

        if let Some(key) = settings.transcription.api_key_for(provider) {
            // Check if key is from environment variable
            let key_source = if settings
                .transcription
                .api_keys
                .contains_key(provider.as_str())
            {
                mask_key(&key)
            } else {
                format!("from ${}", provider.api_key_env_var())
            };
            interactive::info(&format!(
                "  + {:<10} ({}){}",
                provider.display_name(),
                key_source,
                active_marker
            ));
        } else {
            interactive::info(&format!(
                "  - {:<10} (not configured)",
                provider.display_name()
            ));
        }
    }

    // Determine menu options based on state
    if configured.is_empty() {
        // No keys configured - go straight to provider selection
        return setup_new_provider(&mut settings, CLOUD_PROVIDERS);
    }

    // Build dynamic menu
    let mut menu_options: Vec<String> = Vec::new();
    let mut menu_actions: Vec<MenuAction> = Vec::new();

    // Option 1: Use current provider (if configured)
    if configured
        .iter()
        .any(|(p, _)| *p == settings.transcription.provider)
    {
        menu_options.push(format!(
            "Use {} (current)",
            settings.transcription.provider.display_name()
        ));
        let current = settings.transcription.provider.clone();
        menu_actions.push(Box::new(move |s| {
            interactive::info(&format!(
                "Keeping {} as active provider.",
                current.display_name()
            ));
            finish_setup(s, &current)
        }));
    }

    // Options for switching to other configured providers
    for (provider, _) in &configured {
        if *provider != settings.transcription.provider {
            menu_options.push(format!("Switch to {}", provider.display_name()));
            let p = provider.clone();
            menu_actions.push(Box::new(move |s| {
                s.transcription.provider = p.clone();
                interactive::info(&format!("Switched to {}.", p.display_name()));
                finish_setup(s, &p)
            }));
        }
    }

    // Option to configure new provider (if any unconfigured)
    if !unconfigured.is_empty() {
        menu_options.push("Configure a new provider".to_string());
        let unconfigured_clone = unconfigured.clone();
        menu_actions.push(Box::new(move |s| {
            setup_new_provider(s, &unconfigured_clone)
        }));
    }

    // Option to update existing key
    if !configured.is_empty() {
        menu_options.push("Update an existing key".to_string());
        let configured_clone: Vec<_> = configured.iter().map(|(p, _)| p.clone()).collect();
        menu_actions.push(Box::new(move |s| update_existing_key(s, &configured_clone)));
    }

    // Display menu
    let choice = interactive::select("What would you like to do?", &menu_options, Some(0))?;

    // Execute selected action
    let action = menu_actions.into_iter().nth(choice).unwrap();
    action(&mut settings)
}

/// Configure a new provider with API key
fn setup_new_provider(settings: &mut Settings, providers: &[TranscriptionProvider]) -> Result<()> {
    let items: Vec<String> = providers
        .iter()
        .map(|p| format!("{:<10} - {}", p.display_name(), provider_description(p)))
        .collect();

    let choice = interactive::select("Which provider to configure?", &items, Some(0))?;
    let provider = providers[choice].clone();

    interactive::info(&format!(
        "Get your API key from: {}",
        api_key_url(&provider)
    ));

    let api_key = prompt_and_validate_key(&provider)?;

    settings.transcription.provider = provider.clone();
    settings.transcription.set_api_key(&provider, api_key);

    finish_setup(settings, &provider)
}

/// Update an existing API key
fn update_existing_key(settings: &mut Settings, providers: &[TranscriptionProvider]) -> Result<()> {
    let items: Vec<String> = providers
        .iter()
        .map(|p| {
            let current_key = settings.transcription.api_key_for(p).unwrap_or_default();
            format!("{} ({})", p.display_name(), mask_key(&current_key))
        })
        .collect();

    let choice = interactive::select("Which provider to update?", &items, Some(0))?;
    let provider = providers[choice].clone();

    interactive::info(&format!(
        "Current key: {}",
        mask_key(
            &settings
                .transcription
                .api_key_for(&provider)
                .unwrap_or_default()
        )
    ));
    interactive::info(&format!("Get a new key from: {}", api_key_url(&provider)));

    let api_key = prompt_and_validate_key(&provider)?;

    settings.transcription.set_api_key(&provider, api_key);

    interactive::info(&format!("{} key updated", provider.display_name()));

    settings.save()?;
    Ok(())
}

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

/// Finish setup and save settings
fn finish_setup(settings: &mut Settings, provider: &TranscriptionProvider) -> Result<()> {
    // Set default post-processor based on provider
    let default_post_processor = match provider {
        TranscriptionProvider::OpenAI => PostProcessor::OpenAI,
        TranscriptionProvider::Mistral => PostProcessor::Mistral,
        _ => {
            // For other providers, use OpenAI for post-processing if they have an OpenAI key
            if settings
                .transcription
                .api_key_for(&TranscriptionProvider::OpenAI)
                .is_some()
            {
                PostProcessor::OpenAI
            } else {
                PostProcessor::None
            }
        }
    };

    settings.post_processing.processor = default_post_processor.clone();
    settings.save()?;

    interactive::info(&format!(
        "Transcription configured: {}",
        provider.display_name()
    ));

    // Offer post-processing configuration
    let pp_display = match &default_post_processor {
        PostProcessor::None => "None (disabled)",
        PostProcessor::OpenAI => "OpenAI (same API key)",
        PostProcessor::Mistral => "Mistral (same API key)",
        PostProcessor::Ollama => "Ollama",
    };

    interactive::info(
        "Post-processing cleans up transcriptions (removes filler words, fixes grammar)",
    );

    let options = vec![
        format!("Use {} (default)", pp_display),
        "Configure different post-processor".to_string(),
        "Skip for now".to_string(),
    ];

    let choice = interactive::select("Configure post-processing?", &options, Some(0))?;

    match choice {
        0 => {
            // Keep default
        }
        1 => {
            // Run post-processing setup
            configure_post_processing_options(settings)?;
        }
        2 => {
            // Skip - leave as default
        }
        _ => unreachable!(),
    }

    interactive::info(&format!(
        "Configuration saved! Transcription: {}{}",
        provider.display_name(),
        if settings.post_processing.processor != PostProcessor::None {
            format!(", Post-processing: {}", settings.post_processing.processor)
        } else {
            String::new()
        }
    ));

    Ok(())
}

/// Streamlined cloud transcription setup (no post-processing config)
/// Used by the unified wizard
pub fn setup_transcription_cloud() -> Result<()> {
    let mut settings = Settings::load();

    // Build provider display items with [configured] marker
    let items: Vec<String> = CLOUD_PROVIDERS
        .iter()
        .map(|provider| {
            // Check if this provider or its realtime variant is configured
            let configured = if settings.transcription.api_key_for(provider).is_some()
                || (*provider == TranscriptionProvider::OpenAI
                    && settings.transcription.provider == TranscriptionProvider::OpenAIRealtime)
                || (*provider == TranscriptionProvider::Deepgram
                    && settings.transcription.provider == TranscriptionProvider::DeepgramRealtime)
            {
                " [configured]"
            } else {
                ""
            };
            format!(
                "{:<10} - {}{}",
                provider.display_name(),
                provider_description(provider),
                configured
            )
        })
        .collect();

    // Default to current provider if configured, otherwise first
    // Treat realtime variants same as base provider for default selection
    let default = CLOUD_PROVIDERS.iter().position(|p| {
        *p == settings.transcription.provider
            || (*p == TranscriptionProvider::OpenAI
                && settings.transcription.provider == TranscriptionProvider::OpenAIRealtime)
            || (*p == TranscriptionProvider::Deepgram
                && settings.transcription.provider == TranscriptionProvider::DeepgramRealtime)
    });

    let choice = interactive::select("Which provider?", &items, default)?;
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
    if let Some(existing_key) = settings.transcription.api_key_for(&provider) {
        interactive::info(&format!("Current API key: {}", mask_key(&existing_key)));
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
