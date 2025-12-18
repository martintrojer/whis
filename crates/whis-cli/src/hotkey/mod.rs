//! Cross-platform hotkey support
//!
//! - Linux/macOS: Uses rdev for keyboard grab (supports X11, Wayland, and macOS)
//! - Windows: Uses global-hotkey crate (Tauri-maintained)

use anyhow::Result;
use std::sync::mpsc::Receiver;

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod unix_like;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use unix_like as platform;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as platform;

/// Opaque guard that keeps the hotkey listener alive
#[allow(dead_code)]
pub struct HotkeyGuard(platform::HotkeyGuard);

/// Setup the hotkey listener.
/// Returns a receiver for hotkey events and a guard that must be kept alive.
pub fn setup(hotkey_str: &str) -> Result<(Receiver<()>, HotkeyGuard)> {
    let (rx, guard) = platform::setup(hotkey_str)?;
    Ok((rx, HotkeyGuard(guard)))
}
