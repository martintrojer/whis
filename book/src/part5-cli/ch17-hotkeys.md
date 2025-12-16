# Chapter 17: Global Hotkeys

Global hotkeys let Whis listen for key combinations system-wide, even when the app isn't focused. Press `Ctrl+Shift+R` anywhere—browser, terminal, IDE—and start recording. This chapter explores how Whis implements cross-platform hotkeys using two different approaches: `rdev` for Linux and `global-hotkey` for Windows/macOS.

## The Challenge

Registering global hotkeys is OS-specific:

| Platform | Method | Challenges |
|----------|--------|------------|
| **Linux X11** | X11 keyboard grab | Requires input group permissions |
| **Linux Wayland** | Input device capture | Security restrictions, compositor-dependent |
| **macOS** | Carbon/Cocoa APIs | Requires accessibility permissions |
| **Windows** | `RegisterHotKey` API | Must be on GUI thread |

No single Rust crate works perfectly everywhere. Whis uses:
- **Linux**: `rdev` (raw device events)
- **macOS/Windows**: `global-hotkey` (Tauri-maintained)

## Platform Abstraction

The `hotkey` module uses conditional compilation:

```rust
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as platform;

#[cfg(not(target_os = "linux"))]
mod non_linux;
#[cfg(not(target_os = "linux"))]
use non_linux as platform;

pub struct HotkeyGuard(platform::HotkeyGuard);

pub fn setup(hotkey_str: &str) -> Result<(Receiver<()>, HotkeyGuard)> {
    let (rx, guard) = platform::setup(hotkey_str)?;
    Ok((rx, HotkeyGuard(guard)))
}
```

**From `whis-cli/src/hotkey/mod.rs:9-27`**

**Pattern**: Platform-specific modules, unified interface.

**`HotkeyGuard`**: RAII guard that keeps the hotkey registered. When dropped, hotkey is unregistered.

## Parsing Hotkey Strings

Both platforms share the same input format:

```
"ctrl+shift+r"
"alt+f1"
"super+space"
"ctrl+alt+delete"
```

**Format rules**:
- Modifiers: `ctrl`, `shift`, `alt`, `super` (Windows key / Command key)
- Main key: Letter, number, function key, or special key
- Separator: `+`
- Case-insensitive

## Linux Implementation (rdev)

Linux uses the `rdev` crate, which provides low-level keyboard event capture.

### The Hotkey Struct

```rust
#[derive(Debug, Clone)]
pub struct Hotkey {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub super_key: bool,
    pub key: Key,
}
```

**From `whis-cli/src/hotkey/linux.rs:25-32`**

Stores which modifiers are required and the main key.

### Parsing the String

```rust
impl Hotkey {
    pub fn parse(s: &str) -> Result<Self> {
        let lower = s.to_lowercase();
        let parts: Vec<&str> = lower.split('+').map(|p| p.trim()).collect();

        if parts.is_empty() {
            anyhow::bail!("Empty hotkey string");
        }

        let mut ctrl = false;
        let mut shift = false;
        let mut alt = false;
        let mut super_key = false;
        let mut main_key: Option<Key> = None;

        for part in parts {
            match part {
                "ctrl" | "control" => ctrl = true,
                "shift" => shift = true,
                "alt" => alt = true,
                "super" | "meta" | "win" | "cmd" => super_key = true,
                key_str => {
                    main_key = Some(parse_key(key_str)?);
                }
            }
        }

        let key = main_key.context("No main key specified in hotkey")?;

        Ok(Hotkey {
            ctrl,
            shift,
            alt,
            super_key,
            key,
        })
    }
}
```

**From `whis-cli/src/hotkey/linux.rs:34-72`**

**Step-by-step**:

1. **Lowercase**: `"Ctrl+Shift+R"` → `"ctrl+shift+r"`
2. **Split**: `["ctrl", "shift", "r"]`
3. **Classify**: Modifiers vs main key
4. **Validate**: Must have at least one main key

**Aliases**:
- `"control"` → `ctrl`
- `"super"`, `"meta"`, `"win"`, `"cmd"` → `super_key`

### Key Name Mapping

```rust
fn parse_key(s: &str) -> Result<Key> {
    let key = match s {
        "a" => Key::KeyA,
        "b" => Key::KeyB,
        // ... (all letters)
        "0" => Key::Num0,
        "1" => Key::Num1,
        // ... (all digits)
        "f1" => Key::F1,
        "f2" => Key::F2,
        // ... (F1-F12)
        "space" => Key::Space,
        "enter" | "return" => Key::Return,
        "escape" | "esc" => Key::Escape,
        "tab" => Key::Tab,
        "backspace" => Key::Backspace,
        "delete" | "del" => Key::Delete,
        "insert" | "ins" => Key::Insert,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" | "pgup" => Key::PageUp,
        "pagedown" | "pgdn" => Key::PageDown,
        "up" => Key::UpArrow,
        "down" => Key::DownArrow,
        "left" => Key::LeftArrow,
        "right" => Key::RightArrow,
        _ => anyhow::bail!("Unknown key: {s}"),
    };
    Ok(key)
}
```

**From `whis-cli/src/hotkey/linux.rs:75-143`**

Maps string names to `rdev::Key` enum variants.

**Example**:
```rust
parse_key("r")      // → Key::KeyR
parse_key("f1")     // → Key::F1
parse_key("escape") // → Key::Escape
parse_key("xyz")    // → Error: Unknown key
```

### Setup Function

```rust
pub fn setup(hotkey_str: &str) -> Result<(Receiver<()>, HotkeyGuard)> {
    let hotkey = Hotkey::parse(hotkey_str)?;
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        if let Err(e) = listen_for_hotkey(hotkey, move || {
            let _ = tx.send(());
        }) {
            eprintln!("Hotkey error: {e}");
        }
    });

    Ok((rx, HotkeyGuard))
}
```

**From `whis-cli/src/hotkey/linux.rs:9-22`**

**Flow**:
1. Parse hotkey string
2. Create channel: `(tx, rx)`
3. Spawn thread for hotkey listening
4. Return receiver and guard

**Why spawn a thread?**  
`listen_for_hotkey()` blocks indefinitely. Running on a separate thread prevents blocking the main thread.

### The Listening Loop

```rust
pub fn listen_for_hotkey<F>(hotkey: Hotkey, on_press: F) -> Result<()>
where
    F: Fn() + Send + 'static,
{
    let pressed_keys: Arc<Mutex<HashSet<Key>>> = Arc::new(Mutex::new(HashSet::new()));
    let pressed_keys_clone = pressed_keys.clone();

    let callback = move |event: Event| -> Option<Event> {
        match event.event_type {
            EventType::KeyPress(key) => {
                let mut keys = pressed_keys_clone.lock().unwrap();
                keys.insert(key);

                // Check if hotkey combination is pressed
                let ctrl_ok = !hotkey.ctrl
                    || keys.contains(&Key::ControlLeft)
                    || keys.contains(&Key::ControlRight);
                let shift_ok = !hotkey.shift
                    || keys.contains(&Key::ShiftLeft)
                    || keys.contains(&Key::ShiftRight);
                let alt_ok = !hotkey.alt || keys.contains(&Key::Alt) || keys.contains(&Key::AltGr);
                let super_ok = !hotkey.super_key
                    || keys.contains(&Key::MetaLeft)
                    || keys.contains(&Key::MetaRight);
                let key_ok = keys.contains(&hotkey.key);

                if ctrl_ok && shift_ok && alt_ok && super_ok && key_ok {
                    on_press();
                }
            }
            EventType::KeyRelease(key) => {
                let mut keys = pressed_keys_clone.lock().unwrap();
                keys.remove(&key);
            }
            _ => {}
        }
        // Return Some(event) to pass the event through, None to consume it
        Some(event)
    };

    // This blocks and listens for all keyboard events
    if let Err(e) = grab(callback) {
        anyhow::bail!(
            "Failed to grab keyboard: {e:?}\n\nSetup required:\n  sudo usermod -aG input $USER\n  echo 'KERNEL==\"uinput\", GROUP=\"input\", MODE=\"0660\"' | sudo tee /etc/udev/rules.d/99-uinput.rules\n  sudo udevadm control --reload-rules && sudo udevadm trigger\nThen logout and login again."
        );
    }

    Ok(())
}
```

**From `whis-cli/src/hotkey/linux.rs:147-195`**

### Tracking Pressed Keys

```rust
let pressed_keys: Arc<Mutex<HashSet<Key>>> = Arc::new(Mutex::new(HashSet::new()));
```

**Why a `HashSet`?**  
Multiple keys can be pressed simultaneously. We track all currently pressed keys.

**Example state**:
```
User presses Ctrl: {ControlLeft}
User presses Shift: {ControlLeft, ShiftLeft}
User presses R: {ControlLeft, ShiftLeft, KeyR}  ← Hotkey triggered!
User releases R: {ControlLeft, ShiftLeft}
User releases Ctrl: {ShiftLeft}
User releases Shift: {}
```

### Event Callback

```rust
let callback = move |event: Event| -> Option<Event> {
    match event.event_type {
        EventType::KeyPress(key) => {
            let mut keys = pressed_keys_clone.lock().unwrap();
            keys.insert(key);
            // Check if hotkey matches...
        }
        EventType::KeyRelease(key) => {
            let mut keys = pressed_keys_clone.lock().unwrap();
            keys.remove(&key);
        }
        _ => {}
    }
    Some(event) // Pass event through (don't consume)
};
```

**On KeyPress**: Add key to set, check if hotkey combination is complete.

**On KeyRelease**: Remove key from set.

**Return value**:
- `Some(event)`: Pass event to other apps (doesn't block normal typing)
- `None`: Consume event (would prevent typing)

Whis returns `Some(event)` so normal keyboard usage isn't disrupted.

### Checking Hotkey Match

```rust
let ctrl_ok = !hotkey.ctrl
    || keys.contains(&Key::ControlLeft)
    || keys.contains(&Key::ControlRight);
```

**Logic**: 
- If `!hotkey.ctrl`: We don't require Ctrl → Always OK
- Otherwise: Check if either ControlLeft or ControlRight is pressed

**Why both?** Left and right modifiers are separate keys.

**Full check**:
```rust
if ctrl_ok && shift_ok && alt_ok && super_ok && key_ok {
    on_press(); // Hotkey matched!
}
```

All modifiers must match AND the main key must be pressed.

### Permission Requirements

```rust
if let Err(e) = grab(callback) {
    anyhow::bail!(
        "Failed to grab keyboard: {e:?}\n\nSetup required:\n  sudo usermod -aG input $USER\n  echo 'KERNEL==\"uinput\", GROUP=\"input\", MODE=\"0660\"' | sudo tee /etc/udev/rules.d/99-uinput.rules\n  sudo udevadm control --reload-rules && sudo udevadm trigger\nThen logout and login again."
    );
}
```

**Linux requirement**: User must be in `input` group to access `/dev/input/` devices.

**Setup steps**:
1. Add user to input group: `sudo usermod -aG input $USER`
2. Create udev rule for uinput device
3. Reload udev rules
4. **Log out and log back in** (group membership updates)

Without this, `grab()` fails with permission denied.

## Windows/macOS Implementation (global-hotkey)

Non-Linux platforms use the `global-hotkey` crate (maintained by Tauri team).

### Setup Function

```rust
pub fn setup(hotkey_str: &str) -> Result<(Receiver<()>, HotkeyGuard)> {
    let converted = convert_to_global_hotkey_format(hotkey_str)?;
    let hotkey: HotKey = converted
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid hotkey '{}': {:?}", hotkey_str, e))?;

    let manager = GlobalHotKeyManager::new()
        .map_err(|e| anyhow::anyhow!("Failed to create hotkey manager: {:?}", e))?;

    manager.register(hotkey.clone()).map_err(|e| {
        anyhow::anyhow!(
            "Failed to register hotkey '{}': {:?}\n\n\
            This may mean the hotkey is already registered by another application.",
            hotkey_str,
            e
        )
    })?;

    let receiver = GlobalHotKeyEvent::receiver().clone();
    let hotkey_id = hotkey.id();
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        loop {
            if let Ok(event) = receiver.recv() {
                if event.id() == hotkey_id {
                    let _ = tx.send(());
                }
            }
        }
    });

    Ok((rx, HotkeyGuard { _manager: manager }))
}
```

**From `whis-cli/src/hotkey/non_linux.rs:13-46`**

**Flow**:
1. Convert format: `"ctrl+shift+r"` → `"Ctrl+Shift+KeyR"`
2. Parse into `HotKey` struct
3. Create `GlobalHotKeyManager`
4. Register hotkey
5. Spawn thread to listen for events
6. Return receiver and guard

### Format Conversion

`global-hotkey` uses different naming:

| Our Format | global-hotkey Format |
|------------|----------------------|
| `ctrl` | `Ctrl` |
| `shift` | `Shift` |
| `r` | `KeyR` |
| `5` | `Digit5` |
| `f1` | `F1` |
| `space` | `Space` |
| `up` | `ArrowUp` |

**Conversion function** (simplified):

```rust
fn convert_to_global_hotkey_format(s: &str) -> Result<String> {
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    let mut result = Vec::new();
    let mut has_main_key = false;

    for part in parts {
        let lower = part.to_lowercase();
        let converted = match lower.as_str() {
            "ctrl" | "control" => "Ctrl".to_string(),
            "shift" => "Shift".to_string(),
            "alt" => "Alt".to_string(),
            "super" | "meta" | "win" | "cmd" => "Super".to_string(),

            // Single letters -> KeyX format
            key if key.len() == 1 && key.chars().next().unwrap().is_ascii_alphabetic() => {
                has_main_key = true;
                format!("Key{}", key.to_uppercase())
            }

            // Numbers -> DigitX format
            key if key.len() == 1 && key.chars().next().unwrap().is_ascii_digit() => {
                has_main_key = true;
                format!("Digit{}", key)
            }

            "f1" => { has_main_key = true; "F1".to_string() }
            // ... (F2-F12)

            "space" => { has_main_key = true; "Space".to_string() }
            "enter" | "return" => { has_main_key = true; "Enter".to_string() }
            // ... (other special keys)

            _ => anyhow::bail!("Unknown key: {}", part),
        };
        result.push(converted);
    }

    if !has_main_key {
        anyhow::bail!("No main key specified in hotkey");
    }

    Ok(result.join("+"))
}
```

**From `whis-cli/src/hotkey/non_linux.rs:52-205`**

**Examples**:
```rust
convert("ctrl+shift+r")  // → "Ctrl+Shift+KeyR"
convert("alt+5")         // → "Alt+Digit5"
convert("super+f1")      // → "Super+F1"
convert("ctrl+space")    // → "Ctrl+Space"
```

### Registering with OS

```rust
let manager = GlobalHotKeyManager::new()?;
manager.register(hotkey.clone())?;
```

**Under the hood**:
- **macOS**: Uses Cocoa `NSEvent.addGlobalMonitorForEvents`
- **Windows**: Uses Win32 `RegisterHotKey` API

Both require elevated permissions:
- **macOS**: User must grant accessibility permissions
- **Windows**: Works by default (no special permissions)

### Event Loop

```rust
let receiver = GlobalHotKeyEvent::receiver().clone();
let hotkey_id = hotkey.id();
let (tx, rx) = std::sync::mpsc::channel();

std::thread::spawn(move || {
    loop {
        if let Ok(event) = receiver.recv() {
            if event.id() == hotkey_id {
                let _ = tx.send(());
            }
        }
    }
});
```

**`GlobalHotKeyEvent::receiver()`**: Global channel for all hotkey events.

**Filtering by ID**: Multiple hotkeys can be registered. We only care about events matching our hotkey's ID.

**Channel bridge**: Convert `global-hotkey` events to simple `()` signals on our channel.

### The Guard

```rust
pub struct HotkeyGuard {
    _manager: GlobalHotKeyManager,
}
```

**RAII pattern**: When `HotkeyGuard` is dropped, `GlobalHotKeyManager` is dropped, which automatically unregisters the hotkey.

**Usage**:
```rust
{
    let (_rx, guard) = setup("ctrl+shift+r")?;
    // Hotkey is registered
} // guard drops here
// Hotkey is now unregistered
```

## Using the Hotkey System

From the CLI command handler:

```rust
pub fn run(hotkey_str: String) -> Result<()> {
    println!("🎧 Listening for hotkey: {}", hotkey_str);
    println!("Press {} to record. Press Ctrl+C to stop service.", hotkey_str);

    let (rx, _guard) = hotkey::setup(&hotkey_str)?;

    loop {
        // Wait for hotkey press
        rx.recv()?;
        
        println!("🎤 Recording...");
        start_recording()?;
        
        // Wait for hotkey press again (to stop)
        rx.recv()?;
        
        println!("⏹️  Stopped.");
        let audio = stop_recording()?;
        
        println!("🔄 Transcribing...");
        let text = transcribe(audio)?;
        
        println!("✅ {}", text);
        clipboard::copy(&text)?;
    }
}
```

**Simplified from `whis-cli/src/commands/listen.rs`**

**Guard must be kept alive**: `_guard` lives for the entire function. If we didn't store it, it would drop immediately and unregister the hotkey.

## Error Scenarios

### Linux: Permission Denied

```
$ whis listen
Failed to grab keyboard: Os { code: 13, kind: PermissionDenied }

Setup required:
  sudo usermod -aG input $USER
  echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/99-uinput.rules
  sudo udevadm control --reload-rules && sudo udevadm trigger
Then logout and login again.
```

**Solution**: Follow setup instructions, log out and back in.

### macOS: Accessibility Permissions

```
$ whis listen
Failed to create hotkey manager: AccessibilityNotEnabled

Please grant accessibility permissions:
  System Settings → Privacy & Security → Accessibility
  Add Terminal (or your terminal app) to the list
```

**Solution**: Grant accessibility permissions in System Settings.

### Windows: Hotkey Already Registered

```
$ whis listen --hotkey ctrl+shift+r
Failed to register hotkey 'ctrl+shift+r': HotkeyAlreadyRegistered

This may mean the hotkey is already registered by another application.
```

**Solution**: Choose a different hotkey combination.

## Testing Hotkey Parsing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_format_conversion() {
        assert_eq!(
            convert_to_global_hotkey_format("ctrl+shift+r").unwrap(),
            "Ctrl+Shift+KeyR"
        );
        assert_eq!(
            convert_to_global_hotkey_format("alt+5").unwrap(),
            "Alt+Digit5"
        );
        assert_eq!(
            convert_to_global_hotkey_format("super+f1").unwrap(),
            "Super+F1"
        );
    }

    #[test]
    fn test_invalid_hotkey() {
        assert!(convert_to_global_hotkey_format("ctrl+shift+invalidkey").is_err());
        assert!(convert_to_global_hotkey_format("").is_err());
        assert!(convert_to_global_hotkey_format("ctrl+shift").is_err()); // No main key
    }
}
```

**From `whis-cli/src/hotkey/non_linux.rs:207-237`**

Unit tests verify format conversion without actually registering hotkeys.

## Platform Differences Summary

| Feature | Linux (rdev) | macOS/Windows (global-hotkey) |
|---------|--------------|-------------------------------|
| **Permission** | Requires `input` group | macOS: Accessibility, Windows: None |
| **Method** | Raw device grab | OS-specific APIs |
| **Thread** | Spawned (blocking) | Spawned (event loop) |
| **Unregister** | Automatic on drop | Automatic on drop |
| **Pass-through** | Yes (returns `Some(event)`) | N/A (OS handles) |

## Summary

**Key Takeaways:**

1. **Platform abstraction**: Conditional compilation with unified interface
2. **Linux**: `rdev` for raw keyboard grab, requires input group permissions
3. **macOS/Windows**: `global-hotkey` for OS APIs, requires accessibility on macOS
4. **Parsing**: Unified format (`"ctrl+shift+r"`), platform-specific conversion
5. **RAII guard**: Automatic hotkey unregistration on drop
6. **Channel pattern**: Convert events to simple signals

**Where This Matters in Whis:**

- CLI daemon mode (`whis-cli/src/commands/listen.rs`)
- Desktop app global shortcuts (`whis-desktop/src/shortcuts.rs` - Chapter 22)
- Cross-platform hotkey registration

**Patterns Used:**

- **Conditional compilation**: `#[cfg(target_os = "...")]`
- **RAII**: `HotkeyGuard` ensures cleanup
- **Channel bridge**: Platform events → unified signal
- **Thread spawning**: Non-blocking hotkey listening

**Design Decisions:**

1. **Why two libraries?** No single solution works well everywhere
2. **Why spawn threads?** Hotkey listening blocks
3. **Why pass events through?** Don't disrupt normal typing on Linux
4. **Why track all keys?** Detect modifier combinations accurately

---

Next: [Chapter 18: IPC and Daemon Mode](./ch18-ipc.md)

This chapter covers inter-process communication for stopping the daemon and checking status.
