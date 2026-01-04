mod app;
mod args;
mod commands;
mod error;
mod hotkey;
mod ipc;
mod service;
mod ui;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    // Run CLI and handle errors with helpful messages
    if let Err(err) = run() {
        error::display_anyhow_error(err);
        std::process::exit(1);
    }
    Ok(())
}

fn run() -> Result<()> {
    let cli = args::Cli::parse();

    // Enable verbose logging if requested
    whis_core::set_verbose(cli.verbose);

    match cli.command {
        Some(args::Commands::Start { hotkey }) => commands::start::run(hotkey),
        Some(args::Commands::Stop) => commands::stop::run(),
        Some(args::Commands::Restart { hotkey }) => commands::restart::run(hotkey),
        Some(args::Commands::Status) => commands::status::run(),
        Some(args::Commands::Config {
            key,
            value,
            list,
            path,
        }) => commands::config::run(key, value, list, path),
        Some(args::Commands::Preset { action }) => commands::preset::run(action),
        Some(args::Commands::Setup { mode }) => commands::setup::run(mode),
        Some(args::Commands::Model { action }) => commands::model::run(action),
        None => commands::record::run(
            cli.processing.post_process,
            cli.processing.preset,
            cli.input.file,
            cli.input.stdin,
            &cli.input.format,
            cli.output.print,
            cli.processing.duration,
            cli.processing.no_vad,
            cli.output.save_raw,
            cli.processing.legacy_batch,
        ),
    }
}
