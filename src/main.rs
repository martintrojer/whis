mod audio;
mod clipboard;
mod config;
mod transcribe;

use anyhow::Result;
use std::io::{self, Write};

fn main() -> Result<()> {
    println!("🎤 Whisp - Voice to Text");
    println!("========================\n");

    // Load configuration
    let config = match config::Config::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            eprintln!("\nPlease create a .env file with your OpenAI API key:");
            eprintln!("  OPENAI_API_KEY=your-api-key-here\n");
            std::process::exit(1);
        }
    };

    println!("Press Enter to start recording...");
    wait_for_enter()?;

    // Create recorder and start recording
    let mut recorder = audio::AudioRecorder::new()?;
    recorder.start_recording()?;

    println!("\n🔴 RECORDING - Press Enter to stop...\n");
    wait_for_enter()?;

    // Stop recording and get audio data
    let audio_data = recorder.stop_and_save()?;
    println!("Captured {} bytes of audio data", audio_data.len());

    // Transcribe
    let transcription = match transcribe::transcribe_audio(&config.openai_api_key, audio_data) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Transcription error: {}", e);
            std::process::exit(1);
        }
    };

    // Display result
    println!("\n📝 Transcription:");
    println!("─────────────────");
    println!("{}", transcription);
    println!("─────────────────\n");

    // Copy to clipboard
    clipboard::copy_to_clipboard(&transcription)?;

    println!("✓ Done! The transcription has been copied to your clipboard.");

    Ok(())
}

fn wait_for_enter() -> Result<()> {
    let mut input = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    Ok(())
}
