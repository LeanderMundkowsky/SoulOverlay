use std::sync::{Arc, Mutex};

use crate::cache_store::CacheStore;
use crate::config::AppPaths;
use crate::game_tracker;
use crate::hotkey;
use crate::log_watcher;
use crate::settings::Settings;

/// Application state managed by Tauri
pub struct AppState {
    pub paths: AppPaths,
    pub game_tracker: Mutex<Option<game_tracker::GameTracker>>,
    pub game_state: game_tracker::SharedGameState,
    pub log_watcher: Mutex<Option<log_watcher::LogWatcher>>,
    pub cache: Arc<CacheStore>,
    pub current_settings: Mutex<Settings>,
    /// Holds the LL keyboard hook handle. Dropping it stops the hook thread.
    pub hotkey_handle: Mutex<Option<hotkey::HookHandle>>,
}

impl AppState {
    /// Get an Arc reference to the cache store (for spawned tasks).
    pub fn cache_arc(&self) -> Arc<CacheStore> {
        Arc::clone(&self.cache)
    }
}
