# Whisp - Voice to Text CLI

A simple command-line tool that records your voice and transcribes it using OpenAI's Whisper API, then copies the result to your clipboard.

## Features

- Press Enter to start/stop recording
- Automatic transcription using OpenAI Whisper
- Copies transcription directly to clipboard
- Simple and fast workflow

## Prerequisites

- Rust (latest stable version)
- OpenAI API key with access to Whisper API
- Linux with working audio input device
- ALSA or PulseAudio for audio recording

## Installation

1. Clone or download this repository
2. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```
3. Edit `.env` and add your OpenAI API key:
   ```
   OPENAI_API_KEY=sk-...your-key-here...
   ```
4. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

Run the program:
```bash
cargo run --release
```

Or use the compiled binary:
```bash
./target/release/whisp
```

### Workflow

1. Run the program
2. Press **Enter** to start recording
3. Speak into your microphone
4. Press **Enter** again to stop recording
5. Wait for transcription (a few seconds)
6. The transcribed text will be copied to your clipboard automatically!

## Configuration

Create a `.env` file in the project root with:
```
OPENAI_API_KEY=your-api-key-here
```

Get your API key from: https://platform.openai.com/api-keys

## Future Features

- Global keyboard shortcut support (planned)
- Configurable audio settings
- Multiple output formats
- Local Whisper model support

## Troubleshooting

### No audio input device
Make sure your microphone is connected and working. Test with:
```bash
arecord -l
```

### API key errors
- Verify your API key is correct in `.env`
- Check that your OpenAI account has API access enabled
- Ensure you have credits available

### Clipboard issues
Make sure you have a clipboard manager installed. On Linux, `xclip` or `wl-clipboard` (for Wayland) should be available.

## Dependencies

- `cpal` - Cross-platform audio I/O
- `hound` - WAV encoding
- `reqwest` - HTTP client for OpenAI API
- `arboard` - Clipboard access
- `tokio` - Async runtime
- `dotenv` - Environment variable management

## License

MIT
