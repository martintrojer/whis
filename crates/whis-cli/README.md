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
  <a href="https://github.com/frankdierolf/whis/releases">Releases</a>
</p>
</div>

## Why?

- **Built for AI workflows** — speak your prompt, paste to Claude/Copilot
- **Cheap** — ~$0.006/minute via cloud providers (no local GPU required)
- **Simple** — record → transcribe → clipboard
- **Multi-provider** — OpenAI, Mistral, Groq, Deepgram, ElevenLabs, or local Whisper

## Quick Start

```bash
cargo install whis
whis setup cloud   # or: whis setup local
whis
```

## Usage

**One-shot mode:**
```bash
whis    # Recording starts, press Enter to stop
```

**From file or stdin:**
```bash
whis -f audio.mp3              # Transcribe audio file
whis --stdin < audio.mp3       # Read from stdin
yt-dlp -x ... | whis --stdin   # Pipe from other tools
```

**Hotkey mode (background service):**
```bash
whis listen                    # Global Ctrl+Alt+W anywhere
whis listen -k "super+space"   # Custom hotkey
whis status                    # Check if running
whis stop                      # Stop service
```

**Configuration:**
```bash
whis config --openai-api-key sk-...   # Save OpenAI API key
whis config --mistral-api-key ...     # Save Mistral API key
whis config --groq-api-key ...        # Save Groq API key
whis config --provider mistral        # Switch provider
whis config --language en             # Set language hint (ISO-639-1)
whis config --show                    # View current settings
```

**Presets:**
```bash
whis presets              # List all presets
whis presets show my-preset   # View preset details
whis --as my-preset       # Use preset for transcription
```

**LLM Post-Processing:**
```bash
whis --post-process       # Clean up transcription with Ollama
whis config --ollama-model llama3.2   # Set Ollama model
```

**Model Management:**
```bash
whis models whisper       # List local Whisper models
whis models ollama        # List Ollama models
```

## Requirements

- API key from [OpenAI](https://platform.openai.com/api-keys), [Mistral](https://console.mistral.ai/api-keys), [Groq](https://console.groq.com/keys), [Deepgram](https://deepgram.com), or [ElevenLabs](https://elevenlabs.io) — or use local Whisper (no API key needed)
- FFmpeg (`sudo apt install ffmpeg` or `brew install ffmpeg`)
- Linux (X11/Wayland) or macOS

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
