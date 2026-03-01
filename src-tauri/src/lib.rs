mod app_setup;
mod commands;
mod game_tracker;
mod hotkey;
mod log_watcher;
mod logging;
mod platform;
mod settings;
pub mod state;
mod tray;
mod uex_client;
mod window;

use log::info;
use settings::Settings;
use state::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::setup();

    let game_state = std::sync::Arc::new(std::sync::Mutex::new(
        game_tracker::GameTrackerState::default(),
    ));

    let app_state = AppState {
        game_tracker: Mutex::new(None),
        game_state: game_state.clone(),
        log_watcher: Mutex::new(None),
        uex_cache: Mutex::new(uex_client::UexCache::new(60)),
        current_settings: Mutex::new(Settings::default()),
        hotkey_handle: Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            use tauri::Manager;
            // On second instance, show the overlay
            if let Some(w) = app.get_webview_window("overlay") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::uex::uex_search,
            commands::uex::uex_prices,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::overlay::hide_overlay_cmd,
            commands::overlay::show_overlay_cmd,
            commands::debug::get_debug_info,
            commands::debug::get_game_state,
        ])
        .setup(|app| {
            app_setup::initialize(app)?;
            info!("SoulOverlay initialized successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
