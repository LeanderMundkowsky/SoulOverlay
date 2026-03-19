mod activity;
mod app_setup;
mod cache_store;
mod commands;
pub mod config;
mod database;
mod hotkey;
mod image_proxy;
mod log_watcher;
mod logging;
mod platform;
mod process_tracker;
mod providers;
mod settings;
pub mod state;
mod tray;
mod uex;
mod wiki;
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

    let sc_running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    let app_state = AppState {
        paths,
        sc_running: sc_running.clone(),
        process_tracker: Mutex::new(None),
        log_watcher: Mutex::new(None),
        uex: uex::UexClient::new(),
        cache: std::sync::Arc::new(cache_store::CacheStore::new(db_conn)),
        current_settings: Mutex::new(Settings::default()),
        hotkey_handle: Mutex::new(None),
        refreshing_collections: Mutex::new(std::collections::HashSet::new()),
        activity: std::sync::Arc::new(Mutex::new(activity::ActivityLog::new())),
    };

    // Build the tauri-specta invoke handler + export TS bindings in dev mode.
    let builder = create_specta_builder();

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default()
                .header("// @ts-nocheck\n"),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    // Remember which window has focus before Tauri creates any windows,
    // so we can restore focus after setup (window creation steals focus).
    window::capture_pre_launch_foreground();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            use tauri::Manager;
            // On second instance, show the overlay
            if let Some(w) = app.get_webview_window("overlay") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .register_asynchronous_uri_scheme_protocol("uex-img", |_ctx, request, responder| {
            tauri::async_runtime::spawn(async move {
                let response = image_proxy::fetch(request).await;
                responder.respond(response);
            });
        })
        .manage(app_state)
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            // Disable WebView2 built-in devtools, context menu, and status bar
            // on the overlay window before any page content loads.
            use tauri::Manager;
            if let Some(webview) = app.get_webview_window("overlay") {
                let _ = webview.with_webview(|wv| {
                    #[cfg(windows)]
                    unsafe {
                        use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings;
                        if let Ok(core) = wv.controller().CoreWebView2() {
                            if let Ok(s) = core.Settings() {
                                let s: ICoreWebView2Settings = s;
                                let _ = s.SetAreDevToolsEnabled(cfg!(debug_assertions));
                                let _ = s.SetAreDefaultContextMenusEnabled(false);
                                let _ = s.SetIsStatusBarEnabled(false);
                            }
                        }
                    }
                });
            }
            app_setup::initialize(app)?;
            info!("SoulOverlay initialized successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn create_specta_builder() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            commands::api::api_search,
            commands::api::api_search_commodities,
            commands::api::api_search_vehicles,
            commands::api::api_search_items,
            commands::api::api_search_locations,
            commands::api::api_commodity_prices,
            commands::api::api_raw_commodity_prices,
            commands::api::api_item_prices,
            commands::api::api_vehicle_purchase_prices,
            commands::api::api_vehicle_rental_prices,
            commands::api::api_fuel_prices,
            commands::api::api_entity_info,
            commands::api::api_location_terminals,
            commands::api::api_terminal_prices,
            commands::cache::cache_status,
            commands::cache::cache_refresh,
            commands::cache::cache_refresh_all,
            commands::cache::cache_refresh_expired,
            commands::uex::uex_search,
            commands::uex::uex_search_all,
            commands::uex::uex_prices,
            commands::settings::get_default_settings,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::overlay::hide_overlay_cmd,
            commands::overlay::show_overlay_cmd,
            commands::debug::get_debug_info,
            commands::debug::get_game_state,
            commands::favorites::get_favorites,
            commands::favorites::add_favorite,
            commands::favorites::remove_favorite,
            commands::favorites::is_favorite,
            commands::hangar::hangar_get_fleet,
            commands::user::user_get_profile,
            commands::watchlist::get_watchlist,
            commands::watchlist::add_watch_entry,
            commands::watchlist::remove_watch_entry,
            commands::inventory::get_inventory,
            commands::inventory::add_inventory_entry,
            commands::inventory::update_inventory_entry,
            commands::inventory::update_inventory_quantity,
            commands::inventory::remove_inventory_entry,
            commands::inventory::remove_inventory_quantity,
            commands::inventory::transfer_inventory,
            commands::inventory::get_inventory_collections,
            commands::inventory::get_storage_locations,
            commands::updates::check_for_update,
            commands::updates::backup_before_update,
            commands::contested_zones::cz_get_cycle_start,
            commands::contested_zones::cz_get_ships,
            commands::contested_zones::cz_get_maps,
            commands::contested_zones::cz_load_self_timers,
            commands::contested_zones::cz_save_self_timer,
            commands::contested_zones::cz_reset_all_self_timers,
            commands::wiki::wiki_search,
            commands::wiki::wiki_entity_specs,
            commands::wikelo::wikelo_get_trades,
            commands::wikelo::wikelo_get_completions,
            commands::wikelo::wikelo_toggle_completion,
        ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_bindings() {
        create_specta_builder()
            .export(
                specta_typescript::Typescript::default()
                    .header("// @ts-nocheck\n"),
                "../src/bindings.ts",
            )
            .expect("Failed to export typescript bindings");
    }
}
