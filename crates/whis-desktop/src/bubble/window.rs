//! Bubble Window Creation and Positioning
//!
//! Creates the floating bubble overlay window with platform-specific configuration.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use whis_core::settings::BubblePosition;

use crate::state::AppState;

/// Bubble window dimensions
const BUBBLE_SIZE: f64 = 48.0;

/// Offset from screen edges
const BUBBLE_OFFSET: f64 = 50.0;

/// Create the bubble overlay window (hidden by default)
pub fn create_bubble_window(app: &AppHandle) -> Result<(), String> {
    // Calculate initial position (will be updated when shown)
    let (x, y) = calculate_bubble_position(app).unwrap_or((100.0, 100.0));

    let window = WebviewWindowBuilder::new(
        app,
        "bubble",
        WebviewUrl::App("src/bubble/index.html".into()),
    )
    .title("Whis Bubble")
    .position(x, y)
    .inner_size(BUBBLE_SIZE, BUBBLE_SIZE)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .focused(false)
    .visible(false)
    .build()
    .map_err(|e| e.to_string())?;

    // Linux/Wayland: Remove GTK titlebar for proper transparency
    #[cfg(target_os = "linux")]
    {
        use gtk::prelude::GtkWindowExt;
        if let Ok(gtk_window) = window.gtk_window() {
            gtk_window.set_titlebar(Option::<&gtk::Widget>::None);
        }
    }

    Ok(())
}

/// Calculate bubble position based on settings
pub fn calculate_bubble_position(app: &AppHandle) -> Result<(f64, f64), String> {
    // Get primary monitor
    let monitor = app
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .ok_or("No primary monitor")?;

    let size = monitor.size();
    let scale = monitor.scale_factor();
    let width = size.width as f64 / scale;
    let height = size.height as f64 / scale;

    // Get bubble position setting
    let state = app.state::<AppState>();
    let position = state.with_settings(|s| s.ui.bubble.position);

    // Center horizontally
    let x = (width - BUBBLE_SIZE) / 2.0;

    // Vertical position based on setting
    let y = match position {
        BubblePosition::Top => BUBBLE_OFFSET,
        BubblePosition::Center => (height - BUBBLE_SIZE) / 2.0,
        BubblePosition::Bottom | BubblePosition::None => height - BUBBLE_SIZE - BUBBLE_OFFSET,
    };

    Ok((x, y))
}
