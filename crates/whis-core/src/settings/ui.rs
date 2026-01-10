//! User interface and recording settings.
//!
//! This module contains settings for:
//! - Keyboard shortcuts and triggering mode
//! - Audio recording configuration (microphone, VAD, chunking)
//! - Output handling (clipboard backend, presets)
//! - Desktop-specific features (floating bubble overlay)

use serde::{Deserialize, Serialize};

#[cfg(feature = "clipboard")]
use crate::clipboard::ClipboardMethod;

/// CLI keyboard shortcut triggering mode.
///
/// Determines how the CLI (`whis` command) listens for the recording hotkey.
///
/// **NOTE**: This setting only affects the CLI. The desktop app (whis-desktop)
/// auto-detects the best shortcut backend based on your environment.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CliShortcutMode {
    /// Desktop environment handles the hotkey.
    ///
    /// Configure a keyboard shortcut in your desktop settings
    /// (GNOME Settings → Keyboard → Shortcuts) to run: `whis toggle`
    ///
    /// This is the recommended mode as it works reliably across
    /// all Linux desktop environments without special permissions.
    #[default]
    System,

    /// CLI captures the hotkey directly.
    ///
    /// The CLI will listen for the global keyboard shortcut specified
    /// in `shortcut_key`. This requires special permissions on Linux
    /// (typically input group membership or running as root).
    ///
    /// Use this mode if you can't configure system shortcuts or
    /// need the shortcut to work in specific applications.
    Direct,
}

impl CliShortcutMode {
    /// Returns the string representation for config display.
    pub fn as_str(&self) -> &'static str {
        match self {
            CliShortcutMode::System => "system",
            CliShortcutMode::Direct => "direct",
        }
    }
}

impl std::fmt::Display for CliShortcutMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for CliShortcutMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "system" => Ok(CliShortcutMode::System),
            "direct" => Ok(CliShortcutMode::Direct),
            _ => Err(format!(
                "Invalid shortcut mode: '{}'. Use 'system' or 'direct'",
                s
            )),
        }
    }
}

/// Settings for UI behavior and device configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    /// CLI keyboard shortcut triggering mode.
    ///
    /// - `system`: Configure hotkey in desktop settings to run "whis toggle"
    /// - `direct`: CLI captures the hotkey globally (requires permissions)
    ///
    /// **NOTE**: This setting only affects the CLI (`whis` command).
    /// The desktop app auto-detects its shortcut backend.
    ///
    /// See [`CliShortcutMode`] for details on each mode.
    #[serde(default)]
    pub cli_shortcut_mode: CliShortcutMode,

    /// Global keyboard shortcut (e.g., "Ctrl+Alt+W").
    ///
    /// **NOTE**: Only used by CLI when `cli_shortcut_mode` is `direct`.
    /// In `system` mode, this field is ignored - configure your
    /// hotkey in desktop settings instead.
    ///
    /// Format: Modifier keys + key, e.g., "Ctrl+Alt+W", "Super+Shift+R"
    pub shortcut_key: String,

    /// Clipboard backend for pasting transcriptions.
    ///
    /// - `auto`: Auto-detect the best option for your system (recommended)
    ///   - Flatpak: uses wl-copy
    ///   - X11: uses xclip
    ///   - Wayland: uses arboard
    /// - `xclip`: Force X11 xclip (for X11 systems)
    /// - `wl-copy`: Force Wayland wl-copy (for Wayland systems)
    /// - `arboard`: Force cross-platform Rust clipboard library
    ///
    /// Change this if transcription pasting doesn't work correctly.
    #[cfg(feature = "clipboard")]
    #[serde(default)]
    pub clipboard_backend: ClipboardMethod,

    /// Selected microphone device name.
    ///
    /// - `null`: Use system default microphone
    /// - `"Device Name"`: Use specific microphone by name
    ///
    /// Run `whis setup` to see available devices and select one.
    #[serde(default)]
    pub microphone_device: Option<String>,

    /// Voice Activity Detection (VAD) settings.
    ///
    /// When enabled, whis will skip silence during recording,
    /// reducing transcription time and improving accuracy.
    #[serde(default)]
    pub vad: VadSettings,

    /// Currently active output preset name.
    ///
    /// Presets define post-processing transformations like
    /// "professional email", "casual chat", etc.
    ///
    /// - `null`: No preset active (raw transcription)
    /// - `"preset_name"`: Apply named preset from ~/.config/whis/presets/
    #[serde(default)]
    pub active_preset: Option<String>,

    /// Audio chunk duration for progressive transcription (seconds).
    ///
    /// During recording, audio is split into chunks and transcribed
    /// progressively. This setting controls chunk size:
    ///
    /// - Lower (30s): Faster perceived response, but less context
    /// - Default (90s): Good balance of speed and accuracy
    /// - Higher (120s+): Better accuracy for complex speech
    ///
    /// Valid range: 10-300 seconds
    #[serde(default = "default_chunk_duration")]
    pub chunk_duration_secs: u64,

    /// Floating bubble overlay settings (desktop only).
    ///
    /// Shows a small floating indicator during recording.
    /// Experimental feature.
    #[serde(default)]
    pub bubble: BubbleSettings,
}

fn default_chunk_duration() -> u64 {
    crate::configuration::DEFAULT_CHUNK_DURATION_SECS
}

/// Voice Activity Detection configuration.
///
/// VAD automatically detects speech and skips silence,
/// which can significantly reduce transcription time
/// for recordings with pauses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadSettings {
    /// Enable Voice Activity Detection.
    ///
    /// When enabled, silence is skipped during recording,
    /// reducing the amount of audio sent for transcription.
    #[serde(default)]
    pub enabled: bool,

    /// Speech probability threshold (0.0-1.0).
    ///
    /// - Lower (0.3): More sensitive, may include background noise
    /// - Default (0.5): Balanced sensitivity
    /// - Higher (0.7): Less sensitive, may cut off soft speech
    ///
    /// Adjust if VAD is cutting off speech or including too much silence.
    #[serde(default)]
    pub threshold: f32,
}

impl Default for VadSettings {
    fn default() -> Self {
        Self {
            enabled: crate::configuration::DEFAULT_VAD_ENABLED,
            threshold: crate::configuration::DEFAULT_VAD_THRESHOLD,
        }
    }
}

/// Floating bubble overlay position.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BubblePosition {
    /// Disabled (default)
    #[default]
    None,
    /// Top of screen
    Top,
    /// Center of screen
    Center,
    /// Bottom of screen
    Bottom,
}

/// Floating bubble overlay settings (experimental).
///
/// The bubble is a small floating indicator that shows
/// recording status. Desktop only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BubbleSettings {
    /// Enable floating bubble overlay.
    #[serde(default)]
    pub enabled: bool,

    /// Bubble position on screen.
    #[serde(default)]
    pub position: BubblePosition,
}

impl Default for BubbleSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            position: BubblePosition::None,
        }
    }
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            cli_shortcut_mode: CliShortcutMode::default(),
            shortcut_key: crate::configuration::DEFAULT_SHORTCUT.to_string(),
            #[cfg(feature = "clipboard")]
            clipboard_backend: ClipboardMethod::default(),
            microphone_device: None,
            vad: VadSettings::default(),
            active_preset: None,
            chunk_duration_secs: crate::configuration::DEFAULT_CHUNK_DURATION_SECS,
            bubble: BubbleSettings::default(),
        }
    }
}
