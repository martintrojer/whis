<div align="center">
<img src="./icons/128x128.png" alt="whis" width="80" height="80" />
</div>

<h3 align="center">whis-desktop</h3>
<p align="center">
  Your voice, piped to clipboard. With a GUI.
  <br />
  <a href="https://whis.ink">Website</a>
  ·
  <a href="../../README.md">CLI</a>
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
flatpak install flathub ink.whis.Whis
```

## Screenshot

![Whis Desktop](screenshots/1-about.png)

## Features

- **System tray** — lives in your taskbar, out of the way
- **Global shortcut** — Ctrl+Shift+R by default (configurable)
- **Settings UI** — configure API key and shortcuts
- **X11 & Wayland** — works on both

![Settings](screenshots/4-settings.png)

## Installation

**Flatpak** (recommended):
```bash
flatpak install flathub ink.whis.Whis
```

**AppImage**:
```bash
wget https://github.com/frankdierolf/whis/releases/latest/download/Whis_amd64.AppImage
chmod +x Whis_amd64.AppImage
./Whis_amd64.AppImage
```

**Debian/Ubuntu**:
```bash
wget https://github.com/frankdierolf/whis/releases/latest/download/Whis_amd64.deb
sudo dpkg -i Whis_amd64.deb
```

## Requirements

- API key from [OpenAI](https://platform.openai.com/api-keys) or [Mistral](https://console.mistral.ai/api-keys)
- Linux (X11/Wayland)

## Prefer the terminal?

See [whis CLI](../../README.md) — same functionality, no GUI.

## License

MIT
