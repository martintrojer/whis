//! Shared CLI utilities

/// Mask an API key for display (show first 6 and last 4 chars)
pub fn mask_key(key: &str) -> String {
    if key.len() > 10 {
        format!("{}...{}", &key[..6], &key[key.len() - 4..])
    } else {
        "***".to_string()
    }
}
