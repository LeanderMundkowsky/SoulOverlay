use std::sync::Mutex;

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
    pub cache: CacheStore,
    pub current_settings: Mutex<Settings>,
    /// Holds the LL keyboard hook handle. Dropping it stops the hook thread.
    pub hotkey_handle: Mutex<Option<hotkey::HookHandle>>,
}
