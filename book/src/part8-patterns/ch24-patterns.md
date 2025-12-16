# Chapter 24: Common Patterns in Whis

Across the Whis codebase, certain design patterns appear repeatedly. These aren't arbitrary choices—each pattern solves a specific problem that arises in building a cross-platform audio transcription app.

In this chapter, we'll examine:
- `Arc<Mutex<T>>` for shared mutable state
- `Result<T>` and context-aware error handling
- Async runtime patterns with tokio
- Provider trait pattern for extensibility
- State machine for recording lifecycle
- Command pattern in Tauri

## Pattern 1: Arc&lt;Mutex&lt;T&gt;&gt; for Shared State

### The Problem

Audio recording happens in a callback thread (owned by cpal). The main thread needs to access the same sample buffer. Rust's ownership system prevents sharing mutable data across threads without synchronization.

### The Solution

`Arc<Mutex<T>>` combines two primitives:
- **Arc** (Atomic Reference Counted): Allows multiple owners across threads
- **Mutex**: Ensures exclusive access when modifying

From `whis-core/src/audio.rs:39-44`:

```rust
pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
    stream: Option<cpal::Stream>,
}
```

**From `whis-core/src/audio.rs:39-44`**

The `samples` field is wrapped in `Arc<Mutex<Vec<f32>>>`. When starting recording, we clone the Arc:

```rust
let samples = self.samples.clone();  // Increments ref count
samples.lock().unwrap().clear();
```

**From `whis-core/src/audio.rs:69-70`**

The clone is cheap—it only increments an atomic counter. Both the callback thread and main thread now share ownership. When the callback receives audio data:

```rust
let mut samples = samples_clone.lock().unwrap();
samples.extend_from_slice(&data);
```

The `lock()` acquires the mutex, blocking if another thread holds it. The `MutexGuard` released at end of scope automatically unlocks.

### When to Use

Use `Arc<Mutex<T>>` when:
- Multiple threads need to mutate the same data
- Access is infrequent (locking has overhead)
- Contention is low (threads rarely wait on each other)

**Don't use** for high-frequency updates—consider channels (mpsc) instead for queue-like access.

### Real-World Trade-offs

In Whis, audio callback runs ~100 times/second. The lock is held for microseconds (just extending a vector). This is acceptable. For real-time audio processing with lower latency requirements, a lock-free ring buffer would be better.

## Pattern 2: Result&lt;T&gt; and Error Context

### The Problem

Rust requires explicit error handling. Functions that can fail return `Result<T, E>`. But raw error types like `std::io::Error` don't include context about _why_ the error occurred.

### The Solution

The `anyhow` crate provides two key features:
1. **`anyhow::Result<T>`**: Alias for `Result<T, anyhow::Error>`, a boxed error that can hold any error type
2. **`.context()`**: Adds descriptive context to errors as they bubble up

From `whis-core/src/audio.rs:56-64`:

```rust
pub fn start_recording(&mut self) -> Result<()> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("No input device available")?;

    let config = device
        .default_input_config()
        .context("Failed to get default input config")?;
```

**From `whis-core/src/audio.rs:56-64`**

Each `?` operator checks for error. If error occurs, `.context()` wraps it with additional information. The final error message shows the full chain:

```
Failed to get default input config
Caused by:
    Device disconnected
```

This makes debugging much easier than just seeing "Device disconnected" with no context about what was attempting to use the device.

### Pattern: Early Return with ?

The `?` operator is Rust's error propagation syntax:

```rust
let file = std::fs::File::open("audio.mp3")?;
```

This is equivalent to:

```rust
let file = match std::fs::File::open("audio.mp3") {
    Ok(f) => f,
    Err(e) => return Err(e.into()),
};
```

The `?` operator converts the error type automatically (via `From` trait) and returns early.

### When to Use

Use `anyhow::Result<T>` for:
- Applications (not libraries—libraries should use specific error types)
- Error chains where context matters
- Quick prototyping

Use `.context()` on every operation that could fail in a non-obvious way.

## Pattern 3: Async Runtime with Tokio

### The Problem

Parallel transcription needs to:
1. Spawn multiple HTTP requests concurrently
2. Limit concurrency to avoid overwhelming APIs
3. Collect results in order

### The Solution

Tokio provides an async runtime with:
- **`tokio::spawn`**: Spawns a new task (like a lightweight thread)
- **`tokio::sync::Semaphore`**: Limits concurrent access to a resource
- **`await`**: Suspends execution until a future completes

From `whis-core/src/transcribe.rs:52-80`:

```rust
pub async fn parallel_transcribe(
    provider: &TranscriptionProvider,
    api_key: &str,
    language: Option<&str>,
    chunks: Vec<AudioChunk>,
    progress_callback: Option<Box<dyn Fn(usize, usize) + Send + Sync>>,
) -> Result<String> {
    let total_chunks = chunks.len();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .build()?;

    // Semaphore to limit concurrent requests
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));
    let client = Arc::new(client);
    let api_key = Arc::new(api_key.to_string());
    // ...

    let mut handles = Vec::with_capacity(total_chunks);

    for chunk in chunks {
        let semaphore = semaphore.clone();
        // ... clone other Arc values ...

        let handle = tokio::spawn(async move {
            // Acquire semaphore permit (blocks if limit reached)
            let _permit = semaphore.acquire().await.unwrap();
            
            // Make API request
            let result = provider_impl.transcribe_async(/* ... */).await?;
            
            // Permit automatically released when _permit drops
            Ok(result)
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await??);
    }

    Ok(merge_results(results))
}
```

**From `whis-core/src/transcribe.rs:52-120` (simplified)**

Key mechanics:

1. **Semaphore initialization**: `Semaphore::new(3)` allows 3 concurrent permits
2. **Spawn all tasks immediately**: `tokio::spawn` creates tasks but they wait on semaphore
3. **Acquire permit**: `.acquire().await` blocks if all permits in use
4. **Automatic release**: When `_permit` drops (end of scope), permit returns to semaphore

This ensures exactly 3 requests run concurrently. If chunk 4 starts before chunk 1 finishes, it waits.

### Why Not Just Limit spawn() Calls?

If we spawned tasks sequentially (wait for previous 3 to finish before spawning more), we'd waste time. Spawning all tasks upfront means as soon as a slot opens, the next task starts immediately.

### Double Question Mark: `handle.await??`

- First `?`: `JoinHandle::await` returns `Result<T, JoinError>` (task panicked?)
- Second `?`: Inner function returns `Result<String, anyhow::Error>` (API error?)

Both must succeed for the result to be `Ok`.

### When to Use

Use this pattern when:
- You have many independent I/O operations (HTTP requests, DB queries)
- You need to limit concurrency (API rate limits, connection pools)
- You want results as soon as all complete (not streaming)

## Pattern 4: Provider Trait for Extensibility

### The Problem

Whis supports multiple transcription providers (OpenAI, Groq, Deepgram, etc.). Each has different:
- API endpoints
- Authentication methods
- Request/response formats

How do we add new providers without modifying existing code?

### The Solution

The **Strategy Pattern** via Rust traits. Define a trait for common behavior, implement it for each provider, and store implementations in a registry.

From `whis-core/src/provider/mod.rs:151-176`:

```rust
#[async_trait]
pub trait TranscriptionBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn display_name(&self) -> &'static str;

    fn transcribe_sync(
        &self,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult>;

    async fn transcribe_async(
        &self,
        client: &reqwest::Client,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult>;
}
```

**From `whis-core/src/provider/mod.rs:151-176`**

Each provider implements this trait. For example, `OpenAIProvider`:

```rust
pub struct OpenAIProvider;

#[async_trait]
impl TranscriptionBackend for OpenAIProvider {
    fn name(&self) -> &'static str { "openai" }
    fn display_name(&self) -> &'static str { "OpenAI Whisper" }

    fn transcribe_sync(&self, api_key: &str, request: TranscriptionRequest) 
        -> Result<TranscriptionResult> 
    {
        openai_compatible_transcribe_sync(
            "https://api.openai.com/v1/audio/transcriptions",
            "whisper-1",
            api_key,
            request,
        )
    }

    async fn transcribe_async(/* ... */) -> Result<TranscriptionResult> {
        // Async version
    }
}
```

**From `whis-core/src/provider/openai.rs:8-40` (simplified)**

### Registry Pattern

A registry holds all providers as trait objects:

```rust
pub struct ProviderRegistry {
    providers: HashMap<&'static str, Arc<dyn TranscriptionBackend>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut providers = HashMap::new();
        providers.insert("openai", Arc::new(OpenAIProvider));
        providers.insert("groq", Arc::new(GroqProvider));
        providers.insert("deepgram", Arc::new(DeepgramProvider));
        // ...
        Self { providers }
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn TranscriptionBackend>> {
        self.providers.get(name).cloned()
    }
}
```

**From `whis-core/src/provider/mod.rs:179-200`**

Usage:

```rust
let provider = registry().get("openai").unwrap();
let result = provider.transcribe_sync(api_key, request)?;
```

### Why Arc&lt;dyn Trait&gt;?

- `dyn TranscriptionBackend` is a trait object (dynamic dispatch)
- `Arc` allows cloning the reference without cloning the provider
- Enables registry to return clones cheaply

### Adding a New Provider

To add a new provider:

1. Create `new_provider.rs` with a struct
2. Implement `TranscriptionBackend` trait
3. Register in `ProviderRegistry::new()`

No changes to existing code. The registry pattern makes Whis extensible without modification.

### When to Use

Use trait objects with registry when:
- You have multiple implementations of the same interface
- Implementations are known at runtime (user-selected)
- You want to avoid monomorphization cost (trait generics create code for each type)

**Don't use** if you need compile-time dispatch for performance-critical paths.

## Pattern 5: State Machine for Recording Lifecycle

### The Problem

Recording has distinct states: Idle → Recording → Transcribing → Idle. Each state allows different actions:
- **Idle**: Can start recording
- **Recording**: Can stop recording
- **Transcribing**: No action allowed (wait for completion)

How do we prevent invalid transitions (e.g., starting recording while transcribing)?

### The Solution

A simple **state machine** with an enum and state checks:

From `whis-desktop/src/state.rs:6-11`:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecordingState {
    Idle,
    Recording,
    Transcribing,
}
```

**From `whis-desktop/src/state.rs:6-11`**

State is stored in `Mutex` within `AppState`:

```rust
pub struct AppState {
    pub state: Mutex<RecordingState>,
    pub recorder: Mutex<Option<AudioRecorder>>,
    pub transcription_config: Mutex<Option<TranscriptionConfig>>,
    // ...
}
```

**From `whis-desktop/src/state.rs:20-32`**

State transitions happen in the tray menu handler. Simplified version:

```rust
async fn toggle_recording(app_handle: AppHandle) {
    let state = app_handle.state::<AppState>();
    let current = *state.state.lock().unwrap();

    match current {
        RecordingState::Idle => {
            // Start recording
            *state.state.lock().unwrap() = RecordingState::Recording;
            start_audio_recording(state);
        }
        RecordingState::Recording => {
            // Stop recording, start transcription
            *state.state.lock().unwrap() = RecordingState::Transcribing;
            stop_and_transcribe(state).await;
            *state.state.lock().unwrap() = RecordingState::Idle;
        }
        RecordingState::Transcribing => {
            // Do nothing - already transcribing
        }
    }
}
```

The state enum prevents invalid transitions. You can't start recording while transcribing—the match arm does nothing.

### UI Synchronization

The frontend polls `/get_status` command:

```rust
#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> StatusResponse {
    let recording_state = *state.state.lock().unwrap();
    let config_valid = state.transcription_config.lock().unwrap().is_some();
    
    StatusResponse {
        state: match recording_state {
            RecordingState::Idle => "Idle",
            RecordingState::Recording => "Recording",
            RecordingState::Transcribing => "Transcribing",
        },
        config_valid,
    }
}
```

Vue polls every 500ms and updates button state accordingly.

### When to Use

Use state machine pattern when:
- Your system has distinct phases with different valid actions
- Transitions follow predictable rules
- Invalid states should be impossible (enforce with types)

For complex state machines, consider the `state_machine_future` crate or explicit state transition functions.

## Pattern 6: Command Pattern in Tauri

### The Problem

The frontend needs to invoke Rust functions. Tauri provides IPC bridge, but we need:
- Type safety between TypeScript and Rust
- Dependency injection (access to app state)
- Error handling across language boundary

### The Solution

The **Command Pattern** via `#[tauri::command]` macro:

```rust
#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<SaveSettingsResponse, String> {
    // Implementation
}
```

**From `whis-desktop/src/commands.rs:95`**

The macro generates:
- Serialization/deserialization code for parameters and return types
- Parameter injection logic for `AppHandle`, `State<T>`, etc.
- Error conversion (`Result<T, E>` where `E: Display` converts to string)

Frontend calls:

```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke<SaveSettingsResponse>('save_settings', {
  settings: { shortcut: 'Ctrl+R', provider: 'openai', /* ... */ }
});
```

### Dependency Injection

Special parameter types are auto-injected:
- `AppHandle`: Handle to the Tauri app instance
- `State<'_, T>`: Access to managed state
- `Window`: The window that invoked the command

You can mix injected and user-provided parameters:

```rust
#[tauri::command]
pub fn validate_api_key(
    api_key: String,        // From frontend
    provider: String,       // From frontend
    state: State<'_, AppState>,  // Injected
) -> Result<bool, String> {
    // Implementation
}
```

### Error Handling

Return `Result<T, String>` to send errors to frontend:

```rust
#[tauri::command]
pub async fn transcribe_audio(/* ... */) -> Result<String, String> {
    let result = do_transcription().await
        .map_err(|e| e.to_string())?;
    Ok(result)
}
```

If `Err` returned, `invoke()` throws in JavaScript with the string as error message.

### When to Use

Use Tauri commands for:
- Any frontend-initiated action (button clicks, form submissions)
- Querying state from Rust backend
- Async operations that need to run in Rust runtime

Keep commands thin—delegate to service layer functions for testability.

## Pattern Summary Table

| Pattern | Use Case | Key Types |
|---------|----------|-----------|
| `Arc<Mutex<T>>` | Shared mutable state across threads | `Arc`, `Mutex`, `MutexGuard` |
| `Result<T>` + context | Error handling with descriptive chains | `anyhow::Result`, `.context()` |
| Tokio + Semaphore | Concurrent I/O with rate limiting | `tokio::spawn`, `Semaphore` |
| Trait + Registry | Extensible provider system | `dyn Trait`, `Arc<dyn Trait>` |
| State Machine | Lifecycle with valid transitions | `enum`, `match` |
| Command Pattern | IPC with type safety | `#[tauri::command]` |

## When NOT to Use These Patterns

**Arc&lt;Mutex&lt;T&gt;&gt;**:
- High contention → Use channels (mpsc) or lock-free structures
- Read-heavy workload → Use `Arc<RwLock<T>>` (multiple readers, one writer)

**anyhow::Result**:
- Library code → Define specific error types for better API
- Need to match on error variants → Use enum errors

**Trait objects (dyn Trait)**:
- Performance-critical loops → Use generic `<T: Trait>` for monomorphization
- Trait has associated types → Can't be object-safe

**State machine with enum**:
- Many states (>10) → Consider state pattern with separate types
- Need history/rollback → Use event sourcing or command pattern

## Real-World Composition: Parallel Transcription

Let's see how these patterns combine in `parallel_transcribe`:

1. **Provider trait**: Get provider from registry
2. **Arc clones**: Share `client`, `api_key` across tasks
3. **Tokio spawn**: Create task for each chunk
4. **Semaphore**: Limit to 3 concurrent requests
5. **Result + context**: Propagate errors with `.context()`
6. **Async/await**: Suspend on I/O, resume when ready

```rust
pub async fn parallel_transcribe(/* ... */) -> Result<String> {
    // 1. Provider trait
    let provider_impl = registry().get_by_kind(provider);
    
    // 2. Arc clones for sharing
    let semaphore = Arc::new(Semaphore::new(3));
    let client = Arc::new(client);
    let api_key = Arc::new(api_key.to_string());
    
    let mut handles = Vec::new();
    
    for chunk in chunks {
        let semaphore = semaphore.clone();
        let client = client.clone();
        let api_key = api_key.clone();
        let provider_impl = provider_impl.clone();
        
        // 3. Tokio spawn
        let handle = tokio::spawn(async move {
            // 4. Semaphore
            let _permit = semaphore.acquire().await.unwrap();
            
            // 5. Result + context
            provider_impl.transcribe_async(&client, &api_key, request)
                .await
                .context("Failed to transcribe chunk")
        });
        
        handles.push(handle);
    }
    
    // 6. Await all
    let results = futures::future::join_all(handles).await;
    
    Ok(merge_chunks(results))
}
```

Each pattern solves a specific problem, and they compose naturally because Rust's type system enforces correctness.

## Summary

**Key Takeaways:**

1. **Arc&lt;Mutex&lt;T&gt;&gt;**: Share mutable state across threads safely
2. **Result + context**: Build descriptive error chains for debugging
3. **Tokio + Semaphore**: Control concurrent async operations
4. **Trait objects**: Extensible architecture without modifying existing code
5. **State machines**: Enforce valid state transitions at compile time
6. **Command pattern**: Type-safe IPC between frontend and backend

**Where These Patterns Appear:**

- Audio recording: `Arc<Mutex<Vec<f32>>>` for sample buffer
- Error handling: `.context()` on every I/O operation
- Parallel transcription: Tokio + Semaphore for rate limiting
- Provider system: Trait + Registry for extensibility
- Recording state: Enum state machine
- Tauri integration: Command pattern for all frontend calls

**Design Principles:**

- **Composition over inheritance**: Traits compose better than class hierarchies
- **Type safety**: Use enums and types to make invalid states unrepresentable
- **Explicit error handling**: No hidden exceptions, all errors visible in signatures
- **Zero-cost abstractions**: Patterns compile to efficient code

These patterns aren't unique to Whis—they're common in idiomatic Rust applications. Understanding them makes it easier to contribute to Whis and to recognize similar structures in other projects.

---

Next: [Chapter 25: Alternative Approaches](./ch25-alternatives.md)
