mod commands;
pub mod settings;
pub mod shortcuts;
mod state;
pub mod tray;
mod window;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Load settings from disk
            let loaded_settings = settings::Settings::load();

            // Initialize system tray (optional - may fail on tray-less environments)
            let tray_available = match tray::setup_tray(app) {
                Ok(_) => true,
                Err(e) => {
                    eprintln!("Tray unavailable: {e}. Running in window mode.");
                    false
                }
            };

            // Initialize state with tray availability
            app.manage(state::AppState::new(loaded_settings, tray_available));

            // Setup global shortcuts (hybrid: Tauri plugin / Portal / CLI fallback)
            shortcuts::setup_shortcuts(app);

            // Start IPC listener for --toggle CLI commands
            shortcuts::start_ipc_listener(app.handle().clone());

            // Always show main window on startup
            window::show_main_window(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::is_api_configured,
            commands::get_settings,
            commands::save_settings,
            commands::shortcut_backend,
            commands::configure_shortcut,
            commands::configure_shortcut_with_trigger,
            commands::portal_shortcut,
            commands::validate_openai_api_key,
            commands::validate_mistral_api_key,
            commands::validate_groq_api_key,
            commands::validate_deepgram_api_key,
            commands::validate_elevenlabs_api_key,
            commands::reset_shortcut,
            commands::portal_bind_error,
            commands::get_toggle_command,
            commands::toggle_recording,
            commands::can_reopen_window,
            commands::download_whisper_model,
            commands::test_remote_whisper,
            commands::test_ollama_connection,
            commands::list_ollama_models,
            commands::pull_ollama_model,
            commands::start_ollama,
            commands::check_config_readiness,
            commands::get_whisper_models,
            commands::is_whisper_model_valid,
            commands::list_presets,
            commands::apply_preset,
            commands::get_active_preset,
            commands::set_active_preset,
            commands::get_preset_details,
            commands::create_preset,
            commands::update_preset,
            commands::delete_preset,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
