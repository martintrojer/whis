//! Cloud provider setup

use anyhow::{Result, anyhow};
use whis_core::{PostProcessor, Settings, TranscriptionProvider};

use super::interactive;
use super::post_processing::configure_post_processing_options;
use super::provider_helpers::{
    CLOUD_PROVIDERS, api_key_url, get_provider_status, provider_description,
};
use crate::ui::{mask_key, prompt_choice};

/// Type alias for menu action callbacks
type MenuAction = Box<dyn FnOnce(&mut Settings) -> Result<()>>;

/// Setup for cloud providers (full interactive menu)
pub fn setup_cloud() -> Result<()> {
    println!("Cloud Setup");
    println!("===========");
    println!();

    let mut settings = Settings::load();
    let (configured, unconfigured) = get_provider_status(&settings);

    // Show current status
    println!("Provider status:");
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
            println!(
                "  + {:<10} ({}){}",
                provider.display_name(),
                key_source,
                active_marker
            );
        } else {
            println!("  - {:<10} (not configured)", provider.display_name());
        }
    }
    println!();

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
            println!();
            println!("Keeping {} as active provider.", current.display_name());
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
                println!();
                println!("Switched to {}.", p.display_name());
                finish_setup(s, &p)
            }));
        }
    }

    // Option to configure new provider (if any unconfigured)
    if !unconfigured.is_empty() {
        menu_options.push("Configure a new provider".to_string());
        let unconfigured_clone = unconfigured.clone();
        menu_actions.push(Box::new(move |s| {
            println!();
            setup_new_provider(s, &unconfigured_clone)
        }));
    }

    // Option to update existing key
    if !configured.is_empty() {
        menu_options.push("Update an existing key".to_string());
        let configured_clone: Vec<_> = configured.iter().map(|(p, _)| p.clone()).collect();
        menu_actions.push(Box::new(move |s| {
            println!();
            update_existing_key(s, &configured_clone)
        }));
    }

    // Display menu
    println!("What would you like to do?");
    for (i, option) in menu_options.iter().enumerate() {
        println!("  {}. {}", i + 1, option);
    }
    println!();

    let choice = prompt_choice(
        &format!("Select (1-{})", menu_options.len()),
        1,
        menu_options.len(),
    )?;

    // Execute selected action
    let action = menu_actions.into_iter().nth(choice - 1).unwrap();
    action(&mut settings)
}

/// Configure a new provider with API key
fn setup_new_provider(settings: &mut Settings, providers: &[TranscriptionProvider]) -> Result<()> {
    println!("Select provider to configure:");
    for (i, provider) in providers.iter().enumerate() {
        println!(
            "  {}. {:<10} - {}",
            i + 1,
            provider.display_name(),
            provider_description(provider)
        );
    }
    println!();

    let choice = prompt_choice(
        &format!("Select (1-{})", providers.len()),
        1,
        providers.len(),
    )?;
    let provider = providers[choice - 1].clone();

    println!();
    println!("Get your API key from:");
    println!("  {}", api_key_url(&provider));
    println!();

    let api_key = prompt_and_validate_key(&provider)?;

    settings.transcription.provider = provider.clone();
    settings.transcription.set_api_key(&provider, api_key);

    finish_setup(settings, &provider)
}

/// Update an existing API key
fn update_existing_key(settings: &mut Settings, providers: &[TranscriptionProvider]) -> Result<()> {
    println!("Select provider to update:");
    for (i, provider) in providers.iter().enumerate() {
        let current_key = settings
            .transcription
            .api_key_for(provider)
            .unwrap_or_default();
        println!(
            "  {}. {} ({})",
            i + 1,
            provider.display_name(),
            mask_key(&current_key)
        );
    }
    println!();

    let choice = prompt_choice(
        &format!("Select (1-{})", providers.len()),
        1,
        providers.len(),
    )?;
    let provider = providers[choice - 1].clone();

    println!();
    println!(
        "Current key: {}",
        mask_key(
            &settings
                .transcription
                .api_key_for(&provider)
                .unwrap_or_default()
        )
    );
    interactive::info(&format!("Get a new key from: {}", api_key_url(&provider)));
    println!();

    let api_key = prompt_and_validate_key(&provider)?;

    settings.transcription.set_api_key(&provider, api_key);

    println!();
    println!("{} key updated.", provider.display_name());

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

    println!();
    println!("Transcription configured: {}", provider.display_name());
    println!();

    // Offer post-processing configuration
    let pp_display = match &default_post_processor {
        PostProcessor::None => "None (disabled)",
        PostProcessor::OpenAI => "OpenAI (same API key)",
        PostProcessor::Mistral => "Mistral (same API key)",
        PostProcessor::Ollama => "Ollama",
    };

    println!("Post-processing cleans up transcriptions (removes filler words, fixes grammar).");
    println!();
    println!("Would you like to configure post-processing?");
    println!("  1. Use {} (default)", pp_display);
    println!("  2. Configure different post-processor");
    println!("  3. Skip for now");
    println!();

    let choice = prompt_choice("Select (1-3)", 1, 3)?;

    match choice {
        1 => {
            // Keep default
        }
        2 => {
            // Run post-processing setup
            println!();
            configure_post_processing_options(settings)?;
        }
        3 => {
            // Skip - leave as default
        }
        _ => unreachable!(),
    }

    println!();
    println!("Setup complete!");
    println!();
    println!("Transcription: {}", provider.display_name());
    if settings.post_processing.processor != PostProcessor::None {
        println!("Post-processing: {}", settings.post_processing.processor);
    }
    println!();
    println!("Try it out:");
    println!("  whis                # Record and transcribe");
    println!("  whis --post-process # Record, transcribe, and post-process");
    println!();

    Ok(())
}

/// Streamlined cloud transcription setup (no post-processing config)
/// Used by the unified wizard
pub fn setup_transcription_cloud() -> Result<()> {
    let mut settings = Settings::load();

    // Build provider display items with [configured] marker
    use console::style;

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
                style(" [configured]").green().to_string()
            } else {
                String::new()
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
    let default = CLOUD_PROVIDERS
        .iter()
        .position(|p| {
            *p == settings.transcription.provider
                || (*p == TranscriptionProvider::OpenAI
                    && settings.transcription.provider == TranscriptionProvider::OpenAIRealtime)
                || (*p == TranscriptionProvider::Deepgram
                    && settings.transcription.provider == TranscriptionProvider::DeepgramRealtime)
        });

    let choice = interactive::select("Select provider", &items, default)?;
    let mut provider = CLOUD_PROVIDERS[choice].clone();

    // If OpenAI selected, ask for method (Standard vs Streaming)
    if provider == TranscriptionProvider::OpenAI {
        let methods = vec!["Standard - Batch processing", "Streaming - Real-time, lower latency"];

        // Default to current method if already using OpenAI variant
        let default_method =
            if settings.transcription.provider == TranscriptionProvider::OpenAIRealtime {
                1
            } else {
                0
            };

        println!();
        let method_choice = interactive::select("Select method", &methods, Some(default_method))?;
        if method_choice == 1 {
            provider = TranscriptionProvider::OpenAIRealtime;
        }
    }

    // If Deepgram selected, ask for method (Standard vs Streaming)
    if provider == TranscriptionProvider::Deepgram {
        let methods = vec![
            "Standard - Batch processing",
            "Streaming - Real-time, very fast (~150ms)",
        ];

        // Default to current method if already using Deepgram variant
        let default_method =
            if settings.transcription.provider == TranscriptionProvider::DeepgramRealtime {
                1
            } else {
                0
            };

        println!();
        let method_choice = interactive::select("Select method", &methods, Some(default_method))?;
        if method_choice == 1 {
            provider = TranscriptionProvider::DeepgramRealtime;
        }
    }

    // Check if API key already exists for this provider
    if let Some(existing_key) = settings.transcription.api_key_for(&provider) {
        println!();
        println!("Current API key: {}", mask_key(&existing_key));
        let keep = interactive::confirm("Keep current key?", true)?;

        if !keep {
            println!();
            interactive::info(&format!("Get your API key from: {}", api_key_url(&provider)));
            let api_key = prompt_and_validate_key(&provider)?;
            settings.transcription.set_api_key(&provider, api_key);
        }
    } else {
        // No existing key - prompt for new one
        println!();
        interactive::info(&format!("Get your API key from: {}", api_key_url(&provider)));
        let api_key = prompt_and_validate_key(&provider)?;
        settings.transcription.set_api_key(&provider, api_key);
    }

    settings.transcription.provider = provider;
    settings.save()?;

    Ok(())
}
