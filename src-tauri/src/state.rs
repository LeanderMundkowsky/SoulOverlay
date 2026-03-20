use std::collections::HashSet;
use std::sync::{
    atomic::AtomicBool,
    {Arc, Mutex},
};

use crate::cache_store::CacheStore;
use crate::commands::backend::BackendAccount;
use crate::config::AppPaths;
use crate::hotkey;
use crate::log_watcher;
use crate::process_tracker;
use crate::settings::Settings;
use crate::uex::UexClient;
use crate::activity::ActivityLog;

/// Application state managed by Tauri
pub struct AppState {
    pub paths: AppPaths,
    /// Whether StarCitizen.exe is currently running.
    pub sc_running: Arc<AtomicBool>,
    /// Keeps the process tracker thread alive (dropped on shutdown).
    pub process_tracker: Mutex<Option<process_tracker::ProcessTracker>>,
    pub log_watcher: Mutex<Option<log_watcher::LogWatcher>>,
    pub uex: UexClient,
    pub cache: Arc<CacheStore>,
    pub current_settings: Mutex<Settings>,
    /// Holds the LL keyboard hook handle. Dropping it stops the hook thread.
    pub hotkey_handle: Mutex<Option<hotkey::HookHandle>>,
    /// Tracks which collections are currently being refreshed in the background,
    /// preventing duplicate concurrent refreshes from timer + search racing.
    pub refreshing_collections: Mutex<HashSet<String>>,
    /// Activity log: fetch events + last user action + bg timer tracking.
    pub activity: Arc<Mutex<ActivityLog>>,
    /// UEX API key fetched from the SoulOverlay backend at startup.
    /// Empty string if the backend was unreachable.
    pub fetched_api_key: Mutex<String>,
    /// Authenticated backend account (None if not logged in or session expired).
    pub backend_account: Mutex<Option<BackendAccount>>,
}

impl AppState {
    /// Get an Arc reference to the cache store (for spawned tasks).
    pub fn cache_arc(&self) -> Arc<CacheStore> {
        Arc::clone(&self.cache)
    }
}
