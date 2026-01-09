//! API Key Validation Commands
//!
//! Provides Tauri commands for validating API keys from various transcription providers.
//! These commands perform basic format validation before saving to settings.

/// Validate API key format
///
/// - Empty keys are valid (fall back to env var)
/// - If `required_prefix` is set, key must start with it
/// - If `min_length` is set, key must be at least that long
fn validate_key_format(
    key: &str,
    provider: &str,
    required_prefix: Option<&str>,
    min_length: Option<usize>,
) -> Result<bool, String> {
    if key.is_empty() {
        return Ok(true); // Empty falls back to env var
    }

    if let Some(prefix) = required_prefix {
        if !key.starts_with(prefix) {
            return Err(format!(
                "Invalid key format. {} keys start with '{}'",
                provider, prefix
            ));
        }
    }

    if let Some(min_len) = min_length {
        if key.trim().len() < min_len {
            return Err(format!(
                "Invalid {} API key: key appears too short",
                provider
            ));
        }
    }

    Ok(true)
}

#[tauri::command]
pub fn validate_openai_api_key(api_key: String) -> Result<bool, String> {
    validate_key_format(&api_key, "OpenAI", Some("sk-"), None)
}

#[tauri::command]
pub fn validate_mistral_api_key(api_key: String) -> Result<bool, String> {
    validate_key_format(&api_key, "Mistral", None, Some(20))
}

#[tauri::command]
pub fn validate_groq_api_key(api_key: String) -> Result<bool, String> {
    validate_key_format(&api_key, "Groq", Some("gsk_"), None)
}

#[tauri::command]
pub fn validate_deepgram_api_key(api_key: String) -> Result<bool, String> {
    validate_key_format(&api_key, "Deepgram", None, Some(20))
}

#[tauri::command]
pub fn validate_elevenlabs_api_key(api_key: String) -> Result<bool, String> {
    validate_key_format(&api_key, "ElevenLabs", None, Some(20))
}
