//! Cross-platform hotkey support
//!
//! - Linux: Uses rdev for keyboard grab (supports X11 and Wayland)
//! - Windows/macOS: Uses global-hotkey crate (Tauri-maintained)

use anyhow::Result;
use std::sync::mpsc::Receiver;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as platform;

#[cfg(not(target_os = "linux"))]
mod non_linux;
#[cfg(not(target_os = "linux"))]
use non_linux as platform;

/// Opaque guard that keeps the hotkey listener alive
#[allow(dead_code)]
pub struct HotkeyGuard(platform::HotkeyGuard);

/// Setup the hotkey listener.
/// Returns a receiver for hotkey events and a guard that must be kept alive.
pub fn setup(hotkey_str: &str) -> Result<(Receiver<()>, HotkeyGuard)> {
    let (rx, guard) = platform::setup(hotkey_str)?;
    Ok((rx, HotkeyGuard(guard)))
}
