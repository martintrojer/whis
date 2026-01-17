//! Tauri Plugin Shortcut Implementation
//!
//! Implements global keyboard shortcuts using the Tauri plugin.
//! Works on X11, macOS, and Windows platforms where native shortcuts are supported.
//!
//! Supports two modes:
//! - Toggle mode (default): Press to start/stop recording
//! - Push-to-talk mode: Hold to record, release to stop

use std::str::FromStr;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// Setup global shortcuts using Tauri plugin (for X11, macOS, Windows)
pub fn setup_tauri_shortcut(
    app: &tauri::App,
    shortcut_str: &str,
    push_to_talk: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().clone();

    // Attempt to parse the shortcut
    let shortcut =
        Shortcut::from_str(shortcut_str).map_err(|e| format!("Invalid shortcut: {e}"))?;

    // Initialize plugin with handler that supports both toggle and push-to-talk modes
    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app, _shortcut, event| {
                let handle = app_handle.clone();
                match event.state() {
                    ShortcutState::Pressed => {
                        if push_to_talk {
                            println!("Tauri shortcut pressed (push-to-talk: start)");
                            tauri::async_runtime::spawn(async move {
                                crate::recording::start_recording(handle);
                            });
                        } else {
                            println!("Tauri shortcut pressed (toggle)");
                            tauri::async_runtime::spawn(async move {
                                crate::recording::toggle_recording(handle);
                            });
                        }
                    }
                    ShortcutState::Released => {
                        if push_to_talk {
                            println!("Tauri shortcut released (push-to-talk: stop)");
                            tauri::async_runtime::spawn(async move {
                                crate::recording::stop_recording(handle);
                            });
                        }
                        // In toggle mode, release is ignored
                    }
                }
            })
            .build(),
    )?;

    // Register the shortcut
    app.global_shortcut().register(shortcut)?;
    let mode = if push_to_talk {
        "push-to-talk"
    } else {
        "toggle"
    };
    println!("Tauri global shortcut registered: {shortcut_str} ({mode} mode)");

    Ok(())
}

/// Update shortcut. Returns Ok(true) if restart is needed, Ok(false) if applied immediately.
pub fn update_tauri_shortcut(
    app: &AppHandle,
    new_shortcut: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Unregister all existing shortcuts
    app.global_shortcut().unregister_all()?;

    // Parse and register new one
    let shortcut =
        Shortcut::from_str(new_shortcut).map_err(|e| format!("Invalid shortcut: {e}"))?;
    app.global_shortcut().register(shortcut)?;
    println!("Updated Tauri global shortcut to: {new_shortcut}");
    // Note: Changing push-to-talk mode requires restart since handler is set at plugin init
    Ok(false)
}
