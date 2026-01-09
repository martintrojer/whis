//! PulseAudio device enumeration with rich metadata.
//!
//! Uses libpulse-binding to access device properties like form-factor,
//! bus type, and monitor status for reliable device classification.

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::{Context, Result};
use libpulse_binding::{
    callbacks::ListResult,
    context::{self, Context as PulseContext, FlagSet, introspect::SourceInfo},
    mainloop::standard::{IterateResult, Mainloop},
    proplist::Proplist,
};

use super::types::AudioDeviceInfo;

/// Get audio input devices with PulseAudio metadata.
///
/// Returns devices with form_factor, bus, and is_monitor populated.
/// Filters out monitor sources automatically.
pub fn list_pulse_devices() -> Result<Vec<AudioDeviceInfo>> {
    // Create mainloop
    let mainloop = Rc::new(RefCell::new(
        Mainloop::new().context("Failed to create PulseAudio mainloop")?,
    ));

    // Create context
    let context = Rc::new(RefCell::new(
        PulseContext::new(&*mainloop.borrow(), "whis-device-enum")
            .context("Failed to create PulseAudio context")?,
    ));

    // Connect to server
    context
        .borrow_mut()
        .connect(None, FlagSet::NOFLAGS, None)
        .context("Failed to connect to PulseAudio server")?;

    // Wait for context to be ready
    loop {
        match mainloop.borrow_mut().iterate(true) {
            IterateResult::Quit(_) | IterateResult::Err(_) => {
                anyhow::bail!("PulseAudio mainloop iteration failed");
            }
            IterateResult::Success(_) => {}
        }

        match context.borrow().get_state() {
            context::State::Ready => break,
            context::State::Failed | context::State::Terminated => {
                anyhow::bail!("PulseAudio context connection failed");
            }
            _ => {} // Keep waiting
        }
    }

    // Collect devices
    let devices: Rc<RefCell<Vec<AudioDeviceInfo>>> = Rc::new(RefCell::new(Vec::new()));
    let done = Rc::new(RefCell::new(false));

    // Get default source name for marking default device
    let default_source: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    // First, get server info to find default source
    {
        let default_source_clone = default_source.clone();
        let done_clone = done.clone();

        let introspector = context.borrow().introspect();
        introspector.get_server_info(move |info| {
            if let Some(name) = &info.default_source_name {
                *default_source_clone.borrow_mut() = Some(name.to_string());
            }
            *done_clone.borrow_mut() = true;
        });

        // Wait for server info
        while !*done.borrow() {
            match mainloop.borrow_mut().iterate(true) {
                IterateResult::Quit(_) | IterateResult::Err(_) => break,
                IterateResult::Success(_) => {}
            }
        }
    }

    // Reset done flag
    *done.borrow_mut() = false;

    // Get source list
    {
        let devices_clone = devices.clone();
        let done_clone = done.clone();
        let default_source_clone = default_source.clone();

        let introspector = context.borrow().introspect();
        introspector.get_source_info_list(move |result| match result {
            ListResult::Item(info) => {
                if let Some(device) = source_info_to_device(info, &default_source_clone.borrow()) {
                    devices_clone.borrow_mut().push(device);
                }
            }
            ListResult::End | ListResult::Error => {
                *done_clone.borrow_mut() = true;
            }
        });

        // Wait for source list
        while !*done.borrow() {
            match mainloop.borrow_mut().iterate(true) {
                IterateResult::Quit(_) | IterateResult::Err(_) => break,
                IterateResult::Success(_) => {}
            }
        }
    }

    // Disconnect
    context.borrow_mut().disconnect();
    mainloop.borrow_mut().quit(libpulse_binding::def::Retval(0));

    // Extract and return devices
    let result = Rc::try_unwrap(devices)
        .map_err(|_| anyhow::anyhow!("Failed to unwrap devices"))?
        .into_inner();

    if result.is_empty() {
        anyhow::bail!("No audio input devices found via PulseAudio");
    }

    Ok(result)
}

/// Convert PulseAudio SourceInfo to our AudioDeviceInfo.
/// Returns None for monitor sources (we filter them out).
fn source_info_to_device(
    info: &SourceInfo,
    default_source: &Option<String>,
) -> Option<AudioDeviceInfo> {
    // Skip monitor sources (loopback from output)
    let is_monitor = info.monitor_of_sink.is_some();
    if is_monitor {
        return None;
    }

    // Get device name (technical identifier)
    let name = info.name.as_ref()?.to_string();

    // Get display name (human-readable description)
    let display_name = info.description.as_ref().map(|d| d.to_string());

    // Check if this is the default device
    let is_default = default_source
        .as_ref()
        .is_some_and(|default| default == &name);

    // Extract properties from proplist
    let (form_factor, bus) = extract_properties(&info.proplist);

    Some(AudioDeviceInfo {
        name,
        display_name,
        is_default,
        form_factor,
        bus,
        is_monitor,
    })
}

/// Extract device properties from PulseAudio proplist.
fn extract_properties(proplist: &Proplist) -> (Option<String>, Option<String>) {
    let form_factor = proplist
        .get_str("device.form_factor")
        .map(|s| s.to_string());

    let bus = proplist.get_str("device.bus").map(|s| s.to_string());

    (form_factor, bus)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires PulseAudio to be running
    fn test_list_pulse_devices() {
        let devices = list_pulse_devices().unwrap();
        println!("Found {} devices:", devices.len());
        for device in &devices {
            println!(
                "  {} (form_factor: {:?}, bus: {:?}, default: {})",
                device.display_name.as_ref().unwrap_or(&device.name),
                device.form_factor,
                device.bus,
                device.is_default
            );
        }
        assert!(!devices.is_empty());
    }
}
