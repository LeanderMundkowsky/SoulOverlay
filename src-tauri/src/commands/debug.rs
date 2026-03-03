use chrono::Utc;
use serde::Serialize;
use specta::Type;
use tauri::State;

use crate::activity::{FetchEvent, LastUserAction};
use crate::state::AppState;

/// Initial game connection state returned to the frontend on mount.
#[derive(Debug, Serialize, Type)]
pub struct GameState {
    pub sc_detected: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn get_game_state(state: State<'_, AppState>) -> Result<GameState, String> {
    let gs = state.game_state.lock().unwrap();
    Ok(GameState {
        sc_detected: gs.sc_hwnd.is_some(),
    })
}

/// Per-collection debug status (CollectionStatus extended with is_refreshing + expires_at).
#[derive(Debug, Serialize, Type)]
pub struct CollectionDebugInfo {
    pub key: String,
    pub display_name: String,
    pub cached_at: Option<String>,
    pub expires_at: Option<String>,
    pub ttl_secs: i32,
    pub is_expired: bool,
    pub is_refreshing: bool,
    pub entry_count: u32,
}

/// Comprehensive runtime snapshot returned to the debug panel.
#[derive(Debug, Serialize, Type)]
pub struct DebugInfo {
    // ── Game ──────────────────────────────────────────────────────────────────
    pub sc_detected: bool,
    pub sc_focused: bool,
    pub sc_hwnd: Option<i32>,
    pub sc_window_x: i32,
    pub sc_window_y: i32,
    pub sc_window_w: u32,
    pub sc_window_h: u32,
    // ── Settings ──────────────────────────────────────────────────────────────
    pub hotkey: String,
    pub log_path: Option<String>,
    pub overlay_opacity: f32,
    pub uex_api_key_set: bool,
    pub esc_closes_overlay: bool,
    pub reset_on_open: bool,
    pub max_search_results: u32,
    pub cache_ttls: std::collections::HashMap<String, u32>,
    // ── Services ──────────────────────────────────────────────────────────────
    pub log_watcher_active: bool,
    pub hotkey_registered: bool,
    pub refreshing_collections: Vec<String>,
    // ── Cache ─────────────────────────────────────────────────────────────────
    pub cache_total_keys: u32,
    pub cache_collections: Vec<CollectionDebugInfo>,
    // ── Background timer ──────────────────────────────────────────────────────
    /// ISO8601 timestamp of the last background check tick.
    pub last_bg_check_at: Option<String>,
    /// Seconds until the next background check (30s interval).
    pub next_bg_check_in_secs: i32,
    /// How long ago the last background check ran (seconds).
    pub last_bg_check_ago_secs: Option<i32>,
    // ── Last user price lookup ─────────────────────────────────────────────────
    pub last_user_action: Option<LastUserAction>,
    // ── Activity log ──────────────────────────────────────────────────────────
    /// Most recent fetch events, newest first.
    pub fetch_log: Vec<FetchEvent>,
}

const BG_TIMER_INTERVAL_SECS: i64 = 30;

#[tauri::command]
#[specta::specta]
pub async fn get_debug_info(state: State<'_, AppState>) -> Result<DebugInfo, String> {
    let gs = state.game_state.lock().unwrap();
    let settings = state.current_settings.lock().unwrap().clone();
    let log_watcher_active = state.log_watcher.lock().unwrap().is_some();
    let hotkey_registered = state.hotkey_handle.lock().unwrap().is_some();
    let refreshing: Vec<String> = state.refreshing_collections.lock().unwrap().iter().cloned().collect();

    // Cache collections
    let cache_statuses = state.cache.status();
    let cache_collections: Vec<CollectionDebugInfo> = cache_statuses
        .into_iter()
        .map(|s| {
            let expires_at = s.cached_at.as_deref().and_then(|ts| {
                ts.parse::<chrono::DateTime<Utc>>().ok()
                    .map(|t| (t + chrono::Duration::seconds(s.ttl_secs as i64)).to_rfc3339())
            });
            let is_refreshing = refreshing.contains(&s.collection.storage_key());
            CollectionDebugInfo {
                key: s.collection.storage_key(),
                display_name: s.display_name,
                cached_at: s.cached_at,
                expires_at,
                ttl_secs: s.ttl_secs,
                is_expired: s.is_expired,
                is_refreshing,
                entry_count: s.entry_count,
            }
        })
        .collect();

    // Activity log
    let (last_bg_check_at, last_user_action, fetch_log_rev) = {
        let log = state.activity.lock().unwrap();
        let bg_at = log.last_bg_check_at.map(|t| t.to_rfc3339());
        let ua = log.last_user_action.clone();
        let events: Vec<FetchEvent> = log.events.iter().cloned().rev().collect();
        (bg_at, ua, events)
    };

    let (next_bg_check_in_secs, last_bg_check_ago_secs) = {
        if let Some(ref ts) = last_bg_check_at {
            if let Ok(t) = ts.parse::<chrono::DateTime<Utc>>() {
                let ago = (Utc::now() - t).num_seconds();
                let until = (BG_TIMER_INTERVAL_SECS - ago).max(0);
                (until, Some(ago))
            } else {
                (BG_TIMER_INTERVAL_SECS, None)
            }
        } else {
            (BG_TIMER_INTERVAL_SECS, None)
        }
    };

    Ok(DebugInfo {
        sc_detected: gs.sc_hwnd.is_some(),
        sc_focused: gs.is_focused,
        sc_hwnd: gs.sc_hwnd.map(|h| h as i32),
        sc_window_x: gs.window_x,
        sc_window_y: gs.window_y,
        sc_window_w: gs.window_w,
        sc_window_h: gs.window_h,
        hotkey: settings.hotkey,
        log_path: settings.log_path,
        overlay_opacity: settings.overlay_opacity,
        uex_api_key_set: !settings.uex_api_key.is_empty(),
        esc_closes_overlay: settings.esc_closes_overlay,
        reset_on_open: settings.reset_on_open,
        max_search_results: settings.max_search_results,
        cache_ttls: settings.cache_ttls,
        log_watcher_active,
        hotkey_registered,
        refreshing_collections: refreshing,
        cache_total_keys: state.cache.len() as u32,
        cache_collections,
        last_bg_check_at,
        next_bg_check_in_secs: next_bg_check_in_secs as i32,
        last_bg_check_ago_secs: last_bg_check_ago_secs.map(|s| s as i32),
        last_user_action,
        fetch_log: fetch_log_rev,
    })
}

