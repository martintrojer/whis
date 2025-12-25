//! OpenAI Realtime API transcription provider
//!
//! Uses WebSocket to stream audio in real-time for lower latency transcription.
//! Audio is streamed during recording rather than buffered and uploaded after.

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        Message,
        client::IntoClientRequest,
        http::header::{AUTHORIZATION, HeaderValue},
    },
};

use super::{
    OpenAIProvider, TranscriptionBackend, TranscriptionRequest, TranscriptionResult,
};

const WS_URL: &str = "wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview";
const REALTIME_SAMPLE_RATE: u32 = 24000;

/// OpenAI Realtime transcription provider
///
/// Streams audio via WebSocket for lower-latency transcription.
/// Uses the same API key as regular OpenAI.
#[derive(Debug, Default, Clone)]
pub struct OpenAIRealtimeProvider;

// WebSocket protocol messages (Beta API format)

#[derive(Serialize)]
struct SessionUpdate {
    #[serde(rename = "type")]
    msg_type: &'static str,
    session: SessionConfig,
}

#[derive(Serialize)]
struct SessionConfig {
    input_audio_format: &'static str,
    input_audio_transcription: InputAudioTranscription,
    /// Set to None to disable VAD (serializes as null in JSON)
    turn_detection: Option<TurnDetection>,
}

#[derive(Serialize)]
struct InputAudioTranscription {
    model: &'static str,
}

#[derive(Serialize)]
struct TurnDetection {
    #[serde(rename = "type")]
    detection_type: &'static str,
}

#[derive(Serialize)]
struct ResponseCreate {
    #[serde(rename = "type")]
    msg_type: &'static str,
}

#[derive(Serialize)]
struct AudioBufferAppend {
    #[serde(rename = "type")]
    msg_type: &'static str,
    audio: String,
}

#[derive(Serialize)]
struct AudioBufferCommit {
    #[serde(rename = "type")]
    msg_type: &'static str,
}

#[derive(Deserialize, Debug)]
struct RealtimeEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    transcript: Option<String>,
    #[serde(default)]
    error: Option<RealtimeError>,
}

#[derive(Deserialize, Debug)]
struct RealtimeError {
    message: String,
    #[serde(default)]
    #[allow(dead_code)]
    code: Option<String>,
}

impl OpenAIRealtimeProvider {
    /// Transcribe audio from a channel of f32 samples (16kHz mono)
    ///
    /// Connects to OpenAI Realtime API via WebSocket and streams audio chunks
    /// as they arrive. Returns the final transcript when the channel closes.
    pub async fn transcribe_stream(
        api_key: &str,
        mut audio_rx: mpsc::Receiver<Vec<f32>>,
        _language: Option<String>,
    ) -> Result<String> {
        // Build WebSocket request with authorization header
        let mut request = WS_URL.into_client_request()?;
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {api_key}"))?,
        );
        request.headers_mut().insert(
            "OpenAI-Beta",
            HeaderValue::from_static("realtime=v1"),
        );

        // Connect to WebSocket
        let (ws_stream, _response) = connect_async(request)
            .await
            .context("Failed to connect to OpenAI Realtime API")?;

        let (mut write, mut read) = ws_stream.split();

        // Send session configuration
        // Disable VAD - we'll explicitly trigger transcription with response.create
        // Server VAD causes issues with longer recordings because it waits for "turn" boundaries
        let session_update = SessionUpdate {
            msg_type: "session.update",
            session: SessionConfig {
                input_audio_format: "pcm16",
                input_audio_transcription: InputAudioTranscription {
                    model: "whisper-1",
                },
                turn_detection: None, // Disable VAD for transcription-only mode
            },
        };

        write
            .send(Message::Text(serde_json::to_string(&session_update)?.into()))
            .await
            .context("Failed to send session configuration")?;

        // Wait for session.created or session.updated confirmation
        loop {
            match read.next().await {
                Some(Ok(Message::Text(text))) => {
                    let event: RealtimeEvent = serde_json::from_str(&text)
                        .context("Failed to parse server event")?;

                    if event.event_type == "error" {
                        if let Some(err) = event.error {
                            return Err(anyhow!("OpenAI Realtime error: {}", err.message));
                        }
                    }

                    if event.event_type == "session.created"
                        || event.event_type == "session.updated"
                    {
                        break;
                    }
                }
                Some(Ok(Message::Close(_))) => {
                    return Err(anyhow!("WebSocket closed unexpectedly during setup"));
                }
                Some(Err(e)) => {
                    return Err(anyhow!("WebSocket error during setup: {e}"));
                }
                None => {
                    return Err(anyhow!("WebSocket connection closed during setup"));
                }
                _ => {} // Ignore other message types (Ping, Pong, Binary)
            }
        }

        // Stream audio chunks
        let mut chunk_count = 0;
        let mut total_samples = 0;
        while let Some(samples) = audio_rx.recv().await {
            if samples.is_empty() {
                continue;
            }

            chunk_count += 1;
            total_samples += samples.len();

            // Resample from 16kHz to 24kHz (simple linear interpolation)
            let resampled = resample_16k_to_24k(&samples);

            // Convert f32 samples to PCM16 (i16)
            let pcm16: Vec<i16> = resampled
                .iter()
                .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                .collect();

            // Convert to bytes (little-endian)
            let bytes: Vec<u8> = pcm16
                .iter()
                .flat_map(|&s| s.to_le_bytes())
                .collect();

            // Encode as base64
            let audio_base64 = BASE64.encode(&bytes);

            if crate::verbose::is_verbose() && chunk_count == 1 {
                eprintln!("[realtime] First audio chunk: {} samples, {} bytes after resample", samples.len(), bytes.len());
            }

            // Send audio append message
            let append = AudioBufferAppend {
                msg_type: "input_audio_buffer.append",
                audio: audio_base64,
            };

            write
                .send(Message::Text(serde_json::to_string(&append)?.into()))
                .await
                .context("Failed to send audio chunk")?;
        }

        if crate::verbose::is_verbose() {
            eprintln!("[realtime] Sent {} chunks, {} total samples", chunk_count, total_samples);
        }

        // Audio channel closed - commit the buffer
        let commit = AudioBufferCommit {
            msg_type: "input_audio_buffer.commit",
        };

        write
            .send(Message::Text(serde_json::to_string(&commit)?.into()))
            .await
            .context("Failed to commit audio buffer")?;

        // Trigger transcription explicitly (required when VAD is disabled)
        let response = ResponseCreate {
            msg_type: "response.create",
        };
        write
            .send(Message::Text(serde_json::to_string(&response)?.into()))
            .await
            .context("Failed to create response")?;

        // Wait for transcription result with timeout
        let mut final_transcript = String::new();

        let wait_result = timeout(Duration::from_secs(30), async {
            loop {
                match read.next().await {
                    Some(Ok(Message::Text(text))) => {
                        let event: RealtimeEvent = serde_json::from_str(&text)
                            .context("Failed to parse server event")?;

                        // Verbose logging for debugging
                        if crate::verbose::is_verbose() {
                            eprintln!("[realtime] event: {}", event.event_type);
                        }

                        match event.event_type.as_str() {
                            "error" => {
                                if let Some(err) = event.error {
                                    return Err(anyhow!("OpenAI Realtime error: {}", err.message));
                                }
                            }
                            "conversation.item.input_audio_transcription.completed" => {
                                if let Some(transcript) = event.transcript {
                                    final_transcript = transcript.clone();
                                    return Ok(transcript);
                                }
                            }
                            // Ignore other events (response.created, response.done, etc.)
                            _ => {}
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        return Ok(final_transcript.clone());
                    }
                    Some(Err(e)) => {
                        return Err(anyhow!("WebSocket error: {e}"));
                    }
                    None => {
                        return Ok(final_transcript.clone());
                    }
                    _ => {} // Ignore Ping, Pong, Binary
                }
            }
        })
        .await;

        // Close WebSocket gracefully
        let _ = write.send(Message::Close(None)).await;

        match wait_result {
            Ok(Ok(transcript)) => Ok(transcript),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow!("Timeout waiting for transcription result")),
        }
    }
}

/// Simple linear interpolation to resample from 16kHz to 24kHz
///
/// For each output sample, we interpolate between two input samples.
/// Ratio: 24000/16000 = 1.5 (3 output samples per 2 input samples)
fn resample_16k_to_24k(samples: &[f32]) -> Vec<f32> {
    if samples.is_empty() {
        return Vec::new();
    }

    let ratio = REALTIME_SAMPLE_RATE as f64 / 16000.0; // 1.5
    let output_len = ((samples.len() as f64) * ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = i as f64 / ratio;
        let idx0 = src_idx.floor() as usize;
        let idx1 = (idx0 + 1).min(samples.len() - 1);
        let frac = (src_idx - idx0 as f64) as f32;

        let sample = samples[idx0] * (1.0 - frac) + samples[idx1] * frac;
        output.push(sample);
    }

    output
}

#[async_trait]
impl TranscriptionBackend for OpenAIRealtimeProvider {
    fn name(&self) -> &'static str {
        "openai-realtime"
    }

    fn display_name(&self) -> &'static str {
        "OpenAI Realtime"
    }

    /// For file input, fall back to regular OpenAI API
    ///
    /// The Realtime API is designed for streaming mic input.
    /// For pre-recorded files, the standard whisper-1 API is more appropriate.
    fn transcribe_sync(
        &self,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult> {
        // Delegate to regular OpenAI provider for file-based transcription
        OpenAIProvider.transcribe_sync(api_key, request)
    }

    /// For async file input, fall back to regular OpenAI API
    async fn transcribe_async(
        &self,
        client: &reqwest::Client,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult> {
        // Delegate to regular OpenAI provider for file-based transcription
        OpenAIProvider.transcribe_async(client, api_key, request).await
    }
}
