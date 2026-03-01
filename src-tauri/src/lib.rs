mod game_tracker;
mod hotkey;
mod log_watcher;
mod settings;
mod tray;
mod uex_client;
mod window;

use log::{error, info};
use settings::Settings;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_store::StoreExt;

/// Application state managed by Tauri
pub struct AppState {
    pub game_tracker: Mutex<Option<game_tracker::GameTracker>>,
    pub game_state: game_tracker::SharedGameState,
    pub log_watcher: Mutex<Option<log_watcher::LogWatcher>>,
    pub uex_cache: Mutex<uex_client::UexCache>,
    pub current_settings: Mutex<Settings>,
    /// Holds the LL keyboard hook handle. Dropping it stops the hook thread.
    pub hotkey_handle: Mutex<Option<hotkey::HookHandle>>,
}

// -- Tauri Commands --

#[tauri::command]
async fn uex_search(
    query: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<Vec<uex_client::UexResult>, String> {
    // Check cache first
    let cache_key = format!("search:{}", query);
    {
        let cache = state.uex_cache.lock().unwrap();
        if let Some(cached) = cache.get(&cache_key) {
            if let Ok(results) = serde_json::from_value(cached.clone()) {
                return Ok(results);
            }
        }
    }

    let results = uex_client::search(&query, &api_key).await?;

    // Store in cache
    {
        let mut cache = state.uex_cache.lock().unwrap();
        if let Ok(json) = serde_json::to_value(&results) {
            cache.insert(cache_key, json);
        }
        cache.cleanup();
    }

    Ok(results)
}

#[tauri::command]
async fn uex_prices(
    commodity: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<Vec<uex_client::PriceEntry>, String> {
    let cache_key = format!("prices:{}", commodity);
    {
        let cache = state.uex_cache.lock().unwrap();
        if let Some(cached) = cache.get(&cache_key) {
            if let Ok(results) = serde_json::from_value(cached.clone()) {
                return Ok(results);
            }
        }
    }

    let results = uex_client::get_prices(&commodity, &api_key).await?;

    {
        let mut cache = state.uex_cache.lock().unwrap();
        if let Ok(json) = serde_json::to_value(&results) {
            cache.insert(cache_key, json);
        }
        cache.cleanup();
    }

    Ok(results)
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let settings = state.current_settings.lock().unwrap().clone();
    Ok(settings)
}

#[tauri::command]
async fn save_settings(
    new_settings: Settings,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let old_settings = {
        let mut current = state.current_settings.lock().unwrap();
        let old = current.clone();
        *current = new_settings.clone();
        old
    };

    // Persist to store
    if let Ok(store) = app.store("settings.json") {
        let _ = store.set(
            "settings",
            serde_json::to_value(&new_settings).unwrap_or_default(),
        );
        let _ = store.save();
    }

    // Side effects: re-register hotkey if changed
    if old_settings.hotkey != new_settings.hotkey {
        let game_state = state.game_state.clone();
        let new_handle = hotkey::register_hotkey(&app, &new_settings.hotkey, game_state);
        match new_handle {
            Ok(handle) => {
                // Drop the old handle (stops the old hook thread) and store the new one.
                *state.hotkey_handle.lock().unwrap() = Some(handle);
            }
            Err(e) => {
                error!("Failed to register new hotkey: {}", e);
            }
        }
    }

    // Side effects: restart log watcher if path changed
    if old_settings.log_path != new_settings.log_path {
        let new_path = new_settings
            .log_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(log_watcher::default_log_path);

        let mut watcher = state.log_watcher.lock().unwrap();
        if let Some(ref mut w) = *watcher {
            let _ = w.update_path(new_path);
        }
    }

    info!("Settings saved");
    Ok(())
}

#[tauri::command]
async fn hide_overlay_cmd(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let game_state = state.game_state.lock().unwrap();

    #[cfg(windows)]
    {
        if let Some(sc_hwnd_val) = game_state.sc_hwnd {
            drop(game_state);
            window::hide_overlay(&app, sc_hwnd_val);
        } else {
            drop(game_state);
            if let Some(w) = app.get_webview_window("overlay") {
                let _ = w.hide();
            }
        }
    }

    #[cfg(not(windows))]
    {
        drop(game_state);
        window::hide_overlay(&app, ());
    }

    // Keep the hotkey module's visibility tracking in sync.
    hotkey::notify_overlay_hidden();

    Ok(())
}

#[tauri::command]
async fn show_overlay_cmd(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let game_state = state.game_state.lock().unwrap();

    #[cfg(windows)]
    {
        if let Some(sc_hwnd_val) = game_state.sc_hwnd {
            drop(game_state);
            window::show_overlay(&app, sc_hwnd_val);
        } else {
            drop(game_state);
            if let Some(w) = app.get_webview_window("overlay") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
    }

    #[cfg(not(windows))]
    {
        drop(game_state);
        window::show_overlay(&app, ());
    }

    // Keep the hotkey module's visibility tracking in sync.
    hotkey::notify_overlay_shown();

    Ok(())
}

/// Set up logging to both stderr and a log file in %APPDATA%/SoulOverlay/.
/// The log file is rotated on each launch (overwritten, not appended) to keep
/// it at a reasonable size. Falls back to stderr-only if the file can't be created.
fn setup_logging() {
    use std::fs;

    let log_level = log::LevelFilter::Info;

    // Build stderr output
    let stderr_dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(std::io::stderr());

    // Try to set up a file logger in %APPDATA%/SoulOverlay/
    let file_dispatch = (|| -> Option<fern::Dispatch> {
        let app_data = std::env::var("APPDATA").ok()?;
        let log_dir = std::path::PathBuf::from(app_data).join("SoulOverlay");
        fs::create_dir_all(&log_dir).ok()?;
        let log_path = log_dir.join("soul-overlay.log");
        let file = fern::log_file(&log_path).ok()?;
        Some(
            fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{}][{}][{}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                        record.level(),
                        record.target(),
                        message
                    ))
                })
                .chain(file),
        )
    })();

    let mut dispatch = fern::Dispatch::new()
        .level(log_level)
        // Suppress noisy deps
        .level_for("hyper", log::LevelFilter::Warn)
        .level_for("reqwest", log::LevelFilter::Warn)
        .level_for("tao", log::LevelFilter::Warn)
        .level_for("wry", log::LevelFilter::Warn)
        .chain(stderr_dispatch);

    if let Some(fd) = file_dispatch {
        dispatch = dispatch.chain(fd);
        // Can't use log macros yet — logger not initialised
        eprintln!("[INFO] File logging enabled");
    } else {
        eprintln!("[WARN] Could not create log file, logging to stderr only");
    }

    dispatch.apply().unwrap_or_else(|e| {
        eprintln!("[ERROR] Failed to initialize logger: {}", e);
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    setup_logging();

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
            // On second instance, show the overlay
            if let Some(w) = app.get_webview_window("overlay") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            uex_search,
            uex_prices,
            get_settings,
            save_settings,
            hide_overlay_cmd,
            show_overlay_cmd,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // Load persisted settings
            let settings: Settings = if let Ok(store) = handle.store("settings.json") {
                if let Some(val) = store.get("settings") {
                    serde_json::from_value::<Settings>(val).unwrap_or_default()
                } else {
                    Settings::default()
                }
            } else {
                Settings::default()
            };

            // Update managed state with loaded settings
            {
                let state = handle.state::<AppState>();
                let mut current = state.current_settings.lock().unwrap();
                *current = settings.clone();
            }

            // Initialize overlay window
            if let Some(window) = app.get_webview_window("overlay") {
                window::init_overlay_window(&window, &handle);
            }

            // Set up system tray
            if let Err(e) = tray::setup_tray(&handle) {
                error!("Failed to setup tray: {}", e);
            }

            // Start game tracker
            let tracker = game_tracker::GameTracker::new(handle.clone());
            let tracker_game_state = tracker.state();
            tracker.start();

            {
                let state = handle.state::<AppState>();
                // Copy initial game state from tracker into our managed state
                {
                    let mut gs = state.game_state.lock().unwrap();
                    let tracker_gs = tracker_game_state.lock().unwrap();
                    gs.sc_hwnd = tracker_gs.sc_hwnd;
                    gs.is_focused = tracker_gs.is_focused;
                    gs.is_running = tracker_gs.is_running;
                    gs.window_x = tracker_gs.window_x;
                    gs.window_y = tracker_gs.window_y;
                    gs.window_w = tracker_gs.window_w;
                    gs.window_h = tracker_gs.window_h;
                }
                *state.game_tracker.lock().unwrap() = Some(tracker);
            }

            // Register global hotkey (LL keyboard hook)
            let state = handle.state::<AppState>();
            let gs_for_hotkey = state.game_state.clone();
            match hotkey::register_hotkey(&handle, &settings.hotkey, gs_for_hotkey) {
                Ok(handle_hook) => {
                    *state.hotkey_handle.lock().unwrap() = Some(handle_hook);
                }
                Err(e) => {
                    error!("Failed to register hotkey: {}", e);
                }
            }

            // Start log watcher
            let log_path = settings
                .log_path
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(log_watcher::default_log_path);

            match log_watcher::LogWatcher::start(handle.clone(), log_path) {
                Ok(watcher) => {
                    *state.log_watcher.lock().unwrap() = Some(watcher);
                }
                Err(e) => {
                    error!("Failed to start log watcher: {}", e);
                }
            }

            info!("SoulOverlay initialized successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
