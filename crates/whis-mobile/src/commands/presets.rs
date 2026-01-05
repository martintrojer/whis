//! Preset management commands.
//!
//! Handles listing, viewing, creating, updating, and deleting presets.

use std::path::PathBuf;
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use whis_core::preset::{Preset, PresetSource};

/// Get the presets directory for this app using Tauri's path API.
/// This works correctly on Android where dirs::config_dir() returns None.
pub fn get_presets_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|p| p.join("presets"))
        .map_err(|e| format!("Failed to get app config dir: {}", e))
}

/// Preset info for the UI list view.
#[derive(serde::Serialize)]
pub struct PresetInfo {
    pub name: String,
    pub description: String,
    pub is_builtin: bool,
    pub is_active: bool,
}

/// Full preset details for viewing/editing.
#[derive(serde::Serialize)]
pub struct PresetDetails {
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub is_builtin: bool,
}

/// List all available presets (built-in + user).
#[tauri::command]
pub fn list_presets(app: tauri::AppHandle) -> Vec<PresetInfo> {
    // Get active preset from store
    let active_preset = app.store("settings.json").ok().and_then(|store| {
        store
            .get("active_preset")
            .and_then(|v| v.as_str().map(String::from))
    });

    // Use Tauri's app config dir for presets (works on Android)
    let presets = match get_presets_dir(&app) {
        Ok(dir) => Preset::list_all_from(&dir),
        Err(_) => {
            // Fall back to built-ins only if we can't get the dir
            Preset::builtins()
                .into_iter()
                .map(|p| (p, PresetSource::BuiltIn))
                .collect()
        }
    };

    presets
        .into_iter()
        .map(|(p, source)| PresetInfo {
            is_active: active_preset.as_ref().is_some_and(|a| a == &p.name),
            name: p.name,
            description: p.description,
            is_builtin: source == PresetSource::BuiltIn,
        })
        .collect()
}

/// Get full details of a preset.
#[tauri::command]
pub fn get_preset_details(app: tauri::AppHandle, name: String) -> Result<PresetDetails, String> {
    let presets_dir = get_presets_dir(&app)?;
    let (preset, source) = Preset::load_from(&name, &presets_dir)?;

    Ok(PresetDetails {
        name: preset.name,
        description: preset.description,
        prompt: preset.prompt,
        is_builtin: source == PresetSource::BuiltIn,
    })
}

/// Set the active preset.
#[tauri::command]
pub fn set_active_preset(app: tauri::AppHandle, name: Option<String>) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    if let Some(preset_name) = name {
        store.set("active_preset", serde_json::json!(preset_name));
    } else {
        store.delete("active_preset");
    }

    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Get the active preset name.
#[tauri::command]
pub fn get_active_preset(app: tauri::AppHandle) -> Option<String> {
    app.store("settings.json").ok().and_then(|store| {
        store
            .get("active_preset")
            .and_then(|v| v.as_str().map(String::from))
    })
}

// ========== Preset CRUD Commands ==========

/// Input for creating a new preset.
#[derive(serde::Deserialize)]
pub struct CreatePresetInput {
    pub name: String,
    pub description: String,
    pub prompt: String,
}

/// Input for updating an existing preset.
#[derive(serde::Deserialize)]
pub struct UpdatePresetInput {
    pub description: String,
    pub prompt: String,
}

/// Create a new user preset.
#[tauri::command]
pub fn create_preset(
    app: tauri::AppHandle,
    input: CreatePresetInput,
) -> Result<PresetInfo, String> {
    let presets_dir = get_presets_dir(&app)?;

    // Validate name
    Preset::validate_name(&input.name, false)?;

    // Check if preset already exists (check user preset in custom dir)
    if Preset::load_user_from(&input.name, &presets_dir).is_some() {
        return Err(format!("Preset '{}' already exists", input.name));
    }

    // Create and save the preset
    let preset = Preset {
        name: input.name.clone(),
        description: input.description.clone(),
        prompt: input.prompt,
        post_processor: None,
        model: None,
    };

    preset.save_to(&presets_dir)?;

    Ok(PresetInfo {
        name: input.name,
        description: input.description,
        is_builtin: false,
        is_active: false,
    })
}

/// Update an existing user preset.
#[tauri::command]
pub fn update_preset(
    app: tauri::AppHandle,
    name: String,
    input: UpdatePresetInput,
) -> Result<(), String> {
    let presets_dir = get_presets_dir(&app)?;

    // Check it's not a built-in
    if Preset::is_builtin(&name) {
        return Err(format!("Cannot edit built-in preset '{}'", name));
    }

    // Check preset exists
    let (mut preset, _) = Preset::load_from(&name, &presets_dir)?;

    // Update fields
    preset.description = input.description;
    preset.prompt = input.prompt;

    // Save
    preset.save_to(&presets_dir)?;

    Ok(())
}

/// Delete a user preset.
#[tauri::command]
pub fn delete_preset(app: tauri::AppHandle, name: String) -> Result<(), String> {
    let presets_dir = get_presets_dir(&app)?;

    // Delete the preset file
    Preset::delete_from(&name, &presets_dir)?;

    // If this was the active preset, clear it
    if let Ok(store) = app.store("settings.json")
        && let Some(active) = store
            .get("active_preset")
            .and_then(|v| v.as_str().map(String::from))
        && active == name
    {
        store.delete("active_preset");
        let _ = store.save();
    }

    Ok(())
}
