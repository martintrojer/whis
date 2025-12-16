# Chapter 26: Extending Whis

Whis is designed to be extensible. The provider trait pattern, feature flags, and modular architecture make it easy to add new capabilities without modifying existing code.

In this chapter, we'll walk through:
- Adding a new transcription provider
- Adding a new audio format encoder
- Adding presets for polish/summarization
- Plugin architecture ideas for the future

## Adding a New Transcription Provider

The most common extension: adding support for a new transcription API.

### Step 1: Create Provider Module

Create `whis-core/src/provider/my_provider.rs`:

```rust
use anyhow::Result;
use async_trait::async_trait;

use super::{
    TranscriptionBackend, TranscriptionRequest, TranscriptionResult,
};

const API_URL: &str = "https://api.myprovider.com/v1/transcribe";

pub struct MyProvider;

#[async_trait]
impl TranscriptionBackend for MyProvider {
    fn name(&self) -> &'static str {
        "myprovider"
    }

    fn display_name(&self) -> &'static str {
        "My Provider"
    }

    fn transcribe_sync(
        &self,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult> {
        // Implement sync version (blocking HTTP)
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(API_URL)
            .header("Authorization", format!("Bearer {api_key}"))
            .multipart(build_form(request)?)
            .send()?;
        
        parse_response(response)
    }

    async fn transcribe_async(
        &self,
        client: &reqwest::Client,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult> {
        // Implement async version (for parallel transcription)
        let response = client
            .post(API_URL)
            .header("Authorization", format!("Bearer {api_key}"))
            .multipart(build_form(request)?)
            .send()
            .await?;
        
        parse_response_async(response).await
    }
}

fn build_form(request: TranscriptionRequest) -> Result<reqwest::multipart::Form> {
    let form = reqwest::multipart::Form::new()
        .part(
            "audio",
            reqwest::multipart::Part::bytes(request.audio_data)
                .file_name(request.filename)
                .mime_str("audio/mpeg")?,
        );
    
    // Add language if provided
    if let Some(lang) = request.language {
        form = form.text("language", lang);
    }
    
    Ok(form)
}

fn parse_response(response: reqwest::blocking::Response) -> Result<TranscriptionResult> {
    #[derive(serde::Deserialize)]
    struct ApiResponse {
        text: String,
    }
    
    let resp: ApiResponse = response.json()?;
    Ok(TranscriptionResult { text: resp.text })
}
```

### Step 2: Register in mod.rs

Add to `whis-core/src/provider/mod.rs`:

```rust
mod my_provider;
pub use my_provider::MyProvider;

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut providers = HashMap::new();
        providers.insert("openai", Arc::new(OpenAIProvider));
        providers.insert("groq", Arc::new(GroqProvider));
        providers.insert("myprovider", Arc::new(MyProvider)); // Add here
        // ...
        Self { providers }
    }
}
```

### Step 3: Add to Config Enum

Update `whis-core/src/config.rs`:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TranscriptionProvider {
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "groq")]
    Groq,
    #[serde(rename = "myprovider")]
    MyProvider, // Add here
    // ...
}
```

### Step 4: Update UI

Add to `whis-desktop/ui/src/views/ApiKeyView.vue`:

```vue
<button
  class="provider-btn"
  :class="{ active: provider === 'myprovider' }"
  @click="emit('update:provider', 'myprovider')"
>
  My Provider
</button>

<!-- Add API key input -->
<div class="field">
  <label>My Provider API Key</label>
  <div class="api-key-input">
    <input
      :type="keyMasked.myprovider ? 'password' : 'text'"
      :value="getApiKey('myprovider')"
      @input="updateApiKey('myprovider', $event.target.value)"
      placeholder="mp-..."
    />
    <button @click="keyMasked.myprovider = !keyMasked.myprovider">
      {{ keyMasked.myprovider ? 'show' : 'hide' }}
    </button>
  </div>
  <p class="hint">
    Get your key from <a href="https://myprovider.com/keys">myprovider.com</a>
  </p>
</div>
```

### Step 5: Test

```bash
cd crates/whis-core
cargo test provider::my_provider

cd ../whis-cli
cargo run -- record --provider myprovider
```

That's it! The provider is now available in CLI, desktop, and mobile apps. No changes needed to the transcription logic—the registry handles dispatch.

## Adding a New Audio Format

Currently, Whis encodes to MP3. To add OGG Opus support:

### Step 1: Add Encoder Function

Create `whis-core/src/encode_opus.rs`:

```rust
use anyhow::{Context, Result};

pub fn encode_opus(
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
) -> Result<Vec<u8>> {
    // Use opus crate
    let mut encoder = opus::Encoder::new(
        sample_rate,
        opus::Channels::Mono,
        opus::Application::Voip,
    )?;
    
    // Convert f32 to i16
    let pcm: Vec<i16> = samples
        .iter()
        .map(|&s| (s * 32767.0) as i16)
        .collect();
    
    // Encode in frames (960 samples at 48kHz)
    let frame_size = 960;
    let mut output = Vec::new();
    
    for chunk in pcm.chunks(frame_size) {
        let mut encoded = vec![0u8; 4000];
        let len = encoder.encode(chunk, &mut encoded)?;
        output.extend_from_slice(&encoded[..len]);
    }
    
    Ok(output)
}
```

### Step 2: Add Feature Flag

In `Cargo.toml`:

```toml
[features]
default = ["mp3-lame"]
mp3-lame = ["dep:lame"]
opus = ["dep:opus"]

[dependencies]
lame = { version = "0.1", optional = true }
opus = { version = "0.3", optional = true }
```

### Step 3: Conditional Compilation

In `whis-core/src/audio.rs`:

```rust
pub fn encode_recording(&self, data: RecordingData) -> Result<Vec<u8>> {
    #[cfg(feature = "mp3-lame")]
    {
        encode_mp3_lame(&data.samples, data.sample_rate, data.channels)
    }
    
    #[cfg(feature = "opus")]
    {
        encode_opus(&data.samples, data.sample_rate, data.channels)
    }
}
```

### Step 4: Update Filename

Providers expect a filename with correct extension:

```rust
let filename = if cfg!(feature = "opus") {
    "audio.ogg"
} else {
    "audio.mp3"
};
```

Now users can build with Opus:

```bash
cargo build --no-default-features --features opus
```

## Adding Polish Presets

Whis has a `polish` module for post-processing transcripts. Let's add a "Summarize" preset:

### Step 1: Add Preset Enum

In `whis-core/src/preset.rs`:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    None,
    Punctuate,
    Paragraphs,
    Summarize, // New!
}

impl Preset {
    pub fn system_prompt(&self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Punctuate => Some(
                "Add proper punctuation and capitalization. \
                 Output only the corrected text, nothing else."
            ),
            Self::Paragraphs => Some(
                "Format into paragraphs with proper punctuation. \
                 Output only the formatted text."
            ),
            Self::Summarize => Some(
                "Summarize the following transcript in 2-3 sentences. \
                 Focus on key points and main ideas."
            ),
        }
    }
}
```

### Step 2: Use in Polish Function

The `polish_text` function already handles any preset:

```rust
pub async fn polish_text(
    text: &str,
    preset: Preset,
    provider: LLMProvider,
    api_key: &str,
) -> Result<String> {
    let Some(system_prompt) = preset.system_prompt() else {
        return Ok(text.to_string());
    };
    
    // Make API call with system_prompt...
}
```

### Step 3: Add to CLI

In `whis-cli/src/args.rs`:

```rust
#[derive(ValueEnum, Clone, Debug)]
pub enum PresetArg {
    None,
    Punctuate,
    Paragraphs,
    Summarize, // Add here
}
```

### Step 4: Add to UI

In settings view, add radio button or dropdown:

```vue
<select v-model="selectedPreset">
  <option value="none">None</option>
  <option value="punctuate">Add Punctuation</option>
  <option value="paragraphs">Format Paragraphs</option>
  <option value="summarize">Summarize</option>
</select>
```

Now users can run:

```bash
whis record --preset summarize
```

## Plugin Architecture Ideas

Whis doesn't currently support plugins, but here's how it could work:

### Dynamic Loading with libloading

```rust
use libloading::{Library, Symbol};

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn on_transcription_complete(&self, text: &str) -> Result<String>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn load_from_dir(&mut self, dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension() == Some(OsStr::new("so")) {
                self.load_plugin(&path)?;
            }
        }
        Ok(())
    }

    fn load_plugin(&mut self, path: &Path) -> Result<()> {
        unsafe {
            let lib = Library::new(path)?;
            let constructor: Symbol<fn() -> Box<dyn Plugin>> = 
                lib.get(b"create_plugin")?;
            let plugin = constructor();
            self.plugins.push(plugin);
        }
        Ok(())
    }

    pub fn run_plugins(&self, text: &str) -> Result<String> {
        let mut result = text.to_string();
        for plugin in &self.plugins {
            result = plugin.on_transcription_complete(&result)?;
        }
        Ok(result)
    }
}
```

### Plugin Example

A plugin in `plugins/emoji.so`:

```rust
// plugins/emoji/src/lib.rs
use whis_plugin_api::{Plugin, Result};

pub struct EmojiPlugin;

impl Plugin for EmojiPlugin {
    fn name(&self) -> &str {
        "emoji-injector"
    }

    fn on_transcription_complete(&self, text: &str) -> Result<String> {
        let result = text
            .replace("happy", "happy 😊")
            .replace("sad", "sad 😢");
        Ok(result)
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(EmojiPlugin)
}
```

### Challenges

- **ABI stability**: Rust doesn't have stable ABI, plugin must match exact compiler version
- **Safety**: Loading arbitrary code is dangerous
- **Cross-platform**: Dynamic libraries differ (`.so`, `.dylib`, `.dll`)

**Alternative**: WebAssembly plugins via `wasmtime`:

```rust
use wasmtime::*;

pub fn load_wasm_plugin(path: &Path) -> Result<Box<dyn Plugin>> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, path)?;
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[])?;
    
    // Call WASM function
    let process = instance.get_typed_func::<(u32, u32), u32>(&mut store, "process")?;
    // ...
}
```

WASM is safer (sandboxed) and cross-platform, but has more overhead.

## Contributing Guidelines

If you want to add a feature to Whis:

1. **Check existing issues**: Maybe someone's already working on it
2. **Open an issue first**: Discuss the design before coding
3. **Keep it focused**: One feature per PR
4. **Add tests**: At least smoke tests for new providers
5. **Update docs**: README and inline comments
6. **Follow existing style**: Run `cargo fmt` and `cargo clippy`

The codebase is structured to make extensions easy. Most features can be added without touching core logic.

## Summary

**Key Takeaways**:

1. **Provider trait**: Adding transcription providers takes ~50 lines of code
2. **Feature flags**: Use for optional dependencies (encoders, backends)
3. **Presets**: System prompts make it easy to add polish options
4. **Plugin system**: Possible via dynamic libraries or WASM (not implemented yet)

**Extension Points**:

- Transcription providers: Implement `TranscriptionBackend` trait
- Audio formats: Add encoder functions with feature flags
- Polish presets: Add enum variant and system prompt
- UI themes: Modify CSS custom properties in App.vue

**Best Practices**:

- Keep providers isolated (one file each)
- Use feature flags for optional dependencies
- Test on multiple platforms
- Document API key acquisition process
- Handle errors gracefully (no panics)

Whis is designed for extensibility. The registry pattern, trait objects, and modular architecture make it easy to add new providers, formats, and features without modifying existing code.

---

Next: [Chapter 27: Building for Platforms](../part9-operations/ch27-build.md)
