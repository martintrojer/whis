use anyhow::{Result, anyhow};
use whis_core::{Preset, PresetSource};

use crate::args::PresetsAction;

pub fn run(action: Option<PresetsAction>) -> Result<()> {
    match action {
        None | Some(PresetsAction::List) => list(),
        Some(PresetsAction::Show { name }) => show(&name),
        Some(PresetsAction::New { name }) => new(&name),
        Some(PresetsAction::Edit { name }) => edit(&name),
    }
}

fn list() -> Result<()> {
    let presets = Preset::list_all();

    if presets.is_empty() {
        println!("No presets available.");
        return Ok(());
    }

    // Calculate column widths
    let name_width = presets
        .iter()
        .map(|(p, _)| p.name.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let source_width = "built-in".len();

    // Print header
    println!(
        "{:<name_width$}  {:<source_width$}  DESCRIPTION",
        "NAME", "SOURCE"
    );

    // Print presets
    for (preset, source) in presets {
        println!(
            "{:<name_width$}  {:<source_width$}  {}",
            preset.name, source, preset.description
        );
    }

    println!();
    println!("User presets: {}", Preset::presets_dir().display());

    Ok(())
}

fn show(name: &str) -> Result<()> {
    let (preset, source) = Preset::load(name).map_err(|e| anyhow!("{}", e))?;

    println!("Preset: {} ({})", preset.name, source);
    println!();
    println!("Description:");
    println!("  {}", preset.description);
    println!();
    println!("Prompt:");
    for line in preset.prompt.lines() {
        println!("  {}", line);
    }

    // Show overrides if any
    if preset.polisher.is_some() || preset.model.is_some() {
        println!();
        println!("Overrides:");
        if let Some(polisher) = &preset.polisher {
            println!("  Polisher: {}", polisher);
        }
        if let Some(model) = &preset.model {
            println!("  Model: {}", model);
        }
    }

    // Show file location for user presets
    if source == PresetSource::User {
        println!();
        println!(
            "Location: {}",
            Preset::presets_dir()
                .join(format!("{}.json", name))
                .display()
        );
    }

    Ok(())
}

fn new(name: &str) -> Result<()> {
    let template = Preset::template(name);
    let json = serde_json::to_string_pretty(&template)?;
    println!("{}", json);
    eprintln!();
    eprintln!(
        "Save to: {}",
        Preset::presets_dir()
            .join(format!("{}.json", name))
            .display()
    );
    Ok(())
}

fn edit(name: &str) -> Result<()> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "nano".to_string());

    let presets_dir = Preset::presets_dir();
    let file_path = presets_dir.join(format!("{}.json", name));

    // If file doesn't exist, create from template
    if !file_path.exists() {
        std::fs::create_dir_all(&presets_dir)?;
        let template = Preset::template(name);
        std::fs::write(&file_path, serde_json::to_string_pretty(&template)?)?;
        println!("Created new preset: {}", file_path.display());
    }

    // Open in editor
    let status = std::process::Command::new(&editor)
        .arg(&file_path)
        .status()?;

    if status.success() {
        println!("Preset saved: {}", file_path.display());
    }
    Ok(())
}
