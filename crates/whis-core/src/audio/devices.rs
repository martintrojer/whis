//! Audio device enumeration and management.
//!
//! On Linux with PulseAudio, uses libpulse-binding for rich device metadata
//! (form_factor, bus type, monitor detection). Falls back to cpal on other
//! platforms or when PulseAudio is unavailable.

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};

use super::types::AudioDeviceInfo;

#[cfg(all(target_os = "linux", feature = "pulse-metadata"))]
use super::pulse;

#[cfg(target_os = "linux")]
mod alsa_suppress {
    use std::os::raw::{c_char, c_int};
    use std::sync::Once;

    // Use a non-variadic function pointer type for the handler.
    // ALSA's actual signature is variadic, but since our handler ignores all args,
    // we can use a simpler signature that's compatible at the ABI level.
    type SndLibErrorHandlerT =
        unsafe extern "C" fn(*const c_char, c_int, *const c_char, c_int, *const c_char);

    #[link(name = "asound")]
    unsafe extern "C" {
        fn snd_lib_error_set_handler(handler: Option<SndLibErrorHandlerT>) -> c_int;
    }

    // No-op error handler - does nothing, suppresses all ALSA errors
    unsafe extern "C" fn silent_error_handler(
        _file: *const c_char,
        _line: c_int,
        _function: *const c_char,
        _err: c_int,
        _fmt: *const c_char,
    ) {
        // Intentionally empty - suppress all ALSA error output
    }

    static INIT: Once = Once::new();

    /// Initialize ALSA error suppression.
    ///
    /// NOTE: This function can be safely removed without affecting functionality.
    /// It only suppresses noisy log output about unavailable PCM plugins (pulse, jack, oss).
    /// The unsafe FFI code here is purely cosmetic - audio works fine without it.
    pub fn init() {
        INIT.call_once(|| {
            // SAFETY: We provide a valid no-op error handler function.
            // This suppresses ALSA's error messages about unavailable PCM plugins.
            unsafe {
                snd_lib_error_set_handler(Some(silent_error_handler));
            }
        });
    }
}

#[cfg(not(target_os = "linux"))]
mod alsa_suppress {
    pub fn init() {}
}

/// List all available audio input devices on the system.
///
/// On Linux with PulseAudio, returns devices with rich metadata (form_factor, bus, etc.).
/// Falls back to cpal-based enumeration on other platforms or when PulseAudio unavailable.
///
/// Device names are normalized to CPAL-compatible descriptions for consistent lookup
/// during recording (CPAL is used for actual audio capture).
///
/// # Returns
/// A vector of audio device information, including device names and default status.
///
/// # Errors
/// Returns an error if no audio input devices are found.
pub fn list_audio_devices() -> Result<Vec<AudioDeviceInfo>> {
    // Try PulseAudio first on Linux (provides rich metadata)
    #[cfg(all(target_os = "linux", feature = "pulse-metadata"))]
    {
        match pulse::list_pulse_devices() {
            Ok(mut devices) if !devices.is_empty() => {
                // Cross-reference with CPAL to get compatible names for device lookup.
                // PulseAudio returns technical names (alsa_input.usb-...) but CPAL uses
                // human-readable descriptions (USB Microphone Mono). We need CPAL names
                // for the recorder's device lookup to work.
                let cpal_descriptions = get_cpal_descriptions();

                crate::verbose!("CPAL descriptions: {:?}", cpal_descriptions);

                for device in &mut devices {
                    crate::verbose!(
                        "PulseAudio: name={:?}, display={:?}",
                        device.name,
                        device.display_name
                    );

                    // Match PulseAudio display_name to CPAL description using fuzzy matching
                    // (names may differ by punctuation or suffixes like "(PipeWire)")
                    if let Some(display) = &device.display_name {
                        let normalized = normalize_for_matching(display);
                        if let Some(cpal_name) = cpal_descriptions
                            .iter()
                            .find(|c| normalize_for_matching(c) == normalized)
                        {
                            crate::verbose!("Matched: {} -> {}", device.name, cpal_name);
                            device.name = cpal_name.clone();
                        }
                    }
                }
                return Ok(devices);
            }
            Ok(_) => {} // Empty result, fall through to cpal
            Err(_e) => {
                // PulseAudio unavailable, fall through to cpal
                // Could log: eprintln!("PulseAudio enumeration failed: {}, using cpal", e);
            }
        }
    }

    // Fallback: use cpal (cross-platform, less metadata)
    list_cpal_devices()
}

/// Normalize device name for fuzzy matching.
///
/// Strips punctuation, parenthetical suffixes, and normalizes whitespace
/// to allow matching between PulseAudio and CPAL device names.
pub(crate) fn normalize_for_matching(name: &str) -> String {
    // Remove parenthetical suffixes like "(PipeWire)" or "(currently PulseAudio)"
    let base = name.split('(').next().unwrap_or(name);

    // Keep only alphanumeric and whitespace, then normalize
    base.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Check if a CPAL device description matches a stored device name using fuzzy matching.
///
/// Returns true if the CPAL description's key words are contained in the stored name.
/// This handles cases where:
/// - PulseAudio technical names (alsa_input.usb-VENDOR_PRODUCT...) are stored
/// - CPAL descriptions are human-readable (Product Name Mono)
pub(crate) fn fuzzy_device_match(stored_name: &str, cpal_description: &str) -> bool {
    let stored_normalized = normalize_for_matching(stored_name);
    let cpal_normalized = normalize_for_matching(cpal_description);

    // Exact match after normalization
    if stored_normalized == cpal_normalized {
        return true;
    }

    // Word containment: check if all significant words from CPAL description
    // are found in the normalized stored name (handles PulseAudio technical names)
    let cpal_words: Vec<&str> = cpal_normalized.split_whitespace().collect();

    // Filter out common non-distinctive words and very short words
    let significant_words: Vec<&str> = cpal_words
        .iter()
        .filter(|w| {
            w.len() >= 3
                && !matches!(
                    **w,
                    "mono" | "stereo" | "input" | "output" | "analog" | "digital" | "audio" | "usb"
                )
        })
        .copied()
        .collect();

    // If we don't have enough significant words, can't make a reliable match
    if significant_words.is_empty() {
        return false;
    }

    // Check if all significant words from CPAL are in the normalized stored name
    let matches = significant_words
        .iter()
        .filter(|w| stored_normalized.contains(*w))
        .count();

    matches == significant_words.len()
}

/// Get all CPAL device descriptions for cross-referencing with PulseAudio.
fn get_cpal_descriptions() -> Vec<String> {
    alsa_suppress::init();
    let host = cpal::default_host();
    host.input_devices()
        .ok()
        .map(|devices| {
            devices
                .filter_map(|d| d.description().ok().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// List devices using cpal (cross-platform fallback).
fn list_cpal_devices() -> Result<Vec<AudioDeviceInfo>> {
    alsa_suppress::init();

    let host = cpal::default_host();
    let default_device_name = host
        .default_input_device()
        .and_then(|d| d.description().ok())
        .map(|d| d.to_string());

    let mut devices = Vec::new();
    for device in host.input_devices()? {
        if let Ok(desc) = device.description() {
            let raw_name = desc.to_string();

            // Filter out virtual/null devices that aren't real microphones
            if is_virtual_device(&raw_name) {
                continue;
            }

            let display_name = clean_device_name(&raw_name);

            devices.push(AudioDeviceInfo {
                name: raw_name.clone(),
                display_name: Some(display_name),
                is_default: default_device_name.as_ref() == Some(&raw_name),
                // cpal doesn't provide rich metadata
                form_factor: None,
                bus: None,
                is_monitor: false,
            });
        }
    }

    if devices.is_empty() {
        anyhow::bail!("No audio input devices found");
    }

    Ok(devices)
}

/// Check if a device is a virtual/null device that should be filtered out.
fn is_virtual_device(name: &str) -> bool {
    let lower = name.to_lowercase();

    // Filter out null/dummy devices
    if lower.contains("discard all samples")
        || lower.contains("generate zero samples")
        || lower.contains("null")
    {
        return true;
    }

    // Filter out output monitors (not real microphones)
    if lower.contains("output") && lower.contains("monitor") {
        return true;
    }

    // Filter out PipeWire's internal devices
    if lower == "pipewire sound server" {
        return true;
    }

    false
}

/// Clean up a device name for display.
fn clean_device_name(name: &str) -> String {
    let mut cleaned = name.to_string();

    // Remove common verbose suffixes
    let suffixes_to_remove = [
        " (currently PipeWire Media Server)",
        " (currently PulseAudio)",
        " Analog Stereo",
        " Digital Stereo",
        " Stereo",
        " Mono",
    ];

    for suffix in suffixes_to_remove {
        if let Some(pos) = cleaned.find(suffix) {
            cleaned.truncate(pos);
        }
    }

    // Remove trailing commas and whitespace
    cleaned = cleaned.trim_end_matches([',', ' ']).to_string();

    // If empty after cleaning, return original
    if cleaned.is_empty() {
        return name.to_string();
    }

    cleaned
}

/// Initialize platform-specific audio system.
///
/// On Linux, this suppresses ALSA error messages about unavailable PCM plugins.
/// On other platforms, this is a no-op.
pub(super) fn init_platform() {
    alsa_suppress::init();
}
