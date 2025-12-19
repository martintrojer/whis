# Chapter 16: Command-Line Interface with Clap

The `whis` CLI provides multiple commands: record once, run as daemon, configure settings, and manage presets. This chapter explores how Whis uses the `clap` crate to parse arguments with derive macros, subcommands, and automatic help generation.

## What is Clap?

[`clap`](https://github.com/clap-rs/clap) (Command Line Argument Parser) is Rust's most popular CLI library. It provides:

1. **Derive macros**: Define CLI structure with Rust structs
2. **Automatic help**: `-h` and `--help` generated automatically
3. **Type safety**: Arguments parsed into Rust types
4. **Subcommands**: Git-style commands (`git commit`, `git push`)
5. **Validation**: Required args, value constraints, custom validators

**Why clap?** Type-safe, zero-runtime-cost abstractions, excellent error messages.

## The CLI Structure

Whis CLI has this command hierarchy:

```
whis                          # Record once (default)
  -v, --verbose               # Enable debug output
  --polish                    # Polish transcript with LLM
  --as <preset>               # Apply output preset
  -f, --file <path>           # Transcribe from file
  --stdin                     # Read audio from stdin
  --format <fmt>              # Input format for stdin (default: mp3)

whis listen                   # Start daemon with hotkey listener
  --hotkey <KEY>              # Hotkey combination

whis stop                     # Stop daemon
whis status                   # Check daemon status

whis config                   # Configure settings
  --openai-api-key <KEY>
  --provider <NAME>
  --whisper-model-path <PATH> # Local whisper model
  --ollama-url <URL>          # Ollama server for polishing
  --ollama-model <NAME>       # Ollama model name
  --show                      # Display current config

whis presets                  # List presets
  list                        # List all
  show <name>                 # Show preset details
  new <name>                  # Create template
  edit <name>                 # Edit in $EDITOR

whis setup                    # Interactive setup wizard
  cloud                       # Configure cloud API provider
  local                       # Setup local whisper + Ollama

whis models                   # List available models
  whisper                     # Show whisper models with install status
  ollama                      # List Ollama models from server
```

## The Root CLI Struct

```rust
#[derive(Parser)]
#[command(name = "whis")]
#[command(version)]
#[command(about = "Voice-to-text CLI with multiple transcription providers")]
#[command(after_help = "Run 'whis' without arguments to record once (press Enter to stop).")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose output for debugging (audio device, clipboard, etc.)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Polish transcript with LLM (cleanup grammar, filler words)
    #[arg(long)]
    pub polish: bool,

    /// Output preset for transcript (run 'whis presets' to see all)
    #[arg(long = "as", value_name = "PRESET")]
    pub preset: Option<String>,

    /// Transcribe audio from file instead of recording
    #[arg(short = 'f', long, value_hint = ValueHint::FilePath)]
    pub file: Option<PathBuf>,

    /// Read audio from stdin (use with pipes, e.g., `yt-dlp ... | whis --stdin`)
    #[arg(long)]
    pub stdin: bool,

    /// Input audio format when using --stdin (default: mp3)
    #[arg(long, default_value = "mp3")]
    pub format: String,
}
```

**From `whis-cli/src/args.rs:4-36`**

### Derive Macros

**`#[derive(Parser)]`**: Implements `clap::Parser` trait
- Provides `parse()` method: `Cli::parse()`
- Reads `std::env::args()` automatically
- Returns `Cli` instance or exits with error/help

### Command Attributes

**`#[command(name = "whis")]`**: Binary name in help text

**`#[command(version)]`**: Adds `-V` and `--version` flags
- Version from `Cargo.toml` via `env!("CARGO_PKG_VERSION")`

**`#[command(about = "...")]`**: Short description shown in help

**`#[command(after_help = "...")]`**: Additional help text after options

**Example output**:
```
$ whis --help
Voice-to-text CLI with multiple transcription providers

Usage: whis [OPTIONS] [COMMAND]

Commands:
  listen   Start the background service that listens for hotkey triggers
  stop     Stop the background service
  status   Check service status
  config   Configure settings (API keys, provider, etc.)
  presets  Manage output presets
  setup    Quick setup wizard for different usage modes
  models   List available models (whisper, ollama)

Options:
  -v, --verbose         Enable verbose output for debugging
      --polish          Polish transcript with LLM (cleanup grammar, filler words)
      --as <PRESET>     Output preset for transcript (run 'whis presets' to see all)
  -f, --file <FILE>     Transcribe audio from file instead of recording
      --stdin           Read audio from stdin (use with pipes)
      --format <FORMAT> Input audio format when using --stdin [default: mp3]
  -h, --help            Print help
  -V, --version         Print version

Run 'whis' without arguments to record once (press Enter to stop).
```

### Optional Subcommand

```rust
#[command(subcommand)]
pub command: Option<Commands>,
```

**`Option<Commands>`**: Subcommand is optional
- `None`: Run default action (record once)
- `Some(cmd)`: Run specific subcommand

### Global Flags

```rust
#[arg(short, long, global = true)]
pub verbose: bool,
```

**`#[arg(short, long, global = true)]`**: Enables `-v` and `--verbose` flags
- `global = true`: Works with any subcommand (`whis -v config --show`)
- Useful for debugging audio device detection, clipboard operations, etc.

```rust
#[arg(long)]
pub polish: bool,
```

**`#[arg(long)]`**: Enables `--polish` flag
- `bool` type: presence = `true`, absence = `false`
- No value needed: `whis --polish` (not `--polish=true`)

```rust
#[arg(long = "as", value_name = "PRESET")]
pub preset: Option<String>,
```

**`long = "as"`**: Custom flag name (otherwise would be `--preset`)
- Usage: `whis --as markdown`
- `value_name = "PRESET"`: Shows `<PRESET>` in help

**Why `long = "as"`?**
More natural English: "output as markdown" vs "output preset markdown"

### File and Stdin Input

Instead of recording from the microphone, Whis can transcribe from existing audio:

```rust
#[arg(short = 'f', long, value_hint = ValueHint::FilePath)]
pub file: Option<PathBuf>,

#[arg(long)]
pub stdin: bool,

#[arg(long, default_value = "mp3")]
pub format: String,
```

**Usage examples**:
```bash
# Transcribe a local file
whis -f recording.mp3

# Transcribe from a URL (with yt-dlp)
yt-dlp -x --audio-format mp3 -o - "https://youtube.com/..." | whis --stdin

# Transcribe with custom format
ffmpeg -i video.mkv -f wav - | whis --stdin --format wav
```

**`value_hint = ValueHint::FilePath`**: Shell completion hint for file paths

## Subcommands Enum

```rust
#[derive(Subcommand)]
pub enum Commands {
    /// Start the background service that listens for hotkey triggers
    Listen {
        /// Hotkey to trigger recording (e.g., "ctrl+shift+r")
        #[arg(short = 'k', long, default_value = "ctrl+shift+r")]
        hotkey: String,
    },

    /// Stop the background service
    Stop,

    /// Check service status
    Status,

    /// Configure settings (API keys, provider, etc.)
    Config {
        #[arg(long)]
        openai_api_key: Option<String>,

        #[arg(long)]
        provider: Option<String>,

        #[arg(long)]
        show: bool,
        // ... more config options
    },

    /// Manage output presets
    Presets {
        #[command(subcommand)]
        action: Option<PresetsAction>,
    },

    /// Quick setup wizard for different usage modes
    Setup {
        #[command(subcommand)]
        mode: SetupMode,
    },

    /// List available models (whisper, ollama)
    Models {
        #[command(subcommand)]
        action: Option<ModelsAction>,
    },
}
```

**From `whis-cli/src/args.rs:38-130`**

### Subcommand Variants

**Unit variant** (no args):
```rust
Stop,
Status,
```

**Struct variant** (with args):
```rust
Listen {
    #[arg(short = 'k', long, default_value = "ctrl+shift+r")]
    hotkey: String,
},
```

**Nested subcommands**:
```rust
Presets {
    #[command(subcommand)]
    action: Option<PresetsAction>,
},
```

### Listen Command

```rust
Listen {
    #[arg(short = 'k', long, default_value = "ctrl+shift+r")]
    hotkey: String,
},
```

**Arguments**:
- **`short = 'k'`**: Enables `-k` short flag
- **`long`**: Enables `--hotkey` long flag
- **`default_value`**: Default if not provided

**Usage examples**:
```bash
whis listen                        # Uses default: ctrl+shift+r
whis listen -k ctrl+alt+r          # Short flag
whis listen --hotkey super+space   # Long flag
```

> **Key Insight**: Both short and long flags work simultaneously. User can choose their preference.

### Config Command

This has many optional arguments:

```rust
Config {
    #[arg(long)]
    openai_api_key: Option<String>,

    #[arg(long)]
    mistral_api_key: Option<String>,

    #[arg(long)]
    groq_api_key: Option<String>,
    
    // ... (8 more provider keys)

    #[arg(long)]
    provider: Option<String>,

    #[arg(long)]
    show: bool,
}
```

**From `whis-cli/src/args.rs:38-94`**

**All optional**: User can set one or multiple at once

**Examples**:
```bash
# Set one key
whis config --openai-api-key sk-proj-abc123

# Set multiple
whis config --openai-api-key sk-... --provider openai

# Show current config
whis config --show

# Set and show
whis config --groq-api-key gsk-... --show
```

## Nested Subcommands: Presets

```rust
Presets {
    #[command(subcommand)]
    action: Option<PresetsAction>,
},
```

**From `whis-cli/src/args.rs:97-100`**

**Second level enum**:

```rust
#[derive(Subcommand)]
pub enum PresetsAction {
    /// List all available presets (default)
    List,

    /// Show details of a specific preset
    Show {
        #[arg(value_hint = ValueHint::Other)]
        name: String,
    },

    /// Print a JSON template for creating a new preset
    New {
        #[arg(value_hint = ValueHint::Other)]
        name: String,
    },

    /// Edit a preset in your editor ($EDITOR or $VISUAL)
    Edit {
        #[arg(value_hint = ValueHint::Other)]
        name: String,
    },
}
```

**From `whis-cli/src/args.rs:103-128`**

**Usage**:
```bash
whis presets              # Defaults to 'list'
whis presets list         # Explicit list
whis presets show markdown
whis presets new my-preset
whis presets edit my-preset
```

**`value_hint = ValueHint::Other`**: Shell completion hint
- Tells shells this is a freeform value (not a file path)

## The Setup Command

The setup command provides interactive wizards for configuring Whis:

```rust
#[derive(Subcommand)]
pub enum SetupMode {
    /// Configure cloud transcription provider
    Cloud,

    /// Setup fully local transcription (whisper + Ollama)
    Local,
}
```

**From `whis-cli/src/args.rs`**

**Usage examples**:
```bash
# Cloud provider setup (interactive provider selection + API key)
whis setup cloud

# Local setup (downloads whisper model, configures Ollama)
whis setup local
```

The setup wizard handles:
- **Cloud**: Prompts for provider selection, displays API key URLs, validates key format
- **Local**: Downloads whisper model, starts Ollama, pulls polish model

For implementation details, see [Chapter 14b: Local Transcription](../part4-core-advanced/ch14b-local-transcription.md).

## The Models Command

The models command helps users manage Whisper models and check Ollama availability:

```rust
#[derive(Subcommand)]
pub enum ModelsAction {
    /// List available whisper models with install status (default)
    Whisper,

    /// List available Ollama models from server
    Ollama {
        /// Ollama server URL (default: http://localhost:11434)
        #[arg(long)]
        url: Option<String>,
    },
}
```

**From `whis-cli/src/args.rs:175-186`**

**Usage examples**:
```bash
# List whisper models and their install status
$ whis models whisper
Available Whisper models:
  ✓ tiny (75 MB)    - Installed
  ✓ base (142 MB)   - Installed
    small (466 MB)  - Not installed
    medium (1.5 GB) - Not installed

# List Ollama models
$ whis models ollama
Available Ollama models:
  - ministral-3:3b (default)
  - llama3.2:3b
  - mistral:latest

# Check Ollama on a remote server
$ whis models ollama --url http://my-server:11434
```

This command is useful for:
- Checking which Whisper models are downloaded
- Verifying Ollama is running and has models available
- Discovering model options before configuration

## Parsing Arguments in Main

```rust
fn main() -> Result<()> {
    let cli = args::Cli::parse();

    match cli.command {
        Some(args::Commands::Listen { hotkey }) => commands::listen::run(hotkey),
        Some(args::Commands::Stop) => commands::stop::run(),
        Some(args::Commands::Status) => commands::status::run(),
        Some(args::Commands::Config { ... }) => commands::config::run(...),
        Some(args::Commands::Presets { action }) => commands::presets::run(action),
        None => commands::record_once::run(cli.polish, cli.preset),
    }
}
```

**From `whis-cli/src/main.rs:11-52`**

**`Cli::parse()`**:
- Reads `std::env::args()`
- Parses according to `#[derive(Parser)]` rules
- Returns `Cli` struct on success
- Exits with error message on failure (invalid args)
- Exits with help text on `-h` or `--help`

**Pattern matching**:
- `Some(Commands::Listen { hotkey })`: Destructure args
- `Some(Commands::Stop)`: No args to destructure
- `None`: No subcommand → default action

## Auto-Generated Help

Clap generates help text from doc comments:

```rust
/// Start the background service that listens for hotkey triggers
Listen { ... },
```

**Becomes**:
```
Commands:
  listen   Start the background service that listens for hotkey triggers
```

**Field-level docs**:
```rust
/// Hotkey to trigger recording (e.g., "ctrl+shift+r")
#[arg(short = 'k', long, default_value = "ctrl+shift+r")]
hotkey: String,
```

**Becomes**:
```
Options:
  -k, --hotkey <HOTKEY>  Hotkey to trigger recording (e.g., "ctrl+shift+r")
                         [default: ctrl+shift+r]
```

## Error Handling

Clap provides great error messages:

**Missing required arg**:
```bash
$ whis presets show
error: the following required arguments were not provided:
  <NAME>

Usage: whis presets show <NAME>

For more information, try '--help'.
```

**Unknown flag**:
```bash
$ whis --unknown
error: unexpected argument '--unknown' found

Usage: whis [OPTIONS] [COMMAND]

For more information, try '--help'.
```

**Invalid subcommand**:
```bash
$ whis invalid
error: unrecognized subcommand 'invalid'

Usage: whis [OPTIONS] [COMMAND]

For more information, try '--help'.
```

## Validation: Ensuring FFmpeg

Before running commands, Whis checks dependencies:

```rust
pub fn ensure_ffmpeg_installed() -> Result<()> {
    if std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .is_err()
    {
        eprintln!("Error: FFmpeg is not installed or not in PATH.");
        eprintln!("\nwhis requires FFmpeg for audio compression.");
        eprintln!("Please install FFmpeg:");
        eprintln!("  - Ubuntu/Debian: sudo apt install ffmpeg");
        eprintln!("  - macOS: brew install ffmpeg");
        eprintln!("  - Windows: choco install ffmpeg or download from ffmpeg.org");
        eprintln!("  - Or visit: https://ffmpeg.org/download.html\n");
        std::process::exit(1);
    }
    Ok(())
}
```

**From `whis-cli/src/app.rs:12-28`**

**How it works**:
1. Try to run `ffmpeg -version`
2. If fails: Print helpful error with install instructions
3. Exit with code 1 (failure)

**Called early in command handlers**:
```rust
pub fn run(hotkey: String) -> Result<()> {
    ensure_ffmpeg_installed()?;
    // ... rest of command
}
```

This prevents confusing errors later ("file not found: ffmpeg").

## Loading Configuration

Most commands need settings (API key, provider):

```rust
pub fn load_transcription_config() -> Result<TranscriptionConfig> {
    let settings = Settings::load();
    let provider = settings.provider.clone();
    let language = settings.language.clone();

    let api_key = match &provider {
        TranscriptionProvider::LocalWhisper => {
            match settings.get_whisper_model_path() {
                Some(path) => path,
                None => {
                    eprintln!("Error: No whisper model path configured.");
                    eprintln!("\nSet the model path with:");
                    eprintln!("  whis config --whisper-model-path ~/.local/share/whis/models/ggml-small.bin\n");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            match settings.get_api_key_for(&provider) {
                Some(key) => key,
                None => {
                    eprintln!("Error: No {} API key configured.", provider.display_name());
                    eprintln!("  whis config --{}-api-key YOUR_KEY\n", provider.as_str());
                    std::process::exit(1);
                }
            }
        }
    };

    Ok(TranscriptionConfig {
        provider,
        api_key,
        language,
    })
}
```

**From `whis-cli/src/app.rs:30-91`**

**Pattern matching on provider type**:
- **LocalWhisper**: Needs model path, not API key
- **Others**: Need API key

**Helpful errors**: If config is missing, tell user exactly how to fix it.

## Stdin Interaction

For the default record-once command:

```rust
pub fn wait_for_enter() -> Result<()> {
    let mut input = String::new();
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    Ok(())
}
```

**From `whis-cli/src/app.rs:93-98`**

**Usage**:
```rust
println!("🎤 Recording... Press Enter to stop.");
wait_for_enter()?;
println!("⏹️  Stopped.");
```

**`flush()`**: Ensures "Recording..." prints before waiting for input.

## Real-World Command Examples

### Record Once (Default)

```bash
$ whis
🎤 Recording... Press Enter to stop.
[User speaks]
[Press Enter]
⏹️  Stopped.
🔄 Transcribing...
✅ This is the transcribed text.
📋 Copied to clipboard.
```

### Record with Polish

```bash
$ whis --polish
🎤 Recording...
[Record and transcribe]
✨ Polishing with OpenAI...
✅ This is the polished, grammatically correct text.
```

### Record with Preset

```bash
$ whis --as markdown
🎤 Recording...
[Record and transcribe]
✅ Output formatted as markdown:

# Transcription

This is the transcribed text formatted as markdown.
```

### Start Daemon

```bash
$ whis listen
🎧 Listening for hotkey: ctrl+shift+r
Press ctrl+shift+r to record. Press Ctrl+C to stop service.
```

### Configure API Key

```bash
$ whis config --openai-api-key sk-proj-abc123
✅ OpenAI API key saved.

$ whis config --provider openai
✅ Provider set to: OpenAI Whisper

$ whis config --show
Configuration:
  Provider: OpenAI Whisper
  Language: Auto-detect
  Polisher: None
  API Keys: OpenAI (configured), Groq (not set)
```

### Manage Presets

```bash
$ whis presets
Available presets:
  - markdown: Format as markdown document
  - email: Professional email format
  - code-comment: Code comment format

$ whis presets show markdown
Preset: markdown
Description: Format as markdown document
Template:
  # Transcription
  
  {{ text }}

$ whis presets edit my-preset
[Opens $EDITOR with preset template]
```

## Clap Features Not Used (Yet)

Clap has many more features Whis could use:

### Value Validation

```rust
#[arg(long, value_parser = clap::value_parser!(u16).range(1..=65535))]
port: u16,
```

Validates port is 1-65535.

### Custom Value Parser

```rust
fn parse_hex_color(s: &str) -> Result<Color, String> {
    if s.starts_with('#') && s.len() == 7 {
        Ok(Color::from_hex(s))
    } else {
        Err(format!("Invalid hex color: {}", s))
    }
}

#[arg(long, value_parser = parse_hex_color)]
color: Color,
```

### Conflicting Args

```rust
#[arg(long, conflicts_with = "quiet")]
verbose: bool,

#[arg(long, conflicts_with = "verbose")]
quiet: bool,
```

Can't use both `--verbose` and `--quiet`.

### Required If

```rust
#[arg(long, required_if_eq("provider", "openai"))]
openai_api_key: Option<String>,
```

Require API key only if provider is OpenAI.

## Testing CLI Parsing

You can test clap parsing without running commands:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_listen_default() {
        let cli = Cli::parse_from(["whis", "listen"]);
        match cli.command {
            Some(Commands::Listen { hotkey }) => {
                assert_eq!(hotkey, "ctrl+shift+r");
            }
            _ => panic!("Expected Listen command"),
        }
    }

    #[test]
    fn test_parse_listen_custom_hotkey() {
        let cli = Cli::parse_from(["whis", "listen", "--hotkey", "super+space"]);
        match cli.command {
            Some(Commands::Listen { hotkey }) => {
                assert_eq!(hotkey, "super+space");
            }
            _ => panic!("Expected Listen command"),
        }
    }

    #[test]
    fn test_parse_config_show() {
        let cli = Cli::parse_from(["whis", "config", "--show"]);
        match cli.command {
            Some(Commands::Config { show, .. }) => {
                assert!(show);
            }
            _ => panic!("Expected Config command"),
        }
    }
}
```

**`parse_from()`**: Parse from custom args (not `std::env::args()`)

## Summary

**Key Takeaways:**

1. **Clap derive macros**: Define CLI with Rust structs and enums
2. **Automatic help**: `-h`, `--help`, `--version` for free
3. **Type safety**: Args parsed into Rust types (String, bool, etc.)
4. **Subcommands**: Git-style multi-level commands
5. **Default values**: `#[arg(default_value)]` for optional args
6. **Validation**: Check FFmpeg, API keys before running commands
7. **Helpful errors**: Guide users to fix configuration issues

**Where This Matters in Whis:**

- CLI entry point (`whis-cli/src/main.rs`)
- Argument definitions (`whis-cli/src/args.rs`)
- Config validation (`whis-cli/src/app.rs`)
- Command implementations (`whis-cli/src/commands/*`)

**Patterns Used:**

- **Derive macros**: Declarative CLI definition
- **Pattern matching**: Route to command handlers
- **Early validation**: Check dependencies before running
- **Helpful errors**: Exit with actionable error messages

**Design Decisions:**

1. **Why optional subcommand?** Default action (record once) is most common
2. **Why `--as` not `--preset`?** More natural English phrasing
3. **Why nested presets?** Group related operations logically
4. **Why exit on missing config?** Better than cryptic API errors

---

Next: [Chapter 17: Global Hotkeys](./ch17-hotkeys.md)

This chapter covers platform-specific hotkey registration for Linux (X11, Wayland), macOS, and Windows.
