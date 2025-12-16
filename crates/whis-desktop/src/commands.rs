use crate::settings::Settings;
use crate::shortcuts::ShortcutBackendInfo;
use crate::state::{AppState, RecordingState};
use tauri::{AppHandle, State};

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
        has_cached_config || settings.has_api_key()
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
/// The trigger should be in human-readable format like "Ctrl+Shift+R"
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
                || current.language != settings.language,
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
