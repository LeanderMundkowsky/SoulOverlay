/// System tray icon and menu builder.
use log::info;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

use crate::window;

/// Build and register the system tray icon with menu.
pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_hide = MenuItem::with_id(app, "show_hide", "Show/Hide Overlay", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_hide, &settings, &quit])?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("SoulOverlay")
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show_hide" => {
                if let Some(w) = app.get_webview_window("overlay") {
                    if window::is_overlay_visible(app) {
                        let _ = w.hide();
                    } else {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
            }
            "settings" => {
                let _ = app.emit("open-settings", ());
                // Also show the overlay so user can see settings
                if let Some(w) = app.get_webview_window("overlay") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "quit" => {
                info!("Quit requested from tray menu");
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    info!("System tray initialized");
    Ok(())
}
