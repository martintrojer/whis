use clap::{Parser, Subcommand, ValueHint};

#[derive(Parser)]
#[command(name = "whis")]
#[command(version)]
#[command(about = "Voice-to-text CLI with multiple transcription providers")]
#[command(after_help = "Run 'whis' without arguments to record once (press Enter to stop).")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Polish transcript with LLM (cleanup grammar, filler words)
    #[arg(long)]
    pub polish: bool,

    /// Output preset for transcript (run 'whis presets' to see all)
    #[arg(long = "as", value_name = "PRESET")]
    pub preset: Option<String>,
}

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
        /// Set your OpenAI API key
        #[arg(long)]
        openai_api_key: Option<String>,

        /// Set your Mistral API key
        #[arg(long)]
        mistral_api_key: Option<String>,

        /// Set your Groq API key
        #[arg(long)]
        groq_api_key: Option<String>,

        /// Set your Deepgram API key
        #[arg(long)]
        deepgram_api_key: Option<String>,

        /// Set your ElevenLabs API key
        #[arg(long)]
        elevenlabs_api_key: Option<String>,

        /// Set the transcription provider (openai, mistral, groq, deepgram, elevenlabs)
        #[arg(long)]
        provider: Option<String>,

        /// Set the language hint (ISO-639-1 code: en, de, fr, etc.) or "auto" for auto-detect
        #[arg(long)]
        language: Option<String>,

        /// Set the polisher for transcript cleanup (none, openai, or mistral)
        #[arg(long)]
        polisher: Option<String>,

        /// Set custom polish prompt for transcript cleanup
        #[arg(long)]
        polish_prompt: Option<String>,

        /// Show current configuration
        #[arg(long)]
        show: bool,
    },

    /// Manage output presets
    Presets {
        #[command(subcommand)]
        action: Option<PresetsAction>,
    },
}

#[derive(Subcommand)]
pub enum PresetsAction {
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
}
