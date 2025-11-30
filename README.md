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
- **Cheap** — ~$0.006/minute via OpenAI Whisper API (no local GPU)
- **Simple** — record → transcribe → clipboard

## Quick Start

```bash
cargo install whis
whis config --api-key sk-your-key-here
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
whis config --api-key sk-...   # Save API key (persists to ~/.config/whis/)
whis config --show             # View current settings
```

## Installation

```bash
cargo install whis
```

Or download binaries from [GitHub Releases](https://github.com/frankdierolf/whis/releases).

## Requirements

- [OpenAI API key](https://platform.openai.com/api-keys)
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

## License

MIT
