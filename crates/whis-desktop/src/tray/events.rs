//! Tray Event Handlers
//!
//! Handles tray menu clicks and tray icon interactions.
//! Platform-specific behavior for left-click on Linux vs macOS.

use crate::recording;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

/// Handle tray menu item clicks
pub fn handle_menu_event(app: AppHandle, event_id: &str) {
    match event_id {
        "record" => {
            // Use unified toggle_recording that handles tray + bubble
            recording::toggle_recording(app);
        }
        "settings" => {
            open_settings_window(app);
        }
        "quit" => {
            // Emit event to frontend to flush settings before exit
            let _ = app.emit("tray-quit-requested", ());
        }
        _ => {}
    }
}

/// Handle tray icon clicks (Linux only - left-click toggles recording)
#[cfg(not(target_os = "macos"))]
pub fn handle_tray_icon_event(app: AppHandle, event: tauri::tray::TrayIconEvent) {
    use tauri::tray::TrayIconEvent;
    if let TrayIconEvent::Click { button, .. } = event
        && button == tauri::tray::MouseButton::Left
    {
        // Use unified toggle_recording that handles tray + bubble
        recording::toggle_recording(app);
    }
}

#[cfg(target_os = "macos")]
pub fn handle_tray_icon_event(_app: AppHandle, _event: tauri::tray::TrayIconEvent) {
    // On macOS, menu shows on left-click so we don't handle icon events
}

/// Open or focus the settings window
fn open_settings_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    let window = WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("index.html".into()))
        .title("Whis Settings")
        .inner_size(600.0, 400.0)
        .min_inner_size(400.0, 300.0)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .build();

    // Fix Wayland window dragging by unsetting GTK titlebar
    // On Wayland, GTK's titlebar is required for dragging, but decorations(false)
    // removes it. By calling set_titlebar(None), we restore drag functionality
    // while keeping our custom chrome.
    match window {
        Ok(window) => {
            #[cfg(target_os = "linux")]
            {
                use gtk::prelude::GtkWindowExt;
                if let Ok(gtk_window) = window.gtk_window() {
                    gtk_window.set_titlebar(Option::<&gtk::Widget>::None);
                }
            }
            let _ = window.show();
        }
        Err(e) => eprintln!("Failed to create settings window: {e}"),
    }
}
