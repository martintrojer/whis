//! Microphone recording mode - record audio from microphone

use anyhow::Result;
use std::io::{self, IsTerminal, Write};
use std::time::Duration;
use whis_core::{AudioRecorder, PostProcessor, RecordingOutput, Settings, TranscriptionProvider};

use super::super::types::RecordResult;
use crate::app;

/// Microphone recording configuration
#[derive(Debug, Clone)]
pub struct MicrophoneConfig {
    /// Recording duration (None = interactive)
    pub duration: Option<Duration>,
    /// Disable VAD
    pub no_vad: bool,
    /// Provider (for preloading)
    pub provider: TranscriptionProvider,
    /// Whether post-processing will be used (for preloading)
    pub will_post_process: bool,
}

/// Microphone recording mode
pub struct MicrophoneMode {
    config: MicrophoneConfig,
}

impl MicrophoneMode {
    /// Create a new microphone mode
    pub fn new(config: MicrophoneConfig) -> Self {
        Self { config }
    }

    /// Record audio from microphone
    pub fn execute(&self, quiet: bool, _runtime: &tokio::runtime::Runtime) -> Result<RecordResult> {
        let mut recorder = AudioRecorder::new()?;

        // Configure VAD
        self.configure_vad(&mut recorder)?;

        recorder.start_recording()?;

        // Preload models in background
        self.preload_models();

        // Wait for recording to complete
        self.wait_for_recording(quiet)?;

        // Stop and get recording data
        let recording_data = recorder.stop_recording()?;

        // For local providers: use raw samples directly
        // For cloud providers: use MP3 encoded data
        #[cfg(feature = "local-transcription")]
        if self.config.provider == TranscriptionProvider::LocalWhisper
            || self.config.provider == TranscriptionProvider::LocalParakeet
        {
            let samples = recording_data.finalize_raw();

            if self.config.provider == TranscriptionProvider::LocalParakeet {
                // Parakeet: return raw samples, chunking handled in transcribe phase
                let audio = RecordingOutput::Single(Vec::new()); // Unused for local

                return Ok(RecordResult {
                    audio,
                    raw_samples: Some((samples, 16000)),
                });
            } else {
                // Whisper: single chunk
                let audio = RecordingOutput::Single(Vec::new());
                return Ok(RecordResult {
                    audio,
                    raw_samples: Some((samples, 16000)),
                });
            }
        }

        // Cloud provider path
        let audio = recording_data.finalize()?;
        Ok(RecordResult {
            audio,
            raw_samples: None,
        })
    }

    /// Configure VAD based on settings and flags
    fn configure_vad(&self, recorder: &mut AudioRecorder) -> Result<()> {
        let settings = Settings::load();
        // VAD is enabled if settings say so AND --no-vad is not passed
        let vad_enabled = settings.ui.vad.enabled && !self.config.no_vad;
        recorder.set_vad(vad_enabled, settings.ui.vad.threshold);
        Ok(())
    }

    /// Preload models in background to reduce latency
    fn preload_models(&self) {
        // Preload whisper model
        #[cfg(feature = "local-transcription")]
        if self.config.provider == TranscriptionProvider::LocalWhisper {
            // Note: api_key here is actually the model path for local whisper
            let settings = Settings::load();
            if let Some(model_path) = settings.transcription.whisper_model_path() {
                whis_core::whisper_preload_model(&model_path);
            }
        }

        // Preload Ollama model if using for post-processing
        if self.config.will_post_process {
            let settings = Settings::load();
            if settings.post_processing.processor == PostProcessor::Ollama {
                if let (Some(url), Some(model)) = (
                    settings.services.ollama.url(),
                    settings.services.ollama.model(),
                ) {
                    whis_core::preload_ollama(&url, &model);
                }
            }
        }
    }

    /// Wait for recording to complete (timed or interactive)
    fn wait_for_recording(&self, quiet: bool) -> Result<()> {
        if let Some(dur) = self.config.duration {
            // Timed recording
            if !quiet {
                println!("Recording for {} seconds...", dur.as_secs());
                io::stdout().flush()?;
            }
            std::thread::sleep(dur);
        } else {
            // Interactive mode
            let settings = Settings::load();
            let hotkey = &settings.ui.shortcut;

            if !quiet {
                if io::stdin().is_terminal() {
                    println!("Press Enter or {} to stop", hotkey);
                } else {
                    println!("Press {} to stop", hotkey);
                }
                print!("Recording...");
                io::stdout().flush()?;
            }
            app::wait_for_stop(hotkey)?;

            if !quiet && whis_core::verbose::is_verbose() {
                println!();
            }
        }

        Ok(())
    }
}
