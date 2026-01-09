//! Floating Bubble Overlay Module
//!
//! Manages a floating overlay window that shows recording/transcription state.
//! The bubble provides visual feedback independent of the system tray.
//!
//! ## Architecture
//!
//! ```text
//! bubble/
//! ├── window.rs    - Window creation and positioning
//! ├── events.rs    - Show/hide/state change events
//! └── mod.rs       - Public API (this file)
//! ```

pub mod events;
pub mod window;

pub use events::{hide_bubble, show_bubble, update_bubble_state};
pub use window::create_bubble_window;
