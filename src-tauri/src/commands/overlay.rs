use tauri::{AppHandle, State};

use crate::hotkey;
use crate::state::AppState;
use crate::window;

#[tauri::command]
#[specta::specta]
pub async fn hide_overlay_cmd(app: AppHandle, _state: State<'_, AppState>) -> Result<(), String> {
    window::hide_overlay(&app);

    // Keep the hotkey module's visibility tracking in sync.
    hotkey::notify_overlay_hidden();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn show_overlay_cmd(app: AppHandle, _state: State<'_, AppState>) -> Result<(), String> {
    window::show_overlay(&app);

    // Keep the hotkey module's visibility tracking in sync.
    hotkey::notify_overlay_shown();

    Ok(())
}
