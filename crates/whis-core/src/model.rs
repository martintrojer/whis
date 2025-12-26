//! Model download utilities
//!
//! Helps download and manage Whisper and Parakeet model files.

use anyhow::{Context, Result, anyhow};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// Available whisper models with their download URLs
pub const WHISPER_MODELS: &[(&str, &str, &str)] = &[
    (
        "tiny",
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin",
        "~75 MB - Fastest, lower quality",
    ),
    (
        "base",
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
        "~142 MB - Fast, decent quality",
    ),
    (
        "small",
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
        "~466 MB - Balanced (recommended)",
    ),
    (
        "medium",
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin",
        "~1.5 GB - Better quality, slower",
    ),
];

/// Default model for embedded setup
pub const DEFAULT_MODEL: &str = "small";

/// Get the default directory for storing whisper models
pub fn default_models_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("whis")
        .join("models")
}

/// Get the default path for a whisper model
pub fn default_model_path(model_name: &str) -> PathBuf {
    default_models_dir().join(format!("ggml-{}.bin", model_name))
}

/// Check if a model exists at the given path
pub fn model_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Get the URL for a model by name
pub fn get_model_url(name: &str) -> Option<&'static str> {
    WHISPER_MODELS
        .iter()
        .find(|(n, _, _)| *n == name)
        .map(|(_, url, _)| *url)
}

/// Download a whisper model with progress indication (prints to stderr)
pub fn download_model(model_name: &str, dest: &Path) -> Result<()> {
    download_model_with_progress(model_name, dest, |downloaded, total| {
        let progress = if total > 0 {
            (downloaded * 100 / total) as usize
        } else {
            0
        };
        eprint!(
            "\rDownloading: {}% ({:.1} MB / {:.1} MB)  ",
            progress,
            downloaded as f64 / 1_000_000.0,
            total as f64 / 1_000_000.0
        );
        io::stderr().flush().ok();
    })
}

/// Download a whisper model with a custom progress callback
///
/// The callback receives (downloaded_bytes, total_bytes) and is called
/// approximately every 1% of progress or every 500KB, whichever is more frequent.
pub fn download_model_with_progress<F>(model_name: &str, dest: &Path, on_progress: F) -> Result<()>
where
    F: Fn(u64, u64),
{
    let url = get_model_url(model_name).ok_or_else(|| {
        anyhow!(
            "Unknown model: {}. Available: tiny, base, small, medium",
            model_name
        )
    })?;

    // Create parent directory if needed
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).context("Failed to create models directory")?;
    }

    eprintln!("Downloading whisper model '{}'...", model_name);
    eprintln!("URL: {}", url);
    eprintln!("Destination: {}", dest.display());
    eprintln!();

    // Download with progress
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600)) // 10 min timeout for large files
        .build()
        .context("Failed to create HTTP client")?;

    let mut response = client.get(url).send().context("Failed to start download")?;

    if !response.status().is_success() {
        return Err(anyhow!("Download failed: HTTP {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);

    // Create temp file first, then rename on success
    let temp_path = dest.with_extension("bin.tmp");
    let mut file = fs::File::create(&temp_path).context("Failed to create temp file")?;

    let mut downloaded: u64 = 0;
    let mut buffer = [0u8; 8192];
    let mut last_callback_bytes: u64 = 0;

    // Emit initial progress
    on_progress(0, total_size);

    loop {
        let bytes_read = response.read(&mut buffer).context("Download interrupted")?;
        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .context("Failed to write to file")?;
        downloaded += bytes_read as u64;

        // Emit progress every ~1% or 500KB, whichever is more frequent
        let threshold = if total_size > 0 {
            (total_size / 100).min(500_000)
        } else {
            500_000
        };

        if downloaded - last_callback_bytes >= threshold {
            on_progress(downloaded, total_size);
            last_callback_bytes = downloaded;
        }
    }

    // Final progress callback
    on_progress(downloaded, total_size);

    eprintln!(
        "\rDownload complete: {:.1} MB                    ",
        downloaded as f64 / 1_000_000.0
    );

    // Rename temp to final
    fs::rename(&temp_path, dest).context("Failed to finalize download")?;

    Ok(())
}

/// Ensure a model is available, downloading if needed
pub fn ensure_model(model_name: &str) -> Result<PathBuf> {
    let path = default_model_path(model_name);

    if model_exists(&path) {
        return Ok(path);
    }

    download_model(model_name, &path)?;
    Ok(path)
}

/// List available models with descriptions
pub fn list_models() {
    eprintln!("Available whisper models:");
    eprintln!();
    for (name, _, desc) in WHISPER_MODELS {
        let path = default_model_path(name);
        let status = if model_exists(&path) {
            "[installed]"
        } else {
            ""
        };
        eprintln!("  {} - {} {}", name, desc, status);
    }
}

// ============================================================================
// Parakeet Models (NVIDIA Parakeet via ONNX)
// ============================================================================

/// Available Parakeet models with their download URLs
/// Format: (name, url, description, size_mb)
#[cfg(feature = "local-transcription")]
pub const PARAKEET_MODELS: &[(&str, &str, &str, u64)] = &[
    (
        "parakeet-v3",
        "https://blob.handy.computer/parakeet-v3-int8.tar.gz",
        "~478 MB - Fast & accurate (recommended)",
        478,
    ),
    (
        "parakeet-v2",
        "https://blob.handy.computer/parakeet-v2-int8.tar.gz",
        "~478 MB - Previous version",
        478,
    ),
];

/// Default Parakeet model for setup
pub const DEFAULT_PARAKEET_MODEL: &str = "parakeet-v3";

/// Directory name after extraction (tar.gz contains this directory)
#[cfg(feature = "local-transcription")]
fn parakeet_dirname(model_name: &str) -> String {
    match model_name {
        "parakeet-v3" => "parakeet-tdt-0.6b-v3-int8".to_string(),
        "parakeet-v2" => "parakeet-tdt-0.6b-v2-int8".to_string(),
        _ => format!("{}-int8", model_name),
    }
}

/// Get the default path for a Parakeet model directory
#[cfg(feature = "local-transcription")]
pub fn default_parakeet_model_path(model_name: &str) -> PathBuf {
    default_models_dir()
        .join("parakeet")
        .join(parakeet_dirname(model_name))
}

/// Check if a Parakeet model directory exists and is valid
#[cfg(feature = "local-transcription")]
pub fn parakeet_model_exists(path: &Path) -> bool {
    // Parakeet models are directories containing ONNX files
    if !path.exists() || !path.is_dir() {
        return false;
    }

    // Check for essential files
    let encoder = path.join("encoder-model.int8.onnx");
    let decoder = path.join("decoder_joint-model.int8.onnx");
    let vocab = path.join("vocab.txt");

    encoder.exists() && decoder.exists() && vocab.exists()
}

/// Get the URL for a Parakeet model by name
#[cfg(feature = "local-transcription")]
pub fn get_parakeet_model_url(name: &str) -> Option<&'static str> {
    PARAKEET_MODELS
        .iter()
        .find(|(n, _, _, _)| *n == name)
        .map(|(_, url, _, _)| *url)
}

/// Download a Parakeet model with progress indication
#[cfg(feature = "local-transcription")]
pub fn download_parakeet_model(model_name: &str, dest: &Path) -> Result<()> {
    download_parakeet_model_with_progress(model_name, dest, |downloaded, total| {
        let progress = if total > 0 {
            (downloaded * 100 / total) as usize
        } else {
            0
        };
        eprint!(
            "\rDownloading: {}% ({:.1} MB / {:.1} MB)  ",
            progress,
            downloaded as f64 / 1_000_000.0,
            total as f64 / 1_000_000.0
        );
        io::stderr().flush().ok();
    })
}

/// Download a Parakeet model with a custom progress callback
#[cfg(feature = "local-transcription")]
pub fn download_parakeet_model_with_progress<F>(
    model_name: &str,
    dest: &Path,
    on_progress: F,
) -> Result<()>
where
    F: Fn(u64, u64),
{
    use flate2::read::GzDecoder;
    use tar::Archive;

    let url = get_parakeet_model_url(model_name).ok_or_else(|| {
        anyhow!(
            "Unknown Parakeet model: {}. Available: parakeet-v3, parakeet-v2",
            model_name
        )
    })?;

    // Create parent directory if needed
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).context("Failed to create models directory")?;
    }

    eprintln!("Downloading Parakeet model '{}'...", model_name);
    eprintln!("URL: {}", url);
    eprintln!("Destination: {}", dest.display());
    eprintln!();

    // Download with progress to a temp file
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .context("Failed to create HTTP client")?;

    let mut response = client.get(url).send().context("Failed to start download")?;

    if !response.status().is_success() {
        return Err(anyhow!("Download failed: HTTP {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);

    // Download to temp file
    let temp_path = dest.with_extension("tar.gz.tmp");
    let mut file = fs::File::create(&temp_path).context("Failed to create temp file")?;

    let mut downloaded: u64 = 0;
    let mut buffer = [0u8; 8192];
    let mut last_callback_bytes: u64 = 0;

    on_progress(0, total_size);

    loop {
        let bytes_read = response.read(&mut buffer).context("Download interrupted")?;
        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .context("Failed to write to file")?;
        downloaded += bytes_read as u64;

        let threshold = if total_size > 0 {
            (total_size / 100).min(500_000)
        } else {
            500_000
        };

        if downloaded - last_callback_bytes >= threshold {
            on_progress(downloaded, total_size);
            last_callback_bytes = downloaded;
        }
    }

    on_progress(downloaded, total_size);
    drop(file);

    eprintln!(
        "\rDownload complete: {:.1} MB                    ",
        downloaded as f64 / 1_000_000.0
    );

    // Extract tar.gz
    eprintln!("Extracting model...");
    let tar_gz = fs::File::open(&temp_path).context("Failed to open downloaded archive")?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    // Extract to parent directory (archive contains the model directory)
    let parent = dest.parent().unwrap_or(dest);
    archive
        .unpack(parent)
        .context("Failed to extract model archive")?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_path);

    // Verify extraction
    if !parakeet_model_exists(dest) {
        return Err(anyhow!(
            "Model extraction failed: expected files not found in {}",
            dest.display()
        ));
    }

    eprintln!("Model ready: {}", dest.display());

    Ok(())
}

/// Ensure a Parakeet model is available, downloading if needed
#[cfg(feature = "local-transcription")]
pub fn ensure_parakeet_model(model_name: &str) -> Result<PathBuf> {
    let path = default_parakeet_model_path(model_name);

    if parakeet_model_exists(&path) {
        return Ok(path);
    }

    download_parakeet_model(model_name, &path)?;
    Ok(path)
}

/// List available Parakeet models with descriptions
#[cfg(feature = "local-transcription")]
pub fn list_parakeet_models() {
    eprintln!("Available Parakeet models:");
    eprintln!();
    for (name, _, desc, _) in PARAKEET_MODELS {
        let path = default_parakeet_model_path(name);
        let status = if parakeet_model_exists(&path) {
            "[installed]"
        } else {
            ""
        };
        eprintln!("  {} - {} {}", name, desc, status);
    }
}
