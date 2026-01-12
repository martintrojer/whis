//! Global HTTP client singleton
//!
//! Provides a shared HTTP client instance to avoid recreating clients for each request.
//! This eliminates the TLS handshake overhead and root certificate store population
//! that happens when creating a new client.
//!
//! # Usage
//!
//! ```rust,ignore
//! use whis_core::http::get_http_client;
//!
//! // Get the global client (creates on first call)
//! let client = get_http_client()?;
//!
//! // Pre-warm during app startup (optional, non-blocking)
//! whis_core::http::warmup_http_client()?;
//! ```

use anyhow::{Context, Result};
use std::sync::OnceLock;

use crate::provider::DEFAULT_TIMEOUT_SECS;

/// Global HTTP client instance
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// Get the global HTTP client, creating it if necessary.
///
/// The client is configured appropriately for the current platform:
/// - Mobile (mobile-tls feature): Uses bundled Mozilla CA certificates
/// - Desktop: Uses the default platform certificate verifier
///
/// # Returns
///
/// A reference to the global HTTP client.
///
/// # Errors
///
/// Returns an error if the client cannot be created (should be rare).
pub fn get_http_client() -> Result<&'static reqwest::Client> {
    // Try to get existing client first (fast path)
    if let Some(client) = HTTP_CLIENT.get() {
        return Ok(client);
    }

    // Create and store client (slow path, only happens once)
    let client = create_http_client()?;

    // Use get_or_init to handle race condition where multiple threads
    // might try to initialize simultaneously
    Ok(HTTP_CLIENT.get_or_init(|| client))
}

/// Pre-warm the HTTP client by initializing it.
///
/// This is useful for GUI applications to initialize the client during startup,
/// before the user presses any buttons. Call this in the background after
/// the app is mounted.
///
/// If the client is already initialized, this returns immediately.
///
/// # Returns
///
/// `Ok(())` if the client is initialized (or was already initialized).
///
/// # Errors
///
/// Returns an error if the client cannot be created.
pub fn warmup_http_client() -> Result<()> {
    get_http_client()?;
    Ok(())
}

/// Check if the HTTP client is already initialized.
///
/// This can be used to check if warmup has completed.
pub fn is_http_client_ready() -> bool {
    HTTP_CLIENT.get().is_some()
}

/// Create an HTTP client configured for the current platform.
///
/// On mobile (mobile-tls feature), uses bundled Mozilla CA certificates
/// to avoid Android's platform verifier JNI initialization issues.
/// On desktop, uses the default platform certificate verifier.
fn create_http_client() -> Result<reqwest::Client> {
    #[cfg(feature = "mobile-tls")]
    {
        // Mobile: Use bundled webpki-roots to avoid Android TLS issues
        // Build root certificate store from Mozilla's CA bundle
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        // Build rustls config with the bundled roots
        let tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // Create reqwest client with pre-configured TLS
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .use_preconfigured_tls(tls_config)
            .build()
            .context("Failed to create HTTP client")
    }

    #[cfg(not(feature = "mobile-tls"))]
    {
        // Desktop: Use default platform verifier
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .context("Failed to create HTTP client")
    }
}
