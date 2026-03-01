use log::{error, info};
use std::path::PathBuf;
use tauri::{AppHandle, State};

use crate::hotkey;
use crate::log_watcher;
use crate::settings::Settings;
use crate::state::AppState;

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let settings = state.current_settings.lock().unwrap().clone();
    Ok(settings)
}

#[tauri::command]
pub async fn save_settings(
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

    // Persist to disk via AppPaths
    state.paths.save_settings(&new_settings)?;

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
