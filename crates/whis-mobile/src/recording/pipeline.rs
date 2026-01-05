//! Transcription pipeline utilities.
//!
//! Handles post-processing, clipboard operations, and event emission.
//! This mirrors the pattern in whis-desktop's recording/pipeline.rs.

use tauri::Emitter;
use tauri_plugin_clipboard_manager::ClipboardExt;
use whis_core::preset::Preset;
use whis_core::{PostProcessor, post_process};

use crate::commands::presets::get_presets_dir;

/// Check if post-processing is enabled (has post-processor and active preset).
pub fn is_post_processing_enabled(store: &tauri_plugin_store::Store<tauri::Wry>) -> bool {
    // Check post-processor setting
    let post_processor_str = store
        .get("post_processor")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "none".to_string());

    if post_processor_str == "none" {
        return false;
    }

    // Check active preset exists
    store
        .get("active_preset")
        .and_then(|v| v.as_str().map(String::from))
        .is_some()
}

/// Apply post-processing to transcription if enabled.
///
/// Post-processing is applied when:
/// 1. An active preset is set, AND
/// 2. A post-processor is configured (not "none")
///
/// Returns the processed text, or original text on error/skip.
pub async fn apply_post_processing(
    app: &tauri::AppHandle,
    text: String,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> String {
    // Get post-processor setting
    let post_processor_str = store
        .get("post_processor")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "none".to_string());

    let post_processor: PostProcessor = post_processor_str.parse().unwrap_or(PostProcessor::None);

    // Skip if disabled
    if post_processor == PostProcessor::None {
        return text;
    }

    // Get active preset - post-processing only works with a preset
    let active_preset = store
        .get("active_preset")
        .and_then(|v| v.as_str().map(String::from));

    let preset = match active_preset {
        Some(name) => {
            // Use Tauri's app config dir for presets (works on Android)
            let presets_dir = match get_presets_dir(app) {
                Ok(dir) => dir,
                Err(e) => {
                    eprintln!("Failed to get presets dir: {}", e);
                    return text;
                }
            };
            match Preset::load_from(&name, &presets_dir) {
                Ok((preset, _)) => preset,
                Err(e) => {
                    eprintln!("Failed to load preset '{}': {}", name, e);
                    return text;
                }
            }
        }
        None => {
            // No preset active, skip post-processing
            return text;
        }
    };

    // Get API key for post-processor
    let api_key = match post_processor {
        PostProcessor::OpenAI => store.get("openai_api_key"),
        PostProcessor::Mistral => store.get("mistral_api_key"),
        _ => None,
    }
    .and_then(|v| v.as_str().map(String::from));

    let api_key = match api_key {
        Some(key) if !key.is_empty() => key,
        _ => {
            eprintln!(
                "Post-processing: No API key configured for {}",
                post_processor
            );
            return text;
        }
    };

    // Apply post-processing with preset's prompt
    match post_process(&text, &post_processor, &api_key, &preset.prompt, None).await {
        Ok(processed) => processed,
        Err(e) => {
            eprintln!("Post-processing failed: {}", e);
            let _ = app.emit("post-process-warning", e.to_string());
            text // Return original on error
        }
    }
}

/// Copy text to clipboard and emit completion event.
pub fn finish_transcription(app: &tauri::AppHandle, text: &str) -> Result<(), String> {
    // Copy to clipboard
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("Clipboard error: {}", e))?;

    // Emit completion event
    let _ = app.emit("transcription-complete", text.to_string());

    Ok(())
}
