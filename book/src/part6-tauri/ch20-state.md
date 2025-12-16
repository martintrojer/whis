# Chapter 20: Application State Management

Tauri commands need access to shared state: current recording status, settings, API keys, the active audio recorder. This chapter explores how Whis uses `app.manage()` to create global state accessible from all commands, and how `Mutex` enables safe concurrent access from multiple threads.

## The Challenge

Multiple parts of the app need shared data:

**Scenarios**:
- User clicks "Record" in UI → Need to check if already recording
- Settings change in UI → Update cached transcription config
- Tray menu click → Toggle recording state
- Hotkey pressed → Start/stop recording

**Without shared state**, we'd need:
- Database for persistence
- IPC between components
- Complex event passing

**With Tauri's managed state**: One centralized state accessible everywhere.

## The `AppState` Struct

```rust
pub struct AppState {
    pub state: Mutex<RecordingState>,
    pub recorder: Mutex<Option<AudioRecorder>>,
    pub transcription_config: Mutex<Option<TranscriptionConfig>>,
    pub record_menu_item: Mutex<Option<MenuItem<tauri::Wry>>>,
    pub settings: Mutex<Settings>,
    pub portal_shortcut: Mutex<Option<String>>,
    pub portal_bind_error: Mutex<Option<String>>,
    pub tray_available: Mutex<bool>,
}
```

**From `whis-desktop/src/state.rs:20-32`**

### Field-by-Field Breakdown

**`state: Mutex<RecordingState>`**: Current recording state
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecordingState {
    Idle,
    Recording,
    Transcribing,
}
```

**From `whis-desktop/src/state.rs:7-11`**

**Why `Mutex`?** Multiple Tauri commands can run concurrently (different frontend calls). `Mutex` ensures only one thread modifies state at a time.

**`recorder: Mutex<Option<AudioRecorder>>`**: Active audio recorder

- `None` when idle
- `Some(recorder)` while recording
- Taken out (`Option::take()`) when stopping

**`transcription_config: Mutex<Option<TranscriptionConfig>>`**: Cached API config

```rust
pub struct TranscriptionConfig {
    pub provider: TranscriptionProvider,
    pub api_key: String,
    pub language: Option<String>,
}
```

**From `whis-desktop/src/state.rs:14-18`**

**Why cache?** Reading settings from disk every transcription is slow. Cache it on first use.

**`record_menu_item: Mutex<Option<MenuItem<...>>>`**: Reference to tray menu item

Allows updating menu text: "Start Recording" ↔ "Stop Recording".

**`settings: Mutex<Settings>`**: Current settings

Kept in-sync with `~/.config/whis/settings.json`.

**`portal_shortcut: Mutex<Option<String>>`**: Actual shortcut bound by GNOME portal

On Wayland, user might configure `Ctrl+Shift+R`, but portal binds `Ctrl+Alt+R` (conflict resolution). This stores what was actually bound.

**`portal_bind_error: Mutex<Option<String>>`**: Error from portal binding

Displayed in UI if shortcut registration failed.

**`tray_available: Mutex<bool>`**: Whether system tray is available

Some Linux systems don't support tray. UI adapts accordingly.

## Creating State

```rust
impl AppState {
    pub fn new(settings: Settings, tray_available: bool) -> Self {
        Self {
            state: Mutex::new(RecordingState::Idle),
            recorder: Mutex::new(None),
            transcription_config: Mutex::new(None),
            record_menu_item: Mutex::new(None),
            settings: Mutex::new(settings),
            portal_shortcut: Mutex::new(None),
            portal_bind_error: Mutex::new(None),
            tray_available: Mutex::new(tray_available),
        }
    }
}
```

**From `whis-desktop/src/state.rs:35-46`**

**Initial state**:
- Recording state: `Idle`
- No active recorder
- No cached config (loaded lazily)
- Settings from disk
- Tray availability from setup check

## Registering State

In `lib.rs` setup:

```rust
.setup(|app| {
    let loaded_settings = settings::Settings::load();
    let tray_available = match tray::setup_tray(app) {
        Ok(_) => true,
        Err(_) => false,
    };

    app.manage(AppState::new(loaded_settings, tray_available));

    Ok(())
})
```

**From `whis-desktop/src/lib.rs:14-27`**

**`app.manage()`**: Registers state globally
- Only one instance per type
- Accessible in all commands via `State<AppState>`
- Lives for app lifetime

## Accessing State in Commands

### The `State` Extractor

```rust
#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusResponse, String> {
    let current_state = *state.state.lock().unwrap();

    let config_valid = {
        let settings = state.settings.lock().unwrap();
        settings.has_api_key()
    };

    Ok(StatusResponse {
        state: match current_state {
            RecordingState::Idle => "Idle".to_string(),
            RecordingState::Recording => "Recording".to_string(),
            RecordingState::Transcribing => "Transcribing".to_string(),
        },
        config_valid,
    })
}
```

**From `whis-desktop/src/commands.rs:24-42`**

**`State<'_, AppState>`**: Tauri injects managed state
- `state.state` accesses the `Mutex<RecordingState>` field
- `state.settings` accesses the `Mutex<Settings>` field

**Lifetime `'_`**: Elided lifetime, inferred by compiler. State lives at least as long as the command.

### Locking Pattern

```rust
let current_state = *state.state.lock().unwrap();
```

**Steps**:
1. **`state.state`**: Access the `Mutex<RecordingState>` field
2. **`.lock()`**: Acquire mutex lock, returns `Result<MutexGuard<RecordingState>, PoisonError>`
3. **`.unwrap()`**: Panic if mutex is poisoned (previous panic while holding lock)
4. **`*guard`**: Dereference guard to get `RecordingState`
5. **Copy**: `RecordingState` is `Copy`, so `*guard` copies the value

**Lock scope**:
```rust
{
    let settings = state.settings.lock().unwrap();
    settings.has_api_key() // Lock held here
} // Lock released here
```

Short-lived locks prevent deadlocks.

## Modifying State

### Updating Recording State

```rust
// Start recording
{
    let mut state_guard = state.state.lock().unwrap();
    *state_guard = RecordingState::Recording;
}

// Stop recording
{
    let mut state_guard = state.state.lock().unwrap();
    *state_guard = RecordingState::Transcribing;
}
```

**`let mut`**: Mutable guard allows mutation via `*guard`.

### Taking the Recorder

```rust
// When stopping recording
let mut recorder = state
    .recorder
    .lock()
    .unwrap()
    .take()
    .ok_or("No active recording")?;

// recorder is now owned, Option in state is None
recorder.stop_recording()?;
```

**`Option::take()`**: Replaces `Some(recorder)` with `None`, returns owned `AudioRecorder`.

**Why take?** `AudioRecorder` must be moved to stop it (consumes `self`).

### Updating Settings

```rust
#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<SaveSettingsResponse, String> {
    // Update in-memory state
    {
        let mut state_settings = state.settings.lock().unwrap();
        *state_settings = settings.clone();
        state_settings.save().map_err(|e| e.to_string())?;
    }

    // Clear cached config if provider changed
    *state.transcription_config.lock().unwrap() = None;

    Ok(SaveSettingsResponse { needs_restart: false })
}
```

**From `whis-desktop/src/commands.rs:96-131`**

**Steps**:
1. Lock settings mutex
2. Replace settings in-memory
3. Save to disk
4. Release lock
5. Clear cached config (invalidate)

## Lazy Loading Transcription Config

First transcription:

```rust
pub fn get_or_create_transcription_config(
    state: &State<AppState>
) -> Result<TranscriptionConfig> {
    // Check cache first
    let cached = state.transcription_config.lock().unwrap().clone();
    if let Some(config) = cached {
        return Ok(config);
    }

    // Not cached, load from settings
    let settings = state.settings.lock().unwrap();
    let provider = settings.provider.clone();
    let api_key = settings
        .get_api_key_for(&provider)
        .ok_or("No API key configured")?;
    let language = settings.language.clone();

    let config = TranscriptionConfig {
        provider,
        api_key,
        language,
    };

    // Cache it
    *state.transcription_config.lock().unwrap() = Some(config.clone());

    Ok(config)
}
```

**Hypothetical from transcription logic**

**Pattern**:
1. Check cache (fast path)
2. If miss, load from settings (slow path)
3. Store in cache for next time

**Invalidation**: When settings change, clear cache.

## Thread Safety Guarantees

### Why `Mutex` is Necessary

Tauri commands run on tokio's thread pool:

```
Thread 1: get_status()      | Lock state | Read | Unlock |
Thread 2: toggle_recording() |            | Lock state | Write | Unlock |
```

Without `Mutex`, race condition:
```
Thread 1: Read state = Idle
Thread 2: Read state = Idle
Thread 1: Start recording
Thread 2: Start recording  ← Both start! Overwrite each other!
```

With `Mutex`, sequential access:
```
Thread 1: Lock | Read Idle | Start recording | Set Recording | Unlock |
Thread 2:      | Wait...                                     | Lock | Read Recording | Return "already recording" | Unlock |
```

### Mutex Poisoning

If a thread panics while holding a lock, the mutex becomes "poisoned":

```rust
let guard = state.state.lock().unwrap(); // Panics if poisoned
```

**When does this happen?**
```rust
{
    let mut guard = state.state.lock().unwrap();
    *guard = RecordingState::Recording;
    panic!("oops"); // Mutex is now poisoned
}

// Later
let guard = state.state.lock().unwrap(); // Panics: "Mutex poisoned"
```

**Whis approach**: `unwrap()` panics immediately. This crashes the app, which is acceptable for a desktop app (better than corrupted state).

**Alternative**: `lock().expect("mutex poisoned")` with custom message.

## State Lifetime

The state lives as long as the Tauri app:

```
App starts
    ↓
AppState::new() creates state
    ↓
app.manage(state) registers it
    ↓
Commands access state via State<AppState>
    ↓
App exits
    ↓
AppState dropped (mutexes dropped, memory freed)
```

**No manual cleanup needed**: RAII handles it.

## Real-World Example: Toggle Recording

Combining multiple state fields:

```rust
pub fn toggle_recording(app: &AppHandle, state: &State<AppState>) -> Result<()> {
    let current_state = *state.state.lock().unwrap();

    match current_state {
        RecordingState::Idle => {
            // Start recording
            let mut recorder = AudioRecorder::new()?;
            recorder.start_recording()?;

            *state.recorder.lock().unwrap() = Some(recorder);
            *state.state.lock().unwrap() = RecordingState::Recording;

            // Update tray menu
            if let Some(menu_item) = state.record_menu_item.lock().unwrap().as_ref() {
                menu_item.set_text("Stop Recording")?;
            }

            // Update tray icon
            update_tray_icon(app, "recording")?;

            Ok(())
        }
        RecordingState::Recording => {
            // Stop recording
            *state.state.lock().unwrap() = RecordingState::Transcribing;

            let mut recorder = state
                .recorder
                .lock()
                .unwrap()
                .take()
                .ok_or("No active recording")?;

            update_tray_icon(app, "transcribing")?;

            // Stop and finalize in background
            tokio::spawn(async move {
                let recording_data = recorder.stop_recording()?;
                let output = tokio::task::spawn_blocking(move || {
                    recording_data.finalize()
                })
                .await??;

                // Transcribe...

                Ok::<_, anyhow::Error>(())
            });

            Ok(())
        }
        RecordingState::Transcribing => {
            Err("Already transcribing, please wait".into())
        }
    }
}
```

**Simplified from `whis-desktop/src/tray.rs`**

**State interactions**:
1. Read `state.state` to check current state
2. Update `state.recorder` (store or take)
3. Update `state.state` to new state
4. Update `state.record_menu_item` text
5. Tray icon update (separate)

## Validation Commands

Some commands only read state:

```rust
#[tauri::command]
pub async fn is_api_configured(state: State<'_, AppState>) -> Result<bool, String> {
    let settings = state.settings.lock().unwrap();
    Ok(settings.has_api_key())
}
```

**From `whis-desktop/src/commands.rs:18-21`**

**Read-only**: No mutation, just check configuration.

## Settings Synchronization

Settings can change in two ways:

**1. From UI** (settings dialog):
```rust
#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<SaveSettingsResponse, String> {
    let mut state_settings = state.settings.lock().unwrap();
    *state_settings = settings.clone();
    state_settings.save().map_err(|e| e.to_string())?;
    Ok(...)
}
```

**2. External edit** (user edits `~/.config/whis/settings.json`):
```rust
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let mut settings = state.settings.lock().unwrap();
    *settings = Settings::load(); // Reload from disk
    Ok(settings.clone())
}
```

**From `whis-desktop/src/commands.rs:51-56`**

**Pattern**: Always reload when reading, to catch external changes.

## Portal State (Wayland-Specific)

On Wayland with XDG Desktop Portal:

```rust
#[tauri::command]
pub fn portal_shortcut(state: State<'_, AppState>) -> Result<Option<String>, String> {
    // Check cache first
    let cached = state.portal_shortcut.lock().unwrap().clone();
    if cached.is_some() {
        return Ok(cached);
    }

    // Read from dconf (GNOME stores shortcuts there)
    Ok(crate::shortcuts::read_portal_shortcut_from_dconf())
}
```

**From `whis-desktop/src/commands.rs:83-92`**

**Why cache?** Reading from dconf is an external process call (slow).

**Why dconf?** GNOME stores portal shortcuts in dconf database. Reading it shows user what was actually bound.

## Error Handling in Commands

Commands return `Result<T, String>`:

```rust
#[tauri::command]
pub async fn toggle_recording(app: AppHandle) -> Result<(), String> {
    crate::tray::toggle_recording_public(app);
    Ok(())
}
```

**From `whis-desktop/src/commands.rs:45-48`**

**Error type**: `String` for simplicity
- Automatically serialized to JSON
- Displayed in frontend catch blocks

**Alternative**: Custom error types with `#[derive(serde::Serialize)]`.

## State Patterns Summary

### 1. Read-Only Access

```rust
let value = *state.field.lock().unwrap();
```

Short lock, copy value, release.

### 2. Mutation

```rust
{
    let mut guard = state.field.lock().unwrap();
    *guard = new_value;
} // Lock released
```

### 3. Take Ownership

```rust
let owned = state.field.lock().unwrap().take().ok_or("missing")?;
// Use owned value (mutex now contains None)
```

### 4. Lazy Initialization

```rust
let cached = state.cache.lock().unwrap().clone();
if let Some(value) = cached {
    return Ok(value);
}

// Compute value
let value = expensive_operation()?;
*state.cache.lock().unwrap() = Some(value.clone());
Ok(value)
```

### 5. Cache Invalidation

```rust
*state.cache.lock().unwrap() = None;
```

Clear cache when underlying data changes.

## Testing State

Unit testing with state:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = AppState::default();
        let current = *state.state.lock().unwrap();
        assert_eq!(current, RecordingState::Idle);
    }

    #[test]
    fn test_state_transition() {
        let state = AppState::default();

        // Idle → Recording
        *state.state.lock().unwrap() = RecordingState::Recording;
        assert_eq!(*state.state.lock().unwrap(), RecordingState::Recording);

        // Recording → Transcribing
        *state.state.lock().unwrap() = RecordingState::Transcribing;
        assert_eq!(*state.state.lock().unwrap(), RecordingState::Transcribing);
    }
}
```

**Pattern**: Create state, acquire locks, verify behavior.

## Summary

**Key Takeaways:**

1. **`app.manage()`**: Register global state, one instance per type
2. **`State<T>` extractor**: Injected into commands by Tauri
3. **`Mutex<T>`**: Thread-safe interior mutability for concurrent access
4. **Short-lived locks**: Acquire, use, release quickly to avoid deadlocks
5. **`Option::take()`**: Transfer ownership out of shared state
6. **Lazy loading**: Cache expensive operations, invalidate on change
7. **RAII**: State lives for app lifetime, automatic cleanup

**Where This Matters in Whis:**

- State definition (`whis-desktop/src/state.rs`)
- Command handlers (`whis-desktop/src/commands.rs`)
- Tray interactions (`whis-desktop/src/tray.rs`)
- Shortcut handling (`whis-desktop/src/shortcuts.rs`)

**Patterns Used:**

- **Managed state**: Tauri's dependency injection
- **Interior mutability**: `Mutex<T>` for shared mutable state
- **RAII**: State cleanup automatic
- **Lazy initialization**: Cache on first use
- **Cache invalidation**: Clear when data changes

**Design Decisions:**

1. **Why `Mutex` not `RwLock`?** Writes are common (state changes), read-heavy optimization not needed
2. **Why `unwrap()` not error handling?** Mutex poisoning is catastrophic, panic acceptable
3. **Why cache transcription config?** Avoid disk reads on every recording
4. **Why `Option<AudioRecorder>`?** Recorder only exists while recording

---

Next: [Chapter 21: Tauri Commands](./ch21-commands.md)

This chapter dives deeper into Tauri commands: the `#[tauri::command]` macro, parameter injection, async commands, and the Rust ↔ JavaScript bridge.
