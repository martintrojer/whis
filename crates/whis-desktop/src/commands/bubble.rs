//! Bubble Overlay Commands
//!
//! Provides Tauri commands for bubble interactions.

use tauri::AppHandle;

/// Toggle recording from bubble click
/// Mirrors the tray toggle behavior
#[tauri::command]
pub async fn bubble_toggle_recording(app: AppHandle) -> Result<(), String> {
    crate::recording::toggle_recording(app);
    Ok(())
}
