use crate::ipc;
use anyhow::Result;

pub fn run() -> Result<()> {
    let mut client = ipc::IpcClient::connect()?;
    match client.send_message(ipc::IpcMessage::Toggle)? {
        ipc::IpcResponse::Recording => println!("Recording..."),
        ipc::IpcResponse::Idle => println!("Stopped"),
        ipc::IpcResponse::Transcribing => println!("Transcribing..."),
        ipc::IpcResponse::Success => println!("Done"),
        ipc::IpcResponse::Error(e) => anyhow::bail!(e),
    }
    Ok(())
}
