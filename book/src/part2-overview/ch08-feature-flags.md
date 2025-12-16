# Chapter 8: Feature Flags and Conditional Compilation

Whis runs on desktops, mobile devices, and servers. Each platform has different capabilities: desktop has FFmpeg installed, mobile doesn't; servers don't have clipboards; some users want local transcription, others use APIs. 

How does one codebase handle all these variations without bloating every build with unused code?

**Answer: Cargo feature flags.**

This chapter explains how Whis uses `#[cfg(feature = "...")]` to compile different code for different platforms, keeping binaries small and dependencies minimal.

## What Are Feature Flags?

Feature flags let you conditionally include/exclude code and dependencies at **compile time**.

Think of them as `#ifdef` in C, but type-safe and integrated into Rust's module system.

**Example from `whis-core/Cargo.toml`**:

```toml
[features]
default = ["ffmpeg", "clipboard", "local-whisper"]
ffmpeg = ["hound"]
clipboard = ["arboard"]
embedded-encoder = ["mp3lame-encoder"]
local-whisper = ["whisper-rs", "rubato", "minimp3"]
```

**What this means**:
- By default, `whis-core` compiles with `ffmpeg`, `clipboard`, and `local-whisper` features
- Enabling `ffmpeg` adds the `hound` crate (WAV writer)
- Enabling `embedded-encoder` adds `mp3lame-encoder` (MP3 encoding library)
- Enabling `local-whisper` adds whisper.cpp bindings for offline transcription
- `ffmpeg` and `embedded-encoder` are **mutually exclusive in practice** (you use one or the other)

## How Features Affect Code

Let's look at a real example from `whis-core/src/encoding.rs`:

```rust
// Only include this module if the "ffmpeg" feature is enabled
#[cfg(feature = "ffmpeg")]
pub mod ffmpeg {
    use std::process::Command;
    use anyhow::{Context, Result};
    
    pub fn encode_to_mp3(wav_path: &str, mp3_path: &str) -> Result<()> {
        let output = Command::new("ffmpeg")
            .arg("-i").arg(wav_path)
            .arg("-codec:a").arg("libmp3lame")
            .arg("-b:a").arg("128k")
            .arg(mp3_path)
            .output()
            .context("Failed to spawn FFmpeg process")?;
        
        if !output.status.success() {
            anyhow::bail!("FFmpeg encoding failed");
        }
        Ok(())
    }
}

// Only include this module if the "embedded-encoder" feature is enabled
#[cfg(feature = "embedded-encoder")]
pub mod embedded {
    use mp3lame_encoder::{Encoder, Quality};
    use anyhow::Result;
    
    pub fn encode_to_mp3(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>> {
        let mut encoder = Encoder::new(
            sample_rate as u32,
            Quality::Best,
        )?;
        
        let mp3_data = encoder.encode(samples)?;
        Ok(mp3_data)
    }
}
```

**What happens during compilation**:

- **Desktop build** (`cargo build --features ffmpeg`):
  - `#[cfg(feature = "ffmpeg")]` code is compiled
  - `#[cfg(feature = "embedded-encoder")]` code is **completely removed**
  - `mp3lame-encoder` crate is **not downloaded or compiled**
  
- **Mobile build** (`cargo build --features embedded-encoder`):
  - `#[cfg(feature = "embedded-encoder")]` code is compiled
  - `#[cfg(feature = "ffmpeg")]` code is **completely removed**
  - No FFmpeg dependency, no shell commands

> **Key Insight**: This isn't runtime conditional logic (`if/else`). The unused code literally doesn't exist in the compiled binary. This reduces binary size and eliminates dependencies.

## Whis Feature Matrix

Here's every feature flag in `whis-core` and what it does:

### 1. `ffmpeg` (Desktop Default)

**Enables**: WAV file writing + FFmpeg process spawning

**Dependencies added**:
- `hound` - WAV file writer

**Code enabled**:
- `whis-core/src/encoding.rs` - `ffmpeg::encode_to_mp3()`

**Why**: Desktop systems have FFmpeg installed via package managers. It's faster and produces smaller MP3s than pure-Rust encoders.

**Used by**: `whis-cli`, `whis-desktop`

### 2. `embedded-encoder` (Mobile)

**Enables**: Pure-Rust MP3 encoding (no external process)

**Dependencies added**:
- `mp3lame-encoder` - Rust bindings to LAME MP3 encoder

**Code enabled**:
- `whis-core/src/encoding.rs` - `embedded::encode_to_mp3()`

**Why**: Mobile apps can't rely on system packages. Android doesn't ship FFmpeg. iOS is sandboxed. This embeds the encoder directly in the binary.

**Used by**: `whis-mobile`

**Tradeoff**: Larger binary (+2 MB) but no external dependencies.

### 3. `clipboard` (Desktop Default)

**Enables**: System clipboard integration

**Dependencies added**:
- `arboard` - Cross-platform clipboard library

**Code enabled**:
- `whis-core/src/clipboard.rs` - `copy_to_clipboard()`

**Why**: Desktop apps need to copy transcriptions to clipboard for paste workflows.

**Used by**: `whis-cli`, `whis-desktop`

**Why optional?** Headless servers and CI environments don't have clipboards. This prevents build failures in those environments.

### 4. `local-whisper` (Desktop Default)

**Enables**: Local Whisper model inference (no API required)

**Dependencies added**:
- `whisper-rs` - Rust bindings to `whisper.cpp`
- `rubato` - Sample rate converter
- `minimp3` - MP3 decoder

**Code enabled**:
- `whis-core/src/provider/local_whisper.rs` - Local Whisper provider
- `whis-core/src/model.rs` - Model download utilities
- `whis-core/src/resample.rs` - Audio resampling to 16kHz

**Why**: Privacy-conscious users or offline environments can run Whisper models locally.

**Tradeoff**: 
- Requires downloading ~500MB model files
- Slower than cloud APIs (CPU-only, no GPU acceleration)
- Larger binary (+45 MB)

**Used by**: `whis-cli`, `whis-desktop` (included in defaults)

> **Note**: Local LLM polishing uses Ollama via HTTP, which doesn't require a feature flag. See [Chapter 14b: Local Transcription](../part4-core-advanced/ch14b-local-transcription.md) for details on the Ollama integration.

## How Apps Choose Features

Each app crate specifies which features it needs in its `Cargo.toml`:

**`whis-desktop/Cargo.toml`**:
```toml
[dependencies.whis-core]
path = "../whis-core"
default-features = true  # ffmpeg + clipboard
features = []            # No additional features
```

**`whis-mobile/Cargo.toml`**:
```toml
[dependencies.whis-core]
path = "../whis-core"
default-features = false           # Disable ffmpeg + clipboard
features = ["embedded-encoder"]    # Use embedded MP3 encoder
```

**`whis-cli/Cargo.toml`** (with local Whisper):
```toml
[dependencies.whis-core]
path = "../whis-core"
default-features = true
features = ["local-whisper"]  # Add local transcription
```

## Conditional Compilation in Action

Let's trace what happens in `whis-core/src/lib.rs`:

```rust
pub mod audio;
pub mod transcribe;
pub mod settings;

#[cfg(feature = "clipboard")]
pub mod clipboard;

#[cfg(feature = "ffmpeg")]
pub mod encoding;

pub mod provider {
    pub mod openai;
    pub mod groq;
    pub mod deepgram;
    
    #[cfg(feature = "local-whisper")]
    pub mod local;
}
```

**Desktop build** (`whis-desktop`):
- `clipboard` module: ✅ Included
- `encoding` module: ✅ Included
- `provider::local`: ❌ Excluded

**Mobile build** (`whis-mobile`):
- `clipboard` module: ❌ Excluded
- `encoding` module: ❌ Excluded (uses different path)
- `provider::local`: ❌ Excluded

**CLI with local Whisper**:
- `clipboard` module: ✅ Included
- `encoding` module: ✅ Included
- `provider::local`: ✅ Included

## More Complex Conditionals

You can combine conditions:

```rust
// Only on Linux with clipboard feature
#[cfg(all(target_os = "linux", feature = "clipboard"))]
fn copy_to_x11_clipboard(text: &str) -> Result<()> {
    // X11-specific clipboard code
}

// On any Unix except mobile builds
#[cfg(all(unix, not(feature = "embedded-encoder")))]
fn find_ffmpeg() -> Option<PathBuf> {
    // Search PATH for ffmpeg binary
}

// Either feature works
#[cfg(any(feature = "ffmpeg", feature = "embedded-encoder"))]
pub fn encode_audio(samples: &[f32]) -> Result<Vec<u8>> {
    #[cfg(feature = "ffmpeg")]
    return ffmpeg::encode(samples);
    
    #[cfg(feature = "embedded-encoder")]
    return embedded::encode(samples);
}
```

## Why Not Just Use Runtime `if` Statements?

You might think: "Why not just check at runtime?"

```rust
// ❌ This approach wastes space
pub fn encode_audio(samples: &[f32]) -> Result<Vec<u8>> {
    if cfg!(mobile) {
        embedded::encode(samples)
    } else {
        ffmpeg::encode(samples)
    }
}
```

**Problems**:
1. Both `embedded::encode()` and `ffmpeg::encode()` get compiled into the binary
2. Both dependencies (`mp3lame-encoder` and `hound`) get included
3. Binary is larger than necessary
4. Unused code can't be optimized away

**With feature flags**:
1. Only the needed code path exists in the binary
2. Only the needed dependencies are compiled
3. Binary is smaller (sometimes 30-50% smaller)
4. Compiler can inline and optimize more aggressively

## Binary Size Impact

Here's the real-world impact on `whis-mobile`:

| Build | Features | Binary Size | Dependencies |
|-------|----------|-------------|--------------|
| Desktop | `ffmpeg, clipboard` | 8.2 MB | 127 crates |
| Mobile (full) | `embedded-encoder, local-whisper` | 24.1 MB | 142 crates |
| Mobile (minimal) | `embedded-encoder` | 12.3 MB | 98 crates |

**Takeaway**: Removing `local-whisper` cuts the mobile binary in half. Features have real costs.

## Checking Features at Runtime

Sometimes you need to know what features were enabled:

```rust
pub fn available_encoders() -> Vec<&'static str> {
    let mut encoders = vec![];
    
    #[cfg(feature = "ffmpeg")]
    encoders.push("ffmpeg");
    
    #[cfg(feature = "embedded-encoder")]
    encoders.push("embedded");
    
    encoders
}
```

This lets the app display "Available encoders: ffmpeg" in the settings UI.

## Summary

**Key Takeaways:**

1. **Feature flags** = compile-time conditional code inclusion
2. **`#[cfg(feature = "...")]`** removes unused code from the binary entirely
3. **Whis features**: `ffmpeg` (desktop), `embedded-encoder` (mobile), `clipboard` (desktop), `local-whisper` (desktop default)
4. **Why use them?** Smaller binaries, faster compilation, no unnecessary dependencies

**Where This Matters in Whis:**

- Mobile builds exclude clipboard and FFmpeg code
- Desktop builds exclude embedded encoder code
- Local Whisper is opt-in because it's a huge dependency
- Each platform gets exactly what it needs, nothing more

**Decision Framework:**

When adding new code to `whis-core`, ask:
1. Does every platform need this? → No feature flag needed
2. Is this platform-specific? → Use `#[cfg(target_os = "...")]`
3. Is this an optional dependency? → Create a new feature flag
4. Does this add >5 crates? → Definitely make it optional

**What's Next:**

Part II gave you the big picture: what Whis does, how crates are organized, and how features enable platform-specific code.

Part III dives into **implementation details**. You'll explore how audio recording actually works, how providers are implemented, and how the transcription pipeline handles chunking and parallelism.

---

Next: [Part III: Core Systems](../part3-core-systems/README.md)
