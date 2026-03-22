use log::{error, info};
use std::path::PathBuf;
use tauri::{App, Emitter, Manager};

use crate::commands;
use crate::hotkey;
use crate::log_watcher;
use crate::process_tracker;
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
        // Sync the debug logging flag with the persisted setting
        state.debug_logging.store(
            loaded.debug_logging,
            std::sync::atomic::Ordering::Relaxed,
        );
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

    // Start process tracker (detects StarCitizen.exe running/stopping)
    let tracker = process_tracker::ProcessTracker::new(handle.clone());
    tracker.start();
    *handle.state::<AppState>().process_tracker.lock().unwrap() = Some(tracker);

    // Register global hotkey
    let state = handle.state::<AppState>();
    match hotkey::register_hotkey(&settings.hotkey, &handle) {
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
    // First fetches the UEX API key from the SoulOverlay backend, then prefetches caches.
    let prefetch_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        fetch_and_store_api_key(&prefetch_handle).await;
        commands::backend::fetch_account_on_startup(&prefetch_handle).await;
        commands::inventory::migrate_legacy_inventory(&prefetch_handle).await;
        commands::inventory::sync_inventory_from_backend(&prefetch_handle).await;
        info!("Starting background cache prefetch...");
        let state = prefetch_handle.state::<AppState>();
        commands::cache::prefetch_all(&state).await;
        // Seed the timer so the debug panel countdown starts immediately
        if let Ok(mut log) = state.activity.lock() {
            log.last_bg_check_at = Some(chrono::Utc::now());
        }
        info!("Background cache prefetch complete");
    });

    // Spawn a long-lived background timer that refreshes expired collections every 60s.
    let timer_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        use crate::cache_store::Collection;

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        // The first tick fires immediately — skip it since prefetch_all just ran.
        interval.tick().await;

        loop {
            interval.tick().await;
            let state = timer_handle.state::<AppState>();

            // Record this tick in the activity log
            {
                if let Ok(mut log) = state.activity.lock() {
                    log.last_bg_check_at = Some(chrono::Utc::now());
                }
            }

            let mut refreshed_any = false;

            for collection in Collection::all() {
                if commands::cache::guarded_refresh(&state, collection).await {
                    refreshed_any = true;
                }
            }

            if refreshed_any {
                let _ = timer_handle.emit("cache-updated", ());
            }

            // Check backend reachability and emit status event
            let backend_ok = commands::backend::check_backend_status().await;
            let _ = timer_handle.emit("backend-status", serde_json::json!({ "ok": backend_ok }));
        }
    });

    // Spawn a delayed update check (~5s after launch).
    let update_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        info!("Running startup update check...");
        match commands::updates::check_for_update(update_handle.clone()).await {
            Ok(Some(info)) => {
                info!("Update available: v{}", info.version);
                let _ = update_handle.emit("update-available", info);
            }
            Ok(None) => {
                info!("No updates available at startup");
            }
            Err(e) => {
                // Don't surface startup check failures to the user
                error!("Startup update check failed: {}", e);
            }
        }
    });

    // Show a brief startup hint with the hotkey
    spawn_startup_hint(&handle, &settings.hotkey);

    // Restore focus to whatever was active before the app launched (Windows only).
    // Tauri/WebView2 window creation steals focus even for invisible windows.
    window::restore_pre_launch_foreground();

    Ok(())
}

/// Fetch the UEX API key from the SoulOverlay backend and store it in AppState.
/// Silently proceeds with an empty key if the backend is unreachable or returns an error.
async fn fetch_and_store_api_key(handle: &tauri::AppHandle) {
    use crate::constants::{BACKEND_CONFIG_ENDPOINT, BACKEND_URL, SOUL_APP_TOKEN};
    use log::debug;
    use tauri::Manager;

    let url = format!("{}{}", BACKEND_URL, BACKEND_CONFIG_ENDPOINT);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    debug!("[backend] → GET {} (app-token: {})", BACKEND_CONFIG_ENDPOINT, if SOUL_APP_TOKEN.is_empty() { "not set" } else { "set" });
    let t = std::time::Instant::now();
    match client
        .get(&url)
        .header("X-Soul-App-Token", SOUL_APP_TOKEN)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            debug!("[backend] ← GET {} {} ({}ms)", BACKEND_CONFIG_ENDPOINT, resp.status(), t.elapsed().as_millis());
            match resp.json::<serde_json::Value>().await {
                Ok(json) => {
                    if let Some(key) = json["data"]["uexApiKey"].as_str() {
                        if !key.is_empty() {
                            let state = handle.state::<AppState>();
                            *state.fetched_api_key.lock().unwrap() = key.to_string();
                            info!("UEX API key fetched from backend successfully");
                            return;
                        }
                    }
                    error!("Backend config response missing uexApiKey");
                }
                Err(e) => error!("Failed to parse backend config response: {}", e),
            }
        }
        Ok(resp) => {
            debug!("[backend] ← GET {} {} ({}ms)", BACKEND_CONFIG_ENDPOINT, resp.status(), t.elapsed().as_millis());
            error!("Backend returned {} for config endpoint", resp.status());
        }
        Err(e) => {
            debug!("[backend] ✗ GET {} ({}ms)", BACKEND_CONFIG_ENDPOINT, t.elapsed().as_millis());
            info!("Backend unreachable, proceeding without UEX API key: {}", e);
        }
    }
}


/// Show a brief startup hint with the configured hotkey.
/// Auto-closes after ~7.5s (the CSS animation completes its fade-out at 7s).
fn spawn_startup_hint(handle: &tauri::AppHandle, hotkey: &str) {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

    let logical_w = 320.0_f64;
    let logical_h = 44.0_f64;

    // Compute the physical center of the primary monitor so we can position accurately
    let scale = handle
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| m.scale_factor())
        .unwrap_or(1.0);
    let (mx, my, mw, _) = window::get_primary_monitor_geometry();
    let phys_w = (logical_w * scale) as i32;
    let x = mx + (mw as i32 - phys_w) / 2;
    let y = my + (24.0 * scale) as i32;

    let encoded_hotkey = hotkey.replace('+', "%2B");
    let url = format!("hint.html?hotkey={}", encoded_hotkey);

    let result = WebviewWindowBuilder::new(handle, "startup-hint", WebviewUrl::App(url.into()))
        .title("SoulOverlay")
        .inner_size(logical_w, logical_h)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .visible(false)
        .build();

    match result {
        Ok(w) => {
            #[cfg(windows)]
            if let Ok(raw) = w.hwnd() {
                use windows::Win32::Foundation::HWND;
                use windows::Win32::UI::WindowsAndMessaging::{
                    GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST,
                    SWP_NOACTIVATE, SWP_NOSIZE, SWP_SHOWWINDOW, WS_EX_NOACTIVATE,
                    WS_EX_TOOLWINDOW,
                };

                let hwnd = HWND(raw.0);
                unsafe {
                    // Hide from Alt+Tab, prevent focus stealing
                    let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                    SetWindowLongPtrW(
                        hwnd,
                        GWL_EXSTYLE,
                        ex | WS_EX_TOOLWINDOW.0 as isize | WS_EX_NOACTIVATE.0 as isize,
                    );
                    // Position at top-center and show
                    let _ = SetWindowPos(
                        hwnd,
                        HWND_TOPMOST,
                        x,
                        y,
                        0,
                        0,
                        SWP_NOSIZE | SWP_SHOWWINDOW | SWP_NOACTIVATE,
                    );
                }
            }

            #[cfg(not(windows))]
            {
                // On Linux, use Tauri's cross-platform positioning
                let _ = w.set_position(tauri::PhysicalPosition::new(x, y));
                let _ = w.show();
            }

            // Auto-close slightly after the CSS fade-out animation finishes
            let h = handle.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(7500)).await;
                if let Some(w) = h.get_webview_window("startup-hint") {
                    let _ = w.close();
                }
            });

            info!("Startup hint window shown");
        }
        Err(e) => {
            error!("Failed to create startup hint: {}", e);
        }
    }
}

