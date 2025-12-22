<div align="center">
<img src="https://raw.githubusercontent.com/frankdierolf/whis/main/crates/whis-desktop/icons/128x128.png" alt="whis" width="80" height="80" />

<h3>whis</h3>
<p>
  Your voice, piped to clipboard.
  <br />
  <a href="https://whis.ink">Website</a>
  ·
  <a href="https://github.com/frankdierolf/whis/tree/main/crates/whis-desktop">Desktop</a>
  ·
  <a href="https://github.com/frankdierolf/whis/tree/main/crates/whis-mobile">Mobile</a>
  ·
  <a href="https://github.com/frankdierolf/whis/releases">Releases</a>
</p>
</div>

## Introduction

The terminal-native voice-to-text tool. Record, transcribe, paste — all from your shell. Supports hotkey mode, presets, and pipes nicely with AI assistants.

## Quick Start

```bash
cargo install whis
whis setup cloud   # or: whis setup local
whis
```

## Usage

```bash
# Record once
whis                           # Press Enter to stop — text copied!

# Hotkey mode (background)
whis listen                    # ctrl+alt+w toggles recording
whis listen -k "super+space"   # Custom hotkey
whis stop                      # Stop background service
whis status                    # Check if running

# From file or stdin
whis -f audio.mp3
whis --stdin --format mp3      # Read from stdin (for piping)

# Output options
whis --print                   # Print to stdout instead of clipboard
whis -d 10                     # Record for 10 seconds (non-interactive)
whis -v                        # Verbose output

# Presets
whis --as email                # Use preset (auto-enables post-processing)
whis presets                   # List all
whis presets new xyz           # Create new preset
whis presets edit xyz          # Edit in $EDITOR

# Post-process with LLM
whis --post-process            # Clean up with Ollama

# Configuration
whis config                    # Show current settings
whis config provider openai    # Set provider
whis config language en        # Set language hint
whis models                    # List available models
```

## Environment Variables

API keys can be set via environment variables instead of `whis setup`:

```bash
OPENAI_API_KEY=sk-...
MISTRAL_API_KEY=...
GROQ_API_KEY=gsk_...
DEEPGRAM_API_KEY=...
ELEVENLABS_API_KEY=...
OLLAMA_URL=http://localhost:11434   # Default
OLLAMA_MODEL=qwen2.5:1.5b           # Default post-processing model
```

## Requirements

- API key from [OpenAI](https://platform.openai.com/api-keys), Mistral, Groq, Deepgram, or ElevenLabs — or use local Whisper (no API key needed)
- FFmpeg (`sudo apt install ffmpeg` or `brew install ffmpeg`)
- Linux (X11/Wayland), macOS, or Windows

**For hotkey mode** (one-time setup on Linux):
```bash
sudo usermod -aG input $USER
echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/99-uinput.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
# Logout and login again
```

## Prefer a GUI?

See [whis-desktop](https://github.com/frankdierolf/whis/tree/main/crates/whis-desktop) — same functionality, with system tray.

## License

MIT
