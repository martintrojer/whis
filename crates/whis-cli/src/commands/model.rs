//! Model listing commands for whisper, parakeet, and ollama

use anyhow::{Context, Result};
use serde::Deserialize;
use std::time::Duration;
use whis_core::model::{ModelType, WhisperModel};
use whis_core::ollama;

#[cfg(feature = "local-transcription")]
use whis_core::model::ParakeetModel;

use crate::args::{ModelAction, ModelType as ModelTypeArg};

/// Run the model command
pub fn run(action: Option<ModelAction>) -> Result<()> {
    match action {
        None | Some(ModelAction::List { model_type: None }) => list_whisper_models(),
        Some(ModelAction::List {
            model_type: Some(ModelTypeArg::Whisper),
        }) => list_whisper_models(),
        Some(ModelAction::List {
            model_type: Some(ModelTypeArg::Parakeet),
        }) => list_parakeet_models(),
        Some(ModelAction::List {
            model_type: Some(ModelTypeArg::Ollama { url }),
        }) => list_ollama_models(url),
    }
}

/// List available whisper models with install status
fn list_whisper_models() -> Result<()> {
    println!("Available whisper models:\n");

    // Calculate column widths
    let name_width = WhisperModel
        .models()
        .iter()
        .map(|model| model.name.len())
        .max()
        .unwrap_or(6)
        .max(4);

    // Print header
    println!(
        "{:<name_width$}  STATUS       DESCRIPTION",
        "NAME",
        name_width = name_width
    );
    println!("{}", "-".repeat(60));

    // Print each model
    for model in WhisperModel.models() {
        let path = WhisperModel.default_path(model.name);
        let status = if WhisperModel.verify(&path) {
            "[installed]"
        } else {
            ""
        };

        println!(
            "{:<name_width$}  {:<11}  {}",
            model.name,
            status,
            model.description,
            name_width = name_width
        );
    }

    println!();
    println!("Models directory: {}", WhisperModel.default_dir().display());
    println!();
    println!("To download a model, run: whis setup local");

    Ok(())
}

/// List available Parakeet models with install status
fn list_parakeet_models() -> Result<()> {
    println!("Available Parakeet models:\n");

    // Calculate column widths
    let name_width = ParakeetModel
        .models()
        .iter()
        .map(|model| model.name.len())
        .max()
        .unwrap_or(6)
        .max(4);

    // Print header
    println!(
        "{:<name_width$}  STATUS       DESCRIPTION",
        "NAME",
        name_width = name_width
    );
    println!("{}", "-".repeat(60));

    // Print each model
    for model in ParakeetModel.models() {
        let path = ParakeetModel.default_path(model.name);
        let status = if ParakeetModel.verify(&path) {
            "[installed]"
        } else {
            ""
        };

        println!(
            "{:<name_width$}  {:<11}  {}",
            model.name,
            status,
            model.description,
            name_width = name_width
        );
    }

    println!();
    println!(
        "Models directory: {}",
        WhisperModel.default_dir().join("parakeet").display()
    );
    println!();
    println!("To download a model, run: whis setup local");

    Ok(())
}

/// Response from Ollama /api/tags endpoint
#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<OllamaModel>,
}

/// Model info from Ollama
#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    #[serde(default)]
    size: u64,
}

/// List available Ollama models from server
fn list_ollama_models(url: Option<String>) -> Result<()> {
    let url = url.as_deref().unwrap_or(ollama::DEFAULT_OLLAMA_URL);

    // Check if Ollama is running
    if !ollama::is_ollama_running(url).unwrap_or(false) {
        println!("Ollama is not running at {}\n", url);
        println!("Start Ollama with: ollama serve");
        println!("Or specify a different URL: whis model list ollama --url http://...");
        return Ok(());
    }

    // Fetch models from Ollama
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .context("Failed to create HTTP client")?;

    let tags_url = format!("{}/api/tags", url.trim_end_matches('/'));
    let response = client
        .get(&tags_url)
        .send()
        .context("Failed to connect to Ollama")?;

    if !response.status().is_success() {
        anyhow::bail!("Ollama returned error: {}", response.status());
    }

    let tags: TagsResponse = response.json().context("Failed to parse Ollama response")?;

    if tags.models.is_empty() {
        println!("No models installed in Ollama at {}\n", url);
        println!("Pull a model with: ollama pull <model>");
        println!("Example: ollama pull qwen2.5:1.5b");
        return Ok(());
    }

    println!("Ollama models at {}:\n", url);

    // Calculate column widths
    let name_width = tags
        .models
        .iter()
        .map(|m| m.name.len())
        .max()
        .unwrap_or(4)
        .max(4);

    // Print header
    println!("{:<name_width$}  SIZE", "NAME", name_width = name_width);
    println!("{}", "-".repeat(name_width + 12));

    // Print each model
    for model in &tags.models {
        println!(
            "{:<name_width$}  {}",
            model.name,
            format_size(model.size),
            name_width = name_width
        );
    }

    Ok(())
}

/// Format bytes as human-readable size
fn format_size(bytes: u64) -> String {
    if bytes == 0 {
        return String::new();
    }

    if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.0} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.0} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}
