//! Tauri command handlers organized by domain.
//!
//! ## Modules
//!
//! - `system` - Status and validation commands
//! - `presets` - Preset CRUD operations
//! - `recording` - Audio transcription commands

pub mod presets;
mod recording;
mod system;

pub use presets::*;
pub use recording::*;
pub use system::*;
