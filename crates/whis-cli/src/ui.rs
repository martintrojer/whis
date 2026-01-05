//! Shared CLI prompt utilities
//!
//! This module provides common interactive prompt helpers used across the CLI.

use anyhow::Result;
use std::io::{self, Write};

/// Mask an API key for display (show first 6 and last 4 chars)
pub fn mask_key(key: &str) -> String {
    if key.len() > 10 {
        format!("{}...{}", &key[..6], &key[key.len() - 4..])
    } else {
        "***".to_string()
    }
}

/// Prompt for a numeric choice
pub fn prompt_choice(prompt: &str, min: usize, max: usize) -> Result<usize> {
    prompt_choice_with_default(prompt, min, max, None)
}

/// Prompt for a numeric choice with optional default
pub fn prompt_choice_with_default(
    prompt: &str,
    min: usize,
    max: usize,
    default: Option<usize>,
) -> Result<usize> {
    loop {
        print!("{}: ", prompt);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let trimmed = input.trim();

        // If empty and we have a default, use it
        if trimmed.is_empty()
            && let Some(d) = default
        {
            return Ok(d);
        }

        match trimmed.parse::<usize>() {
            Ok(n) if n >= min && n <= max => return Ok(n),
            _ => println!("Please enter a number between {} and {}", min, max),
        }
    }
}

/// Prompt for secret input (API key, password, etc.)
///
/// **DEPRECATED**: Use `interactive::password()` instead. This function does not
/// properly hide input (security vulnerability - input is visible on screen).
#[deprecated(
    since = "0.6.5",
    note = "Use interactive::password() for properly hidden password input"
)]
pub fn prompt_secret(prompt: &str) -> Result<String> {
    // Note: Input will be visible. For true hidden input, use rpassword crate.
    print!("{}: ", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Prompt for yes/no with default (Y/n or y/N format)
///
/// **DEPRECATED**: Use `interactive::confirm()` instead for better UX with
/// arrow key support and visual indicators.
#[deprecated(
    since = "0.6.5",
    note = "Use interactive::confirm() for interactive yes/no prompts"
)]
pub fn prompt_yes_no(prompt: &str, default: bool) -> Result<bool> {
    let suffix = if default { "[Y/n]" } else { "[y/N]" };
    print!("{} {}: ", prompt, suffix);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim().to_lowercase();

    if trimmed.is_empty() {
        return Ok(default);
    }

    match trimmed.as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => Ok(default),
    }
}
