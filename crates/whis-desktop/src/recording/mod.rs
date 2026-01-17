//! Recording Orchestration Module
//!
//! Manages the complete recording workflow:
//! - Configuration loading and validation
//! - Audio recording control (start/stop)
//! - Transcription pipeline (transcribe → post-process → clipboard)
//!
//! ## Architecture
//!
//! ```text
//! recording/
//! ├── config.rs      - Configuration loading from settings
//! ├── control.rs     - Start/stop recording logic
//! ├── pipeline.rs    - Transcription pipeline orchestration
//! └── mod.rs         - Public API (toggle, start, stop)
//! ```

pub mod config;
pub mod control;
pub mod pipeline;

// Re-export public APIs
pub use config::load_transcription_config;
pub use control::start_recording_sync;
pub use pipeline::stop_and_transcribe;

use crate::state::{AppState, RecordingState};
use crate::{bubble, tray};
use tauri::{AppHandle, Manager};
use whis_core::error;

/// Toggle recording state (start if idle, stop if recording)
/// Called from global shortcuts, tray menu, and IPC
pub fn toggle_recording(app: AppHandle) {
    let state = app.state::<AppState>();
    let current_state = *state.state.lock().unwrap();

    match current_state {
        RecordingState::Idle => {
            // Start recording
            if let Err(e) = start_recording_sync(&app, &state) {
                error!("Failed to start recording: {e}");
            } else {
                // Update UI (tray and bubble)
                tray::menu::update_tray(&app, RecordingState::Recording);
                bubble::show_bubble(&app);
            }
        }
        RecordingState::Recording => {
            // Stop recording and transcribe
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                // Update UI to transcribing state
                tray::menu::update_tray(&app_clone, RecordingState::Transcribing);
                bubble::update_bubble_state(&app_clone, RecordingState::Transcribing);

                // Run transcription pipeline
                if let Err(e) = stop_and_transcribe(&app_clone).await {
                    error!("Failed to transcribe: {e}");
                }

                // Update UI back to idle
                tray::menu::update_tray(&app_clone, RecordingState::Idle);
                bubble::hide_bubble(&app_clone);
            });
        }
        RecordingState::Transcribing => {
            // Already transcribing, ignore
        }
    }
}

/// Start recording (push-to-talk mode: called on key press)
/// Only starts if currently idle; ignores if already recording or transcribing
pub fn start_recording(app: AppHandle) {
    let state = app.state::<AppState>();
    let current_state = *state.state.lock().unwrap();

    if current_state != RecordingState::Idle {
        return; // Only start from idle state
    }

    if let Err(e) = start_recording_sync(&app, &state) {
        error!("Failed to start recording: {e}");
    } else {
        // Update UI (tray icon turns red, show bubble)
        tray::menu::update_tray(&app, RecordingState::Recording);
        bubble::show_bubble(&app);
    }
}

/// Stop recording and transcribe (push-to-talk mode: called on key release)
/// Only stops if currently recording; ignores if idle or already transcribing
pub fn stop_recording(app: AppHandle) {
    let state = app.state::<AppState>();
    let current_state = *state.state.lock().unwrap();

    if current_state != RecordingState::Recording {
        return; // Only stop from recording state
    }

    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        // Update UI to transcribing state (tray icon changes to processing)
        tray::menu::update_tray(&app_clone, RecordingState::Transcribing);
        bubble::update_bubble_state(&app_clone, RecordingState::Transcribing);

        // Run transcription pipeline
        if let Err(e) = stop_and_transcribe(&app_clone).await {
            error!("Failed to transcribe: {e}");
        }

        // Update UI back to idle (tray icon returns to normal)
        tray::menu::update_tray(&app_clone, RecordingState::Idle);
        bubble::hide_bubble(&app_clone);
    });
}
