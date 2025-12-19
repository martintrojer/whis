# Chapter 14: The Provider System

Whis supports seven transcription providers: OpenAI, Groq, Mistral, Deepgram, ElevenLabs, local Whisper, and remote Whisper. Each has different APIs, auth methods, and response formats. This chapter explores how Whis abstracts these differences using trait objects, a registry pattern, and `Arc<dyn Trait>` for polymorphism.

## The Problem: Multiple APIs

Each provider has unique characteristics:

| Provider | Auth | Request Format | Response Format | Speed |
|----------|------|----------------|-----------------|-------|
| OpenAI | Bearer token | Multipart form | `{"text": "..."}` | Slow (2-4× real-time) |
| Groq | Bearer token | Multipart form | `{"text": "..."}` | Fast (0.3× real-time) |
| Mistral | Bearer token | Multipart form | `{"text": "..."}` | Medium |
| Deepgram | Token header | Raw audio body | `{"results": {...}}` | Very fast |
| ElevenLabs | API key header | Multipart form | Custom format | Medium |
| Local Whisper | Model path | In-memory | In-memory | Varies (CPU) |
| Remote Whisper | Server URL | Multipart form | `{"text": "..."}` | Varies |

> **Note**: Local and remote whisper providers are covered in depth in [Chapter 14b: Local Transcription](./ch14b-local-transcription.md), including model management, audio resampling, Ollama integration, and Docker deployment.

**Without abstraction**, every call site would need:
```rust
match provider {
    OpenAI => call_openai(...),
    Groq => call_groq(...),
    Deepgram => call_deepgram(...),
    // ... etc
}
```

**With abstraction**, we get:
```rust
provider.transcribe(audio_data, api_key).await?
```

## The `TranscriptionBackend` Trait

This trait defines the interface all providers must implement:

```rust
#[async_trait]
pub trait TranscriptionBackend: Send + Sync {
    /// Unique identifier (e.g., "openai", "deepgram")
    fn name(&self) -> &'static str;

    /// Display name for UI (e.g., "OpenAI Whisper", "Deepgram Nova")
    fn display_name(&self) -> &'static str;

    /// Synchronous transcription (for simple single-file case)
    fn transcribe_sync(
        &self,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult>;

    /// Async transcription for parallel chunk processing
    async fn transcribe_async(
        &self,
        client: &reqwest::Client,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult>;
}
```

**From `whis-core/src/provider/mod.rs:156-178`**

**Key design decisions**:

### 1. `Send + Sync` Bounds

```rust
pub trait TranscriptionBackend: Send + Sync { ... }
```

**Why?** Providers are shared across threads:
- `Send`: Can be moved to async tasks (`tokio::spawn`)
- `Sync`: Can be shared via `Arc` (multiple tasks read simultaneously)

Without these, you'd get:
```
error[E0277]: `dyn TranscriptionBackend` cannot be sent between threads safely
```

### 2. Two Methods: Sync and Async

**`transcribe_sync()`**: For CLI where blocking is acceptable
```rust
let result = provider.transcribe_sync(api_key, request)?;
```

**`transcribe_async()`**: For parallel chunk transcription
```rust
let result = provider.transcribe_async(&client, api_key, request).await?;
```

> **Key Insight**: Having both avoids requiring `tokio` runtime for simple CLI use. The sync version blocks, the async version uses `.await`.

### 3. `#[async_trait]`

```rust
#[async_trait]
pub trait TranscriptionBackend: Send + Sync {
    async fn transcribe_async(...) -> Result<...>;
}
```

Rust doesn't support `async fn` in traits natively (yet). The `#[async_trait]` macro from the `async-trait` crate works around this by boxing the future:

**What it expands to** (simplified):
```rust
fn transcribe_async(...) -> Pin<Box<dyn Future<Output = Result<...>> + Send + 'async_trait>>
```

This enables dynamic dispatch with async methods.

## Request and Response Types

### `TranscriptionRequest`

```rust
#[derive(Clone)]
pub struct TranscriptionRequest {
    pub audio_data: Vec<u8>,      // MP3 bytes
    pub language: Option<String>,  // e.g., "en", "de"
    pub filename: String,          // e.g., "recording.mp3"
}
```

**From `whis-core/src/provider/mod.rs:36-41`**

**Why include `filename`?**  
Multipart form uploads require a filename. Some APIs use it for logging/debugging.

### `TranscriptionResult`

```rust
pub struct TranscriptionResult {
    pub text: String,
}
```

**From `whis-core/src/provider/mod.rs:44-46`**

Simple wrapper. Future versions might include:
- Confidence scores
- Word-level timestamps
- Speaker diarization

## Implementing a Provider: OpenAI

Let's see how `OpenAIProvider` implements the trait:

```rust
pub struct OpenAIProvider;

#[async_trait]
impl TranscriptionBackend for OpenAIProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn display_name(&self) -> &'static str {
        "OpenAI Whisper"
    }

    fn transcribe_sync(
        &self,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult> {
        openai_compatible_transcribe_sync(API_URL, MODEL, api_key, request)
    }

    async fn transcribe_async(
        &self,
        client: &reqwest::Client,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult> {
        openai_compatible_transcribe_async(client, API_URL, MODEL, api_key, request).await
    }
}
```

**From `whis-core/src/provider/openai.rs:15-44`**

**Observations**:

1. **Zero-sized type**: `OpenAIProvider` has no fields, just methods
2. **Delegates to helpers**: Shares code with Groq and Mistral (same API format)
3. **Constants**: API_URL and MODEL are module-level

**Constants**:
```rust
const API_URL: &str = "https://api.openai.com/v1/audio/transcriptions";
const MODEL: &str = "whisper-1";
```

**From `whis-core/src/provider/openai.rs:11-12`**

## Shared Helper: OpenAI-Compatible Providers

OpenAI, Groq, and Mistral all use the same API format. Whis shares implementation:

### Sync Version

```rust
pub(crate) fn openai_compatible_transcribe_sync(
    api_url: &str,
    model: &str,
    api_key: &str,
    request: TranscriptionRequest,
) -> Result<TranscriptionResult> {
    // 1. Create HTTP client with timeout
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .build()
        .context("Failed to create HTTP client")?;

    // 2. Build multipart form
    let mut form = reqwest::blocking::multipart::Form::new()
        .text("model", model.to_string())
        .part(
            "file",
            reqwest::blocking::multipart::Part::bytes(request.audio_data)
                .file_name(request.filename)
                .mime_str("audio/mpeg")?,
        );

    if let Some(lang) = request.language {
        form = form.text("language", lang);
    }

    // 3. Send POST request
    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {api_key}"))
        .multipart(form)
        .send()
        .context("Failed to send request")?;

    // 4. Check status
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("API error ({status}): {error_text}");
    }

    // 5. Parse response
    let text = response.text().context("Failed to get response text")?;
    let resp: OpenAICompatibleResponse =
        serde_json::from_str(&text).context("Failed to parse API response")?;

    Ok(TranscriptionResult { text: resp.text })
}
```

**From `whis-core/src/provider/mod.rs:57-101`**

**Step-by-step**:

### Step 1: HTTP Client with Timeout

```rust
let client = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
    .build()?;
```

**Timeout value**:
```rust
pub const DEFAULT_TIMEOUT_SECS: u64 = 300; // 5 minutes
```

**Why 5 minutes?**  
Large files (20 MB chunks) can take 2-3 minutes to transcribe. 5 minutes provides buffer.

### Step 2: Multipart Form

```rust
let mut form = reqwest::blocking::multipart::Form::new()
    .text("model", model)
    .part(
        "file",
        reqwest::blocking::multipart::Part::bytes(request.audio_data)
            .file_name(request.filename)
            .mime_str("audio/mpeg")?,
    );
```

**Multipart form structure** (HTTP):
```
POST /v1/audio/transcriptions HTTP/1.1
Content-Type: multipart/form-data; boundary=----Boundary1234

------Boundary1234
Content-Disposition: form-data; name="model"

whisper-1
------Boundary1234
Content-Disposition: form-data; name="file"; filename="audio.mp3"
Content-Type: audio/mpeg

<binary MP3 data>
------Boundary1234--
```

`reqwest` handles all the boundary generation and encoding.

### Step 3: Optional Language Hint

```rust
if let Some(lang) = request.language {
    form = form.text("language", lang);
}
```

If user specified language (e.g., `"en"` for English), add it to form. Otherwise, API auto-detects.

### Step 4: Authorization Header

```rust
.header("Authorization", format!("Bearer {api_key}"))
```

**OpenAI format**: `Bearer sk-proj-abc123...`

Groq and Mistral use the same format (why this helper works for all three).

### Step 5: Error Handling

```rust
if !response.status().is_success() {
    let status = response.status();
    let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
    anyhow::bail!("API error ({status}): {error_text}");
}
```

**Common error status codes**:
- `401`: Invalid API key
- `429`: Rate limit exceeded
- `413`: File too large
- `500`: Server error

Return error text from API (often includes helpful details).

### Step 6: JSON Parsing

```rust
#[derive(Deserialize)]
struct OpenAICompatibleResponse {
    text: String,
}

let resp: OpenAICompatibleResponse = serde_json::from_str(&text)?;
Ok(TranscriptionResult { text: resp.text })
```

**Example response JSON**:
```json
{
  "text": "This is the transcribed audio."
}
```

Serde deserializes directly into the struct.

## Different API: Deepgram

Deepgram uses a completely different API format:

### Request Differences

1. **Auth**: `Token` header (not `Bearer`)
2. **Body**: Raw audio bytes (not multipart)
3. **Options**: Query parameters (not form fields)

```rust
let mut url = reqwest::Url::parse(API_URL)?;
url.query_pairs_mut()
    .append_pair("model", MODEL)
    .append_pair("smart_format", "true");

if let Some(lang) = &request.language {
    url.query_pairs_mut().append_pair("language", lang);
}

let response = client
    .post(url)
    .header("Authorization", format!("Token {api_key}"))  // Not "Bearer"!
    .header("Content-Type", "audio/mpeg")
    .body(request.audio_data)  // Raw bytes, not multipart
    .send()
    .await?;
```

**From `whis-core/src/provider/deepgram.rs:112-128`**

**URL with query params**:
```
https://api.deepgram.com/v1/listen?model=nova-2&smart_format=true&language=en
```

### Response Differences

Deepgram returns nested JSON:

```json
{
  "results": {
    "channels": [
      {
        "alternatives": [
          {
            "transcript": "This is the transcribed audio."
          }
        ]
      }
    ]
  }
}
```

**Parsing**:

```rust
#[derive(Deserialize)]
struct Response {
    results: Results,
}

#[derive(Deserialize)]
struct Results {
    channels: Vec<Channel>,
}

#[derive(Deserialize)]
struct Channel {
    alternatives: Vec<Alternative>,
}

#[derive(Deserialize)]
struct Alternative {
    transcript: String,
}

let resp: Response = serde_json::from_str(&text)?;
let transcript = resp
    .results
    .channels
    .first()
    .and_then(|c| c.alternatives.first())
    .map(|a| a.transcript.clone())
    .unwrap_or_default();
```

**From `whis-core/src/provider/deepgram.rs:19-101`**

**Navigation**:
- Get first channel: `.first()`
- Get first alternative: `.and_then(|c| c.alternatives.first())`
- Extract transcript: `.map(|a| a.transcript.clone())`
- Default to empty string: `.unwrap_or_default()`

This safely handles missing fields.

## The Provider Registry

How do we dynamically select a provider at runtime?

### Registry Struct

```rust
pub struct ProviderRegistry {
    providers: HashMap<&'static str, Arc<dyn TranscriptionBackend>>,
}
```

**From `whis-core/src/provider/mod.rs:181-183`**

**Key type**: `Arc<dyn TranscriptionBackend>`

- **`dyn TranscriptionBackend`**: Trait object (runtime polymorphism)
- **`Arc`**: Shared ownership (can clone cheap references)
- **`&'static str`**: Provider name as key (e.g., `"openai"`)

### Building the Registry

```rust
impl ProviderRegistry {
    pub fn new() -> Self {
        let mut providers: HashMap<&'static str, Arc<dyn TranscriptionBackend>> = HashMap::new();

        providers.insert("openai", Arc::new(OpenAIProvider));
        providers.insert("mistral", Arc::new(MistralProvider));
        providers.insert("groq", Arc::new(GroqProvider));
        providers.insert("deepgram", Arc::new(DeepgramProvider));
        providers.insert("elevenlabs", Arc::new(ElevenLabsProvider));

        #[cfg(feature = "local-whisper")]
        providers.insert("local-whisper", Arc::new(LocalWhisperProvider));

        Self { providers }
    }
}
```

**From `whis-core/src/provider/mod.rs:186-201`**

**Type erasure in action**:

Each provider is a different concrete type:
- `OpenAIProvider` (0 bytes)
- `DeepgramProvider` (0 bytes)
- `LocalWhisperProvider` (might have state)

But they're all stored as `Arc<dyn TranscriptionBackend>` (same type).

**Feature flag**: Local Whisper only included if compiled with `local-whisper` feature (enabled by default).

For implementation details of LocalWhisperProvider, see [Chapter 14b: Local Transcription](./ch14b-local-transcription.md).

### Using the Registry

```rust
pub fn get(&self, name: &str) -> Option<Arc<dyn TranscriptionBackend>> {
    self.providers.get(name).cloned()
}

pub fn get_by_kind(&self, kind: &TranscriptionProvider) -> Arc<dyn TranscriptionBackend> {
    self.get(kind.as_str())
        .expect("All enum variants must have providers registered")
}
```

**From `whis-core/src/provider/mod.rs:204-217`**

**Usage**:

```rust
let registry = ProviderRegistry::new();

// By string name
let provider = registry.get("openai").unwrap();
provider.transcribe_sync(api_key, request)?;

// By enum
let provider = registry.get_by_kind(&TranscriptionProvider::Groq);
provider.transcribe_async(&client, api_key, request).await?;
```

**Arc clone is cheap**:
```rust
let p1 = registry.get("openai").unwrap(); // Arc<dyn ...>
let p2 = p1.clone();                       // Just increments ref count
```

Both `p1` and `p2` point to the same `OpenAIProvider` instance.

## Global Registry with `OnceLock`

Creating the registry is cheap but wasteful to do repeatedly. Whis uses a global:

```rust
pub fn registry() -> &'static ProviderRegistry {
    static REGISTRY: OnceLock<ProviderRegistry> = OnceLock::new();
    REGISTRY.get_or_init(ProviderRegistry::new)
}
```

**From `whis-core/src/provider/mod.rs:227-230`**

**`OnceLock`** (stable Rust 1.70+):
- Lazy initialization
- Thread-safe
- Initialized exactly once

**Usage**:
```rust
let provider = provider::registry().get("groq").unwrap();
```

No need to pass registry around—it's globally accessible.

## Dynamic Dispatch: How It Works

When you call:
```rust
let provider: Arc<dyn TranscriptionBackend> = registry.get("openai")?;
provider.transcribe_sync(api_key, request)?;
```

**What happens**:

1. **Trait object**: `dyn TranscriptionBackend` is a fat pointer:
   ```
   [data pointer, vtable pointer]
   ```

2. **Vtable**: Compiler generates a table of function pointers for `OpenAIProvider`:
   ```rust
   vtable_OpenAIProvider = {
       name: OpenAIProvider::name,
       display_name: OpenAIProvider::display_name,
       transcribe_sync: OpenAIProvider::transcribe_sync,
       transcribe_async: OpenAIProvider::transcribe_async,
   }
   ```

3. **Method call**: `provider.transcribe_sync(...)` becomes:
   ```rust
   (provider.vtable.transcribe_sync)(provider.data, api_key, request)
   ```

This is **runtime polymorphism** (vs compile-time generic monomorphization).

**Tradeoff**:
- Slower: Indirect function call (not inlined)
- Smaller binaries: One `transcribe_all_chunks` function works for all providers
- Flexibility: Can select provider at runtime based on user config

## Real-World Usage

### CLI: Blocking Transcription

```rust
// whis-cli/src/commands/transcribe.rs (simplified)
pub fn transcribe_file(provider_name: &str, file_path: &Path, api_key: &str) -> Result<String> {
    let registry = provider::registry();
    let provider = registry.get(provider_name)
        .context("Unknown provider")?;

    let audio_data = std::fs::read(file_path)?;
    let request = TranscriptionRequest {
        audio_data,
        language: None,
        filename: file_path.file_name().unwrap().to_str().unwrap().to_string(),
    };

    let result = provider.transcribe_sync(api_key, request)?;
    Ok(result.text)
}
```

Simple and synchronous—perfect for CLI.

### Desktop: Async Transcription

```rust
// whis-desktop/src/commands.rs (simplified)
#[tauri::command]
pub async fn transcribe(
    provider_kind: TranscriptionProvider,
    audio_data: Vec<u8>,
    api_key: String,
) -> Result<String, String> {
    let registry = provider::registry();
    let provider = registry.get_by_kind(&provider_kind);

    let client = reqwest::Client::new();
    let request = TranscriptionRequest {
        audio_data,
        language: None,
        filename: "recording.mp3".to_string(),
    };

    let result = provider
        .transcribe_async(&client, &api_key, request)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.text)
}
```

Async for non-blocking GUI.

## Why This Architecture?

### Extensibility

Adding a new provider requires:
1. Create `src/provider/newprovider.rs`
2. Implement `TranscriptionBackend`
3. Register in `ProviderRegistry::new()`

No changes needed to:
- CLI commands
- Desktop GUI
- Transcription pipeline
- Chunking logic

### Testability

Mock provider for tests:

```rust
struct MockProvider;

#[async_trait]
impl TranscriptionBackend for MockProvider {
    fn name(&self) -> &'static str { "mock" }
    fn display_name(&self) -> &'static str { "Mock" }
    
    fn transcribe_sync(&self, _: &str, request: TranscriptionRequest) -> Result<TranscriptionResult> {
        // Return canned response
        Ok(TranscriptionResult {
            text: "Mock transcription".to_string(),
        })
    }
    
    async fn transcribe_async(&self, ...) -> Result<TranscriptionResult> {
        Ok(TranscriptionResult {
            text: "Mock transcription".to_string(),
        })
    }
}
```

Use in tests without hitting real APIs.

### Flexibility

User switches providers by changing config:
```json
{
  "provider": "groq"
}
```

No recompilation, no code changes.

## Summary

**Key Takeaways:**

1. **Trait abstraction**: `TranscriptionBackend` unifies 7 providers
2. **Sync + async**: Two methods for blocking and non-blocking contexts
3. **Shared helpers**: OpenAI-compatible providers reuse code
4. **Registry pattern**: HashMap of `Arc<dyn Trait>` for runtime selection
5. **Global registry**: `OnceLock` for lazy, thread-safe initialization
6. **Dynamic dispatch**: Vtable indirection for runtime polymorphism

**Where This Matters in Whis:**

- Providers registered at startup (`whis-core/src/provider/mod.rs`)
- Selected based on user config (`Settings::provider`)
- Used in CLI (`whis-cli/src/commands/transcribe.rs`)
- Used in desktop GUI (`whis-desktop/src/commands.rs`)
- Used in parallel transcription (Chapter 15)

**Patterns Used:**

- **Trait objects**: `dyn TranscriptionBackend` for polymorphism
- **Registry pattern**: Centralized provider lookup
- **Zero-sized types**: Providers with no state (just methods)
- **Arc for sharing**: Cheap clones across threads
- **`#[async_trait]`**: Async methods in traits

**Design Decisions:**

1. **Why both sync and async?** CLI doesn't need tokio, GUI does
2. **Why Arc over Box?** Multiple tasks need shared access
3. **Why OnceLock?** Lazy init without lazy_static dependency
4. **Why helpers?** Avoid duplicating OpenAI-compatible logic 3×

---

Next: [Chapter 14b: Local Transcription](./ch14b-local-transcription.md)
