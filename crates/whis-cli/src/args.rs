use clap::{Args, Parser, Subcommand, ValueHint};
use std::path::PathBuf;
use std::time::Duration;

/// Parse a duration string like "10s", "30s", "1m", "90"
fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("duration cannot be empty".to_string());
    }

    // Check for suffix
    if let Some(num_str) = s.strip_suffix('s') {
        let secs: u64 = num_str
            .parse()
            .map_err(|_| format!("invalid number: {}", num_str))?;
        Ok(Duration::from_secs(secs))
    } else if let Some(num_str) = s.strip_suffix('m') {
        let mins: u64 = num_str
            .parse()
            .map_err(|_| format!("invalid number: {}", num_str))?;
        Ok(Duration::from_secs(mins * 60))
    } else {
        // No suffix, assume seconds
        let secs: u64 = s.parse().map_err(|_| format!("invalid duration: {}", s))?;
        Ok(Duration::from_secs(secs))
    }
}

/// Input source options for recording/transcription
#[derive(Args)]
pub struct InputSource {
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

/// Processing options for transcription
#[derive(Args)]
pub struct ProcessingOptions {
    /// Post-process transcript with LLM (cleanup grammar, filler words)
    #[arg(long)]
    pub post_process: bool,

    /// Output preset for transcript (run 'whis preset list' to see all)
    #[arg(long = "as", value_name = "PRESET")]
    pub preset: Option<String>,

    /// Record for a fixed duration (e.g., "10s", "30s", "1m")
    /// Useful for non-interactive environments like AI assistant shell modes
    #[arg(short = 'd', long, value_parser = parse_duration)]
    pub duration: Option<Duration>,

    /// Disable Voice Activity Detection (records all audio including silence)
    #[arg(long)]
    pub no_vad: bool,
}

/// Output options for transcription results
#[derive(Args)]
pub struct OutputOptions {
    /// Print output to stdout instead of copying to clipboard
    #[arg(long)]
    pub print: bool,

    /// Save recorded audio to WAV file (16kHz mono f32)
    #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
    pub save_raw: Option<PathBuf>,
}

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

    // Input source options (file, stdin, or microphone)
    #[command(flatten)]
    pub input: InputSource,

    // Processing options (post-processing, presets, duration, VAD)
    #[command(flatten)]
    pub processing: ProcessingOptions,

    // Output options (print, save raw audio)
    #[command(flatten)]
    pub output: OutputOptions,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    /// Start the background service that listens for hotkey triggers
    Start {
        /// Hotkey to trigger recording (e.g., "ctrl+alt+w" or "cmd+option+w" on macOS)
        #[arg(short = 'k', long, default_value = "ctrl+alt+w")]
        hotkey: String,
    },

    /// Stop the background service
    Stop,

    /// Restart the background service (preserves hotkey if not specified)
    Restart {
        /// Hotkey to trigger recording (if not specified, preserves current hotkey or uses default)
        #[arg(short = 'k', long)]
        hotkey: Option<String>,
    },

    /// Check service status
    Status,

    /// Toggle recording state (for compositor keybindings)
    Toggle,

    /// Interactive setup wizard
    Setup {
        #[command(subcommand)]
        mode: Option<SetupMode>,
    },

    /// Configure settings (git-style interface)
    Config {
        /// Configuration key to get or set
        key: Option<String>,

        /// Value to set (omit to get current value)
        value: Option<String>,

        /// List all configuration settings
        #[arg(long, conflicts_with_all = ["key", "value"])]
        list: bool,

        /// Show configuration file path
        #[arg(long, conflicts_with_all = ["key", "value", "list"])]
        path: bool,
    },

    /// Manage output presets
    Preset {
        #[command(subcommand)]
        action: Option<PresetAction>,
    },

    /// List available models (whisper, parakeet, ollama)
    Model {
        #[command(subcommand)]
        action: Option<ModelAction>,
    },
}

#[derive(Subcommand)]
pub enum SetupMode {
    /// Setup for cloud providers (OpenAI, Mistral, Groq, etc.)
    #[command(hide = true)]
    Cloud,

    /// Setup for fully local (on-device) transcription and post-processing
    #[command(hide = true)]
    Local,

    /// Configure post-processing (Ollama model selection, cleanup settings)
    #[command(hide = true)]
    PostProcessing,
}

#[derive(Subcommand)]
pub enum PresetAction {
    /// List all available presets (default)
    List,

    /// Show details of a specific preset
    Show {
        /// Name of the preset to show
        #[arg(value_hint = ValueHint::Other)]
        name: String,
    },

    /// Print a JSON template for creating a new preset
    New {
        /// Name for the new preset
        #[arg(value_hint = ValueHint::Other)]
        name: String,
    },

    /// Edit a preset in your editor ($EDITOR or $VISUAL)
    Edit {
        /// Name of the preset to edit (creates if doesn't exist)
        #[arg(value_hint = ValueHint::Other)]
        name: String,
    },

    /// Delete a user preset
    Delete {
        /// Name of the preset to delete
        #[arg(value_hint = ValueHint::Other)]
        name: String,
    },
}

#[derive(Subcommand)]
pub enum ModelAction {
    /// List available models
    List {
        #[command(subcommand)]
        model_type: Option<ModelType>,
    },
}

#[derive(Subcommand)]
pub enum ModelType {
    /// List available whisper models with install status (default)
    Whisper,

    /// List available Parakeet models with install status
    Parakeet,

    /// List available Ollama models from server
    Ollama {
        /// Ollama server URL (default: http://localhost:11434)
        #[arg(long)]
        url: Option<String>,
    },
}
