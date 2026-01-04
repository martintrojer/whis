//! Recording Control
//!
//! Handles starting and stopping audio recording with state management.

use super::config::load_transcription_config;
use crate::state::{AppState, RecordingState};
use tauri::AppHandle;
use tokio::sync::{mpsc, oneshot};
#[cfg(feature = "local-transcription")]
use whis_core::progressive_transcribe_local;
use whis_core::{
    AudioRecorder, ChunkerConfig, PostProcessor, ProgressiveChunker, Settings,
    TranscriptionProvider, ollama, preload_ollama, progressive_transcribe_cloud,
};

/// Start recording with progressive transcription (default mode)
///
/// Starts streaming audio recording and spawns background tasks for:
/// - Progressive audio chunking (90s target, VAD-aware)
/// - Transcription during recording (parallel for cloud providers, sequential for local providers)
///
/// The transcription result will be available via the oneshot channel
/// stored in AppState when recording completes.
pub fn start_recording_sync(_app: &AppHandle, state: &AppState) -> Result<(), String> {
    // Load transcription config if not already loaded
    let (provider, api_key, language) = {
        let mut config_guard = state.transcription_config.lock().unwrap();
        if config_guard.is_none() {
            *config_guard = Some(load_transcription_config(state)?);
        }
        let config = config_guard.as_ref().unwrap();
        (
            config.provider.clone(),
            config.api_key.clone(),
            config.language.clone(),
        )
    };

    // Create recorder and start streaming
    let mut recorder = AudioRecorder::new().map_err(|e| e.to_string())?;

    // Configure VAD from settings
    let vad_enabled = {
        let settings = state.settings.lock().unwrap();
        settings.ui.vad.enabled
    };
    let vad_threshold = state.settings.lock().unwrap().ui.vad.threshold;
    recorder.set_vad(vad_enabled, vad_threshold);

    // Start streaming recording
    let device_name = state.settings.lock().unwrap().ui.microphone_device.clone();
    let mut audio_rx_bounded = if let Some(device) = device_name.as_deref() {
        recorder
            .start_recording_streaming_with_device(Some(device))
            .map_err(|e| e.to_string())?
    } else {
        recorder
            .start_recording_streaming()
            .map_err(|e| e.to_string())?
    };

    // Create unbounded channel adapter
    let (audio_tx_unbounded, audio_rx_unbounded) = mpsc::unbounded_channel();
    tauri::async_runtime::spawn(async move {
        while let Some(samples) = audio_rx_bounded.recv().await {
            if audio_tx_unbounded.send(samples).is_err() {
                break;
            }
        }
    });

    // Create channels for chunking
    let (chunk_tx, chunk_rx) = mpsc::unbounded_channel();

    // Create chunker config
    let chunker_config = ChunkerConfig {
        target_duration_secs: 90,
        min_duration_secs: 60,
        max_duration_secs: 120,
        vad_aware: vad_enabled,
    };

    // Spawn chunker task
    let mut chunker = ProgressiveChunker::new(chunker_config, chunk_tx);
    tauri::async_runtime::spawn(async move {
        let _ = chunker.consume_stream(audio_rx_unbounded, None).await;
    });

    // Create oneshot channel for transcription result
    let (result_tx, result_rx) = oneshot::channel();

    // Preload models in background to reduce latency (before spawning async tasks)
    {
        let settings = state.settings.lock().unwrap();

        // Preload the configured local transcription model (Whisper OR Parakeet, not both)
        #[cfg(feature = "local-transcription")]
        match provider {
            TranscriptionProvider::LocalWhisper => {
                if let Some(model_path) = settings.transcription.whisper_model_path() {
                    whis_core::whisper_preload_model(&model_path);
                }
            }
            TranscriptionProvider::LocalParakeet => {
                if let Some(model_path) = settings.transcription.parakeet_model_path() {
                    whis_core::preload_parakeet(&model_path);
                }
            }
            _ => {} // Cloud providers don't need preload
        }

        // Preload Ollama if post-processing enabled
        if settings.post_processing.processor == PostProcessor::Ollama {
            let ollama_url = settings
                .services
                .ollama
                .url()
                .unwrap_or_else(|| ollama::DEFAULT_OLLAMA_URL.to_string());
            let ollama_model = settings
                .services
                .ollama
                .model()
                .unwrap_or_else(|| ollama::DEFAULT_OLLAMA_MODEL.to_string());

            preload_ollama(&ollama_url, &ollama_model);
        }
    }

    // Spawn transcription task
    tauri::async_runtime::spawn(async move {
        let result: Result<String, String> = {
            #[cfg(feature = "local-transcription")]
            if provider == TranscriptionProvider::LocalParakeet {
                match Settings::load().transcription.parakeet_model_path() {
                    Some(model_path) => progressive_transcribe_local(&model_path, chunk_rx, None)
                        .await
                        .map_err(|e| e.to_string()),
                    None => Err("Parakeet model path not configured".to_string()),
                }
            } else {
                progressive_transcribe_cloud(
                    &provider,
                    &api_key,
                    language.as_deref(),
                    chunk_rx,
                    None,
                )
                .await
                .map_err(|e| e.to_string())
            }

            #[cfg(not(feature = "local-transcription"))]
            progressive_transcribe_cloud(&provider, &api_key, language.as_deref(), chunk_rx, None)
                .await
                .map_err(|e| e.to_string())
        };

        let _ = result_tx.send(result);
    });

    // Store receiver for later retrieval
    *state.transcription_rx.lock().unwrap() = Some(result_rx);
    *state.recorder.lock().unwrap() = Some(recorder);
    *state.state.lock().unwrap() = RecordingState::Recording;

    println!("Recording started (progressive mode)...");
    Ok(())
}
