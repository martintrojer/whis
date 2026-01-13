//! CLI error display with helpful hints and actionable messages
//!
//! This module provides user-friendly error messages with contextual hints
//! for resolving common issues.

use whis_core::{AudioError, ProviderError, WhisError};

/// Display an error to stderr with helpful hints
///
/// This function matches on specific error types and provides:
/// - Clear, user-friendly error messages
/// - Contextual hints for resolution
/// - Actionable next steps (e.g., which command to run)
pub fn display_error(err: &WhisError) {
    match err {
        // Provider errors with helpful hints
        WhisError::Provider(ProviderError::MissingApiKey { provider }) => {
            eprintln!("Error: No API key configured for {}", provider);
            eprintln!();
            eprintln!("Hint: Run one of these commands to configure:");
            eprintln!("  whis setup cloud      # Interactive setup wizard");
            eprintln!("  whis config --help    # Manual configuration");
        }

        WhisError::Provider(ProviderError::InvalidApiKey { provider, reason }) => {
            eprintln!("Error: Invalid API key for {}", provider);
            eprintln!("Reason: {}", reason);
            eprintln!();
            eprintln!("Hint: Check your API key at:");
            match provider.as_str() {
                "OpenAI" => eprintln!("  https://platform.openai.com/api-keys"),
                "Mistral" => eprintln!("  https://console.mistral.ai/api-keys/"),
                "Groq" => eprintln!("  https://console.groq.com/keys"),
                "Deepgram" => eprintln!("  https://console.deepgram.com/"),
                "ElevenLabs" => eprintln!("  https://elevenlabs.io/app/settings/api-keys"),
                _ => eprintln!("  (Check your provider's console)"),
            }
        }

        WhisError::Provider(ProviderError::TranscriptionFailed(msg)) => {
            eprintln!("Error: Transcription failed");
            eprintln!("{}", msg);
            eprintln!();
            eprintln!("Hint: Common causes:");
            eprintln!("  - Network connectivity issues");
            eprintln!("  - Audio file format not supported");
            eprintln!("  - API service temporarily unavailable");
        }

        WhisError::Provider(ProviderError::RateLimitExceeded(provider)) => {
            eprintln!("Error: Rate limit exceeded for {}", provider);
            eprintln!();
            eprintln!("Hint: Wait a few moments and try again, or:");
            eprintln!("  - Upgrade your API plan for higher limits");
            eprintln!("  - Use a different provider (run: whis config --provider <name>)");
        }

        WhisError::Provider(ProviderError::NetworkError(msg)) => {
            eprintln!("Error: Network error");
            eprintln!("{}", msg);
            eprintln!();
            eprintln!("Hint: Check your internet connection");
        }

        WhisError::Provider(ProviderError::LocalModelError(msg)) => {
            eprintln!("Error: Local model error");
            eprintln!("{}", msg);
            eprintln!();
            eprintln!("Hint: Download or verify models:");
            eprintln!("  whis setup local      # Download and configure local models");
            eprintln!("  whis models           # List available models");
        }

        // Audio errors with helpful hints
        WhisError::Audio(AudioError::DeviceNotFound(device)) => {
            eprintln!("Error: Audio device not found: {}", device);
            eprintln!();
            eprintln!("The configured device may have been disconnected or renamed.");
            eprintln!();
            eprintln!("Hint: Reconfigure your microphone:");
            eprintln!("  whis setup            # Run setup wizard again");
        }

        WhisError::Audio(AudioError::RecordingFailed(msg)) => {
            eprintln!("Error: Recording failed");
            eprintln!("{}", msg);
            eprintln!();
            eprintln!("Hint: Common causes:");
            eprintln!("  - Microphone not connected or disabled");
            eprintln!("  - Permission denied (check system settings)");
            eprintln!("  - Another application is using the microphone");
        }

        WhisError::Audio(AudioError::EncodingFailed(msg)) => {
            eprintln!("Error: Audio encoding failed");
            eprintln!("{}", msg);
            eprintln!();
            eprintln!("Hint: This is usually a system configuration issue.");
            eprintln!("  Try reinstalling whis or report this issue.");
        }

        WhisError::Audio(AudioError::LoadFailed(msg)) => {
            eprintln!("Error: Failed to load audio file");
            eprintln!("{}", msg);
            eprintln!();
            eprintln!("Hint: Supported formats: WAV, MP3, MP4, M4A, FLAC, OGG");
        }

        // Configuration errors
        WhisError::Config(msg) => {
            eprintln!("Error: Configuration error");
            eprintln!("{}", msg);
            eprintln!();
            eprintln!("Hint: Check your configuration:");
            eprintln!("  whis config --show    # View current settings");
        }

        // Model errors
        WhisError::Model(msg) => {
            eprintln!("Error: Model error");
            eprintln!("{}", msg);
            eprintln!();
            eprintln!("Hint: Download or verify models:");
            eprintln!("  whis setup local      # Interactive model setup");
            eprintln!("  whis models           # List available models");
        }

        // Settings errors
        WhisError::Settings(msg) => {
            eprintln!("Error: Settings error");
            eprintln!("{}", msg);
            eprintln!();
            eprintln!("Hint: Reset or reconfigure:");
            eprintln!("  whis setup            # Run setup wizard");
        }

        // I/O errors
        WhisError::Io(err) => {
            eprintln!("Error: I/O error");
            eprintln!("{}", err);
            eprintln!();
            match err.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    eprintln!("Hint: Permission denied - check file/directory permissions");
                }
                std::io::ErrorKind::NotFound => {
                    eprintln!("Hint: File or directory not found - check the path");
                }
                std::io::ErrorKind::AlreadyExists => {
                    eprintln!("Hint: File already exists - remove it or use a different path");
                }
                _ => {
                    eprintln!("Hint: Check file paths and permissions");
                }
            }
        }

        // Generic errors
        WhisError::Other(msg)
        | WhisError::Audio(AudioError::Other(msg))
        | WhisError::Provider(ProviderError::Other(msg)) => {
            eprintln!("Error: {}", msg);
            eprintln!();
            eprintln!("Hint: For more help, run:");
            eprintln!("  whis --help");
        }

        // Catch-all for other variants
        _ => {
            eprintln!("Error: {}", err);
            eprintln!();
            eprintln!("Hint: For more help, run:");
            eprintln!("  whis --help");
        }
    }
}

/// Display an error and exit with code 1
///
/// This is a convenience function for direct error handling.
#[allow(dead_code)]
pub fn display_error_and_exit(err: &WhisError) -> ! {
    display_error(err);
    std::process::exit(1);
}

/// Convert anyhow::Error to WhisError and display
///
/// This is a bridge function for gradual migration from anyhow.
pub fn display_anyhow_error(err: anyhow::Error) {
    // Try to downcast to WhisError first
    if let Some(whis_err) = err.downcast_ref::<WhisError>() {
        display_error(whis_err);
    } else {
        // Fall back to generic error display
        eprintln!("Error: {:#}", err);
    }
}
