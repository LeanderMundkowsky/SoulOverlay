use log::{error, info};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};

use crate::cache_store::Collection;
use crate::hotkey;
use crate::log_watcher;
use crate::settings::Settings;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn get_default_settings() -> Result<Settings, String> {
    Ok(Settings::default())
}

#[tauri::command]
#[specta::specta]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let settings = state.current_settings.lock().unwrap().clone();
    Ok(settings)
}

#[tauri::command]
#[specta::specta]
pub async fn save_settings(
    new_settings: Settings,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let old_settings = {
        let mut current = state.current_settings.lock().unwrap();
        let old = current.clone();
        // Preserve the backend token — it's managed by backend commands, not the settings form
        let mut merged = new_settings.clone();
        merged.backend_api_token = old.backend_api_token.clone();
        *current = merged;
        old
    };

    // Persist to disk — use the in-memory value (with preserved token)
    let settings_to_save = state.current_settings.lock().unwrap().clone();
    state.paths.save_settings(&settings_to_save)?;

    // Side effects: re-register hotkey if changed
    if old_settings.hotkey != settings_to_save.hotkey {
        let new_handle = hotkey::register_hotkey(&settings_to_save.hotkey, &app);
        match new_handle {
            Ok(handle) => {
                *state.hotkey_handle.lock().unwrap() = Some(handle);
            }
            Err(e) => {
                error!("Failed to register new hotkey: {}", e);
            }
        }
    }

    // Side effects: restart log watcher if path changed
    if old_settings.log_path != settings_to_save.log_path {
        let new_path = settings_to_save
            .log_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(log_watcher::default_log_path);

        let mut watcher = state.log_watcher.lock().unwrap();
        if let Some(ref mut w) = *watcher {
            let _ = w.update_path(new_path);
        }
    }

    // Side effects: update cache entry TTLs for any changed values
    {
        for collection in Collection::all() {
            let old_val = collection.ttl_for(&old_settings);
            let new_val = collection.ttl_for(&settings_to_save);
            if old_val != new_val {
                info!("Updating cached TTL for '{}': {}s → {}s", collection.storage_key(), old_val, new_val);
                state.cache.update_collection_ttl(*collection, new_val);
            }
        }
        if old_settings.cache_ttls != settings_to_save.cache_ttls {
            let _ = app.emit("cache-updated", ());
        }
    }

    // Side effects: toggle debug logging at runtime
    if old_settings.debug_logging != settings_to_save.debug_logging {
        state
            .debug_logging
            .store(settings_to_save.debug_logging, Ordering::Relaxed);
        info!(
            "Debug logging {}",
            if settings_to_save.debug_logging {
                "enabled"
            } else {
                "disabled"
            }
        );
    }

    info!("Settings saved");
    Ok(())
}
