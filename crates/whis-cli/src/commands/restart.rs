use crate::ipc;
use anyhow::Result;

pub fn run() -> Result<()> {
    // Stop the service if running
    if ipc::is_service_running() {
        let mut client = ipc::IpcClient::connect()?;
        let _ = client.send_message(ipc::IpcMessage::Stop)?;
        println!("Service stopped");

        // Wait a moment for graceful shutdown
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    // Start the service (uses settings for shortcut_mode)
    crate::commands::start::run(false)
}
