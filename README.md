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

<p align="center">
  <a href="https://crates.io/crates/whis"><img src="https://img.shields.io/crates/v/whis" alt="crates.io"></a>
  <a href="https://github.com/frankdierolf/whis/releases"><img src="https://img.shields.io/github/v/release/frankdierolf/whis" alt="GitHub release"></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT"></a>
</p>

## Why?

- **Built for AI workflows** — speak your prompt, paste to Claude/Copilot
- **Cheap** — ~$0.006/minute via cloud providers (no local GPU required)
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
whis listen                    # Global hotkey (Ctrl+Alt+W / Cmd+Option+W)
whis listen -k "super+space"   # Custom hotkey
whis status                    # Check if running
whis stop                      # Stop service
```

**Cross-platform hotkeys:**

| Platform | Default | Example Custom |
|----------|---------|----------------|
| Linux/Windows | `Ctrl+Alt+W` | `super+space`, `ctrl+shift+r` |
| macOS | `Cmd+Option+W` | `cmd+space`, `ctrl+shift+r` |

The shortcut format is consistent across platforms. On macOS:
- `ctrl` = Control (⌃)
- `alt` = Option (⌥)
- `super` or `cmd` = Command (⌘)

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

## Desktop App

Looking for a GUI with system tray? See [whis-desktop](./crates/whis-desktop/).

## Building from Source

```bash
git clone https://github.com/frankdierolf/whis.git
cd whis
cargo build --release -p whis
```

The binary will be at `target/release/whis`.

## Development

This project uses [just](https://github.com/casey/just) for task automation:

```bash
just              # List all commands
just build        # Build CLI
just desktop-dev  # Run desktop app in dev mode
just lint         # Run clippy
just ci           # Pre-commit check (fmt + lint)
```

See [CONTRIBUTING.md](./CONTRIBUTING.md) for full setup instructions.

## Documentation

- **[Book](./book/)** — Deep dive into the codebase architecture
- **[API Docs](https://docs.rs/whis-core)** — Rust API documentation

## For AI Agents

When exploring this codebase, start with:

```bash
just --list          # See all available commands
cat justfile         # Understand build/test/run workflow
```

Key facts:
- Rust workspace with 5 crates: `whis-core` (library), `whis-cli`, `whis-desktop`, `whis-mobile`, `whis-screenshots` (dev utility)
- Voice-to-text transcription using cloud providers (OpenAI, Mistral, Groq, Deepgram, ElevenLabs) or local Whisper
- Desktop app: Tauri + Vue. CLI: clap.
- Entry points: `crates/whis-cli/src/main.rs`, `crates/whis-desktop/src/main.rs`

## License

MIT
