//! Output pipeline phase

use anyhow::Result;
use std::io::{self, IsTerminal};
use whis_core::{Settings, copy_to_clipboard};

use super::super::types::ProcessedResult;

/// Output mode configuration
pub enum OutputMode {
    /// Print to stdout
    Print,
    /// Copy to clipboard
    Clipboard,
}

/// Execute output phase
pub fn output(result: ProcessedResult, mode: OutputMode, quiet: bool) -> Result<()> {
    let text = result.text.trim();

    match mode {
        OutputMode::Print => {
            // Print to stdout (for piping, scripts, etc.)
            println!("{}", text);
        }
        OutputMode::Clipboard => {
            // Copy to clipboard using configured method
            let settings = Settings::load();
            copy_to_clipboard(text, settings.ui.clipboard_method)?;

            if !quiet {
                // Only show confirmation if not in quiet mode
                if io::stdout().is_terminal() {
                    println!("Copied to clipboard!");
                }
            }
        }
    }

    Ok(())
}
