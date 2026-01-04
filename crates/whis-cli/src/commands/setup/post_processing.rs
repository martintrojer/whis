//! Post-processing setup (Ollama, OpenAI, Mistral)

use anyhow::{Result, anyhow};
use std::io::{self, Write};
use whis_core::{PostProcessor, Settings, TranscriptionProvider, ollama};

use super::cloud::prompt_and_validate_key;
use super::provider_helpers::{PP_PROVIDERS, api_key_url};
use crate::ui::{mask_key, prompt_choice, prompt_choice_with_default, prompt_yes_no};

/// Setup for post-processing configuration (standalone command)
pub fn setup_post_processing() -> Result<()> {
    println!("Post-Processing Setup");
    println!("=====================");
    println!();
    println!("Post-processing cleans up transcriptions (removes filler words, fixes grammar).");
    println!();

    let mut settings = Settings::load();

    // Show current status
    println!(
        "Current post-processor: {}",
        match settings.post_processing.processor {
            PostProcessor::None => "None (disabled)".to_string(),
            PostProcessor::OpenAI => "OpenAI".to_string(),
            PostProcessor::Mistral => "Mistral".to_string(),
            PostProcessor::Ollama => format!(
                "Ollama ({})",
                settings
                    .services
                    .ollama
                    .model
                    .as_deref()
                    .unwrap_or(ollama::DEFAULT_OLLAMA_MODEL)
            ),
        }
    );
    println!();

    // Choose post-processor type
    println!("Choose post-processor:");
    println!("  1. Ollama (local, free, recommended)");
    println!("  2. OpenAI (cloud, requires API key)");
    println!("  3. Mistral (cloud, requires API key)");
    println!("  4. None (disable post-processing)");
    println!();

    let choice = prompt_choice("Select (1-4)", 1, 4)?;
    println!();

    match choice {
        1 => {
            // Ollama setup
            let ollama_url = ollama::DEFAULT_OLLAMA_URL;

            // Check if Ollama is installed
            if !ollama::is_ollama_installed() {
                println!("Ollama is not installed.");
                println!();
                println!("Install Ollama:");
                println!("  Linux:  curl -fsSL https://ollama.com/install.sh | sh");
                println!("  macOS:  brew install ollama");
                println!("  Website: https://ollama.com/download");
                println!();
                return Err(anyhow!("Please install Ollama and run setup again"));
            }

            // Start Ollama if not running
            ollama::ensure_ollama_running(ollama_url)?;

            // Select model
            let model = select_ollama_model(ollama_url, settings.services.ollama.model.as_deref())?;

            settings.post_processing.processor = PostProcessor::Ollama;
            settings.services.ollama.url = Some(ollama_url.to_string());
            settings.services.ollama.model = Some(model.clone());
            settings.save()?;

            println!();
            println!("Setup complete!");
            println!("  Post-processor: Ollama ({})", model);
        }
        2 => {
            // OpenAI setup
            if settings
                .transcription
                .api_key_for(&TranscriptionProvider::OpenAI)
                .is_none()
            {
                println!("OpenAI API key not configured.");
                println!("Get your API key from: https://platform.openai.com/api-keys");
                println!();
                let api_key = prompt_and_validate_key(&TranscriptionProvider::OpenAI)?;
                settings
                    .transcription
                    .set_api_key(&TranscriptionProvider::OpenAI, api_key);
            }

            settings.post_processing.processor = PostProcessor::OpenAI;
            settings.save()?;

            println!();
            println!("Setup complete!");
            println!("  Post-processor: OpenAI");
        }
        3 => {
            // Mistral setup
            if settings
                .transcription
                .api_key_for(&TranscriptionProvider::Mistral)
                .is_none()
            {
                println!("Mistral API key not configured.");
                println!("Get your API key from: https://console.mistral.ai/api-keys");
                println!();
                let api_key = prompt_and_validate_key(&TranscriptionProvider::Mistral)?;
                settings
                    .transcription
                    .set_api_key(&TranscriptionProvider::Mistral, api_key);
            }

            settings.post_processing.processor = PostProcessor::Mistral;
            settings.save()?;

            println!();
            println!("Setup complete!");
            println!("  Post-processor: Mistral");
        }
        4 => {
            // Disable post-processing
            settings.post_processing.processor = PostProcessor::None;
            settings.save()?;

            println!("Post-processing disabled.");
        }
        _ => unreachable!(),
    }

    println!();
    println!("Try it out:");
    println!("  whis --post-process # Record, transcribe, and post-process");

    Ok(())
}

/// Configure post-processing options (used within cloud setup flow)
pub fn configure_post_processing_options(settings: &mut Settings) -> Result<()> {
    println!("Choose post-processor:");
    println!("  1. Ollama (local, free)");
    if settings
        .transcription
        .api_key_for(&TranscriptionProvider::OpenAI)
        .is_some()
    {
        println!("  2. OpenAI (cloud, uses existing key)");
    } else {
        println!("  2. OpenAI (cloud, requires API key)");
    }
    if settings
        .transcription
        .api_key_for(&TranscriptionProvider::Mistral)
        .is_some()
    {
        println!("  3. Mistral (cloud, uses existing key)");
    } else {
        println!("  3. Mistral (cloud, requires API key)");
    }
    println!("  4. None (disable post-processing)");
    println!();

    let choice = prompt_choice("Select (1-4)", 1, 4)?;
    println!();

    match choice {
        1 => {
            // Ollama setup
            let ollama_url = ollama::DEFAULT_OLLAMA_URL;

            // Check if Ollama is installed
            if !ollama::is_ollama_installed() {
                println!("Ollama is not installed.");
                println!();
                println!("Install Ollama:");
                println!("  Linux:  curl -fsSL https://ollama.com/install.sh | sh");
                println!("  macOS:  brew install ollama");
                println!("  Website: https://ollama.com/download");
                println!();
                println!("You can run 'whis setup post-processing' later to configure Ollama.");
                return Ok(());
            }

            // Start Ollama if not running
            ollama::ensure_ollama_running(ollama_url)?;

            // Select model
            let model = select_ollama_model(ollama_url, settings.services.ollama.model.as_deref())?;

            settings.post_processing.processor = PostProcessor::Ollama;
            settings.services.ollama.url = Some(ollama_url.to_string());
            settings.services.ollama.model = Some(model);
            settings.save()?;
        }
        2 => {
            // OpenAI
            if settings
                .transcription
                .api_key_for(&TranscriptionProvider::OpenAI)
                .is_none()
            {
                println!("OpenAI API key not configured.");
                println!("Get your API key from: https://platform.openai.com/api-keys");
                println!();
                let api_key = prompt_and_validate_key(&TranscriptionProvider::OpenAI)?;
                settings
                    .transcription
                    .set_api_key(&TranscriptionProvider::OpenAI, api_key);
            }
            settings.post_processing.processor = PostProcessor::OpenAI;
            settings.save()?;
        }
        3 => {
            // Mistral
            if settings
                .transcription
                .api_key_for(&TranscriptionProvider::Mistral)
                .is_none()
            {
                println!("Mistral API key not configured.");
                println!("Get your API key from: https://console.mistral.ai/api-keys");
                println!();
                let api_key = prompt_and_validate_key(&TranscriptionProvider::Mistral)?;
                settings
                    .transcription
                    .set_api_key(&TranscriptionProvider::Mistral, api_key);
            }
            settings.post_processing.processor = PostProcessor::Mistral;
            settings.save()?;
        }
        4 => {
            // Disable
            settings.post_processing.processor = PostProcessor::None;
            settings.save()?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Interactive Ollama model selection
/// Shows installed models + recommended options, allows pulling new models
pub fn select_ollama_model(url: &str, current_model: Option<&str>) -> Result<String> {
    // Get installed models from Ollama
    let installed = ollama::list_models(url).unwrap_or_default();
    let installed_names: Vec<&str> = installed.iter().map(|m| m.name.as_str()).collect();

    // Build menu options
    let mut options: Vec<(String, String, bool)> = Vec::new(); // (name, display, needs_download)

    // Add installed models first
    for model in &installed {
        let is_recommended = model.name.starts_with("qwen2.5:1.5b");
        let is_current = current_model == Some(&model.name);
        let size = if model.size > 0 {
            format!(" ({})", model.size_str())
        } else {
            String::new()
        };
        let markers = match (is_recommended, is_current) {
            (true, true) => " - Recommended [current]",
            (true, false) => " - Recommended",
            (false, true) => " [current]",
            (false, false) => "",
        };
        options.push((
            model.name.clone(),
            format!("{}{}{}", model.name, size, markers),
            false,
        ));
    }

    // Add recommended models that aren't installed
    println!("Select Ollama model:");
    let has_installed = !options.is_empty();
    if has_installed {
        for (i, (_, display, _)) in options.iter().enumerate() {
            println!("  {}. {}", i + 1, display);
        }
    }

    // Check which recommended models need to be added
    let mut other_options: Vec<(String, String, bool)> = Vec::new();
    for (name, size, desc) in ollama::OLLAMA_MODEL_OPTIONS {
        let is_installed = installed_names
            .iter()
            .any(|n| n.starts_with(name.split(':').next().unwrap_or(name)));
        if !is_installed {
            other_options.push((
                name.to_string(),
                format!("{} ({}) - {} [will download]", name, size, desc),
                true,
            ));
        }
    }

    if !other_options.is_empty() {
        if has_installed {
            println!();
            println!("Other options:");
        }
        for (i, (_, display, _)) in other_options.iter().enumerate() {
            println!("  {}. {}", options.len() + i + 1, display);
        }
        options.extend(other_options);
    }

    // Add custom model option
    let custom_index = options.len() + 1;
    println!();
    println!("  {}. Enter custom model name", custom_index);
    println!();

    // Determine default (current model or first recommended)
    let default = if let Some(current) = current_model {
        options
            .iter()
            .position(|(n, _, _)| n == current)
            .map(|i| i + 1)
    } else {
        Some(1)
    };

    let prompt = if let Some(d) = default {
        format!("Select (1-{}) [{}]", custom_index, d)
    } else {
        format!("Select (1-{})", custom_index)
    };

    let choice = prompt_choice_with_default(&prompt, 1, custom_index, default)?;

    if choice == custom_index {
        // Custom model
        print!("Enter model name: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let model_name = input.trim().to_string();

        if model_name.is_empty() {
            return Err(anyhow!("Model name cannot be empty"));
        }

        // Check if model exists, pull if needed
        if !ollama::has_model(url, &model_name)? {
            println!("Pulling model '{}'...", model_name);
            ollama::pull_model(url, &model_name)?;
        }

        return Ok(model_name);
    }

    // Selected from list
    let (model_name, _, needs_download) = &options[choice - 1];

    if *needs_download {
        println!("Pulling model '{}'...", model_name);
        ollama::pull_model(url, model_name)?;
    } else {
        println!("Using model: {}", model_name);
    }

    Ok(model_name.clone())
}

/// Helper to set up Ollama fresh (used by wizard)
pub fn setup_ollama_fresh(settings: &mut Settings) -> Result<()> {
    let ollama_url = ollama::DEFAULT_OLLAMA_URL;

    // Check if Ollama is installed
    if !ollama::is_ollama_installed() {
        println!();
        println!("Ollama not installed. Install from: https://ollama.com/download");
        println!("Run 'whis setup' again after installing.");
        settings.post_processing.processor = PostProcessor::None;
        return Ok(());
    }

    // Start Ollama
    print!("Starting Ollama... ");
    io::stdout().flush()?;
    ollama::ensure_ollama_running(ollama_url)?;
    println!("Done!");

    // Use default model
    let ollama_model = ollama::DEFAULT_OLLAMA_MODEL;
    if !ollama::has_model(ollama_url, ollama_model)? {
        print!("Downloading {}... ", ollama_model);
        io::stdout().flush()?;
        ollama::pull_model(ollama_url, ollama_model)?;
        println!("Done!");
    } else {
        println!("Model {} ready.", ollama_model);
    }

    settings.post_processing.processor = PostProcessor::Ollama;
    settings.services.ollama.url = Some(ollama_url.to_string());
    settings.services.ollama.model = Some(ollama_model.to_string());

    Ok(())
}

/// Independent post-processing step (called after transcription setup in wizard)
pub fn setup_post_processing_step(prefer_cloud: bool) -> Result<()> {
    let mut settings = Settings::load();

    println!();
    println!("Post-processing?");
    println!("  1. Cloud (OpenAI, Mistral)");
    println!("  2. Ollama (local, free)");
    println!("  3. Skip");
    println!();

    // Default: cloud if came from cloud transcription, Ollama if came from local
    let default = if prefer_cloud { 1 } else { 2 };
    let choice = prompt_choice_with_default("Select", 1, 3, Some(default))?;

    match choice {
        1 => setup_cloud_post_processing(&mut settings)?,
        2 => {
            // Check if Ollama already configured
            let ollama_configured = settings.post_processing.processor == PostProcessor::Ollama;
            if ollama_configured {
                let current_model = settings
                    .services
                    .ollama
                    .model
                    .as_deref()
                    .unwrap_or(ollama::DEFAULT_OLLAMA_MODEL);
                println!();
                println!("Current model: {}", current_model);
                let keep = prompt_yes_no("Keep current config?", true)?;

                if !keep {
                    setup_ollama_fresh(&mut settings)?;
                } else {
                    // Just verify Ollama is accessible
                    let ollama_url = settings
                        .services
                        .ollama
                        .url
                        .as_deref()
                        .unwrap_or(ollama::DEFAULT_OLLAMA_URL);
                    if ollama::is_ollama_installed()
                        && ollama::is_ollama_running(ollama_url).unwrap_or(false)
                    {
                        println!("Ollama ready.");
                    }
                }
            } else {
                setup_ollama_fresh(&mut settings)?;
            }
        }
        3 => {
            settings.post_processing.processor = PostProcessor::None;
        }
        _ => unreachable!(),
    }

    settings.save()?;
    Ok(())
}

/// Setup cloud post-processing (OpenAI or Mistral)
fn setup_cloud_post_processing(settings: &mut Settings) -> Result<()> {
    println!();
    println!("Provider:");
    for (i, provider) in PP_PROVIDERS.iter().enumerate() {
        let marker = if settings.transcription.api_key_for(provider).is_some() {
            " [configured]"
        } else {
            ""
        };
        println!("  {}. {}{}", i + 1, provider.display_name(), marker);
    }
    println!();

    let choice = prompt_choice_with_default("Select", 1, PP_PROVIDERS.len(), Some(1))?;
    let provider = PP_PROVIDERS[choice - 1].clone();

    // Check if API key already exists
    if let Some(existing_key) = settings.transcription.api_key_for(&provider) {
        println!();
        println!("Current API key: {}", mask_key(&existing_key));
        let keep = prompt_yes_no("Keep current key?", true)?;

        if !keep {
            println!();
            println!("Get your API key from: {}", api_key_url(&provider));
            let api_key = prompt_and_validate_key(&provider)?;
            settings.transcription.set_api_key(&provider, api_key);
        }
    } else {
        println!();
        println!("Get your API key from: {}", api_key_url(&provider));
        let api_key = prompt_and_validate_key(&provider)?;
        settings.transcription.set_api_key(&provider, api_key);
    }

    settings.post_processing.processor = match provider {
        TranscriptionProvider::OpenAI => PostProcessor::OpenAI,
        TranscriptionProvider::Mistral => PostProcessor::Mistral,
        _ => unreachable!(),
    };

    Ok(())
}
