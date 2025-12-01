<div align="center">

<h3>whis-core</h3>
<p>
  Core library for whis voice-to-text functionality.
  <br />
  <a href="https://whis.ink">Website</a>
  ·
  <a href="../whis-cli/">CLI</a>
  ·
  <a href="../whis-desktop/">Desktop</a>
</p>
</div>

## Features

- **Audio recording** — capture microphone input via cpal
- **Transcription** — send audio to OpenAI Whisper API
- **Parallel processing** — split long recordings into chunks
- **Clipboard** — copy results to system clipboard
- **Config management** — persistent settings in `~/.config/whis/`

## Usage

```rust
use whis_core::{AudioRecorder, Config, transcribe_audio, copy_to_clipboard};

// Load config (includes API key)
let config = Config::load()?;

// Record audio
let recorder = AudioRecorder::new()?;
let audio = recorder.record_until_stopped()?;

// Transcribe
let text = transcribe_audio(&config.api_key, &audio).await?;

// Copy to clipboard
copy_to_clipboard(&text)?;
```

## Modules

| Module | Description |
|--------|-------------|
| `audio` | `AudioRecorder`, `AudioChunk`, recording utilities |
| `transcribe` | Whisper API integration, parallel chunked transcription |
| `clipboard` | System clipboard operations |
| `config` | API key and settings persistence |
| `settings` | User preferences (hotkeys, etc.) |

## License

MIT
