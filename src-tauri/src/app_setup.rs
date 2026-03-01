use log::{error, info};
use std::path::PathBuf;
use tauri::{App, Manager};

use crate::commands;
use crate::game_tracker;
use crate::hotkey;
use crate::log_watcher;
use crate::state::AppState;
use crate::tray;
use crate::window;

/// Run all first-launch initialization inside the Tauri `.setup()` callback.
pub fn initialize(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();

    // Load persisted settings from disk via AppPaths
    let settings = {
        let state = handle.state::<AppState>();
        let loaded = state.paths.load_settings();
        let mut current = state.current_settings.lock().unwrap();
        *current = loaded.clone();
        loaded
    };

    // Initialize overlay window
    if let Some(window) = app.get_webview_window("overlay") {
        window::init_overlay_window(&window, &handle);
    }

    // Position overlay to cover the full primary monitor at startup
    {
        let (mx, my, mw, mh) = window::get_primary_monitor_geometry();
        window::set_window_geometry(&handle, mx, my, mw, mh);
        info!(
            "Overlay positioned to primary monitor: {}x{} at ({}, {})",
            mw, mh, mx, my
        );
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

    // Spawn background prefetch of cached collections.
    // Uses a separate tokio task so it doesn't block the UI startup.
    let prefetch_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        info!("Starting background cache prefetch...");
        let state = prefetch_handle.state::<AppState>();
        commands::cache::prefetch_all(&state).await;
        info!("Background cache prefetch complete");
    });

    Ok(())
}
