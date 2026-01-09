use crate::{app, hotkey, ipc, service};
use anyhow::Result;
use whis_core::settings::CliShortcutMode;
use whis_core::Settings;

pub fn run() -> Result<()> {
    // Check if FFmpeg is available
    app::ensure_ffmpeg_installed()?;

    // Check if service is already running
    if ipc::is_service_running() {
        eprintln!("Error: whis service is already running.");
        eprintln!("Use 'whis stop' to stop the existing service first.");
        std::process::exit(1);
    }

    // Load settings and transcription configuration
    let settings = Settings::load();
    let config = app::load_transcription_config()?;

    // Create Tokio runtime
    let runtime = tokio::runtime::Runtime::new()?;

    // Based on cli_shortcut_mode, decide how to run
    match settings.ui.cli_shortcut_mode {
        CliShortcutMode::Direct => {
            // Try to set up hotkey via evdev/rdev
            let shortcut = &settings.ui.shortcut_key;
            match hotkey::setup(shortcut) {
                Ok((hotkey_rx, _guard)) => {
                    println!("Listening. Press {} to record. Ctrl+C to stop.", shortcut);

                    runtime.block_on(async {
                        let service = service::Service::new(config)?;
                        tokio::select! {
                            result = service.run(Some(hotkey_rx)) => result,
                            _ = tokio::signal::ctrl_c() => {
                                println!("\nShutting down...");
                                Ok(())
                            }
                        }
                    })
                }
                Err(e) => {
                    eprintln!("Error: Could not set up hotkey: {}", e);
                    eprintln!();
                    eprintln!("To use direct hotkey capture, run:");
                    eprintln!("  sudo usermod -aG input $USER");
                    eprintln!("Then logout and login again.");
                    eprintln!();
                    eprintln!("Or switch to system mode:");
                    eprintln!("  whis config cli-shortcut-mode system");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            // "system" mode (or any other value) - IPC only
            println!("Listening. Press your configured shortcut to record. Ctrl+C to stop.");

            runtime.block_on(async {
                let service = service::Service::new(config)?;
                tokio::select! {
                    result = service.run(None) => result,
                    _ = tokio::signal::ctrl_c() => {
                        println!("\nShutting down...");
                        Ok(())
                    }
                }
            })
        }
    }
}
