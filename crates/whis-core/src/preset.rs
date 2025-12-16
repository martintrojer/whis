use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

/// A preset for transcript polishing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    /// Unique identifier (derived from filename, not serialized)
    #[serde(skip)]
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// The system prompt for the LLM
    pub prompt: String,

    /// Optional: Override the polisher for this preset (openai, mistral)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polisher: Option<String>,

    /// Optional: Override the model for this preset
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Where a preset was loaded from
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetSource {
    BuiltIn,
    User,
}

impl std::fmt::Display for PresetSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresetSource::BuiltIn => write!(f, "built-in"),
            PresetSource::User => write!(f, "user"),
        }
    }
}

impl Preset {
    /// Get the presets directory (~/.config/whis/presets/)
    pub fn presets_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("whis")
            .join("presets")
    }

    /// Get all built-in presets
    pub fn builtins() -> Vec<Preset> {
        vec![
            Preset {
                name: "ai-prompt".to_string(),
                description: "Clean transcript for AI assistant prompts".to_string(),
                prompt: "Clean up this voice transcript for use as an AI assistant prompt. \
                    Fix grammar and punctuation. Remove filler words. \
                    Keep it close to plain text, but use minimal markdown when it improves clarity: \
                    lists (ordered/unordered) for multiple items, bold for emphasis, headings only when absolutely necessary. \
                    Preserve the speaker's intent and technical terminology. \
                    Output only the cleaned text."
                    .to_string(),
                polisher: None,
                model: None,
            },
            Preset {
                name: "email".to_string(),
                description: "Format transcript as an email".to_string(),
                prompt: "Clean up this voice transcript into an email. \
                    Fix grammar and punctuation. Remove filler words. \
                    Keep it concise. Match the sender's original tone (casual or formal). \
                    Do NOT add placeholder names or unnecessary formalities. \
                    Output only the cleaned text."
                    .to_string(),
                polisher: None,
                model: None,
            },
            Preset {
                name: "notes".to_string(),
                description: "Light cleanup for personal notes".to_string(),
                prompt: "Lightly clean up this voice transcript for personal notes. \
                    Fix major grammar issues and remove excessive filler words. \
                    Preserve the speaker's natural voice and thought structure. \
                    IMPORTANT: Start directly with the cleaned content. NEVER add any introduction, preamble, or meta-commentary like 'Here are the notes'. \
                    Output ONLY the cleaned transcript, nothing else."
                    .to_string(),
                polisher: None,
                model: None,
            },
        ]
    }

    /// Load a user preset from file. The filename (without .json) is used as the canonical name.
    fn load_user_preset(name: &str) -> Option<Preset> {
        let path = Self::presets_dir().join(format!("{}.json", name));
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<Preset>(&content) {
                Ok(mut preset) => {
                    preset.name = name.to_string();
                    Some(preset)
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse preset '{}': {}",
                        path.display(),
                        e
                    );
                    None
                }
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => {
                eprintln!("Warning: Failed to read preset '{}': {}", path.display(), e);
                None
            }
        }
    }

    /// Load a preset by name (user file takes precedence over built-in)
    pub fn load(name: &str) -> Result<(Preset, PresetSource), String> {
        // Check user presets first
        if let Some(preset) = Self::load_user_preset(name) {
            return Ok((preset, PresetSource::User));
        }

        // Fall back to built-in
        if let Some(preset) = Self::builtins().into_iter().find(|p| p.name == name) {
            return Ok((preset, PresetSource::BuiltIn));
        }

        Err(format!(
            "Unknown preset '{}'\nAvailable: {}",
            name,
            Self::all_names().join(", ")
        ))
    }

    /// List all available presets (user + built-in, deduplicated)
    pub fn list_all() -> Vec<(Preset, PresetSource)> {
        let mut presets: HashMap<String, (Preset, PresetSource)> = HashMap::new();

        // Add built-ins first
        for preset in Self::builtins() {
            presets.insert(preset.name.clone(), (preset, PresetSource::BuiltIn));
        }

        // Add user presets (overwrite built-ins if same name)
        if let Ok(entries) = fs::read_dir(Self::presets_dir()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    let Some(filename_stem) = path.file_stem().and_then(|s| s.to_str()) else {
                        continue;
                    };
                    match fs::read_to_string(&path) {
                        Ok(content) => match serde_json::from_str::<Preset>(&content) {
                            Ok(mut preset) => {
                                // Use filename as canonical name (ignore internal name field)
                                preset.name = filename_stem.to_string();
                                presets.insert(preset.name.clone(), (preset, PresetSource::User));
                            }
                            Err(e) => {
                                eprintln!(
                                    "Warning: Failed to parse preset '{}': {}",
                                    path.display(),
                                    e
                                );
                            }
                        },
                        Err(e) => {
                            eprintln!("Warning: Failed to read preset '{}': {}", path.display(), e);
                        }
                    }
                }
            }
        }

        // Sort by name
        let mut result: Vec<_> = presets.into_values().collect();
        result.sort_by(|a, b| a.0.name.cmp(&b.0.name));
        result
    }

    /// Get all available preset names
    pub fn all_names() -> Vec<String> {
        Self::list_all().into_iter().map(|(p, _)| p.name).collect()
    }

    /// Create a template preset for the given name
    pub fn template(name: &str) -> Preset {
        Preset {
            name: name.to_string(),
            description: "Describe what this preset does".to_string(),
            prompt: "Your system prompt here".to_string(),
            polisher: None,
            model: None,
        }
    }
}
