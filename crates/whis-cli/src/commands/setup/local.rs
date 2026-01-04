//! Local transcription setup

use anyhow::{Result, anyhow};
use std::io::{self, Write};
use whis_core::{PostProcessor, Settings, TranscriptionProvider, model, ollama};

use super::post_processing::select_ollama_model;
use crate::ui::{prompt_choice, prompt_choice_with_default};

#[cfg(feature = "local-transcription")]
use whis_core::model::{ModelType, ParakeetModel, WhisperModel};

/// Setup for fully local (on-device) transcription (full interactive wizard)
pub fn setup_local() -> Result<()> {
    println!("Local Setup");
    println!("===========");
    println!();
    println!("This will set up fully local transcription:");
    println!("  - Transcription model (runs on CPU)");
    println!("  - Ollama for transcript post-processing (runs locally)");
    println!();

    // Step 1: Choose transcription engine
    println!("Step 1: Choose Transcription Engine");
    println!("------------------------------------");
    println!("  1. Parakeet - RECOMMENDED (NVIDIA model, fast & accurate)");
    println!("  2. Whisper - OpenAI model (multiple sizes available)");
    println!();

    let engine_choice = prompt_choice("Select engine (1-2)", 1, 2)?;
    println!();

    let (provider, model_path) = match engine_choice {
        1 => {
            // Parakeet setup - show available models
            println!("Step 2: Choose Parakeet Model");
            println!("-----------------------------");
            for (i, model) in ParakeetModel.models().iter().enumerate() {
                let path = ParakeetModel.default_path(model.name);
                let status = if ParakeetModel.verify(&path) {
                    " [installed]"
                } else {
                    ""
                };
                println!(
                    "  {}. {} - {}{}",
                    i + 1,
                    model.name,
                    model.description,
                    status
                );
            }
            println!();

            let model_choice = prompt_choice(
                &format!("Select model (1-{})", ParakeetModel.models().len()),
                1,
                ParakeetModel.models().len(),
            )?;
            let model = &ParakeetModel.models()[model_choice - 1];
            println!();

            println!("Setting up {}...", model.name);
            let path = ParakeetModel.default_path(model.name);
            if ParakeetModel.verify(&path) {
                println!("Model '{}' already installed at:", model.name);
                println!("  {}", path.display());
            } else {
                println!("Downloading '{}' model...", model.name);
                model::download::download(&ParakeetModel, model.name, &path)?;
            }
            (TranscriptionProvider::LocalParakeet, path)
        }
        2 => {
            // Whisper setup - show available models
            println!("Step 2: Choose Whisper Model");
            println!("----------------------------");
            for (i, model) in WhisperModel.models().iter().enumerate() {
                let path = WhisperModel.default_path(model.name);
                let status = if WhisperModel.verify(&path) {
                    " [installed]"
                } else {
                    ""
                };
                println!(
                    "  {}. {} - {}{}",
                    i + 1,
                    model.name,
                    model.description,
                    status
                );
            }
            println!();

            let model_choice = prompt_choice(
                &format!("Select model (1-{})", WhisperModel.models().len()),
                1,
                WhisperModel.models().len(),
            )?;
            let model = &WhisperModel.models()[model_choice - 1];
            println!();

            println!("Setting up {}...", model.name);
            let path = WhisperModel.default_path(model.name);
            if WhisperModel.verify(&path) {
                println!("Model '{}' already installed at:", model.name);
                println!("  {}", path.display());
            } else {
                println!("Downloading '{}' model...", model.name);
                model::download::download(&WhisperModel, model.name, &path)?;
            }
            (TranscriptionProvider::LocalWhisper, path)
        }
        _ => unreachable!(),
    };
    println!();

    // Step 3: Setup Ollama
    println!("Step 3: Ollama (for post-processing)");
    println!("------------------------------------");

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

    // Let user select model (shows installed + recommended options)
    let ollama_model = select_ollama_model(ollama_url, None)?;
    println!();

    // Step 4: Save configuration
    println!("Step 4: Saving Configuration");
    println!("----------------------------");

    let mut settings = Settings::load();
    settings.transcription.provider = provider.clone();
    match &provider {
        TranscriptionProvider::LocalParakeet => {
            settings.transcription.local_models.parakeet_path =
                Some(model_path.to_string_lossy().to_string());
        }
        TranscriptionProvider::LocalWhisper => {
            settings.transcription.local_models.whisper_path =
                Some(model_path.to_string_lossy().to_string());
        }
        _ => {}
    }
    settings.post_processing.processor = PostProcessor::Ollama;
    settings.services.ollama.url = Some(ollama_url.to_string());
    settings.services.ollama.model = Some(ollama_model.clone());
    settings.save()?;

    println!("Configuration saved to: {}", Settings::path().display());
    println!();
    println!("Setup complete!");
    println!();
    println!("Your setup:");
    println!("  Transcription:    {}", provider.display_name());
    println!("  Post-processing:  Ollama ({})", ollama_model);
    println!();
    println!("Try it out:");
    println!("  whis                # Record and transcribe locally");
    println!("  whis --post-process # Record, transcribe, and post-process locally");
    println!();
    println!("Note: Ollama will auto-start when needed.");

    Ok(())
}

/// Streamlined local transcription setup (no post-processing config)
/// Used by the unified wizard
pub fn setup_transcription_local() -> Result<()> {
    let mut settings = Settings::load();

    // Determine current engine and show with [current] marker
    let current_engine = match settings.transcription.provider {
        TranscriptionProvider::LocalParakeet => Some(1),
        TranscriptionProvider::LocalWhisper => Some(2),
        _ => None,
    };

    println!("Model:");
    let parakeet_marker = if current_engine == Some(1) {
        " [current]"
    } else {
        ""
    };
    let whisper_marker = if current_engine == Some(2) {
        " [current]"
    } else {
        ""
    };
    println!(
        "  1. Parakeet (recommended) - Fast, accurate{}",
        parakeet_marker
    );
    println!(
        "  2. Whisper - OpenAI model, multiple sizes{}",
        whisper_marker
    );
    println!();

    // Default to current engine if local, otherwise Parakeet
    let default = current_engine.unwrap_or(1);
    let engine_choice = prompt_choice_with_default("Select", 1, 2, Some(default))?;
    println!();

    let (provider, model_path) = match engine_choice {
        1 => {
            // Parakeet - use recommended model directly
            let model = &ParakeetModel.models()[0]; // First is recommended
            let path = ParakeetModel.default_path(model.name);

            if ParakeetModel.verify(&path) {
                println!("Model ready: {}", model.name);
            } else {
                print!("Downloading {}... ", model.name);
                io::stdout().flush()?;
                model::download::download(&ParakeetModel, model.name, &path)?;
                println!("Done!");
            }

            (TranscriptionProvider::LocalParakeet, path)
        }
        2 => {
            // Whisper - show model options
            println!("Whisper model:");
            for (i, model) in WhisperModel.models().iter().enumerate() {
                let path = WhisperModel.default_path(model.name);
                let status = if WhisperModel.verify(&path) {
                    " [installed]"
                } else {
                    ""
                };
                println!(
                    "  {}. {} - {}{}",
                    i + 1,
                    model.name,
                    model.description,
                    status
                );
            }
            println!();

            let model_choice = prompt_choice_with_default(
                "Select",
                1,
                WhisperModel.models().len(),
                Some(2), // Default to "base" which is usually index 2
            )?;
            let model = &WhisperModel.models()[model_choice - 1];
            println!();

            let path = WhisperModel.default_path(model.name);
            if WhisperModel.verify(&path) {
                println!("Model ready: {}", model.name);
            } else {
                print!("Downloading {}... ", model.name);
                io::stdout().flush()?;
                model::download::download(&WhisperModel, model.name, &path)?;
                println!("Done!");
            }

            (TranscriptionProvider::LocalWhisper, path)
        }
        _ => unreachable!(),
    };

    // Save transcription config
    settings.transcription.provider = provider.clone();
    match &provider {
        TranscriptionProvider::LocalParakeet => {
            settings.transcription.local_models.parakeet_path =
                Some(model_path.to_string_lossy().to_string());
        }
        TranscriptionProvider::LocalWhisper => {
            settings.transcription.local_models.whisper_path =
                Some(model_path.to_string_lossy().to_string());
        }
        _ => {}
    }
    settings.save()?;

    Ok(())
}
