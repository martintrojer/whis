<div align="center">
<img src="./crates/whis-desktop/icons/128x128.png" alt="whis" width="80" height="80" />
</div>

<h3 align="center">whis</h3>
<p align="center">
  Your voice, piped to clipboard.
  <br />
  <a href="https://whis.ink">Website</a>
  ·
  <a href="./crates/whis-desktop/">Desktop</a>
  ·
  <a href="https://github.com/frankdierolf/whis/releases">Releases</a>
</p>

## Why?

- **Built for AI workflows** — speak your prompt, paste to Claude/Copilot
- **Cheap** — ~$0.006/minute via OpenAI Whisper or Mistral Voxtral (no local GPU)
- **Simple** — record → transcribe → clipboard
- **Multi-provider** — OpenAI, Mistral, Groq, Deepgram, ElevenLabs, or local Whisper

## Quick Start

```bash
cargo install whis
whis config --openai-api-key sk-your-key-here  # or --mistral-api-key
whis
```

## Screenshot

![whis Demo](demo.gif)

## Usage

**One-shot mode:**
```bash
whis    # Recording starts, press Enter to stop
```

**Hotkey mode (background service):**
```bash
whis listen                    # Global Ctrl+Shift+R anywhere
whis listen -k "super+space"   # Custom hotkey
whis status                    # Check if running
whis stop                      # Stop service
```

**Configuration:**
```bash
whis config --openai-api-key sk-...   # Save OpenAI API key
whis config --mistral-api-key ...     # Save Mistral API key
whis config --provider mistral        # Switch to Mistral Voxtral
whis config --language en             # Set language hint (ISO-639-1)
whis config --show                    # View current settings
```

**From file or stdin:**
```bash
whis -f recording.mp3              # Transcribe audio file
cat audio.mp3 | whis --stdin       # Read from stdin
```

**Presets:**
```bash
whis presets list     # Show available output presets
whis --as email       # Apply preset to output
```

## Installation

```bash
cargo install whis
```

Or download binaries from [GitHub Releases](https://github.com/frankdierolf/whis/releases).

## Requirements

- API key from [OpenAI](https://platform.openai.com/api-keys) or [Mistral](https://console.mistral.ai/api-keys)
- FFmpeg (`sudo apt install ffmpeg` or `brew install ffmpeg`)
- Linux (X11/Wayland), macOS, or Windows

**For hotkey mode** (one-time setup on Linux):
```bash
sudo usermod -aG input $USER
echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/99-uinput.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
# Logout and login again
```

## Desktop App

Looking for a GUI with system tray? See [whis-desktop](./crates/whis-desktop/).

## License

MIT
