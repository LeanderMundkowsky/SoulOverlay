mod app_setup;
mod cache_store;
mod commands;
pub mod config;
mod database;
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

use log::{error, info};
use settings::Settings;
use state::AppState;
use std::sync::Mutex;

pub fn run() {
    // 1. Resolve all paths first (creates data dir)
    let paths = match config::AppPaths::init() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[FATAL] Failed to initialize paths: {}", e);
            std::process::exit(1);
        }
    };

    // 2. Set up logging (needs data dir to exist)
    logging::setup(&paths.log_file);

    // 3. Initialize SQLite database
    let db_conn = match database::init(&paths.db_file) {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            eprintln!("[FATAL] Database initialization failed: {}", e);
            std::process::exit(1);
        }
    };

    let game_state = std::sync::Arc::new(std::sync::Mutex::new(
        game_tracker::GameTrackerState::default(),
    ));

    let app_state = AppState {
        paths,
        game_tracker: Mutex::new(None),
        game_state: game_state.clone(),
        log_watcher: Mutex::new(None),
        cache: cache_store::CacheStore::new(db_conn),
        current_settings: Mutex::new(Settings::default()),
        hotkey_handle: Mutex::new(None),
    };

    tauri::Builder::default()
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
            commands::api::api_search,
            commands::api::api_search_commodities,
            commands::api::api_search_vehicles,
            commands::api::api_search_items,
            commands::api::api_search_locations,
            commands::api::api_commodity_prices,
            commands::cache::cache_status,
            commands::cache::cache_refresh,
            commands::cache::cache_refresh_all,
            commands::uex::uex_search,
            commands::uex::uex_search_all,
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
