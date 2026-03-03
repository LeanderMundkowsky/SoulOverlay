use tauri::{AppHandle, Emitter, Manager, State};

use crate::hotkey;
use crate::state::AppState;
use crate::window;

#[tauri::command]
#[specta::specta]
pub async fn hide_overlay_cmd(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let game_state = state.game_state.lock().unwrap();

    if let Some(sc_hwnd_val) = game_state.sc_hwnd {
        drop(game_state);
        window::hide_overlay(&app, sc_hwnd_val);
    } else {
        drop(game_state);
        if let Some(w) = app.get_webview_window("overlay") {
            let _ = w.hide();
        }
    }

    // Keep the hotkey module's visibility tracking in sync.
    hotkey::notify_overlay_hidden();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn show_overlay_cmd(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let game_state = state.game_state.lock().unwrap();

    if let Some(sc_hwnd_val) = game_state.sc_hwnd {
        drop(game_state);
        window::show_overlay(&app, sc_hwnd_val);
    } else {
        drop(game_state);
        if let Some(w) = app.get_webview_window("overlay") {
            let _ = w.show();
            let _ = w.set_focus();
        }
        let _ = app.emit("overlay-shown", ());
    }

    // Keep the hotkey module's visibility tracking in sync.
    hotkey::notify_overlay_shown();

    Ok(())
}
