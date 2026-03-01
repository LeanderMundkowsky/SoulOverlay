use serde::Serialize;
use tauri::State;

use crate::state::AppState;

/// Initial game connection state returned to the frontend on mount.
#[derive(Debug, Serialize)]
pub struct GameState {
    pub sc_detected: bool,
}

#[tauri::command]
pub async fn get_game_state(state: State<'_, AppState>) -> Result<GameState, String> {
    let gs = state.game_state.lock().unwrap();
    Ok(GameState {
        sc_detected: gs.sc_hwnd.is_some(),
    })
}

/// Snapshot of runtime state returned to the frontend debug panel.
#[derive(Debug, Serialize)]
pub struct DebugInfo {
    // Game tracker
    pub sc_detected: bool,
    pub sc_focused: bool,
    pub sc_hwnd: Option<isize>,
    pub sc_window_x: i32,
    pub sc_window_y: i32,
    pub sc_window_w: u32,
    pub sc_window_h: u32,
    // Settings
    pub hotkey: String,
    pub log_path: Option<String>,
    pub overlay_opacity: f32,
    pub uex_api_key_set: bool,
    // Cache
    pub uex_cache_entries: usize,
    // Log watcher
    pub log_watcher_active: bool,
}

#[tauri::command]
pub async fn get_debug_info(state: State<'_, AppState>) -> Result<DebugInfo, String> {
    let gs = state.game_state.lock().unwrap();
    let settings = state.current_settings.lock().unwrap().clone();
    let cache_entries = state.cache.len();
    let log_watcher_active = state.log_watcher.lock().unwrap().is_some();

    Ok(DebugInfo {
        sc_detected: gs.sc_hwnd.is_some(),
        sc_focused: gs.is_focused,
        sc_hwnd: gs.sc_hwnd,
        sc_window_x: gs.window_x,
        sc_window_y: gs.window_y,
        sc_window_w: gs.window_w,
        sc_window_h: gs.window_h,
        hotkey: settings.hotkey.clone(),
        log_path: settings.log_path.clone(),
        overlay_opacity: settings.overlay_opacity,
        uex_api_key_set: !settings.uex_api_key.is_empty(),
        uex_cache_entries: cache_entries,
        log_watcher_active,
    })
}
