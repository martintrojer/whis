use std::sync::Mutex;
use whis_core::{AudioRecorder, Settings};
pub use whis_core::RecordingState;

pub struct AppState {
    pub recording_state: Mutex<RecordingState>,
    pub recorder: Mutex<Option<AudioRecorder>>,
    pub settings: Mutex<Settings>,
}
