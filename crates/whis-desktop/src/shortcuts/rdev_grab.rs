//! RdevGrab Shortcut Implementation
//!
//! Uses rdev::grab() for global keyboard shortcuts on Linux Wayland.
//! This is the same approach used by whis-cli, which works on both X11 and Wayland.
//!
//! Supports two modes:
//! - Toggle mode (default): Press to start/stop recording
//! - Push-to-talk mode: Hold to record, release to stop
//!
//! Requirements:
//! - User must be in the `input` group
//! - uinput device must be accessible

use std::sync::mpsc;
use std::time::Duration;
use tauri::AppHandle;
use whis_core::hotkey::Hotkey;

/// Guard that keeps the keyboard grab thread alive.
/// When dropped, the thread continues until process exit.
pub struct RdevGrabGuard {
    #[allow(dead_code)]
    thread_handle: std::thread::JoinHandle<()>,
}

/// Setup global shortcuts using rdev::grab() on Linux Wayland.
/// Returns a guard that keeps the keyboard grab thread alive.
pub fn setup_rdev_grab(
    app: &tauri::App,
    shortcut_str: &str,
    push_to_talk: bool,
) -> Result<RdevGrabGuard, Box<dyn std::error::Error>> {
    let hotkey = Hotkey::parse(shortcut_str)?;
    let app_handle = app.handle().clone();

    // Channel to receive startup result from the thread
    let (startup_tx, startup_rx) = mpsc::channel::<Result<(), String>>();

    let thread_handle = std::thread::spawn(move || {
        match start_keyboard_grab(hotkey, app_handle, push_to_talk) {
            Ok(()) => {
                // This only returns if grab() exits cleanly (unlikely)
            }
            Err(e) => {
                // Send error back to main thread
                let _ = startup_tx.send(Err(e));
            }
        }
    });

    // Wait for either success signal or error (with timeout)
    // Note: rdev::grab() blocks indefinitely on success, so we use a short timeout
    // If we don't receive an error within 500ms, assume startup succeeded
    match startup_rx.recv_timeout(Duration::from_millis(500)) {
        Ok(Ok(())) => Ok(RdevGrabGuard { thread_handle }),
        Ok(Err(e)) => Err(e.into()),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // No error received = grab is running successfully
            Ok(RdevGrabGuard { thread_handle })
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            Err("Hotkey thread terminated unexpectedly".into())
        }
    }
}

/// Start the keyboard grab and listen for hotkey events.
/// This function blocks indefinitely while the grab is active.
fn start_keyboard_grab(
    hotkey: Hotkey,
    app_handle: AppHandle,
    push_to_talk: bool,
) -> Result<(), String> {
    let app_handle_release = app_handle.clone();

    // Use shared callback from whis-core with separate press/release handlers
    let callback = whis_core::hotkey::create_grab_callback(
        hotkey,
        move || {
            // On key press
            let handle = app_handle.clone();
            if push_to_talk {
                tauri::async_runtime::spawn(async move {
                    crate::recording::start_recording(handle);
                });
            } else {
                tauri::async_runtime::spawn(async move {
                    crate::recording::toggle_recording(handle);
                });
            }
        },
        move || {
            // On key release (only used in push-to-talk mode)
            if push_to_talk {
                let handle = app_handle_release.clone();
                tauri::async_runtime::spawn(async move {
                    crate::recording::stop_recording(handle);
                });
            }
        },
    );

    // rdev::grab() blocks the thread
    if let Err(e) = rdev::grab(callback) {
        return Err(format!("Failed to grab keyboard: {e:?}"));
    }

    Ok(())
}
