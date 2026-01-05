//! Deepgram Live Streaming API transcription provider
//!
//! Uses WebSocket to stream audio in real-time for lower latency transcription.
//! Simpler and faster than OpenAI Realtime API.
//!
//! Key advantages over OpenAI Realtime:
//! - No base64 encoding overhead (sends raw binary)
//! - No resampling needed (16kHz native vs OpenAI's 24kHz)
//! - Simpler protocol (just send binary frames vs complex message types)
//! - Lower latency (~150ms vs 300-500ms)
//! - Supports interim results for progressive transcription

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use futures_util::{SinkExt, Stream, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::time::{Duration, timeout};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        Message,
        client::IntoClientRequest,
        http::header::{AUTHORIZATION, HeaderValue},
    },
};

use super::{DeepgramProvider, TranscriptionBackend, TranscriptionRequest, TranscriptionResult};

const WS_URL: &str = "wss://api.deepgram.com/v1/listen";
const MODEL: &str = "nova-2";
const SAMPLE_RATE: u32 = 16000;
const KEEPALIVE_INTERVAL_SECS: u64 = 5;

/// Deepgram Live Streaming provider
///
/// Streams audio via WebSocket for real-time, low-latency transcription.
/// Uses the same API key as batch Deepgram (DEEPGRAM_API_KEY).
#[derive(Debug, Default, Clone)]
pub struct DeepgramRealtimeProvider;

// Response message types

#[derive(Deserialize, Debug)]
struct DeepgramEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    is_final: bool,
    #[serde(default)]
    #[allow(dead_code)]
    speech_final: bool,
    #[serde(default)]
    channel: Option<Channel>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Channel {
    alternatives: Vec<Alternative>,
}

#[derive(Deserialize, Debug)]
struct Alternative {
    transcript: String,
    #[allow(dead_code)]
    confidence: f64,
}

impl DeepgramRealtimeProvider {
    /// Transcribe audio from a channel of f32 samples (16kHz mono)
    ///
    /// Connects to Deepgram Live Streaming API via WebSocket and streams audio chunks
    /// as they arrive. Returns the final transcript when the channel closes.
    pub async fn transcribe_stream(
        api_key: &str,
        mut audio_rx: mpsc::Receiver<Vec<f32>>,
        language: Option<String>,
    ) -> Result<String> {
        // 1. Build WebSocket URL with query params
        let mut url = format!(
            "{WS_URL}?model={MODEL}&encoding=linear16&sample_rate={SAMPLE_RATE}\
             &channels=1&smart_format=true&interim_results=true"
        );

        if let Some(lang) = language {
            url.push_str(&format!("&language={}", lang));
        }

        // 2. Build request with Authorization header
        let mut request = url.into_client_request()?;
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Token {api_key}"))?,
        );

        // 3. Connect WebSocket
        let (ws_stream, _response) = connect_async(request)
            .await
            .context("Failed to connect to Deepgram Live Streaming API")?;

        let (write, read) = ws_stream.split();
        let write = Arc::new(Mutex::new(write));

        // 4. Spawn read task to collect transcripts
        let read_handle = tokio::spawn(async move { collect_transcripts(read).await });

        // 5. Spawn keepalive task
        let (keepalive_cancel_tx, keepalive_cancel_rx) = oneshot::channel();
        let keepalive_handle = tokio::spawn({
            let write = Arc::clone(&write);
            async move { keepalive_task(write, keepalive_cancel_rx).await }
        });

        // 6. Stream audio chunks as binary frames
        let mut chunk_count = 0;
        let mut total_samples = 0;

        while let Some(samples) = audio_rx.recv().await {
            if samples.is_empty() {
                continue;
            }

            chunk_count += 1;
            total_samples += samples.len();

            // Convert f32 to PCM16 i16
            let pcm16: Vec<i16> = samples
                .iter()
                .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                .collect();

            // Convert to bytes (little-endian)
            let bytes: Vec<u8> = pcm16.iter().flat_map(|&s| s.to_le_bytes()).collect();

            // Send as binary WebSocket message (NOT base64!)
            write
                .lock()
                .await
                .send(Message::Binary(bytes.into()))
                .await
                .context("Failed to send audio chunk")?;
        }

        if crate::verbose::is_verbose() {
            eprintln!(
                "[deepgram-realtime] Sent {} chunks, {} total samples",
                chunk_count, total_samples
            );
        }

        // 7. Cancel keepalive task
        let _ = keepalive_cancel_tx.send(());
        let _ = keepalive_handle.await;

        // 8. Send Finalize message to flush buffer
        write
            .lock()
            .await
            .send(Message::Text(r#"{"type":"Finalize"}"#.to_string().into()))
            .await
            .context("Failed to send Finalize message")?;

        // 9. Wait for final transcript with timeout
        let transcript_result = timeout(Duration::from_secs(30), read_handle).await;

        // 10. Close connection gracefully
        let _ = write.lock().await.send(Message::Close(None)).await;

        match transcript_result {
            Ok(Ok(Ok(transcript))) => Ok(transcript),
            Ok(Ok(Err(e))) => Err(e),
            Ok(Err(e)) => Err(anyhow!("Read task panicked: {e}")),
            Err(_) => Err(anyhow!("Timeout waiting for transcription result")),
        }
    }
}

/// KeepAlive task that sends periodic messages during silence
///
/// Deepgram requires KeepAlive or audio data within 10 seconds.
/// This task sends KeepAlive every 5 seconds to prevent timeout.
async fn keepalive_task<W>(write: Arc<Mutex<W>>, mut cancel_rx: oneshot::Receiver<()>) -> Result<()>
where
    W: SinkExt<Message> + Unpin,
    W::Error: std::error::Error + Send + Sync + 'static,
{
    let mut interval = tokio::time::interval(Duration::from_secs(KEEPALIVE_INTERVAL_SECS));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if crate::verbose::is_verbose() {
                    eprintln!("[deepgram-realtime] Sending KeepAlive");
                }

                let msg = r#"{"type":"KeepAlive"}"#;
                if write.lock().await.send(Message::Text(msg.to_string().into())).await.is_err() {
                    break;
                }
            }
            _ = &mut cancel_rx => {
                if crate::verbose::is_verbose() {
                    eprintln!("[deepgram-realtime] KeepAlive task cancelled");
                }
                break;
            }
        }
    }

    Ok(())
}

/// Collect final transcripts from WebSocket messages
///
/// Processes incoming Deepgram events and accumulates final transcriptions.
/// Ignores interim results (is_final=false) to avoid duplicates in final output.
async fn collect_transcripts<S>(mut read: S) -> Result<String>
where
    S: Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    let mut final_transcript = String::new();

    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                let event: DeepgramEvent =
                    serde_json::from_str(&text).context("Failed to parse Deepgram event")?;

                if crate::verbose::is_verbose() && event.event_type != "Metadata" {
                    eprintln!(
                        "[deepgram-realtime] event: {} (is_final={})",
                        event.event_type, event.is_final
                    );
                }

                match event.event_type.as_str() {
                    "Results" => {
                        // Only collect final results
                        if event.is_final {
                            if let Some(channel) = event.channel {
                                if let Some(alt) = channel.alternatives.first() {
                                    if !alt.transcript.is_empty() {
                                        final_transcript.push_str(&alt.transcript);
                                        final_transcript.push(' ');
                                    }
                                }
                            }
                        }
                        // Ignore interim results (is_final=false)
                    }
                    "Metadata" => {
                        // Connection metadata, ignore for now
                    }
                    "error" => {
                        if let Some(desc) = event.description {
                            return Err(anyhow!("Deepgram error: {}", desc));
                        }
                        return Err(anyhow!("Deepgram error (no description)"));
                    }
                    _ => {
                        // Unknown event type, ignore
                    }
                }
            }
            Message::Close(frame) => {
                if crate::verbose::is_verbose() {
                    eprintln!("[deepgram-realtime] WebSocket closed: {:?}", frame);
                }
                break;
            }
            Message::Ping(_) | Message::Pong(_) => {
                // Tungstenite handles ping/pong automatically
            }
            Message::Binary(_) => {
                // Unexpected binary message from server, ignore
            }
            Message::Frame(_) => {
                // Raw frame, ignore
            }
        }
    }

    Ok(final_transcript.trim().to_string())
}

#[async_trait]
impl TranscriptionBackend for DeepgramRealtimeProvider {
    fn name(&self) -> &'static str {
        "deepgram-realtime"
    }

    fn display_name(&self) -> &'static str {
        "Deepgram Realtime"
    }

    /// For file input, fall back to regular Deepgram API
    ///
    /// The Live Streaming API is designed for real-time mic input.
    /// For pre-recorded files, the batch API is more appropriate.
    fn transcribe_sync(
        &self,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult> {
        // Delegate to regular Deepgram provider for file-based transcription
        DeepgramProvider.transcribe_sync(api_key, request)
    }

    /// For async file input, fall back to regular Deepgram API
    async fn transcribe_async(
        &self,
        client: &reqwest::Client,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult> {
        // Delegate to regular Deepgram provider for file-based transcription
        DeepgramProvider
            .transcribe_async(client, api_key, request)
            .await
    }
}
