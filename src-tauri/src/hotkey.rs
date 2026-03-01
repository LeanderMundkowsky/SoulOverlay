/// Hotkey management module.
/// Handles registration/unregistration of the global overlay toggle hotkey.
use log::info;
use tauri::AppHandle;

use crate::game_tracker::SharedGameState;
use crate::window;

/// Register the global hotkey for toggling the overlay.
/// The hotkey string should be in the format "Alt+Shift+S" (Tauri accelerator format).
pub fn register_hotkey(
    app: &AppHandle,
    hotkey: &str,
    game_state: SharedGameState,
) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let app_clone = app.clone();

    // Unregister first (ignore errors if not registered)
    let _ = app.global_shortcut().unregister(hotkey);

    app.global_shortcut()
        .on_shortcut(hotkey, move |_app, _shortcut, event| {
            use tauri_plugin_global_shortcut::ShortcutState;
            if event.state == ShortcutState::Pressed {
                handle_hotkey_press(&app_clone, &game_state);
            }
        })
        .map_err(|e| format!("Failed to register hotkey '{}': {}", hotkey, e))?;

    info!("Registered global hotkey: {}", hotkey);
    Ok(())
}

/// Unregister a previously registered hotkey.
pub fn unregister_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    app.global_shortcut()
        .unregister(hotkey)
        .map_err(|e| format!("Failed to unregister hotkey '{}': {}", hotkey, e))?;

    info!("Unregistered global hotkey: {}", hotkey);
    Ok(())
}

/// Handle a hotkey press — toggle the overlay visibility.
fn handle_hotkey_press(app: &AppHandle, game_state: &SharedGameState) {
    let state = game_state.lock().unwrap();

    let is_visible = window::is_overlay_visible(app);

    if is_visible {
        // Hide overlay
        #[cfg(windows)]
        {
            if let Some(sc_hwnd_val) = state.sc_hwnd {
                drop(state); // Release lock before calling window functions
                window::hide_overlay(app, sc_hwnd_val);
            } else {
                drop(state);
                // No SC window known, just hide
                use tauri::Manager;
                if let Some(w) = app.get_webview_window("overlay") {
                    let _ = w.hide();
                }
            }
        }
        #[cfg(not(windows))]
        {
            drop(state);
            window::hide_overlay(app, ());
        }
    } else {
        // Show overlay
        #[cfg(windows)]
        {
            if let Some(sc_hwnd_val) = state.sc_hwnd {
                drop(state);
                window::show_overlay(app, sc_hwnd_val);
            } else {
                info!("Hotkey pressed but SC not detected — showing overlay anyway");
                drop(state);
                // Show even without SC for testing
                use tauri::Manager;
                if let Some(w) = app.get_webview_window("overlay") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        }
        #[cfg(not(windows))]
        {
            drop(state);
            window::show_overlay(app, ());
        }
    }
}
