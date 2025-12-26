use crate::shortcuts::ShortcutBackendInfo;
use crate::state::AppState;
use tauri::{AppHandle, State};
use whis_core::{RecordingState, Settings};

#[derive(serde::Serialize)]
pub struct StatusResponse {
    pub state: String,
    pub config_valid: bool,
}

#[derive(serde::Serialize)]
pub struct SaveSettingsResponse {
    pub needs_restart: bool,
}

#[tauri::command]
pub async fn is_api_configured(state: State<'_, AppState>) -> Result<bool, String> {
    let settings = state.settings.lock().unwrap();
    Ok(settings.has_api_key())
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusResponse, String> {
    let current_state = *state.state.lock().unwrap();

    // Check if API key is configured for the current provider
    let config_valid = {
        let has_cached_config = state.transcription_config.lock().unwrap().is_some();
        let settings = state.settings.lock().unwrap();
        has_cached_config || settings.is_provider_configured()
    };

    Ok(StatusResponse {
        state: match current_state {
            RecordingState::Idle => "Idle".to_string(),
            RecordingState::Recording => "Recording".to_string(),
            RecordingState::Transcribing => "Transcribing".to_string(),
        },
        config_valid,
    })
}

#[tauri::command]
pub async fn toggle_recording(app: AppHandle) -> Result<(), String> {
    crate::tray::toggle_recording_public(app);
    Ok(())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let mut settings = state.settings.lock().unwrap();
    // Refresh from disk to ensure latest
    *settings = Settings::load();
    Ok(settings.clone())
}

#[tauri::command]
pub fn shortcut_backend() -> ShortcutBackendInfo {
    crate::shortcuts::backend_info()
}

#[tauri::command]
pub async fn configure_shortcut(app: AppHandle) -> Result<Option<String>, String> {
    crate::shortcuts::open_configure_shortcuts(app)
        .await
        .map_err(|e| e.to_string())
}

/// Configure shortcut with a preferred trigger from in-app key capture
/// The trigger should be in human-readable format like "Ctrl+Alt+W" or "Cmd+Option+W"
#[tauri::command]
pub async fn configure_shortcut_with_trigger(
    app: AppHandle,
    trigger: String,
) -> Result<Option<String>, String> {
    crate::shortcuts::configure_with_preferred_trigger(Some(&trigger), app)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn portal_shortcut(state: State<'_, AppState>) -> Result<Option<String>, String> {
    // First check if we have it cached in state
    let cached = state.portal_shortcut.lock().unwrap().clone();
    if cached.is_some() {
        return Ok(cached);
    }

    // Otherwise try reading from dconf (GNOME stores shortcuts there)
    Ok(crate::shortcuts::read_portal_shortcut_from_dconf())
}

#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<SaveSettingsResponse, String> {
    // Check what changed
    let (config_changed, shortcut_changed) = {
        let current = state.settings.lock().unwrap();
        (
            current.provider != settings.provider
                || current.api_keys != settings.api_keys
                || current.language != settings.language
                || current.whisper_model_path != settings.whisper_model_path,
            current.shortcut != settings.shortcut,
        )
    };

    {
        let mut state_settings = state.settings.lock().unwrap();
        *state_settings = settings.clone();
        state_settings.save().map_err(|e| e.to_string())?;
    }

    // Clear cached transcription config if provider or API key changed
    if config_changed {
        *state.transcription_config.lock().unwrap() = None;
    }

    // Only update shortcut if it actually changed
    let needs_restart = if shortcut_changed {
        crate::shortcuts::update_shortcut(&app, &settings.shortcut).map_err(|e| e.to_string())?
    } else {
        false
    };

    Ok(SaveSettingsResponse { needs_restart })
}

#[tauri::command]
pub fn validate_openai_api_key(api_key: String) -> Result<bool, String> {
    // Validate format: OpenAI keys start with "sk-"
    if api_key.is_empty() {
        return Ok(true); // Empty is valid (will fall back to env var)
    }

    if !api_key.starts_with("sk-") {
        return Err("Invalid key format. OpenAI keys start with 'sk-'".to_string());
    }

    Ok(true)
}

#[tauri::command]
pub fn validate_mistral_api_key(api_key: String) -> Result<bool, String> {
    // Empty is valid (will fall back to env var)
    if api_key.is_empty() {
        return Ok(true);
    }

    // Basic validation: Mistral keys should be reasonably long
    let trimmed = api_key.trim();
    if trimmed.len() < 20 {
        return Err("Invalid Mistral API key: key appears too short".to_string());
    }

    Ok(true)
}

#[tauri::command]
pub fn validate_groq_api_key(api_key: String) -> Result<bool, String> {
    if api_key.is_empty() {
        return Ok(true); // Empty is valid (will fall back to env var)
    }

    if !api_key.starts_with("gsk_") {
        return Err("Invalid key format. Groq keys start with 'gsk_'".to_string());
    }

    Ok(true)
}

#[tauri::command]
pub fn validate_deepgram_api_key(api_key: String) -> Result<bool, String> {
    if api_key.is_empty() {
        return Ok(true);
    }

    if api_key.trim().len() < 20 {
        return Err("Invalid Deepgram API key: key appears too short".to_string());
    }

    Ok(true)
}

#[tauri::command]
pub fn validate_elevenlabs_api_key(api_key: String) -> Result<bool, String> {
    if api_key.is_empty() {
        return Ok(true);
    }

    if api_key.trim().len() < 20 {
        return Err("Invalid ElevenLabs API key: key appears too short".to_string());
    }

    Ok(true)
}

/// Reset portal shortcuts by clearing dconf (GNOME)
/// This allows rebinding after restart
#[cfg(target_os = "linux")]
#[tauri::command]
pub fn reset_shortcut() -> Result<(), String> {
    std::process::Command::new("dconf")
        .args([
            "reset",
            "-f",
            "/org/gnome/settings-daemon/global-shortcuts/",
        ])
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub fn reset_shortcut() -> Result<(), String> {
    Ok(())
}

/// Get any error from portal shortcut binding
#[tauri::command]
pub fn portal_bind_error(state: State<'_, AppState>) -> Option<String> {
    state.portal_bind_error.lock().unwrap().clone()
}

/// Get the correct toggle command based on installation type
#[tauri::command]
pub fn get_toggle_command() -> String {
    if std::path::Path::new("/.flatpak-info").exists() {
        "flatpak run ink.whis.Whis --toggle".to_string()
    } else {
        "whis-desktop --toggle".to_string()
    }
}

/// Check if user can reopen the window after closing
/// Returns true if tray is available OR a working shortcut exists
#[tauri::command]
pub fn can_reopen_window(state: State<'_, AppState>) -> bool {
    // If tray is available, user can always reopen from there
    if *state.tray_available.lock().unwrap() {
        return true;
    }

    // Check shortcut backend - some always work, some need verification
    let backend_info = crate::shortcuts::backend_info();
    match backend_info.backend.as_str() {
        "TauriPlugin" => true, // X11 shortcuts always work
        "ManualSetup" => true, // IPC toggle always available
        "PortalGlobalShortcuts" => {
            // Portal needs a bound shortcut without errors
            let has_shortcut = state.portal_shortcut.lock().unwrap().is_some();
            let no_error = state.portal_bind_error.lock().unwrap().is_none();
            has_shortcut && no_error
        }
        _ => false,
    }
}

/// Progress event payload for model download
#[derive(Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
}

/// Download a whisper model for local transcription
/// Emits 'download-progress' events with { downloaded, total } during download
/// Returns the path where the model was saved
#[tauri::command]
pub async fn download_whisper_model(app: AppHandle, model_name: String) -> Result<String, String> {
    use tauri::Emitter;

    // Run blocking download in a separate thread
    tauri::async_runtime::spawn_blocking(move || {
        let path = whis_core::model::default_model_path(&model_name);

        // Skip download if model already exists
        if path.exists() {
            return Ok(path.to_string_lossy().to_string());
        }

        // Download with progress callback
        whis_core::model::download_model_with_progress(&model_name, &path, |downloaded, total| {
            let _ = app.emit("download-progress", DownloadProgress { downloaded, total });
        })
        .map_err(|e| e.to_string())?;

        Ok(path.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Check if the configured whisper model path points to an existing file
#[tauri::command]
pub fn is_whisper_model_valid(state: State<'_, AppState>) -> bool {
    let settings = state.settings.lock().unwrap();
    settings
        .get_whisper_model_path()
        .map(|p| std::path::Path::new(&p).exists())
        .unwrap_or(false)
}

/// Get available whisper models for download
#[tauri::command]
pub fn get_whisper_models() -> Vec<WhisperModelInfo> {
    whis_core::model::WHISPER_MODELS
        .iter()
        .map(|(name, _, desc)| {
            let path = whis_core::model::default_model_path(name);
            WhisperModelInfo {
                name: name.to_string(),
                description: desc.to_string(),
                installed: path.exists(),
                path: path.to_string_lossy().to_string(),
            }
        })
        .collect()
}

#[derive(serde::Serialize)]
pub struct WhisperModelInfo {
    pub name: String,
    pub description: String,
    pub installed: bool,
    pub path: String,
}

/// Preset info for the UI
#[derive(serde::Serialize)]
pub struct PresetInfo {
    pub name: String,
    pub description: String,
    pub is_builtin: bool,
}

/// List all available presets (built-in + user)
#[tauri::command]
pub fn list_presets() -> Vec<PresetInfo> {
    use whis_core::preset::{Preset, PresetSource};

    Preset::list_all()
        .into_iter()
        .map(|(p, source)| PresetInfo {
            name: p.name,
            description: p.description,
            is_builtin: source == PresetSource::BuiltIn,
        })
        .collect()
}

/// Apply a preset - updates settings with the preset's configuration and sets it as active
#[tauri::command]
pub async fn apply_preset(name: String, state: State<'_, AppState>) -> Result<(), String> {
    use whis_core::preset::Preset;

    let (preset, _) = Preset::load(&name)?;

    {
        let mut settings = state.settings.lock().unwrap();

        // Apply preset's post-processing prompt
        settings.post_processing_prompt = Some(preset.prompt.clone());

        // Apply preset's post-processor override if specified
        if let Some(post_processor_str) = &preset.post_processor
            && let Ok(post_processor) = post_processor_str.parse()
        {
            settings.post_processor = post_processor;
        }

        // Set this preset as active
        settings.active_preset = Some(name);

        // Save the settings
        settings.save().map_err(|e| e.to_string())?;
    }

    // Clear cached transcription config since settings changed
    *state.transcription_config.lock().unwrap() = None;

    Ok(())
}

/// Get the active preset name (if any)
#[tauri::command]
pub fn get_active_preset(state: State<'_, AppState>) -> Option<String> {
    let settings = state.settings.lock().unwrap();
    settings.active_preset.clone()
}

/// Set the active preset
#[tauri::command]
pub async fn set_active_preset(
    name: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut settings = state.settings.lock().unwrap();
    settings.active_preset = name;
    settings.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Full preset details for editing
#[derive(serde::Serialize)]
pub struct PresetDetails {
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub post_processor: Option<String>,
    pub model: Option<String>,
    pub is_builtin: bool,
}

/// Input for creating a new preset
#[derive(serde::Deserialize)]
pub struct CreatePresetInput {
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub post_processor: Option<String>,
    pub model: Option<String>,
}

/// Input for updating an existing preset
#[derive(serde::Deserialize)]
pub struct UpdatePresetInput {
    pub description: String,
    pub prompt: String,
    pub post_processor: Option<String>,
    pub model: Option<String>,
}

/// Get full details of a preset for viewing/editing
#[tauri::command]
pub fn get_preset_details(name: String) -> Result<PresetDetails, String> {
    use whis_core::preset::{Preset, PresetSource};

    let (preset, source) = Preset::load(&name)?;

    Ok(PresetDetails {
        name: preset.name,
        description: preset.description,
        prompt: preset.prompt,
        post_processor: preset.post_processor,
        model: preset.model,
        is_builtin: source == PresetSource::BuiltIn,
    })
}

/// Create a new user preset
#[tauri::command]
pub fn create_preset(input: CreatePresetInput) -> Result<PresetInfo, String> {
    use whis_core::preset::Preset;

    // Validate name
    Preset::validate_name(&input.name, false)?;

    // Check if preset already exists
    if Preset::load(&input.name).is_ok() {
        return Err(format!("A preset named '{}' already exists", input.name));
    }

    // Create and save the preset
    let preset = Preset {
        name: input.name.clone(),
        description: input.description.clone(),
        prompt: input.prompt,
        post_processor: input.post_processor,
        model: input.model,
    };

    preset.save()?;

    Ok(PresetInfo {
        name: input.name,
        description: input.description,
        is_builtin: false,
    })
}

/// Update an existing user preset
#[tauri::command]
pub fn update_preset(name: String, input: UpdatePresetInput) -> Result<PresetInfo, String> {
    use whis_core::preset::Preset;

    // Check it's not a built-in
    if Preset::is_builtin(&name) {
        return Err(format!("Cannot edit built-in preset '{}'", name));
    }

    // Check preset exists
    let (mut preset, _) = Preset::load(&name)?;

    // Update fields
    preset.description = input.description.clone();
    preset.prompt = input.prompt;
    preset.post_processor = input.post_processor;
    preset.model = input.model;

    // Save
    preset.save()?;

    Ok(PresetInfo {
        name,
        description: input.description,
        is_builtin: false,
    })
}

/// Delete a user preset
#[tauri::command]
pub fn delete_preset(name: String, state: State<'_, AppState>) -> Result<(), String> {
    use whis_core::preset::Preset;

    // Delete the preset file
    Preset::delete(&name)?;

    // If this was the active preset, clear it
    {
        let mut settings = state.settings.lock().unwrap();
        if settings.active_preset.as_deref() == Some(&name) {
            settings.active_preset = None;
            settings.save().map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

/// Test connection to Ollama server
/// Must be async with spawn_blocking because reqwest::blocking::Client
/// creates an internal tokio runtime that would panic if called from Tauri's async context
#[tauri::command]
pub async fn test_ollama_connection(url: String) -> Result<bool, String> {
    let url = if url.trim().is_empty() {
        whis_core::ollama::DEFAULT_OLLAMA_URL.to_string()
    } else {
        url
    };

    tauri::async_runtime::spawn_blocking(move || whis_core::ollama::is_ollama_running(&url))
        .await
        .map_err(|e| e.to_string())?
}

/// List available models from Ollama
#[tauri::command]
pub async fn list_ollama_models(url: String) -> Result<Vec<String>, String> {
    let url = if url.trim().is_empty() {
        whis_core::ollama::DEFAULT_OLLAMA_URL.to_string()
    } else {
        url
    };

    // Run blocking call in separate thread
    tauri::async_runtime::spawn_blocking(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;

        let tags_url = format!("{}/api/tags", url.trim_end_matches('/'));
        let response = client
            .get(&tags_url)
            .send()
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama returned error: {}", response.status()));
        }

        #[derive(serde::Deserialize)]
        struct TagsResponse {
            models: Vec<ModelInfo>,
        }

        #[derive(serde::Deserialize)]
        struct ModelInfo {
            name: String,
        }

        let tags: TagsResponse = response
            .json()
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        Ok(tags.models.into_iter().map(|m| m.name).collect())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Progress payload for Ollama pull events
#[derive(Clone, serde::Serialize)]
pub struct OllamaPullProgress {
    pub downloaded: u64,
    pub total: u64,
}

/// Pull an Ollama model with progress events
/// Emits 'ollama-pull-progress' events with { downloaded, total } during download
#[tauri::command]
pub async fn pull_ollama_model(
    app: tauri::AppHandle,
    url: String,
    model: String,
) -> Result<(), String> {
    use tauri::Emitter;

    let url = if url.trim().is_empty() {
        whis_core::ollama::DEFAULT_OLLAMA_URL.to_string()
    } else {
        url
    };

    // Run blocking calls in separate thread to avoid tokio runtime conflicts
    // (reqwest::blocking::Client creates its own runtime internally)
    tauri::async_runtime::spawn_blocking(move || {
        // Validate Ollama is running before attempting pull
        whis_core::ollama::ensure_ollama_running(&url).map_err(|e| e.to_string())?;

        whis_core::ollama::pull_model_with_progress(&url, &model, |downloaded, total| {
            let _ = app.emit(
                "ollama-pull-progress",
                OllamaPullProgress { downloaded, total },
            );
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Ollama status check result
#[derive(Clone, serde::Serialize)]
pub struct OllamaStatus {
    pub installed: bool,
    pub running: bool,
    pub error: Option<String>,
}

/// Check Ollama installation and running status
/// Returns structured status without attempting to start Ollama
#[tauri::command]
pub async fn check_ollama_status(url: String) -> OllamaStatus {
    let url = if url.trim().is_empty() {
        whis_core::ollama::DEFAULT_OLLAMA_URL.to_string()
    } else {
        url
    };

    tauri::async_runtime::spawn_blocking(move || {
        let installed = whis_core::ollama::is_ollama_installed();

        if !installed {
            return OllamaStatus {
                installed: false,
                running: false,
                error: Some("Ollama is not installed".to_string()),
            };
        }

        match whis_core::ollama::is_ollama_running(&url) {
            Ok(true) => OllamaStatus {
                installed: true,
                running: true,
                error: None,
            },
            Ok(false) => OllamaStatus {
                installed: true,
                running: false,
                error: Some("Ollama is not running".to_string()),
            },
            Err(e) => OllamaStatus {
                installed: true,
                running: false,
                error: Some(e),
            },
        }
    })
    .await
    .unwrap_or(OllamaStatus {
        installed: false,
        running: false,
        error: Some("Failed to check status".to_string()),
    })
}

/// Start Ollama server if not running
/// Returns "started" if we started it, "running" if already running
/// Must be async with spawn_blocking because reqwest::blocking::Client
/// creates an internal tokio runtime that would panic if called from Tauri's async context
#[tauri::command]
pub async fn start_ollama(url: String) -> Result<String, String> {
    let url = if url.trim().is_empty() {
        whis_core::ollama::DEFAULT_OLLAMA_URL.to_string()
    } else {
        url
    };

    tauri::async_runtime::spawn_blocking(move || {
        match whis_core::ollama::ensure_ollama_running(&url) {
            Ok(true) => Ok("started".to_string()),  // Was started
            Ok(false) => Ok("running".to_string()), // Already running
            Err(e) => Err(e.to_string()),           // Not installed / error
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Configuration readiness check result
#[derive(serde::Serialize)]
pub struct ConfigReadiness {
    pub transcription_ready: bool,
    pub transcription_error: Option<String>,
    pub post_processing_ready: bool,
    pub post_processing_error: Option<String>,
}

/// Check if transcription and post-processing are properly configured
/// Called on app load and settings changes to show proactive warnings
#[tauri::command]
pub async fn check_config_readiness(
    provider: String,
    post_processor: String,
    api_keys: std::collections::HashMap<String, String>,
    whisper_model_path: Option<String>,
    ollama_url: Option<String>,
) -> ConfigReadiness {
    // Check transcription readiness
    let (transcription_ready, transcription_error) = match provider.as_str() {
        "local-whisper" => match &whisper_model_path {
            Some(path) if std::path::Path::new(path).exists() => (true, None),
            Some(_) => (false, Some("Whisper model file not found".to_string())),
            None => (false, Some("Whisper model path not configured".to_string())),
        },
        provider => {
            // Normalize provider for API key lookup (openai-realtime uses openai key)
            let key_provider = if provider == "openai-realtime" {
                "openai"
            } else {
                provider
            };

            if api_keys.get(key_provider).is_none_or(|k| k.is_empty()) {
                (
                    false,
                    Some(format!("{} API key not configured", capitalize(provider))),
                )
            } else {
                (true, None)
            }
        }
    };

    // Check post-processing readiness
    let (post_processing_ready, post_processing_error) = match post_processor.as_str() {
        "none" => (true, None),
        "ollama" => {
            let url = ollama_url.unwrap_or_else(|| "http://localhost:11434".to_string());
            let result = tauri::async_runtime::spawn_blocking(move || {
                whis_core::ollama::is_ollama_running(&url)
            })
            .await
            .ok()
            .and_then(|r| r.ok());

            match result {
                Some(true) => (true, None),
                _ => (false, Some("Ollama not running".to_string())),
            }
        }
        post_processor => {
            if api_keys.get(post_processor).is_none_or(|k| k.is_empty()) {
                (
                    false,
                    Some(format!(
                        "{} API key not configured",
                        capitalize(post_processor)
                    )),
                )
            } else {
                (true, None)
            }
        }
    };

    ConfigReadiness {
        transcription_ready,
        transcription_error,
        post_processing_ready,
        post_processing_error,
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// List available audio input devices
#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<whis_core::AudioDeviceInfo>, String> {
    whis_core::list_audio_devices().map_err(|e| e.to_string())
}
