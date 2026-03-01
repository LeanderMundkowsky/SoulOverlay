/// Window management for the overlay.
///
/// On Windows, this uses Win32 API calls to:
/// - Set WS_EX_TOOLWINDOW (no taskbar entry)
/// - Position the overlay over the SC window
/// - Show/hide with proper focus management
///
/// On non-Windows (dev builds on Linux), provides no-op stubs.
use log::info;
use tauri::AppHandle;

#[cfg(windows)]
use tauri::WebviewWindow;

#[cfg(windows)]
use windows::Win32::Foundation::{BOOL, HWND, POINT, RECT};
#[cfg(windows)]
use windows::Win32::Graphics::Gdi::ClientToScreen;
#[cfg(windows)]
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, GetClientRect, GetWindowLongPtrW, GetWindowThreadProcessId,
    SetForegroundWindow, SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_EXSTYLE, HWND_TOPMOST,
    SET_WINDOW_POS_FLAGS, SW_HIDE, SW_SHOW, WS_EX_TOOLWINDOW,
};

/// Helper: Get the Win32 HWND from a Tauri WebviewWindow.
/// Tauri 2 returns HWND from the `windows` crate version it depends on (0.61),
/// which may differ from our project's `windows` 0.58. Both define
/// HWND(*mut c_void), so we convert via the raw pointer.
#[cfg(windows)]
fn get_hwnd(window: &WebviewWindow) -> Option<HWND> {
    match window.hwnd() {
        Ok(h) => Some(HWND(h.0)),
        Err(_) => None,
    }
}

/// Initialize the overlay window with WS_EX_TOOLWINDOW style on Windows.
/// Call this after the app is ready.
#[cfg(windows)]
pub fn init_overlay_window(window: &WebviewWindow) {
    let hwnd = match get_hwnd(window) {
        Some(h) => h,
        None => {
            log::error!("Failed to get overlay HWND for init");
            return;
        }
    };

    unsafe {
        // Add WS_EX_TOOLWINDOW to extended style (hides from taskbar and Alt+Tab)
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_TOOLWINDOW.0 as isize);
    }

    info!("Overlay window initialized with WS_EX_TOOLWINDOW");
}

#[cfg(not(windows))]
pub fn init_overlay_window(_window: &tauri::WebviewWindow) {
    info!("init_overlay_window: no-op on non-Windows");
}

/// Show the overlay window, stealing focus from SC.
#[cfg(windows)]
pub fn show_overlay(app: &AppHandle, sc_hwnd_val: isize) {
    use tauri::Manager;

    let window = match app.get_webview_window("overlay") {
        Some(w) => w,
        None => return,
    };

    let overlay_hwnd = match get_hwnd(&window) {
        Some(h) => h,
        None => return,
    };

    let sc_hwnd = crate::game_tracker::hwnd_from_isize(sc_hwnd_val);

    unsafe {
        // Attach input threads so we can steal focus
        let sc_thread = GetWindowThreadProcessId(sc_hwnd, None);
        let our_thread = GetCurrentThreadId();

        if sc_thread != our_thread {
            let _ = AttachThreadInput(our_thread, sc_thread, BOOL::from(true));
        }

        let _ = BringWindowToTop(overlay_hwnd);
        let _ = ShowWindow(overlay_hwnd, SW_SHOW);
        let _ = SetForegroundWindow(overlay_hwnd);

        if sc_thread != our_thread {
            let _ = AttachThreadInput(our_thread, sc_thread, BOOL::from(false));
        }
    }

    info!("Overlay shown");
}

#[cfg(not(windows))]
pub fn show_overlay(app: &AppHandle, _sc_hwnd: ()) {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.show();
        let _ = window.set_focus();
    }
    info!("show_overlay: basic show on non-Windows");
}

/// Hide the overlay and return focus to SC.
#[cfg(windows)]
pub fn hide_overlay(app: &AppHandle, sc_hwnd_val: isize) {
    use tauri::Manager;

    let window = match app.get_webview_window("overlay") {
        Some(w) => w,
        None => return,
    };

    let overlay_hwnd = match get_hwnd(&window) {
        Some(h) => h,
        None => return,
    };

    let sc_hwnd = crate::game_tracker::hwnd_from_isize(sc_hwnd_val);

    unsafe {
        let _ = ShowWindow(overlay_hwnd, SW_HIDE);
        let _ = SetForegroundWindow(sc_hwnd);
    }

    info!("Overlay hidden, focus returned to SC");
}

#[cfg(not(windows))]
pub fn hide_overlay(app: &AppHandle, _sc_hwnd: ()) {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.hide();
    }
    info!("hide_overlay: basic hide on non-Windows");
}

/// Position and resize the overlay to match the SC window geometry.
#[cfg(windows)]
pub fn set_window_geometry(app: &AppHandle, x: i32, y: i32, w: u32, h: u32) {
    use tauri::Manager;

    let window = match app.get_webview_window("overlay") {
        Some(w) => w,
        None => return,
    };

    let overlay_hwnd = match get_hwnd(&window) {
        Some(h) => h,
        None => return,
    };

    unsafe {
        let _ = SetWindowPos(
            overlay_hwnd,
            HWND_TOPMOST,
            x,
            y,
            w as i32,
            h as i32,
            SET_WINDOW_POS_FLAGS(0),
        );
    }

    info!("Overlay geometry set to {}x{} at ({}, {})", w, h, x, y);
}

#[cfg(not(windows))]
pub fn set_window_geometry(app: &AppHandle, _x: i32, _y: i32, w: u32, h: u32) {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: w,
            height: h,
        }));
    }
    info!("set_window_geometry: basic resize on non-Windows");
}

/// Get the client area geometry of a window.
#[cfg(windows)]
pub fn get_window_geometry(hwnd: HWND) -> Option<(i32, i32, u32, u32)> {
    unsafe {
        let mut rect = RECT::default();
        if GetClientRect(hwnd, &mut rect).is_ok() {
            let mut point = POINT { x: 0, y: 0 };
            if ClientToScreen(hwnd, &mut point).as_bool() {
                let w = (rect.right - rect.left) as u32;
                let h = (rect.bottom - rect.top) as u32;
                return Some((point.x, point.y, w, h));
            }
        }
    }
    None
}

/// Check if the overlay window is currently visible.
pub fn is_overlay_visible(app: &AppHandle) -> bool {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("overlay") {
        return window.is_visible().unwrap_or(false);
    }
    false
}
