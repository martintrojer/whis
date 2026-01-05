//! Post-processing setup (Ollama, OpenAI, Mistral)

use anyhow::{Result, anyhow};
use std::io::{self, Write};
use whis_core::{PostProcessor, Settings, TranscriptionProvider, ollama};

use super::cloud::prompt_and_validate_key;
use super::interactive;
use super::provider_helpers::{PP_PROVIDERS, api_key_url};
use crate::ui::{mask_key, prompt_choice};

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
    use console::style;

    // Get installed models from Ollama
    let installed = ollama::list_models(url).unwrap_or_default();
    let installed_names: Vec<&str> = installed.iter().map(|m| m.name.as_str()).collect();

    // Build display items and parallel model data
    let mut items = Vec::new();
    let mut model_data: Vec<Option<(String, bool)>> = Vec::new(); // (name, needs_download)

    // Installed section
    if !installed.is_empty() {
        items.push(style("─── Installed ───").dim().to_string());
        model_data.push(None); // Separator

        for model in &installed {
            let is_recommended = model.name.starts_with("qwen2.5:1.5b");
            let is_current = current_model == Some(&model.name);
            let size = if model.size > 0 {
                format!(" ({})", model.size_str())
            } else {
                String::new()
            };
            let markers = match (is_recommended, is_current) {
                (true, true) => style(" - Recommended [current]").green().to_string(),
                (true, false) => " - Recommended".to_string(),
                (false, true) => style(" [current]").green().to_string(),
                (false, false) => String::new(),
            };
            items.push(format!("{}{}{}", model.name, size, markers));
            model_data.push(Some((model.name.clone(), false)));
        }
    }

    // Recommended section (not installed)
    let not_installed: Vec<_> = ollama::OLLAMA_MODEL_OPTIONS
        .iter()
        .filter(|(name, _, _)| {
            !installed_names
                .iter()
                .any(|n| n.starts_with(name.split(':').next().unwrap_or(name)))
        })
        .collect();

    if !not_installed.is_empty() {
        items.push(style("─── Available (will download) ───").dim().to_string());
        model_data.push(None); // Separator

        for (name, size, desc) in not_installed {
            items.push(format!("{} ({}) - {}", name, size, desc));
            model_data.push(Some((name.to_string(), true)));
        }
    }

    // Custom option
    items.push(style("─── Custom ───").dim().to_string());
    model_data.push(None); // Separator
    items.push("Enter custom model name".to_string());
    model_data.push(None); // Custom trigger

    // Find default index (skip separators)
    let default = if let Some(current) = current_model {
        model_data
            .iter()
            .position(|m| m.as_ref().map(|(n, _)| n.as_str()) == Some(current))
    } else {
        // First real model (skip first separator)
        model_data.iter().position(|m| m.is_some())
    };

    // Interactive select
    let choice = interactive::select("Select Ollama model", &items, default)?;

    // Handle selection
    match &model_data[choice] {
        Some((model_name, needs_download)) => {
            // Selected a model from the list
            if *needs_download {
                println!("Pulling model '{}'...", model_name);
                ollama::pull_model(url, model_name)?;
            } else {
                println!("Using model: {}", model_name);
            }
            Ok(model_name.clone())
        }
        None => {
            // Either separator or custom model
            if items[choice].contains("custom") {
                // Custom model input
                let model_name = interactive::input("Enter model name (e.g., llama3.2:1b)", None)?;

                if model_name.is_empty() {
                    return Err(anyhow!("Model name cannot be empty"));
                }

                // Check if model exists, pull if needed
                if !ollama::has_model(url, &model_name)? {
                    println!("Pulling model '{}'...", model_name);
                    ollama::pull_model(url, &model_name)?;
                }

                Ok(model_name)
            } else {
                // Separator selected - shouldn't happen with proper navigation
                Err(anyhow!("Invalid selection"))
            }
        }
    }
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
    let options = vec![
        "Cloud (OpenAI/Mistral) - Requires API key",
        "Ollama (local) - Free, runs on your machine",
        "Skip",
    ];
    let default = if prefer_cloud { 0 } else { 1 };
    let choice = interactive::select("Select post-processing", &options, Some(default))?;

    match choice {
        0 => setup_cloud_post_processing(&mut settings)?,
        1 => {
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
                let keep = interactive::confirm("Keep current config?", true)?;

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
        2 => {
            settings.post_processing.processor = PostProcessor::None;
        }
        _ => unreachable!(),
    }

    settings.save()?;
    Ok(())
}

/// Setup cloud post-processing (OpenAI or Mistral)
fn setup_cloud_post_processing(settings: &mut Settings) -> Result<()> {
    use console::style;

    println!();

    // Build provider items with [configured] marker
    let items: Vec<String> = PP_PROVIDERS
        .iter()
        .map(|provider| {
            let marker = if settings.transcription.api_key_for(provider).is_some() {
                style(" [configured]").green().to_string()
            } else {
                String::new()
            };
            format!("{}{}", provider.display_name(), marker)
        })
        .collect();

    let choice = interactive::select("Select provider", &items, Some(0))?;
    let provider = PP_PROVIDERS[choice].clone();

    // Check if API key already exists
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
