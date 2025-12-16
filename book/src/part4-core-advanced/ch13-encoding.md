# Chapter 13: Audio Encoding

Raw PCM audio is huge: one minute at 44.1kHz mono = ~10 MB. Transcription APIs won't accept files this large. This chapter dives deep into how Whis converts raw samples to compressed MP3, exploring both the FFmpeg path (desktop) and embedded LAME encoder (mobile), plus resampling for Whisper's 16kHz requirement.

## Why MP3?

Whis uses MP3 because:
1. **Universal support**: All transcription APIs accept MP3
2. **Good compression**: 10:1 to 15:1 ratio for speech
3. **Mature encoders**: LAME is 20+ years old, battle-tested
4. **Low complexity**: Simpler than Opus, Vorbis, AAC

**Compression example**:
- Raw PCM: 5 minutes × 44100 Hz × 2 bytes = **26.5 MB**
- MP3 (128 kbps): 5 minutes × 128 kbps = **4.7 MB**

That's an **82% reduction**.

## The Two Encoding Paths

Recall from Chapter 8 that Whis has two feature flags:

```toml
[features]
ffmpeg = ["hound"]                  # Desktop: FFmpeg process
embedded-encoder = ["mp3lame-encoder"]  # Mobile: LAME library
```

At compile time, exactly one encoder is enabled:

```rust
#[cfg(feature = "ffmpeg")]
fn samples_to_mp3(&self, samples: &[f32], suffix: &str) -> Result<Vec<u8>> {
    self.samples_to_mp3_ffmpeg(samples, suffix)
}

#[cfg(all(feature = "embedded-encoder", not(feature = "ffmpeg")))]
fn samples_to_mp3(&self, samples: &[f32], _suffix: &str) -> Result<Vec<u8>> {
    self.samples_to_mp3_embedded(samples)
}

#[cfg(not(any(feature = "ffmpeg", feature = "embedded-encoder")))]
fn samples_to_mp3(&self, _samples: &[f32], _suffix: &str) -> Result<Vec<u8>> {
    anyhow::bail!("No MP3 encoder available. Enable 'ffmpeg' or 'embedded-encoder'.")
}
```

**From `whis-core/src/audio.rs:198-213`**

This ensures:
- You can't compile without an encoder (compilation fails)
- You can't accidentally use both (embedded takes precedence if both enabled)
- The correct encoder is always available at runtime

## Path 1: FFmpeg (Desktop)

### Overview

The FFmpeg path:
1. Convert f32 samples → i16 samples
2. Write i16 samples to temp WAV file (`hound` crate)
3. Spawn FFmpeg process to convert WAV → MP3
4. Read MP3 file back into memory
5. Clean up temp files

### Step 1: Sample Format Conversion

```rust
let i16_samples: Vec<i16> = samples
    .iter()
    .map(|&s| {
        let clamped = s.clamp(-1.0, 1.0);
        (clamped * i16::MAX as f32) as i16
    })
    .collect();
```

**From `whis-core/src/audio.rs:219-225`**

**Why i16?**  
WAV files typically use integer formats. `i16` (16-bit signed) is the CD-quality standard.

**Conversion math**:
- `f32` range: [-1.0, 1.0]
- `i16` range: [-32768, 32767]
- Formula: `i16_value = f32_value × 32767`

**Example**:
```rust
let f32_sample = 0.5;
let i16_sample = (0.5 * 32767.0) as i16;  // = 16383
```

**Why clamp?**  
Audio processing can occasionally produce values slightly outside [-1.0, 1.0] due to floating-point errors. Clamping prevents overflow.

### Step 2: Unique Temp File Names

```rust
let temp_dir = std::env::temp_dir();
let unique_id = format!(
    "{}_{}_{suffix}",
    std::process::id(),
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos(),
);
let wav_path = temp_dir.join(format!("whis_{unique_id}.wav"));
let mp3_path = temp_dir.join(format!("whis_{unique_id}.mp3"));
```

**From `whis-core/src/audio.rs:228-238`**

**Uniqueness components**:
1. **Process ID**: `std::process::id()` - Prevents conflicts between multiple Whis instances
2. **Nanosecond timestamp**: `SystemTime::now().as_nanos()` - Prevents conflicts within same process
3. **Suffix**: `"chunk0"`, `"chunk1"`, etc. - For debugging (identifies which chunk)

**Example filename**:
```
/tmp/whis_12345_1640000000000000000_chunk0.wav
```

**Why temp files?**  
FFmpeg is designed to work with files, not stdin/stdout for audio. While you *can* pipe data to FFmpeg, it's unreliable for MP3 encoding.

### Step 3: Writing WAV with `hound`

```rust
{
    let spec = hound::WavSpec {
        channels: self.channels,       // 1 or 2
        sample_rate: self.sample_rate, // e.g., 44100
        bits_per_sample: 16,           // i16
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(&wav_path, spec)?;
    for sample in i16_samples {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;
}
```

**From `whis-core/src/audio.rs:241-253`**

**`hound::WavSpec` explained**:
- **`channels`**: Mono (1) or stereo (2)
- **`sample_rate`**: Samples per second (e.g., 44100)
- **`bits_per_sample`**: 16 bits = 2 bytes per sample
- **`sample_format`**: `Int` (signed integer) vs `Float` (f32)

**WAV file structure** (simplified):
```
[RIFF header]
[fmt chunk]  ← Contains spec (channels, rate, bits)
[data chunk] ← Raw samples: i16, i16, i16, ...
```

`hound` handles all the header writing automatically.

**Scoped block**:
```rust
{
    let mut writer = ...;
    writer.finalize()?;
} // writer drops here, file is fully written and closed
```

This ensures the file is flushed before FFmpeg tries to read it.

### Step 4: Spawning FFmpeg

```rust
let output = std::process::Command::new("ffmpeg")
    .args([
        "-hide_banner",
        "-loglevel", "error",
        "-i", wav_path.to_str().unwrap(),
        "-codec:a", "libmp3lame",
        "-b:a", "128k",
        "-y",
        mp3_path.to_str().unwrap(),
    ])
    .output()
    .context("Failed to execute ffmpeg. Make sure ffmpeg is installed.")?;
```

**From `whis-core/src/audio.rs:256-271`**

**FFmpeg arguments breakdown**:

| Argument | Purpose |
|----------|---------|
| `-hide_banner` | Don't show FFmpeg version info |
| `-loglevel error` | Only output errors (not warnings/info) |
| `-i <wav>` | Input file path |
| `-codec:a libmp3lame` | Use LAME MP3 encoder |
| `-b:a 128k` | Bitrate: 128 kilobits/second |
| `-y` | Overwrite output file if exists |
| `<mp3>` | Output file path |

**Why 128 kbps?**  
For speech:
- 96 kbps: Noticeable artifacts
- 128 kbps: Transparent (indistinguishable from source)
- 192 kbps: Overkill, larger files with no benefit

**Blocking vs async**:
- `Command::output()` blocks until FFmpeg finishes
- For 5 minutes of audio, this takes ~1-2 seconds
- Chapter 12 showed using `spawn_blocking` to avoid blocking the async runtime

### Step 5: Error Handling

```rust
// Clean up WAV immediately
let _ = std::fs::remove_file(&wav_path);

if !output.status.success() {
    let _ = std::fs::remove_file(&mp3_path);
    let stderr = String::from_utf8_lossy(&output.stderr);
    anyhow::bail!("FFmpeg conversion failed: {stderr}");
}
```

**From `whis-core/src/audio.rs:274-280`**

**Cleanup strategy**:
1. Delete WAV file immediately (don't need it anymore)
2. If FFmpeg failed, delete (possibly corrupt) MP3 file
3. Return error with FFmpeg's stderr output

**Why `let _ = ...`?**  
File deletion can fail (already deleted, permissions, etc.). We don't care—it's a temp file.

### Step 6: Reading MP3 and Final Cleanup

```rust
let mp3_data = std::fs::read(&mp3_path)
    .context("Failed to read converted MP3 file")?;

let _ = std::fs::remove_file(&mp3_path);

Ok(mp3_data)
```

**From `whis-core/src/audio.rs:283-288`**

Read entire MP3 file into `Vec<u8>`, delete temp file, return data.

**Memory consideration**:  
A 5-minute MP3 at 128 kbps = ~4.7 MB in RAM. This is acceptable for modern systems.

## Path 2: Embedded LAME (Mobile)

### Overview

The embedded path:
1. Convert f32 samples → i16 samples (same as FFmpeg path)
2. Configure LAME encoder
3. Encode directly to in-memory buffer
4. Flush encoder
5. Return MP3 data

No temp files, no process spawning.

### Step 1: Sample Conversion (Identical)

```rust
let i16_samples: Vec<i16> = samples
    .iter()
    .map(|&s| {
        let clamped = s.clamp(-1.0, 1.0);
        (clamped * i16::MAX as f32) as i16
    })
    .collect();
```

**From `whis-core/src/audio.rs:298-304`**

Same logic as FFmpeg path—LAME also wants i16 samples.

### Step 2: Configuring the Encoder

```rust
let mut builder = Builder::new()
    .context("Failed to create LAME builder")?;

builder
    .set_num_channels(self.channels as u8)
    .map_err(|e| anyhow::anyhow!("Failed to set channels: {:?}", e))?;

builder
    .set_sample_rate(self.sample_rate)
    .map_err(|e| anyhow::anyhow!("Failed to set sample rate: {:?}", e))?;

builder
    .set_brate(mp3lame_encoder::Bitrate::Kbps128)
    .map_err(|e| anyhow::anyhow!("Failed to set bitrate: {:?}", e))?;

builder
    .set_quality(mp3lame_encoder::Quality::Best)
    .map_err(|e| anyhow::anyhow!("Failed to set quality: {:?}", e))?;

let mut encoder = builder
    .build()
    .map_err(|e| anyhow::anyhow!("Failed to initialize LAME encoder: {:?}", e))?;
```

**From `whis-core/src/audio.rs:307-323`**

**Builder pattern**:
- `Builder::new()` creates configuration object
- Call setters: `set_num_channels()`, `set_sample_rate()`, etc.
- `build()` finalizes and returns encoder

**Quality settings**:
- `Bitrate::Kbps128`: Same as FFmpeg path
- `Quality::Best`: Highest quality encoding (slower, but mobile has CPU headroom)

### Step 3: Pre-allocating Output Buffer

```rust
let mut mp3_data = Vec::new();
let max_size = mp3lame_encoder::max_required_buffer_size(i16_samples.len());
mp3_data.reserve(max_size);
```

**From `whis-core/src/audio.rs:326-328`**

**Why pre-allocate?**  
MP3 encoding writes directly to a buffer. We need to allocate enough space upfront.

**`max_required_buffer_size()`** calculates worst-case size:
- MP3 frame overhead
- CBR (constant bitrate) padding
- Header/metadata

Typically ~1.25× the expected compressed size.

### Step 4: Encoding (Mono vs Stereo)

```rust
let encoded_size = if self.channels == 1 {
    let input = MonoPcm(&i16_samples);
    encoder
        .encode(input, mp3_data.spare_capacity_mut())
        .map_err(|e| anyhow::anyhow!("Failed to encode MP3: {:?}", e))?
} else {
    let input = InterleavedPcm(&i16_samples);
    encoder
        .encode(input, mp3_data.spare_capacity_mut())
        .map_err(|e| anyhow::anyhow!("Failed to encode MP3: {:?}", e))?
};
```

**From `whis-core/src/audio.rs:331-341`**

**Two input types**:

1. **`MonoPcm`**: Single-channel samples
   ```rust
   [s0, s1, s2, s3, ...]  // One sample per element
   ```

2. **`InterleavedPcm`**: Multi-channel samples interleaved
   ```rust
   [L0, R0, L1, R1, L2, R2, ...]  // Left, right, left, right
   ```

**`spare_capacity_mut()`**:  
Returns a mutable slice over the uninitialized capacity of the `Vec`. The encoder writes directly into this space without copying.

> **Key Insight**: Using `spare_capacity_mut()` is unsafe but efficient. We reserved exactly enough space, so writing won't overflow.

### Step 5: Updating Vec Length (Unsafe)

```rust
// SAFETY: encoder.encode returns the number of bytes written
unsafe {
    mp3_data.set_len(encoded_size);
}
```

**From `whis-core/src/audio.rs:344-346`**

**Why unsafe?**  
`Vec::set_len()` is unsafe because it changes the logical length without initializing memory. We're asserting that `encoder.encode()` wrote exactly `encoded_size` bytes to the buffer.

**This is safe because**:
1. We pre-allocated enough space
2. The encoder wrote exactly `encoded_size` bytes
3. Those bytes are now initialized

### Step 6: Flushing Remaining Data

```rust
let flush_size = encoder
    .flush::<FlushNoGap>(mp3_data.spare_capacity_mut())
    .map_err(|e| anyhow::anyhow!("Failed to flush MP3 encoder: {:?}", e))?;

// SAFETY: flush returns the number of bytes written
unsafe {
    mp3_data.set_len(mp3_data.len() + flush_size);
}

Ok(mp3_data)
```

**From `whis-core/src/audio.rs:349-358`**

**Why flush?**  
MP3 encoding processes samples in frames (typically 1152 samples). The encoder buffers partial frames. `flush()` forces encoding of remaining buffered samples.

**`FlushNoGap`**:  
Flushes without adding silence padding. This keeps the audio tight without extra blank space at the end.

## FFmpeg vs Embedded: Comparison

| Aspect | FFmpeg | Embedded LAME |
|--------|--------|---------------|
| **Platforms** | Desktop (Linux, macOS, Windows) | Mobile (Android, iOS) |
| **Dependencies** | Requires FFmpeg binary in PATH | Self-contained (library) |
| **Performance** | ~0.5-1 sec per minute | ~0.7-1.2 sec per minute |
| **Binary size** | +0 bytes (external tool) | +2 MB (embedded encoder) |
| **Memory** | Uses temp files (~2× audio size disk) | In-memory only |
| **Concurrency** | Can run multiple FFmpeg processes | One encoder instance per thread |
| **Quality** | Excellent (LAME 3.100+) | Excellent (same LAME) |

**When to use each**:
- **FFmpeg**: Desktop, FFmpeg already installed, want smaller binary
- **Embedded**: Mobile, no system dependencies allowed, need portability

## Resampling for Whisper

OpenAI's Whisper model expects **16 kHz mono audio**. If you capture at 44.1 kHz, you need to downsample.

### When Resampling Happens

Whis resamples **only for local Whisper** (`local-whisper` feature):

```rust
#[cfg(feature = "local-whisper")]
pub fn resample_to_16k(samples: &[f32], source_rate: u32, channels: u16) -> Result<Vec<f32>> {
    // ... implementation
}
```

**From `whis-core/src/resample.rs:20`**

**Why only local?**  
Cloud APIs (OpenAI, Groq, etc.) accept any sample rate and resample server-side. Only `whisper.cpp` strictly requires 16 kHz.

### Step 1: Stereo to Mono

```rust
let mono_samples = if channels > 1 {
    stereo_to_mono(samples, channels)
} else {
    samples.to_vec()
};
```

**From `whis-core/src/resample.rs:24-28`**

**Stereo to mono conversion** (simple averaging):

```rust
fn stereo_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    samples
        .chunks(channels as usize)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}
```

**From `whis-core/src/resample.rs:66-71`**

**Example**:
```
Stereo: [L=0.5, R=0.3, L=0.8, R=0.2]
        ↓ Average pairs
Mono:   [0.4, 0.5]
```

### Step 2: Check if Already 16 kHz

```rust
if source_rate == WHISPER_SAMPLE_RATE {
    return Ok(mono_samples);
}
```

**From `whis-core/src/resample.rs:31-33`**

Fast path: if already 16 kHz, just return mono samples.

### Step 3: Create Resampler

```rust
let mut resampler = FftFixedIn::<f32>::new(
    source_rate as usize,        // e.g., 44100
    WHISPER_SAMPLE_RATE as usize, // 16000
    1024,                         // chunk size
    2,                            // sub-chunks
    1,                            // channels (mono)
)
.context("Failed to create resampler")?;
```

**From `whis-core/src/resample.rs:36-43`**

**`rubato` crate**:  
High-quality FFT-based resampler. `FftFixedIn` means fixed input chunk size.

**Parameters**:
- **Input rate**: 44100 Hz
- **Output rate**: 16000 Hz
- **Chunk size**: 1024 samples (power of 2, efficient for FFT)
- **Sub-chunks**: 2 (internal buffering)
- **Channels**: 1 (mono)

### Step 4: Process in Chunks

```rust
let mut output = Vec::new();
let chunk_size = resampler.input_frames_max();

for chunk in mono_samples.chunks(chunk_size) {
    let mut padded = chunk.to_vec();
    if padded.len() < chunk_size {
        padded.resize(chunk_size, 0.0);  // Zero-pad last chunk
    }

    let result = resampler
        .process(&[padded], None)
        .context("Resampling failed")?;
    output.extend_from_slice(&result[0]);
}

Ok(output)
```

**From `whis-core/src/resample.rs:46-61`**

**Why chunks?**  
Resampler processes fixed-size blocks. We split the input into chunks and resample each.

**Zero-padding**:  
The last chunk might be smaller than `chunk_size`. Pad with zeros so the resampler has a full chunk.

**Output size**:
```
Output length = Input length × (16000 / 44100)
              = Input length × 0.3628
```

44.1 kHz → 16 kHz reduces data by ~64%.

## WAV Format Internals

Since Whis uses `hound` to write WAV files, let's briefly understand the format:

### RIFF Structure

WAV files use the RIFF (Resource Interchange File Format) container:

```
[0-3]   "RIFF"              // Magic bytes
[4-7]   File size - 8       // Total file size minus first 8 bytes
[8-11]  "WAVE"              // Format identifier

[12-15] "fmt "              // Format chunk identifier
[16-19] Chunk size (16)     // Format chunk size
[20-21] Audio format (1)    // 1 = PCM
[22-23] Channels            // 1 = mono, 2 = stereo
[24-27] Sample rate         // e.g., 44100
[28-31] Byte rate           // sample_rate × channels × bytes_per_sample
[32-33] Block align         // channels × bytes_per_sample
[34-35] Bits per sample     // e.g., 16

[36-39] "data"              // Data chunk identifier
[40-43] Data size           // Number of sample bytes
[44+]   Sample data         // Raw PCM samples: i16, i16, i16, ...
```

`hound` handles all of this automatically when you provide `WavSpec`.

## Performance Optimizations

### 1. Temp File Location

```rust
let temp_dir = std::env::temp_dir();
```

**Platform behavior**:
- **Linux**: `/tmp` (often tmpfs, RAM-backed)
- **macOS**: `/var/folders/...` (SSD)
- **Windows**: `C:\Users\<name>\AppData\Local\Temp` (disk)

On Linux with tmpfs, temp file I/O is nearly free (pure RAM operations).

### 2. Avoiding Copies

The embedded encoder writes directly to `spare_capacity_mut()`:
```rust
encoder.encode(input, mp3_data.spare_capacity_mut())
```

This avoids:
1. Allocating a new buffer
2. Copying encoded data
3. Reallocating `Vec` during append

### 3. Parallel Encoding (Future)

Currently, chunks are encoded sequentially:
```rust
for chunk in chunks {
    let mp3 = encode(chunk)?;  // Sequential
}
```

**Future optimization**:
```rust
let mp3s: Vec<_> = chunks.par_iter()
    .map(|chunk| encode(chunk))
    .collect()?;  // Parallel with rayon
```

This could halve encoding time for large recordings.

## Summary

**Key Takeaways:**

1. **Two encoders**: FFmpeg (desktop, external) vs LAME (mobile, embedded)
2. **Sample conversion**: f32 → i16 for both encoders
3. **FFmpeg path**: Write WAV → Spawn process → Read MP3 → Cleanup
4. **Embedded path**: Configure encoder → Encode to buffer → Flush
5. **Resampling**: Only for local Whisper (44.1 kHz → 16 kHz)
6. **Quality**: 128 kbps MP3 is transparent for speech

**Where This Matters in Whis:**

- Desktop builds use FFmpeg (`whis-desktop/Cargo.toml`)
- Mobile builds use embedded LAME (`whis-mobile/Cargo.toml`)
- Local Whisper users need resampling (`whis-core/src/resample.rs`)
- Parallel chunk encoding in Chapter 12's `finalize()`

**Patterns Used:**

- **Feature flags**: Conditional compilation for encoder selection
- **Builder pattern**: LAME encoder configuration
- **RAII**: Scoped blocks for automatic file handle cleanup
- **Unsafe**: Controlled use for zero-copy buffer manipulation

**Design Decisions:**

1. **Why MP3 over Opus?** Universal API support
2. **Why 128 kbps?** Transparent for speech, reasonable file size
3. **Why temp files?** FFmpeg design, reliable on all platforms
4. **Why rubato?** High-quality FFT resampling

---

Next: [Chapter 14: The Provider System](./ch14-providers.md)
