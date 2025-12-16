use crate::{app, hotkey, ipc, service};
use anyhow::Result;

/// Guard to clean up PID and socket files on exit
struct CleanupGuard;

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        ipc::remove_pid_file();
    }
}

pub fn run(hotkey_str: String) -> Result<()> {
    // Check if FFmpeg is available
    app::ensure_ffmpeg_installed()?;

    // Check if service is already running
    if ipc::is_service_running() {
        eprintln!("Error: whis service is already running.");
        eprintln!("Use 'whis stop' to stop the existing service first.");
        std::process::exit(1);
    }

    // Load transcription configuration (provider + API key)
    let config = app::load_transcription_config()?;

    // Write PID file
    ipc::write_pid_file()?;

    // Set up cleanup on exit
    let _cleanup = CleanupGuard;

    // Setup hotkey listener
    // This handles platform differences internally
    println!("Registering hotkey: {}", hotkey_str);
    let (hotkey_rx, _guard) = hotkey::setup(&hotkey_str)?;

    // Create Tokio runtime
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        // Create service
        let service = service::Service::new(config)?;

        // Run service loop
        tokio::select! {
            result = service.run(Some(hotkey_rx)) => result,
            _ = tokio::signal::ctrl_c() => {
                println!("\nShutting down...");
                Ok(())
            }
        }
    })
}
