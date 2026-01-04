use anyhow::{Context, Result};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use tokio::time::sleep;

use crate::app::TranscriptionConfig;
use crate::ipc::{IpcMessage, IpcResponse, IpcServer};
use std::time::Duration;
use whis_core::{
    AudioRecorder, DEFAULT_POST_PROCESSING_PROMPT, PostProcessor, RecordingOutput, Settings,
    TranscriptionProvider, batch_transcribe, copy_to_clipboard, post_process, transcribe_audio,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ServiceState {
    Idle,
    Recording,
    Transcribing,
}

pub struct Service {
    state: Arc<Mutex<ServiceState>>,
    recorder: Arc<Mutex<Option<AudioRecorder>>>,
    provider: TranscriptionProvider,
    api_key: String,
    language: Option<String>,
    recording_counter: Arc<Mutex<u32>>,
}

impl Service {
    pub fn new(config: TranscriptionConfig) -> Result<Self> {
        Ok(Self {
            state: Arc::new(Mutex::new(ServiceState::Idle)),
            recorder: Arc::new(Mutex::new(None)),
            provider: config.provider,
            api_key: config.api_key,
            language: config.language,
            recording_counter: Arc::new(Mutex::new(0)),
        })
    }

    /// Run the service main loop
    pub async fn run(&self, hotkey_rx: Option<Receiver<()>>) -> Result<()> {
        // Create IPC server
        let ipc_server = IpcServer::new().context("Failed to create IPC server")?;

        // Enable model caching for local-whisper in listen mode
        // This keeps the model loaded between transcriptions for faster response
        #[cfg(feature = "local-transcription")]
        if self.provider == TranscriptionProvider::LocalWhisper {
            whis_core::whisper_set_keep_loaded(true);
        }

        // Show startup message with shortcut hint
        let settings = Settings::load();
        println!("Press {} to record. Ctrl+C to stop.", settings.ui.shortcut);

        loop {
            // Check for incoming IPC connections (non-blocking)
            if let Some(mut conn) = ipc_server.try_accept()? {
                match conn.receive() {
                    Ok(message) => {
                        let response = self.handle_message(message).await;
                        let _ = conn.send(response);
                    }
                    Err(e) => {
                        eprintln!("Error receiving message: {e}");
                        let _ = conn.send(IpcResponse::Error(e.to_string()));
                    }
                }
            }

            // Check for hotkey toggle signal (non-blocking)
            if let Some(ref rx) = hotkey_rx
                && rx.try_recv().is_ok()
            {
                self.handle_toggle().await;
            }

            // Small sleep to prevent busy waiting
            sleep(Duration::from_millis(10)).await;
        }
    }

    /// Handle an IPC message
    async fn handle_message(&self, message: IpcMessage) -> IpcResponse {
        match message {
            IpcMessage::Stop => {
                println!("Stop signal received");
                // Return Ok response before exiting
                tokio::spawn(async {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    std::process::exit(0);
                });
                IpcResponse::Success
            }
            IpcMessage::Status => {
                let state = *self.state.lock().unwrap();
                match state {
                    ServiceState::Idle => IpcResponse::Idle,
                    ServiceState::Recording => IpcResponse::Recording,
                    ServiceState::Transcribing => IpcResponse::Transcribing,
                }
            }
        }
    }

    /// Handle toggle command (start/stop recording)
    async fn handle_toggle(&self) -> IpcResponse {
        let current_state = *self.state.lock().unwrap();

        match current_state {
            ServiceState::Idle => {
                // Increment recording counter and start recording
                let count = {
                    let mut c = self.recording_counter.lock().unwrap();
                    *c += 1;
                    *c
                };
                match self.start_recording().await {
                    Ok(_) => {
                        println!("#{count} Recording...");
                        IpcResponse::Recording
                    }
                    Err(e) => {
                        println!("#{count} error: {e}");
                        IpcResponse::Error(e.to_string())
                    }
                }
            }
            ServiceState::Recording => {
                // Stop recording and transcribe
                *self.state.lock().unwrap() = ServiceState::Transcribing;
                let count = *self.recording_counter.lock().unwrap();

                println!("#{count} Transcribing...");

                match self.stop_and_transcribe(count).await {
                    Ok(_) => {
                        *self.state.lock().unwrap() = ServiceState::Idle;
                        println!("#{count} done");
                        println!(); // blank line between transcriptions
                        IpcResponse::Success
                    }
                    Err(e) => {
                        *self.state.lock().unwrap() = ServiceState::Idle;
                        println!("#{count} error: {e}");
                        println!();
                        IpcResponse::Error(e.to_string())
                    }
                }
            }
            ServiceState::Transcribing => {
                // Already transcribing, ignore
                IpcResponse::Transcribing
            }
        }
    }

    /// Start recording audio
    async fn start_recording(&self) -> Result<()> {
        let mut recorder = AudioRecorder::new()?;

        // Configure VAD from settings
        #[cfg(feature = "vad")]
        {
            let settings = Settings::load();
            recorder.set_vad(settings.ui.vad.enabled, settings.ui.vad.threshold);
        }

        recorder.start_recording()?;

        // Preload whisper model in background while recording
        // By the time recording finishes, model should be loaded
        #[cfg(feature = "local-transcription")]
        if self.provider == TranscriptionProvider::LocalWhisper {
            whis_core::whisper_preload_model(&self.api_key);
        }

        *self.recorder.lock().unwrap() = Some(recorder);
        *self.state.lock().unwrap() = ServiceState::Recording;

        Ok(())
    }

    /// Stop recording and transcribe
    async fn stop_and_transcribe(&self, count: u32) -> Result<()> {
        // Get the recorder
        let mut recorder = self
            .recorder
            .lock()
            .unwrap()
            .take()
            .context("No active recording")?;

        // Stop recording and get the Send-safe recording data
        // (cpal::Stream is dropped here, making RecordingData movable across threads)
        let recording_data = recorder.stop_recording()?;

        // Transcribe based on provider type
        let api_key = self.api_key.clone();
        let provider = self.provider.clone();
        let language = self.language.clone();

        // For local whisper: use raw samples directly (skip MP3 encoding)
        // For cloud providers: encode to MP3 for upload
        #[cfg(feature = "local-transcription")]
        let transcription = if provider == TranscriptionProvider::LocalWhisper {
            // Fast path: raw samples -> whisper (no MP3 encode/decode)
            let samples = recording_data.finalize_raw();
            let model_path = api_key.clone();
            tokio::task::spawn_blocking(move || {
                whis_core::transcribe_raw(&model_path, &samples, language.as_deref())
                    .map(|r| r.text)
            })
            .await
            .context("Failed to join task")??
        } else {
            // Cloud path: MP3 encoding required
            let audio_result = tokio::task::spawn_blocking(move || recording_data.finalize())
                .await
                .context("Failed to join task")??;

            match audio_result {
                RecordingOutput::Single(audio_data) => tokio::task::spawn_blocking(move || {
                    transcribe_audio(&provider, &api_key, language.as_deref(), audio_data)
                })
                .await
                .context("Failed to join task")??,
                RecordingOutput::Chunked(chunks) => {
                    batch_transcribe(&provider, &api_key, language.as_deref(), chunks, None).await?
                }
            }
        };

        #[cfg(not(feature = "local-transcription"))]
        let transcription = {
            // Finalize recording (blocking operation, run in tokio blocking task)
            let audio_result = tokio::task::spawn_blocking(move || recording_data.finalize())
                .await
                .context("Failed to join task")??;

            match audio_result {
                RecordingOutput::Single(audio_data) => tokio::task::spawn_blocking(move || {
                    transcribe_audio(&provider, &api_key, language.as_deref(), audio_data)
                })
                .await
                .context("Failed to join task")??,
                RecordingOutput::Chunked(chunks) => {
                    batch_transcribe(&provider, &api_key, language.as_deref(), chunks, None).await?
                }
            }
        };

        // Print completion message immediately after transcription finishes
        println!("#{count} Done.");

        // Apply post-processing if configured
        let settings = Settings::load();
        let final_text = if settings.post_processing.processor != PostProcessor::None {
            if let Some(post_processor_api_key) = settings
                .post_processing
                .api_key(&settings.transcription.api_keys)
            {
                println!("#{count} Post-processing...");

                let prompt = settings
                    .post_processing
                    .prompt
                    .as_deref()
                    .unwrap_or(DEFAULT_POST_PROCESSING_PROMPT);

                match post_process(
                    &post_processor_api_key,
                    &settings.post_processing.processor,
                    &transcription,
                    prompt,
                    None,
                )
                .await
                {
                    Ok(processed) => processed,
                    Err(_) => transcription, // Silently fallback in service mode
                }
            } else {
                transcription
            }
        } else {
            transcription
        };

        // Copy to clipboard (blocking operation)
        let clipboard_method = settings.ui.clipboard_method.clone();
        tokio::task::spawn_blocking(move || copy_to_clipboard(&final_text, clipboard_method))
            .await
            .context("Failed to join task")??;

        Ok(())
    }
}
