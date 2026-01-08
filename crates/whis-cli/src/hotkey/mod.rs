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

/// Validate a hotkey string and return normalized form if valid
///
/// Examples of valid hotkeys: "ctrl+alt+w", "super+shift+r", "cmd+option+w"
pub fn validate(hotkey_str: &str) -> Result<String> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let hotkey = unix_like::Hotkey::parse(hotkey_str)?;
        // Return normalized form
        let mut parts = Vec::new();
        if hotkey.ctrl {
            parts.push("Ctrl");
        }
        if hotkey.alt {
            parts.push("Alt");
        }
        if hotkey.shift {
            parts.push("Shift");
        }
        if hotkey.super_key {
            parts.push("Super");
        }
        // Add the main key (convert from rdev Key to string)
        parts.push(key_to_string(&hotkey.key));
        Ok(parts.join("+"))
    }
    #[cfg(target_os = "windows")]
    {
        // Windows validation via global-hotkey
        windows::validate(hotkey_str)
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn key_to_string(key: &rdev::Key) -> &'static str {
    use rdev::Key;
    match key {
        Key::KeyA => "A",
        Key::KeyB => "B",
        Key::KeyC => "C",
        Key::KeyD => "D",
        Key::KeyE => "E",
        Key::KeyF => "F",
        Key::KeyG => "G",
        Key::KeyH => "H",
        Key::KeyI => "I",
        Key::KeyJ => "J",
        Key::KeyK => "K",
        Key::KeyL => "L",
        Key::KeyM => "M",
        Key::KeyN => "N",
        Key::KeyO => "O",
        Key::KeyP => "P",
        Key::KeyQ => "Q",
        Key::KeyR => "R",
        Key::KeyS => "S",
        Key::KeyT => "T",
        Key::KeyU => "U",
        Key::KeyV => "V",
        Key::KeyW => "W",
        Key::KeyX => "X",
        Key::KeyY => "Y",
        Key::KeyZ => "Z",
        Key::Num0 => "0",
        Key::Num1 => "1",
        Key::Num2 => "2",
        Key::Num3 => "3",
        Key::Num4 => "4",
        Key::Num5 => "5",
        Key::Num6 => "6",
        Key::Num7 => "7",
        Key::Num8 => "8",
        Key::Num9 => "9",
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        Key::Space => "Space",
        Key::Return => "Enter",
        Key::Escape => "Escape",
        Key::Tab => "Tab",
        Key::Backspace => "Backspace",
        Key::Delete => "Delete",
        Key::Insert => "Insert",
        Key::Home => "Home",
        Key::End => "End",
        Key::PageUp => "PageUp",
        Key::PageDown => "PageDown",
        Key::UpArrow => "Up",
        Key::DownArrow => "Down",
        Key::LeftArrow => "Left",
        Key::RightArrow => "Right",
        _ => "?",
    }
}
