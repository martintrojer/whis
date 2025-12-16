use crate::ipc;
use anyhow::Result;

pub fn run() -> Result<()> {
    if !ipc::is_service_running() {
        println!("Status: Not running");
        println!("Start with: whis listen");
        return Ok(());
    }

    let mut client = ipc::IpcClient::connect()?;
    let response = client.send_message(ipc::IpcMessage::Status)?;

    match response {
        ipc::IpcResponse::Idle => println!("Status: Running (idle)"),
        ipc::IpcResponse::Recording => println!("Status: Running (recording)"),
        ipc::IpcResponse::Transcribing => println!("Status: Running (transcribing)"),
        ipc::IpcResponse::Error(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
        _ => println!("Status: Running"),
    }

    Ok(())
}
