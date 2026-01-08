//! User interface settings for desktop applications.

use serde::{Deserialize, Serialize};

#[cfg(feature = "clipboard")]
use crate::clipboard::ClipboardMethod;

/// Settings for UI behavior and device configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    /// Shortcut mode: "system" (desktop settings) or "direct" (whis handles hotkey)
    #[serde(default = "default_shortcut_mode")]
    pub shortcut_mode: String,

    /// Global keyboard shortcut (only used if shortcut_mode is "direct")
    pub shortcut: String,

    /// Clipboard method for copying text (auto, xclip, wl-copy, arboard)
    #[cfg(feature = "clipboard")]
    #[serde(default)]
    pub clipboard_method: ClipboardMethod,

    /// Selected microphone device name (None = system default)
    #[serde(default)]
    pub microphone_device: Option<String>,

    /// Voice Activity Detection settings
    #[serde(default)]
    pub vad: VadSettings,

    /// Currently active preset name (if any)
    #[serde(default)]
    pub active_preset: Option<String>,

    /// Audio chunk duration in seconds for progressive transcription
    /// Smaller = faster response, larger = better accuracy
    #[serde(default = "default_chunk_duration")]
    pub chunk_duration_secs: u64,
}

fn default_chunk_duration() -> u64 {
    crate::defaults::DEFAULT_CHUNK_DURATION_SECS
}

fn default_shortcut_mode() -> String {
    crate::defaults::DEFAULT_SHORTCUT_MODE.to_string()
}

/// Voice Activity Detection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadSettings {
    /// Enable Voice Activity Detection to skip silence during recording
    #[serde(default)]
    pub enabled: bool,

    /// VAD speech probability threshold (0.0-1.0, default 0.5)
    #[serde(default)]
    pub threshold: f32,
}

impl Default for VadSettings {
    fn default() -> Self {
        Self {
            enabled: crate::defaults::DEFAULT_VAD_ENABLED,
            threshold: crate::defaults::DEFAULT_VAD_THRESHOLD,
        }
    }
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            shortcut_mode: crate::defaults::DEFAULT_SHORTCUT_MODE.to_string(),
            shortcut: crate::defaults::DEFAULT_SHORTCUT.to_string(),
            #[cfg(feature = "clipboard")]
            clipboard_method: ClipboardMethod::default(),
            microphone_device: None,
            vad: VadSettings::default(),
            active_preset: None,
            chunk_duration_secs: crate::defaults::DEFAULT_CHUNK_DURATION_SECS,
        }
    }
}
