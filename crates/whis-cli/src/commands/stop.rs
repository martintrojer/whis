use crate::ipc;
use anyhow::Result;

pub fn run() -> Result<()> {
    let mut client = ipc::IpcClient::connect()?;
    let _ = client.send_message(ipc::IpcMessage::Stop)?;
    println!("Service stopped");
    Ok(())
}
