//! Provider-related helpers for API key lookup and validation.
//!
//! This module centralizes provider → API key mapping and validation logic,
//! avoiding duplication across commands and config loading.

use whis_core::Settings;

/// Get the Tauri store key for a provider's API key.
///
/// Delegates to `Settings::api_key_store_key` from whis-core for a single
/// source of truth across all apps.
///
/// # Examples
///
/// ```
/// assert_eq!(api_key_store_key("openai"), Some("openai_api_key"));
/// assert_eq!(api_key_store_key("openai-realtime"), Some("openai_api_key"));
/// assert_eq!(api_key_store_key("deepgram"), Some("deepgram_api_key"));
/// assert_eq!(api_key_store_key("unknown"), None);
/// ```
pub fn api_key_store_key(provider: &str) -> Option<&'static str> {
    Settings::api_key_store_key(provider)
}

/// Validate API key format for a given provider.
///
/// Performs basic format validation (prefix, length) without
/// making network calls. Returns `true` if the key looks valid.
///
/// # Provider-specific validation:
///
/// - **OpenAI**: Must start with `sk-` and be longer than 20 chars
/// - **Groq**: Must start with `gsk_` and be longer than 20 chars
/// - **Others**: Just checks minimum length (> 20 chars)
pub fn validate_api_key_format(key: &str, provider: &str) -> bool {
    match provider {
        "openai" | "openai-realtime" => key.starts_with("sk-") && key.len() > 20,
        "groq" => key.starts_with("gsk_") && key.len() > 20,
        "mistral" | "deepgram" | "deepgram-realtime" | "elevenlabs" => key.len() > 20,
        _ => false,
    }
}

/// Check if a provider supports realtime streaming.
///
/// Returns `true` for providers that can use WebSocket-based
/// realtime transcription.
#[allow(dead_code)]
pub fn supports_realtime_streaming(provider: &str) -> bool {
    matches!(
        provider,
        "openai" | "openai-realtime" | "deepgram" | "deepgram-realtime"
    )
}
