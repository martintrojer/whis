//! Record Command - Voice-to-Text Pipeline
//!
//! This module implements the main recording/transcription workflow using a clean
//! pipeline architecture. It handles audio input from multiple sources, transcribes
//! it using configured providers, optionally post-processes the text, and outputs
//! the result.
//!
//! # Architecture
//!
//! The record command follows a four-phase pipeline:
//!
//! ```text
//! ┌─────────────┐      ┌──────────────┐      ┌─────────────┐      ┌────────────┐
//! │   Record    │  →   │  Transcribe  │  →   │   Process   │  →   │   Output   │
//! │  (modes/)   │      │ (pipeline/)  │      │ (pipeline/) │      │(pipeline/) │
//! └─────────────┘      └──────────────┘      └─────────────┘      └────────────┘
//!     ↓                      ↓                     ↓                    ↓
//! Microphone          Cloud/Local API         LLM Cleanup         Clipboard/
//! File Input          Parallel Chunks         Preset Transform    Stdout
//! Stdin Stream        Language Detection
//! ```
//!
//! # Pipeline Phases
//!
//! 1. **Record Phase** (`modes/`): Capture or load audio
//!    - `MicrophoneMode`: Record from system microphone with VAD
//!    - `FileMode`: Load and decode audio file
//!    - `StdinMode`: Stream audio from stdin
//!
//! 2. **Transcribe Phase** (`pipeline/transcribe.rs`): Convert audio to text
//!    - Cloud providers: Single or chunked API calls
//!    - Local providers: Raw samples with progressive transcription
//!
//! 3. **Process Phase** (`pipeline/process.rs`): Enhance transcript
//!    - Apply LLM post-processing (grammar, filler words)
//!    - Transform with output presets
//!
//! 4. **Output Phase** (`pipeline/output.rs`): Deliver result
//!    - Copy to clipboard (default)
//!    - Print to stdout (--print flag)
//!
//! # Usage
//!
//! ```rust
//! // Default: Record from microphone, copy to clipboard
//! commands::record::run(false, None, None, false, "mp3", false, None, false, None)?;
//!
//! // Transcribe file with post-processing
//! commands::record::run(
//!     true,                          // post_process
//!     None,                          // preset_name
//!     Some(PathBuf::from("audio.mp3")), // file_path
//!     false,                         // stdin_mode
//!     "mp3",                         // input_format
//!     false,                         // print
//!     None,                          // duration
//!     false,                         // no_vad
//!     None,                          // save_raw
//! )?;
//! ```
//!
//! # Configuration
//!
//! The record command respects user settings from `~/.config/whis/config.toml`:
//! - Transcription provider and API keys
//! - Post-processing preferences
//! - VAD settings and hotkeys
//! - Clipboard method

mod modes;
mod pipeline;
mod types;

use anyhow::Result;
use std::path::PathBuf;
use std::time::Duration;

use crate::app;
use types::{InputSource, RecordConfig};

/// Execute the record command with clean pipeline phases
pub fn run(
    post_process: bool,
    preset_name: Option<String>,
    file_path: Option<PathBuf>,
    stdin_mode: bool,
    input_format: &str,
    print: bool,
    duration: Option<Duration>,
    no_vad: bool,
    save_raw: Option<PathBuf>,
) -> Result<()> {
    // Create configuration
    let config = RecordConfig::new(post_process, preset_name, print, duration, no_vad, save_raw)?;
    let quiet = config.is_quiet();

    // Create Tokio runtime for async operations
    let runtime = tokio::runtime::Runtime::new()?;

    // Ensure FFmpeg is available
    app::ensure_ffmpeg_installed()?;

    // Load transcription configuration
    let transcription_config = app::load_transcription_config()?;

    // Determine input source
    let input_source = if let Some(path) = file_path {
        InputSource::File(path)
    } else if stdin_mode {
        InputSource::Stdin {
            format: input_format.to_string(),
        }
    } else {
        InputSource::Microphone
    };

    // Phase 1: Record/Load audio
    let record_result = match input_source {
        InputSource::File(path) => {
            let mode = modes::FileMode::new(path);
            mode.execute(quiet)?
        }
        InputSource::Stdin { format } => {
            let mode = modes::StdinMode::new(format);
            mode.execute(quiet)?
        }
        InputSource::Microphone => {
            let mic_config = modes::MicrophoneConfig {
                duration: config.duration,
                no_vad: config.no_vad,
                provider: transcription_config.provider.clone(),
                will_post_process: config.post_process || config.preset.is_some(),
            };
            let mode = modes::MicrophoneMode::new(mic_config);
            mode.execute(quiet, &runtime)?
        }
    };

    // Save raw samples if requested
    if let Some(save_path) = &config.save_raw {
        if let Some((samples, sample_rate)) = &record_result.raw_samples {
            save_raw_samples_as_wav(samples, *sample_rate, save_path)?;
            if !quiet {
                eprintln!("✓ Saved raw audio to: {}", save_path.display());
            }
        }
    }

    // Phase 2: Transcribe audio to text
    let transcription_cfg = pipeline::TranscriptionConfig {
        provider: transcription_config.provider,
        api_key: transcription_config.api_key,
        language: transcription_config.language,
    };
    let transcription_result = runtime.block_on(pipeline::transcribe(
        record_result,
        &transcription_cfg,
        quiet,
    ))?;

    // Phase 3: Post-process and apply presets
    let processing_cfg = pipeline::ProcessingConfig {
        enabled: config.post_process,
        preset: config.preset,
    };
    let processed_result = runtime.block_on(pipeline::process(
        transcription_result,
        &processing_cfg,
        quiet,
    ))?;

    // Phase 4: Output (print or clipboard)
    let output_mode = if config.print {
        pipeline::OutputMode::Print
    } else {
        pipeline::OutputMode::Clipboard
    };
    pipeline::output(processed_result, output_mode, quiet)?;

    Ok(())
}

/// Save raw audio samples as WAV file
fn save_raw_samples_as_wav(samples: &[f32], sample_rate: u32, path: &PathBuf) -> Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)?;

    for &sample in samples {
        let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(sample_i16)?;
    }

    writer.finalize()?;
    Ok(())
}
