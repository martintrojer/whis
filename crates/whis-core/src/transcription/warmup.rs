//! Connection warmup utilities
//!
//! Pre-warms HTTP and WebSocket connections to reduce cold start latency.
//! Only warms connections for providers that are actually configured.
//!
//! # Usage Pattern
//!
//! Call warmup during the "mounted → button press" window:
//! - **CLI**: After recording starts, while user is talking
//! - **Desktop/Mobile**: After Vue app mounts, in background
//!
//! ```rust,ignore
//! use whis_core::warmup::{WarmupConfig, warmup_configured};
//!
//! // Build config from user settings
//! let config = WarmupConfig {
//!     provider: Some("deepgram".to_string()),
//!     provider_api_key: Some("sk-...".to_string()),
//!     post_processor: Some("openai".to_string()),
//!     post_processor_api_key: Some("sk-...".to_string()),
//! };
//!
//! // Warm up in background (non-blocking)
//! tokio::spawn(async move {
//!     let _ = warmup_configured(&config).await;
//! });
//! ```

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::time::{Duration, timeout};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        Message,
        client::IntoClientRequest,
        http::header::{AUTHORIZATION, HeaderValue},
    },
};

use crate::http::{get_http_client, warmup_http_client};

/// Warmup timeout for individual operations
const WARMUP_TIMEOUT_SECS: u64 = 5;

/// WebSocket endpoints for realtime providers
const OPENAI_REALTIME_WS_URL: &str =
    "wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview";
const DEEPGRAM_REALTIME_WS_URL: &str =
    "wss://api.deepgram.com/v1/listen?model=nova-2&encoding=linear16&sample_rate=16000&channels=1";

/// HTTP endpoints for batch providers and post-processors
const OPENAI_API_URL: &str = "https://api.openai.com";
const DEEPGRAM_API_URL: &str = "https://api.deepgram.com";
const GROQ_API_URL: &str = "https://api.groq.com";
const MISTRAL_API_URL: &str = "https://api.mistral.ai";

/// Configuration for connection warmup.
///
/// Only configured providers will be warmed up.
#[derive(Debug, Default, Clone)]
pub struct WarmupConfig {
    /// Transcription provider name (e.g., "openai", "deepgram", "openai-realtime")
    pub provider: Option<String>,
    /// API key for the transcription provider
    pub provider_api_key: Option<String>,

    /// Post-processing provider name (e.g., "openai", "mistral")
    pub post_processor: Option<String>,
    /// API key for the post-processor
    pub post_processor_api_key: Option<String>,
}

/// Warm up connections based on configuration.
///
/// This function:
/// 1. Always warms up the HTTP client (fast, no network)
/// 2. Warms up the configured transcription provider (HTTP or WebSocket)
/// 3. Warms up the configured post-processor (HTTP only)
///
/// All warmup operations are best-effort. Failures are logged but not propagated.
///
/// # Arguments
///
/// * `config` - Configuration specifying which providers to warm up
pub async fn warmup_configured(config: &WarmupConfig) -> Result<()> {
    // Always warm up the HTTP client first (required for any HTTP operation)
    if let Err(e) = warmup_http_client()
        && crate::verbose::is_verbose()
    {
        eprintln!("[warmup] HTTP client warmup failed: {}", e);
    }

    // Collect warmup futures to run in parallel
    let mut warmup_tasks = Vec::new();

    // Warm up transcription provider
    if let (Some(provider), Some(api_key)) = (&config.provider, &config.provider_api_key) {
        let provider = provider.clone();
        let api_key = api_key.clone();
        warmup_tasks.push(tokio::spawn(async move {
            warmup_provider(&provider, &api_key).await
        }));
    }

    // Warm up post-processor (only HTTP, no WebSocket)
    if let (Some(processor), Some(api_key)) =
        (&config.post_processor, &config.post_processor_api_key)
    {
        // Skip if post-processor is "none" or same as transcription provider (already warmed)
        if processor != "none" {
            let processor = processor.clone();
            let api_key = api_key.clone();
            warmup_tasks.push(tokio::spawn(async move {
                warmup_post_processor(&processor, &api_key).await
            }));
        }
    }

    // Wait for all warmup tasks (with overall timeout)
    let overall_timeout = Duration::from_secs(WARMUP_TIMEOUT_SECS + 2);
    let _ = timeout(overall_timeout, async {
        for task in warmup_tasks {
            // Ignore individual task failures
            let _ = task.await;
        }
    })
    .await;

    if crate::verbose::is_verbose() {
        eprintln!("[warmup] Connection warmup completed");
    }

    Ok(())
}

/// Warm up a transcription provider.
///
/// For realtime providers (openai-realtime, deepgram-realtime), establishes
/// a WebSocket connection and immediately closes it to warm DNS/TLS.
///
/// For batch providers (openai, deepgram, groq), makes a HEAD request
/// to warm HTTP connection.
async fn warmup_provider(provider: &str, api_key: &str) -> Result<()> {
    match provider {
        "openai-realtime" => {
            warmup_websocket_openai(api_key).await?;
        }
        "deepgram-realtime" => {
            warmup_websocket_deepgram(api_key).await?;
        }
        "openai" => {
            warmup_http_endpoint(OPENAI_API_URL, Some(api_key), "Bearer").await?;
        }
        "deepgram" => {
            warmup_http_endpoint(DEEPGRAM_API_URL, Some(api_key), "Token").await?;
        }
        "groq" => {
            warmup_http_endpoint(GROQ_API_URL, Some(api_key), "Bearer").await?;
        }
        _ => {
            // Unknown provider or local - nothing to warm
            if crate::verbose::is_verbose() {
                eprintln!("[warmup] Skipping unknown/local provider: {}", provider);
            }
        }
    }
    Ok(())
}

/// Warm up a post-processing provider (HTTP only).
async fn warmup_post_processor(processor: &str, api_key: &str) -> Result<()> {
    match processor {
        "openai" => {
            warmup_http_endpoint(OPENAI_API_URL, Some(api_key), "Bearer").await?;
        }
        "mistral" => {
            warmup_http_endpoint(MISTRAL_API_URL, Some(api_key), "Bearer").await?;
        }
        "ollama" => {
            // Ollama is local, no warmup needed for network
            // Could potentially warmup local connection but usually instant
        }
        _ => {
            if crate::verbose::is_verbose() {
                eprintln!("[warmup] Skipping unknown post-processor: {}", processor);
            }
        }
    }
    Ok(())
}

/// Warm up an HTTP endpoint with a HEAD request.
async fn warmup_http_endpoint(url: &str, api_key: Option<&str>, auth_prefix: &str) -> Result<()> {
    let client = get_http_client()?;

    let mut request = client.head(url);

    if let Some(key) = api_key {
        request = request.header("Authorization", format!("{} {}", auth_prefix, key));
    }

    let result = timeout(Duration::from_secs(WARMUP_TIMEOUT_SECS), request.send()).await;

    match result {
        Ok(Ok(_response)) => {
            if crate::verbose::is_verbose() {
                eprintln!("[warmup] HTTP warmup succeeded: {}", url);
            }
        }
        Ok(Err(e)) => {
            // HTTP error - still warms DNS/TLS
            if crate::verbose::is_verbose() {
                eprintln!(
                    "[warmup] HTTP warmup error (still warms TLS): {} - {}",
                    url, e
                );
            }
        }
        Err(_) => {
            if crate::verbose::is_verbose() {
                eprintln!("[warmup] HTTP warmup timeout: {}", url);
            }
        }
    }

    Ok(())
}

/// Warm up OpenAI Realtime WebSocket connection.
///
/// Connects with authentication, then immediately closes.
/// This warms DNS + TLS session cache.
async fn warmup_websocket_openai(api_key: &str) -> Result<()> {
    let result = timeout(Duration::from_secs(WARMUP_TIMEOUT_SECS), async {
        // Build request with auth headers
        let mut request = OPENAI_REALTIME_WS_URL.into_client_request()?;
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))?,
        );
        request
            .headers_mut()
            .insert("OpenAI-Beta", HeaderValue::from_static("realtime=v1"));

        // Connect
        let (ws_stream, _) = connect_async(request).await?;
        let (mut write, _read) = ws_stream.split();

        // Immediately close gracefully
        let _ = write.send(Message::Close(None)).await;

        if crate::verbose::is_verbose() {
            eprintln!("[warmup] OpenAI WebSocket warmup succeeded");
        }

        Ok::<_, anyhow::Error>(())
    })
    .await;

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            if crate::verbose::is_verbose() {
                eprintln!(
                    "[warmup] OpenAI WebSocket warmup failed (still warms DNS/TLS): {}",
                    e
                );
            }
        }
        Err(_) => {
            if crate::verbose::is_verbose() {
                eprintln!("[warmup] OpenAI WebSocket warmup timeout");
            }
        }
    }

    Ok(())
}

/// Warm up Deepgram Realtime WebSocket connection.
///
/// Connects with authentication, then immediately closes.
/// This warms DNS + TLS session cache.
async fn warmup_websocket_deepgram(api_key: &str) -> Result<()> {
    let result = timeout(Duration::from_secs(WARMUP_TIMEOUT_SECS), async {
        // Build request with auth headers
        let mut request = DEEPGRAM_REALTIME_WS_URL.into_client_request()?;
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Token {}", api_key))?,
        );

        // Connect
        let (ws_stream, _) = connect_async(request).await?;
        let (mut write, _read) = ws_stream.split();

        // Send close frame
        let _ = write.send(Message::Close(None)).await;

        if crate::verbose::is_verbose() {
            eprintln!("[warmup] Deepgram WebSocket warmup succeeded");
        }

        Ok::<_, anyhow::Error>(())
    })
    .await;

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            if crate::verbose::is_verbose() {
                eprintln!(
                    "[warmup] Deepgram WebSocket warmup failed (still warms DNS/TLS): {}",
                    e
                );
            }
        }
        Err(_) => {
            if crate::verbose::is_verbose() {
                eprintln!("[warmup] Deepgram WebSocket warmup timeout");
            }
        }
    }

    Ok(())
}
