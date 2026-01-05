//! Interactive prompt helpers using dialoguer
//!
//! Provides themed, consistent prompts for the setup wizard.

use anyhow::Result;
use dialoguer::{
    Input, Password, Select,
    console::{Style, style},
    theme::ColorfulTheme,
};

/// Get the shared theme for all prompts with minimal ASCII styling
pub fn theme() -> ColorfulTheme {
    use dialoguer::console::style;

    ColorfulTheme {
        // Use ASCII [*] for confirmed selections
        success_prefix: style("[*]".to_string()).for_stderr(),
        success_suffix: style("".to_string()).for_stderr(),

        // Multi-select checkboxes
        checked_item_prefix: style("[*]".to_string()).for_stderr(),
        unchecked_item_prefix: style("[ ]".to_string()).for_stderr(),

        // Single select indicators - use [ ] and [>] (both 3 chars, no shift!)
        active_item_prefix: style("[>]".to_string()).for_stderr(),
        inactive_item_prefix: style("[ ]".to_string()).for_stderr(),

        // Prompt styling - use [?] for questions
        prompt_prefix: style("[?]".to_string()).for_stderr(),
        prompt_suffix: style("".to_string()).for_stderr(),

        // Error indicators
        error_prefix: style("[!]".to_string()).for_stderr(),

        // Plain values - no dimming, no colors
        values_style: Style::new(),

        // Override defaults that apply colors
        prompt_style: Style::new(),
        active_item_style: Style::new(),
        inactive_item_style: Style::new(),

        // Use ColorfulTheme defaults for remaining fields
        ..ColorfulTheme::default()
    }
}

/// Select from a list of options with arrow keys
pub fn select<T: std::fmt::Display>(
    prompt: &str,
    items: &[T],
    default: Option<usize>,
) -> Result<usize> {
    let theme = theme();
    let mut select = Select::with_theme(&theme).with_prompt(prompt).items(items);

    if let Some(idx) = default {
        select = select.default(idx);
    }

    Ok(select.interact()?)
}

/// Get text input
pub fn input(prompt: &str, default: Option<&str>) -> Result<String> {
    let theme = theme();
    let mut input = Input::with_theme(&theme).with_prompt(prompt);

    if let Some(d) = default {
        input = input.default(d.to_string());
    }

    Ok(input.interact_text()?)
}

/// Get password/secret input (hidden)
pub fn password(prompt: &str) -> Result<String> {
    let theme = theme();
    Ok(Password::with_theme(&theme)
        .with_prompt(prompt)
        .interact()?)
}

/// Print an error message
pub fn error(text: &str) {
    eprintln!("{} {}", style("[!]").bold(), text);
}

/// Print an info message
pub fn info(text: &str) {
    println!("{} {}", style("[i]").bold(), text);
}

/// Display Ollama installation instructions
pub fn ollama_not_installed() {
    error("Ollama is not installed.");
    info("Install Ollama:");
    info("  Linux:  curl -fsSL https://ollama.com/install.sh | sh");
    info("  macOS:  brew install ollama");
    info("  Website: https://ollama.com/download");
}
