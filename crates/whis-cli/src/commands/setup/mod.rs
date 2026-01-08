//! Setup wizard for different usage modes
//!
//! Provides a streamlined setup experience for:
//! - Cloud users (API key setup)
//! - Local users (on-device transcription)

mod cloud;
mod interactive;
mod local;
mod post_processing;
mod provider_helpers;

use anyhow::Result;
use whis_core::Settings;

use crate::hotkey;

pub fn run() -> Result<()> {
    setup_wizard()
}

/// Unified setup wizard - guides user through all configuration
fn setup_wizard() -> Result<()> {
    let items = vec!["Cloud", "Local"];
    let choice = interactive::select("How do you want to transcribe?", &items, Some(0))?;

    let is_cloud = match choice {
        0 => {
            cloud::setup_transcription_cloud()?;
            true
        }
        1 => {
            local::setup_transcription_local()?;
            false
        }
        _ => unreachable!(),
    };

    post_processing::setup_post_processing_step(is_cloud)?;

    // Shortcut setup step
    setup_shortcut_step()?;

    interactive::info("Configuration saved! Run 'whis' to record and transcribe.");

    Ok(())
}

/// Setup shortcut mode (system or direct)
fn setup_shortcut_step() -> Result<()> {
    let items = vec!["System shortcut", "Direct capture"];
    let choice = interactive::select("Recording trigger?", &items, Some(0))?;

    let mut settings = Settings::load();

    match choice {
        0 => {
            // System mode
            settings.ui.shortcut_mode = "system".to_string();
            settings.save()?;

            interactive::info("Add a shortcut in desktop settings: whis toggle");
        }
        1 => {
            // Direct capture mode
            settings.ui.shortcut_mode = "direct".to_string();

            // Get and validate hotkey
            let default_shortcut = &settings.ui.shortcut;
            let normalized = loop {
                let input = interactive::input("Hotkey?", Some(default_shortcut))?;

                match hotkey::validate(&input) {
                    Ok(normalized) => {
                        settings.ui.shortcut = input;
                        settings.save()?;
                        break normalized;
                    }
                    Err(_) => {
                        interactive::error("Invalid hotkey. Examples: ctrl+alt+w, super+shift+r");
                        continue;
                    }
                }
            };

            // Check input group membership on Linux
            #[cfg(target_os = "linux")]
            {
                if is_in_input_group() {
                    interactive::info(&format!("Hotkey {} ready", normalized));
                } else {
                    interactive::info(&format!("Valid: {}", normalized));
                    interactive::error("You need permission first:");
                    interactive::info("  sudo usermod -aG input $USER");
                    interactive::info("Then logout and login again.");
                }
            }

            #[cfg(not(target_os = "linux"))]
            interactive::info(&format!("Hotkey {} ready", normalized));
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Check if current user is in the 'input' group (Linux only)
#[cfg(target_os = "linux")]
fn is_in_input_group() -> bool {
    use std::process::Command;

    Command::new("groups")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("input"))
        .unwrap_or(false)
}
