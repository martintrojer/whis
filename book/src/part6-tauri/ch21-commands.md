# Chapter 21: Tauri Commands

Tauri commands are the bridge between Rust and JavaScript. When the Vue frontend calls `invoke('get_status')`, Tauri routes it to a Rust function, executes it, serializes the result, and sends it back. This chapter explores the `#[tauri::command]` macro, parameter types, async commands, error handling, and the complete request-response lifecycle.

## The Command Macro

The `#[tauri::command]` macro transforms a Rust function into a callable command:

```rust
#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusResponse, String> {
    let current_state = *state.state.lock().unwrap();

    Ok(StatusResponse {
        state: match current_state {
            RecordingState::Idle => "Idle".to_string(),
            RecordingState::Recording => "Recording".to_string(),
            RecordingState::Transcribing => "Transcribing".to_string(),
        },
        config_valid: true,
    })
}
```

**From `whis-desktop/src/commands.rs:24-42`**

**What the macro does**:
1. Generates serialization code (Rust → JSON)
2. Generates deserialization code (JSON → Rust)
3. Creates async wrapper if needed
4. Handles dependency injection (`State`, `AppHandle`, etc.)
5. Registers command in Tauri's routing table

## Calling from JavaScript

Frontend (TypeScript):

```typescript
import { invoke } from '@tauri-apps/api/core';

interface StatusResponse {
  state: string;
  config_valid: boolean;
}

const status = await invoke<StatusResponse>('get_status');
console.log(status.state); // "Idle" | "Recording" | "Transcribing"
```

**From `whis-desktop/ui/src/views/HomeView.vue`**

**`invoke<T>(command, args?)`**:
- **`T`**: TypeScript type for return value
- **`command`**: String name (function name, snake_case)
- **`args`**: Optional object with parameters

**Under the hood**:
1. `invoke()` serializes args to JSON
2. Sends IPC message to Rust backend
3. Rust deserializes, calls function
4. Rust serializes result to JSON
5. Sends back to frontend
6. `invoke()` promise resolves with typed result

## Registering Commands

In `lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    commands::get_status,
    commands::save_settings,
    commands::toggle_recording,
    commands::validate_openai_api_key,
    commands::portal_shortcut,
    // ... (20+ commands)
])
```

**From `whis-desktop/src/lib.rs:42-61`**

**`generate_handler![]`** macro:
- Takes list of command functions
- Generates routing code
- Maps command names to functions

**Convention**: Command name = function name in snake_case.

## Parameter Types

### Simple Types

```rust
#[tauri::command]
pub fn validate_openai_api_key(api_key: String) -> Result<bool, String> {
    if api_key.is_empty() {
        return Ok(true);
    }

    if !api_key.starts_with("sk-") {
        return Err("Invalid key format. OpenAI keys start with 'sk-'".to_string());
    }

    Ok(true)
}
```

**From `whis-desktop/src/commands.rs:134-145`**

**JavaScript call**:
```typescript
const valid = await invoke<boolean>('validate_openai_api_key', {
  api_key: 'sk-proj-abc123...'
});
```

**Serialization**: `String` automatically deserialized from JSON string.

### Complex Types

```rust
#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<SaveSettingsResponse, String> {
    // ...
}
```

**From `whis-desktop/src/commands.rs:96-131`**

**JavaScript call**:
```typescript
interface Settings {
  provider: string;
  api_keys: Record<string, string>;
  language: string | null;
  // ...
}

const result = await invoke<SaveSettingsResponse>('save_settings', {
  settings: {
    provider: 'openai',
    api_keys: { openai: 'sk-...' },
    language: 'en',
  }
});
```

**Requirement**: `Settings` must implement `serde::Deserialize`.

### Injected Parameters

These are **not** passed from JavaScript—Tauri injects them:

**`State<'_, AppState>`**: Managed state
```rust
#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusResponse, String>
```

**`AppHandle`**: Handle to app instance
```rust
#[tauri::command]
pub async fn toggle_recording(app: AppHandle) -> Result<(), String>
```

**`Window`**: Handle to specific window
```rust
#[tauri::command]
pub async fn close_window(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}
```

**JavaScript only passes non-injected params**:
```typescript
// Rust: fn save_settings(state: State, settings: Settings)
// JS only passes settings:
await invoke('save_settings', { settings: {...} });
```

## Return Types

### Simple Success

```rust
#[tauri::command]
pub fn get_toggle_command() -> String {
    if std::path::Path::new("/.flatpak-info").exists() {
        "flatpak run ink.whis.Whis --toggle".to_string()
    } else {
        "whis-desktop --toggle".to_string()
    }
}
```

**From `whis-desktop/src/commands.rs:221-227`**

Returns `String` directly (no `Result`).

**JavaScript**:
```typescript
const cmd = await invoke<string>('get_toggle_command');
// cmd = "whis-desktop --toggle"
```

### Result Type

```rust
#[tauri::command]
pub fn reset_shortcut() -> Result<(), String> {
    std::process::Command::new("dconf")
        .args(["reset", "-f", "/org/gnome/settings-daemon/global-shortcuts/"])
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

**From `whis-desktop/src/commands.rs:205-211`**

**On success**: `Ok(())` → JavaScript promise resolves with `null`

**On error**: `Err("message")` → JavaScript promise rejects with error

**JavaScript**:
```typescript
try {
  await invoke('reset_shortcut');
  console.log('Success');
} catch (error) {
  console.error('Failed:', error);
}
```

### Custom Response Types

```rust
#[derive(serde::Serialize)]
pub struct StatusResponse {
    pub state: String,
    pub config_valid: bool,
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusResponse, String> {
    Ok(StatusResponse {
        state: "Idle".to_string(),
        config_valid: true,
    })
}
```

**From `whis-desktop/src/commands.rs:7-42`**

**Requirement**: Must implement `serde::Serialize`.

**JavaScript**:
```typescript
interface StatusResponse {
  state: string;
  config_valid: boolean;
}

const status = await invoke<StatusResponse>('get_status');
```

TypeScript type must match Rust struct.

### Option Types

```rust
#[tauri::command]
pub fn portal_bind_error(state: State<'_, AppState>) -> Option<String> {
    state.portal_bind_error.lock().unwrap().clone()
}
```

**From `whis-desktop/src/commands.rs:215-217`**

**Serialization**:
- `Some("error")` → `"error"`
- `None` → `null`

**JavaScript**:
```typescript
const error = await invoke<string | null>('portal_bind_error');
if (error) {
  console.error('Portal error:', error);
}
```

## Async vs Sync Commands

### Sync Command

```rust
#[tauri::command]
pub fn validate_groq_api_key(api_key: String) -> Result<bool, String> {
    if !api_key.starts_with("gsk_") {
        return Err("Invalid key format. Groq keys start with 'gsk_'".to_string());
    }
    Ok(true)
}
```

**From `whis-desktop/src/commands.rs:164-174`**

Executes immediately, blocks thread until complete.

**Use when**: Pure computation, no I/O, fast execution.

### Async Command

```rust
#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<SaveSettingsResponse, String> {
    // Async operations...
    crate::shortcuts::update_shortcut(&app, &settings.shortcut).await?;
    Ok(SaveSettingsResponse { needs_restart: false })
}
```

**From `whis-desktop/src/commands.rs:96-131`**

Returns a `Future`, executed on tokio runtime.

**Use when**: I/O operations, network calls, long-running tasks.

**JavaScript side**: No difference!
```typescript
// Both look the same
await invoke('validate_groq_api_key', { api_key: '...' });
await invoke('save_settings', { settings: {...} });
```

Tauri handles async transparently.

## Error Handling Patterns

### Simple String Errors

```rust
#[tauri::command]
pub fn validate_deepgram_api_key(api_key: String) -> Result<bool, String> {
    if api_key.trim().len() < 20 {
        return Err("Invalid Deepgram API key: key appears too short".to_string());
    }
    Ok(true)
}
```

**From `whis-desktop/src/commands.rs:177-187`**

**Pros**: Simple, easy to display in UI.

**Cons**: No structured error handling, loses context.

### Converting Errors

```rust
.map_err(|e| e.to_string())
```

Common pattern to convert `anyhow::Error` or other error types to `String`.

**Example**:
```rust
#[tauri::command]
pub async fn configure_shortcut(app: AppHandle) -> Result<Option<String>, String> {
    crate::shortcuts::open_configure_shortcuts(app)
        .await
        .map_err(|e| e.to_string())  // anyhow::Error -> String
}
```

**From `whis-desktop/src/commands.rs:64-68`**

### Custom Error Types

```rust
#[derive(serde::Serialize)]
pub enum CommandError {
    NotFound { resource: String },
    InvalidInput { field: String, reason: String },
    InternalError { message: String },
}

#[tauri::command]
pub fn some_command() -> Result<String, CommandError> {
    Err(CommandError::InvalidInput {
        field: "api_key".to_string(),
        reason: "Too short".to_string(),
    })
}
```

**Hypothetical example**

**JavaScript receives**:
```json
{
  "InvalidInput": {
    "field": "api_key",
    "reason": "Too short"
  }
}
```

Frontend can pattern-match on error type.

## State and Dependency Injection

### Accessing State

```rust
#[tauri::command]
pub fn can_reopen_window(state: State<'_, AppState>) -> bool {
    if *state.tray_available.lock().unwrap() {
        return true;
    }
    
    let has_shortcut = state.portal_shortcut.lock().unwrap().is_some();
    has_shortcut
}
```

**From `whis-desktop/src/commands.rs:232-251`**

**`State<'_, AppState>`**: Injected by Tauri, no JavaScript param needed.

### Using AppHandle

```rust
#[tauri::command]
pub async fn toggle_recording(app: AppHandle) -> Result<(), String> {
    crate::tray::toggle_recording_public(app);
    Ok(())
}
```

**From `whis-desktop/src/commands.rs:45-48`**

**`AppHandle`**: Reference to the Tauri app instance.

**Uses**:
- Get managed state: `app.state::<AppState>()`
- Emit events: `app.emit("event", payload)`
- Get windows: `app.get_webview_window("main")`
- Get tray: `app.tray_by_id("main")`

### Multiple Injections

```rust
#[tauri::command]
pub async fn save_settings(
    app: AppHandle,         // Injected
    state: State<'_, AppState>,  // Injected
    settings: Settings,     // From JavaScript
) -> Result<SaveSettingsResponse, String>
```

**From `whis-desktop/src/commands.rs:96-99`**

**Order doesn't matter** for injected params. Tauri resolves by type.

**JavaScript only sees**:
```typescript
invoke('save_settings', { settings: {...} })
```

## Validation Commands

Whis validates API keys before saving:

```rust
#[tauri::command]
pub fn validate_openai_api_key(api_key: String) -> Result<bool, String> {
    if api_key.is_empty() {
        return Ok(true); // Empty is valid (falls back to env var)
    }

    if !api_key.starts_with("sk-") {
        return Err("Invalid key format. OpenAI keys start with 'sk-'".to_string());
    }

    Ok(true)
}
```

**From `whis-desktop/src/commands.rs:134-145`**

**Frontend usage**:
```typescript
async function validateApiKey(key: string) {
  try {
    await invoke('validate_openai_api_key', { api_key: key });
    // Valid
  } catch (error) {
    // Invalid, show error message
    alert(error);
  }
}
```

**Pattern**: Quick validation without network calls (just format checks).

## Platform-Specific Commands

### Detecting Flatpak

```rust
#[tauri::command]
pub fn get_toggle_command() -> String {
    if std::path::Path::new("/.flatpak-info").exists() {
        "flatpak run ink.whis.Whis --toggle".to_string()
    } else {
        "whis-desktop --toggle".to_string()
    }
}
```

**From `whis-desktop/src/commands.rs:221-227`**

Returns different command based on runtime environment.

### Linux-Specific

```rust
#[tauri::command]
pub fn reset_shortcut() -> Result<(), String> {
    std::process::Command::new("dconf")
        .args(["reset", "-f", "/org/gnome/settings-daemon/global-shortcuts/"])
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

**From `whis-desktop/src/commands.rs:205-211`**

Calls `dconf` (GNOME config tool). Only makes sense on Linux.

**Frontend can check**: Call `shortcut_backend()` first to see if portal is being used.

## Complex Logic: Can Reopen Window?

```rust
#[tauri::command]
pub fn can_reopen_window(state: State<'_, AppState>) -> bool {
    // If tray is available, user can always reopen from there
    if *state.tray_available.lock().unwrap() {
        return true;
    }

    // Check shortcut backend
    let backend_info = crate::shortcuts::backend_info();
    match backend_info.backend.as_str() {
        "TauriPlugin" => true,      // X11 shortcuts always work
        "ManualSetup" => true,       // IPC toggle always available
        "PortalGlobalShortcuts" => {
            // Portal needs a bound shortcut without errors
            let has_shortcut = state.portal_shortcut.lock().unwrap().is_some();
            let no_error = state.portal_bind_error.lock().unwrap().is_none();
            has_shortcut && no_error
        }
        _ => false,
    }
}
```

**From `whis-desktop/src/commands.rs:232-251`**

**Logic**:
1. Tray available? → Always can reopen (click tray icon)
2. X11 shortcuts? → Can reopen (global shortcut works)
3. Manual setup (CLI toggle)? → Can reopen (external command)
4. Portal shortcuts? → Can reopen only if shortcut bound successfully

**Frontend uses this** to show warning before closing window:
```typescript
const canReopen = await invoke<boolean>('can_reopen_window');
if (!canReopen) {
  alert('Warning: You may not be able to reopen this window!');
}
```

## The Complete Request Cycle

Let's trace `get_status` from start to finish:

**1. Frontend initiates**:
```typescript
const status = await invoke<StatusResponse>('get_status');
```

**2. Tauri serializes and sends**:
```json
{
  "cmd": "get_status",
  "callback": 12345,
  "error": 12346,
  "__tauriModule": "Event"
}
```

**3. Backend receives and routes**:
- Tauri core deserializes message
- Looks up `"get_status"` in command registry
- Finds `commands::get_status`

**4. Dependency injection**:
- Sees `state: State<'_, AppState>` param
- Retrieves managed `AppState` instance
- Injects it as parameter

**5. Function executes**:
```rust
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusResponse, String> {
    let current_state = *state.state.lock().unwrap();
    Ok(StatusResponse {
        state: "Idle".to_string(),
        config_valid: true,
    })
}
```

**6. Return value serialized**:
```json
{
  "state": "Idle",
  "config_valid": true
}
```

**7. Sent back to frontend**:
```json
{
  "type": "success",
  "id": 12345,
  "data": {
    "state": "Idle",
    "config_valid": true
  }
}
```

**8. Promise resolves**:
```typescript
const status = { state: "Idle", config_valid: true };
```

**Total time**: Typically <1ms for simple commands.

## Testing Commands

Commands can be tested like regular Rust functions:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_openai_key() {
        // Valid key
        assert!(validate_openai_api_key("sk-proj-abc123".to_string()).is_ok());

        // Invalid format
        assert!(validate_openai_api_key("invalid".to_string()).is_err());

        // Empty is valid
        assert!(validate_openai_api_key("".to_string()).is_ok());
    }

    #[tokio::test]
    async fn test_get_status() {
        let state = AppState::default();
        let result = get_status(tauri::State::from(&state)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().state, "Idle");
    }
}
```

**Note**: `State` injection requires a bit of setup in tests, but logic can be tested.

## Performance Considerations

### Command Overhead

Each `invoke()` has overhead:
- IPC serialization (~0.1ms)
- Thread dispatch (~0.1ms)
- Deserialization (~0.1ms)

**Total**: ~0.3-1ms per command.

**Optimization**: Batch operations where possible.

**Bad** (3 round-trips):
```typescript
const provider = await invoke('get_provider');
const apiKey = await invoke('get_api_key');
const language = await invoke('get_language');
```

**Good** (1 round-trip):
```typescript
const settings = await invoke<Settings>('get_settings');
const { provider, api_key, language } = settings;
```

### Long-Running Commands

**Problem**: Blocking commands freeze UI.

**Solution**: Use async + spawn blocking thread:

```rust
#[tauri::command]
pub async fn expensive_operation() -> Result<String, String> {
    tokio::task::spawn_blocking(|| {
        // CPU-intensive work here
        std::thread::sleep(Duration::from_secs(5));
        Ok("Done".to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
```

Frontend stays responsive while waiting.

## Summary

**Key Takeaways:**

1. **`#[tauri::command]`**: Macro that exposes Rust functions to JavaScript
2. **Registration**: `generate_handler![]` maps command names to functions
3. **Parameter types**: Simple (String, bool), complex (structs), injected (State, AppHandle)
4. **Return types**: Direct values, `Result<T, E>`, `Option<T>`, custom structs
5. **Async support**: `async fn` commands run on tokio runtime
6. **Error handling**: `Result<T, String>` pattern for simplicity
7. **Dependency injection**: State and AppHandle injected automatically
8. **IPC overhead**: ~0.3-1ms per command, batch when possible

**Where This Matters in Whis:**

- Command definitions (`whis-desktop/src/commands.rs`)
- Command registration (`whis-desktop/src/lib.rs`)
- Frontend invocations (`whis-desktop/ui/src/views/*.vue`)
- State access (Chapter 20)
- Shortcuts integration (Chapter 22)

**Patterns Used:**

- **Macro-based registration**: Declarative command exposure
- **Dependency injection**: Tauri provides context
- **Async transparency**: Frontend doesn't know/care if command is async
- **Error type conversion**: `.map_err(|e| e.to_string())`

**Design Decisions:**

1. **Why `String` errors?** Simple to display, no frontend type complexity
2. **Why inject State?** Avoid passing state explicitly, cleaner API
3. **Why async commands?** Non-blocking for I/O operations
4. **Why validation commands?** Client-side validation before saving

---

Next: [Chapter 22: Global Shortcuts](./ch22-shortcuts.md)

This final Part VI chapter covers platform-specific shortcut implementations: Tauri plugin (X11), XDG Portal (Wayland), and manual setup fallback.
