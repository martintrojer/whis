# Chapter 25: Alternative Approaches

Every technical decision involves tradeoffs. Whis's current architecture works well for its use case, but other approaches could work too—with different benefits and costs.

In this chapter, we'll explore:
- Channels vs Arc&lt;Mutex&lt;T&gt;&gt; for audio data
- Local transcription vs cloud APIs
- Alternative Rust frameworks (Leptos, Yew, Dioxus)
- Self-hosting vs managed services

## Channels vs Arc&lt;Mutex&lt;T&gt;&gt;

### Current Approach: Arc&lt;Mutex&lt;T&gt;&gt;

Whis uses `Arc<Mutex<Vec<f32>>>` for sharing audio samples between the cpal callback thread and main thread:

```rust
pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    stream: Option<cpal::Stream>,
}
```

The callback locks the mutex, appends samples, and releases. The main thread locks, reads all samples, and releases.

**Benefits**:
- Simple: Both threads access same Vec
- Low latency: Direct memory access
- Works on all platforms cpal supports

**Drawbacks**:
- Mutex contention: Callback blocks if main thread holds lock
- Not ideal for real-time: Locking in audio callback is discouraged
- Memory growth: Vec keeps growing until recording stops

### Alternative: Channels (mpsc)

Use a bounded channel to send audio chunks from callback to main thread:

```rust
pub struct AudioRecorder {
    sender: Option<mpsc::SyncSender<Vec<f32>>>,
    receiver: mpsc::Receiver<Vec<f32>>,
    stream: Option<cpal::Stream>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::sync_channel(100); // Buffer 100 chunks
        Self {
            sender: Some(tx),
            receiver: rx,
            stream: None,
        }
    }

    fn build_stream(&self, /* ... */) -> Result<Stream> {
        let tx = self.sender.clone().unwrap();
        let stream = device.build_input_stream(
            config,
            move |data: &[f32], _| {
                // Send chunk, drop if channel full
                let _ = tx.try_send(data.to_vec());
            },
            /* ... */
        )?;
        Ok(stream)
    }

    pub fn stop_recording(&mut self) -> Vec<f32> {
        self.stream = None; // Stop stream
        self.sender = None; // Close channel
        
        // Drain all chunks
        let mut all_samples = Vec::new();
        while let Ok(chunk) = self.receiver.try_recv() {
            all_samples.extend_from_slice(&chunk);
        }
        all_samples
    }
}
```

**Benefits**:
- No locking in callback: `try_send()` is lock-free
- Backpressure handling: Drops samples if consumer too slow
- Better real-time behavior: Callback never blocks

**Drawbacks**:
- More complex: Need to manage channel lifetime
- Memory overhead: Each chunk is a separate Vec allocation
- Potential sample loss: Drops data if channel full

**Verdict**: Channels are better for production-quality audio apps. For Whis's use case (voice recording, not real-time processing), the simpler mutex approach is acceptable.

## Local Transcription vs Cloud APIs

### Current Approach: Cloud APIs

Whis sends audio to OpenAI/Groq/Deepgram APIs. No local processing.

**Benefits**:
- Zero setup: User just needs API key
- Always up-to-date: Providers improve models
- Multi-language: Supports 50+ languages out of the box
- Fast: Optimized GPU inference on provider servers

**Drawbacks**:
- Requires internet: Can't work offline
- Privacy: Audio leaves user's machine
- Cost: Pay per minute (though cheap: ~$0.006/min)
- Latency: Network round-trip adds delay

### Alternative: Local Whisper.cpp

Use [whisper.cpp](https://github.com/ggerganov/whisper.cpp) for on-device transcription:

```rust
use whisper_rs::{WhisperContext, FullParams};

pub struct LocalWhisperProvider {
    model_path: PathBuf,
}

impl TranscriptionBackend for LocalWhisperProvider {
    fn transcribe_sync(&self, _api_key: &str, request: TranscriptionRequest) 
        -> Result<TranscriptionResult> 
    {
        // Load model (heavy!)
        let ctx = WhisperContext::new(&self.model_path)?;
        
        // Decode audio to PCM
        let samples = decode_mp3_to_pcm(&request.audio_data)?;
        
        // Run inference
        let mut params = FullParams::new(whisper_rs::SamplingStrategy::Greedy);
        let mut state = ctx.create_state()?;
        state.full(params, &samples)?;
        
        // Get transcription
        let num_segments = state.full_n_segments()?;
        let mut text = String::new();
        for i in 0..num_segments {
            text.push_str(state.full_get_segment_text(i)?);
        }
        
        Ok(TranscriptionResult { text })
    }
}
```

**Benefits**:
- Works offline: No internet required
- Privacy: Audio never leaves machine
- No recurring cost: One-time model download
- Lower latency: No network round-trip

**Drawbacks**:
- Requires model download: 1.5GB for large-v3 model
- CPU/GPU requirements: Slow on older machines
- Limited languages: Best for English, decent for others
- Quality varies: Cloud models often better
- Binary size: Whisper.cpp adds ~50MB to app

**Hybrid Approach**: Whis could support both:
- Default to cloud for ease of use
- Add local transcription as opt-in for privacy-conscious users
- Use feature flags to include/exclude whisper.cpp

This is actually already partially implemented! Check `whis-core/src/provider/local_whisper.rs`.

## Alternative Rust Frameworks

### Current Approach: Tauri + Vue

Whis uses Tauri for the desktop app (Rust backend + web frontend). Vue 3.6 Vapor Mode for UI.

**Benefits**:
- Familiar web stack: HTML/CSS/JS for UI
- Huge ecosystem: npm packages, UI libraries
- Hot reload: Fast iteration during development
- Cross-platform: Same code on Linux/macOS/Windows
- Small binary: ~15MB for full app

**Drawbacks**:
- Two languages: Rust + TypeScript
- IPC overhead: Serialization cost for commands
- Web view dependency: Requires WebKit/WebView2
- Not native UI: Doesn't match platform look and feel

### Alternative 1: Leptos (Rust Full-Stack)

[Leptos](https://github.com/leptos-rs/leptos) is a full-stack Rust framework similar to SolidJS:

```rust
use leptos::*;

#[component]
fn App() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <div>
            <button on:click=move |_| set_count.update(|n| *n + 1)>
                "Count: " {count}
            </button>
        </div>
    }
}
```

For desktop apps, use `tauri-leptos` integration or compile to WebAssembly.

**Benefits**:
- One language: Pure Rust for backend and frontend
- Type safety: Shared types between UI and logic
- Fine-grained reactivity: Like Solid.js
- No JS bundle: WASM is smaller

**Drawbacks**:
- Smaller ecosystem: Fewer UI components
- Steeper learning curve: Different mental model
- WASM overhead: Still needs web view
- Less mature: Leptos is young

### Alternative 2: Native UI (iced, egui)

Use a native Rust UI framework:

```rust
use iced::{Application, Command, Element, Settings, Theme};

struct Whis {
    recording: bool,
}

impl Application for Whis {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ToggleRecording => {
                self.recording = !self.recording;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        button(if self.recording { "Stop" } else { "Start" })
            .on_press(Message::ToggleRecording)
            .into()
    }
}
```

**Benefits**:
- Pure Rust: No web view, no JavaScript
- True native: Can match platform UI
- Better performance: Direct rendering
- Smaller binary: No web view overhead

**Drawbacks**:
- Limited styling: CSS is more flexible
- Fewer components: No rich ecosystem like npm
- Platform differences: More platform-specific code
- Immediate mode UI: Different paradigm (egui)

**Verdict**: For Whis, Tauri+Vue is the right choice. The web ecosystem and iteration speed outweigh the benefits of pure Rust UI. For a simpler utility, native UI would be great.

## Self-Hosting vs Managed Services

### Current Approach: Managed Cloud APIs

Users provide their own API keys for OpenAI/Groq/Deepgram.

**Benefits**:
- No infrastructure: Developer doesn't run servers
- User pays directly: No subscription model needed
- Scalable: Cloud providers handle traffic
- Simple: Just API calls

**Drawbacks**:
- Setup friction: Users must create accounts, get keys
- Privacy concern: Audio goes to third parties
- No control: Subject to provider rate limits, pricing changes

### Alternative: Self-Hosted Inference Server

Run a Whisper server (e.g., [faster-whisper-server](https://github.com/fedirz/faster-whisper-server)):

```yaml
# docker-compose.yml
services:
  whisper:
    image: fedirz/faster-whisper-server:latest-cuda
    ports:
      - "8000:8000"
    volumes:
      - ./models:/app/models
    environment:
      - MODEL=large-v3
```

A future "Remote Whisper" provider could point to user's server using the same `TranscriptionBackend` trait:

```rust
// Example of what a remote whisper provider might look like
pub struct RemoteWhisperProvider {
    server_url: String,
}

impl TranscriptionBackend for RemoteWhisperProvider {
    fn transcribe_sync(&self, _api_key: &str, request: TranscriptionRequest) 
        -> Result<TranscriptionResult> 
    {
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(format!("{}/v1/audio/transcriptions", self.server_url))
            .multipart(/* ... */)
            .send()?;
        
        let result: TranscriptionResult = response.json()?;
        Ok(result)
    }
}
```

**Benefits**:
- Full control: Own infrastructure, no rate limits
- Privacy: Audio stays in user's network
- Cost: Free for personal use (just electricity/cloud compute)
- Customization: Can fine-tune models

**Drawbacks**:
- Complexity: Users must set up Docker/server
- Hardware requirements: GPU for fast inference
- Maintenance: Updates, monitoring, backups
- Upfront cost: GPU machine or cloud instance

**Hybrid Approach**: Whis could support:
- Cloud APIs for easy onboarding
- Self-hosted option for advanced users
- Local processing for offline/privacy needs

This gives users choice without forcing complexity on beginners.

## Other Architecture Decisions

### Monorepo vs Multi-repo

**Current**: Monorepo with workspace crates (`whis-core`, `whis-cli`, `whis-desktop`, `whis-mobile`)

**Alternative**: Separate repos for each app

**Verdict**: Monorepo is better for shared code. All apps use `whis-core`, so keeping them together ensures compatibility.

### SQLite for Settings vs JSON Files

**Current**: JSON files in `~/.config/whis/settings.json`

**Alternative**: SQLite database for settings, presets, history

**Benefits of SQLite**:
- Atomic writes: No corruption from crashes
- Query history: Search past transcriptions
- Complex data: Relationships between entities

**Drawbacks**:
- Overkill: Settings are tiny (~1KB)
- Migration complexity: Schema changes need migrations
- More dependencies: SQLite binary

**Verdict**: JSON is fine for current needs. If Whis adds transcription history or presets, SQLite becomes attractive.

### Static Binary vs Dynamic Linking

**Current**: Static linking (all dependencies in binary)

**Alternative**: Dynamic linking to system libraries

**Verdict**: Static is better for distribution. Users don't need to install dependencies. The extra ~5MB is worth the simplicity.

## Summary

**Key Takeaways**:

1. **Channels vs Arc&lt;Mutex&lt;T&gt;&gt;**: Channels better for real-time, mutex simpler for Whis's use case
2. **Local vs Cloud**: Cloud better for UX, local better for privacy—hybrid is ideal
3. **Framework choice**: Tauri+Vue best for web stack familiarity and ecosystem
4. **Self-hosting**: Optional advanced feature, don't force on all users

**Design Philosophy**:

- **Start simple**: JSON over SQLite, cloud over local, mutex over channels
- **Add complexity when needed**: When simple approach becomes limiting
- **User choice**: Don't force one approach—support multiple paths
- **Pragmatism over purity**: Web view is "not Rust" but gets the job done

**When to Reconsider**:

- If Whis becomes real-time (live transcription while speaking) → Use channels
- If privacy becomes primary concern → Add local transcription by default
- If UI performance is poor → Try native Rust UI
- If users want history/search → Add SQLite

The current architecture balances simplicity, user experience, and maintainability. Alternative approaches exist, but the tradeoffs currently favor the chosen design.

---

Next: [Chapter 26: Extending Whis](./ch26-extending.md)
