use anyhow::{anyhow, Result};
use std::io::{self, Write};
use whis_core::{
    AudioRecorder, Polisher, Preset, RecordingOutput, Settings, TranscriptionProvider,
    copy_to_clipboard, parallel_transcribe, polish, transcribe_audio, DEFAULT_POLISH_PROMPT,
};

use crate::app;

/// Resolve which polisher to use based on priority:
/// 1. Preset override (if specified and valid)
/// 2. Settings polisher (if configured)
/// 3. Transcription provider fallback (OpenAI/Mistral only, others check key availability)
fn resolve_polisher(
    preset: &Option<Preset>,
    settings: &Settings,
    provider: &TranscriptionProvider,
) -> Polisher {
    // 1. Preset override
    if let Some(p) = preset
        && let Some(polisher_str) = &p.polisher
    {
        match polisher_str.parse() {
            Ok(polisher) => return polisher,
            Err(_) => eprintln!("Warning: Invalid polisher '{}' in preset", polisher_str),
        }
    }

    // 2. Settings polisher
    if settings.polisher != Polisher::None {
        return settings.polisher.clone();
    }

    // 3. Transcription provider fallback
    // OpenAI and Mistral have built-in polish capabilities
    // Other providers need an available OpenAI or Mistral key
    match provider {
        TranscriptionProvider::OpenAI => Polisher::OpenAI,
        TranscriptionProvider::Mistral => Polisher::Mistral,
        // For providers without LLM capability, check if we have a polisher key
        TranscriptionProvider::Groq
        | TranscriptionProvider::Deepgram
        | TranscriptionProvider::ElevenLabs => {
            // Try OpenAI first, then Mistral, then disable polishing
            if settings
                .get_api_key_for(&TranscriptionProvider::OpenAI)
                .is_some()
            {
                Polisher::OpenAI
            } else if settings
                .get_api_key_for(&TranscriptionProvider::Mistral)
                .is_some()
            {
                Polisher::Mistral
            } else {
                Polisher::None
            }
        }
    }
}

pub fn run(polish_flag: bool, preset_name: Option<String>) -> Result<()> {
    // Load preset if provided
    let preset: Option<Preset> = if let Some(name) = preset_name {
        let (p, _source) = Preset::load(&name).map_err(|e| anyhow!("{}", e))?;
        Some(p)
    } else {
        None
    };

    // Create Tokio runtime for async operations
    let runtime = tokio::runtime::Runtime::new()?;

    // Check if FFmpeg is available
    app::ensure_ffmpeg_installed()?;

    // Load transcription configuration (provider + API key)
    let config = app::load_transcription_config()?;

    // Create recorder and start recording
    let mut recorder = AudioRecorder::new()?;
    recorder.start_recording()?;

    print!("Recording... (press Enter to stop)");
    io::stdout().flush()?;
    app::wait_for_enter()?;

    // Finalize recording and get output
    let audio_result = recorder.finalize_recording()?;

    // Transcribe based on output type
    let transcription = match audio_result {
        RecordingOutput::Single(audio_data) => {
            // Small file - simple transcription
            print!("\rTranscribing...                        \n");
            io::stdout().flush()?;

            transcribe_audio(
                &config.provider,
                &config.api_key,
                config.language.as_deref(),
                audio_data,
            )?
        }
        RecordingOutput::Chunked(chunks) => {
            // Large file - parallel transcription
            print!("\rTranscribing...                        \n");
            io::stdout().flush()?;

            runtime.block_on(parallel_transcribe(
                &config.provider,
                &config.api_key,
                config.language.as_deref(),
                chunks,
                None,
            ))?
        }
    };

    // Apply polishing if enabled (via flag, preset, or settings)
    let settings = Settings::load();
    let should_polish = polish_flag || preset.is_some() || settings.polisher != Polisher::None;

    let final_text = if should_polish {
        let polisher = resolve_polisher(&preset, &settings, &config.provider);

        // Get API key for polisher using the unified method
        let api_key = settings.get_polisher_api_key();

        if let Some(api_key) = api_key {
            print!("Polishing...");
            io::stdout().flush()?;

            // Priority: preset prompt > settings prompt > default
            let prompt = if let Some(ref p) = preset {
                p.prompt.as_str()
            } else {
                settings
                    .polish_prompt
                    .as_deref()
                    .unwrap_or(DEFAULT_POLISH_PROMPT)
            };

            let model = preset.as_ref().and_then(|p| p.model.as_deref());
            match runtime.block_on(polish(&transcription, &polisher, &api_key, prompt, model)) {
                Ok(polished) => {
                    print!("\r              \r");
                    io::stdout().flush()?;
                    polished
                }
                Err(e) => {
                    eprintln!("\rPolish warning: {e}");
                    eprintln!("Falling back to raw transcript");
                    transcription
                }
            }
        } else {
            eprintln!("Warning: No API key for polisher, skipping polish");
            transcription
        }
    } else {
        transcription
    };

    // Copy to clipboard
    copy_to_clipboard(&final_text)?;

    println!("Copied to clipboard");

    Ok(())
}
