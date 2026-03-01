/// Window management for the overlay.
///
/// Uses Win32 API calls to:
/// - Set WS_EX_TOOLWINDOW (no taskbar entry)
/// - Position the overlay over the SC window
/// - Show/hide with proper focus management
use log::info;
use tauri::{AppHandle, Emitter, WebviewWindow};

use windows::Win32::Foundation::{BOOL, HWND, POINT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    MONITOR_DEFAULTTOPRIMARY,
};
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, GetWindowLongPtrW, GetWindowThreadProcessId, SetForegroundWindow,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_EXSTYLE, GWL_STYLE, HWND_TOPMOST,
    SET_WINDOW_POS_FLAGS, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, SW_SHOW,
    WS_BORDER, WS_DLGFRAME, WS_EX_CLIENTEDGE, WS_EX_TOOLWINDOW, WS_EX_WINDOWEDGE, WS_THICKFRAME,
};

/// Cached overlay HWND as isize so any thread can call Win32 show/hide directly.
/// Set once during init_overlay_window and never changes.
static OVERLAY_HWND: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

/// Get the cached overlay HWND. Returns 0 if not yet initialised.
pub fn get_overlay_hwnd_raw() -> isize {
    OVERLAY_HWND.load(std::sync::atomic::Ordering::SeqCst)
}

/// Original WNDPROC of the overlay window, saved during subclassing.
static ORIGINAL_WNDPROC: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

/// Global AppHandle stored for the subclass proc to use.
static SUBCLASS_APP: std::sync::OnceLock<AppHandle> = std::sync::OnceLock::new();

/// Custom message used by the hotkey hook to request show/hide.
/// WPARAM: 0 = hide, 1 = show. LPARAM: SC HWND as isize (0 if unknown).
/// WM_APP is 0x8000; we add 42 to avoid collisions with other custom messages.
const WM_HOTKEY_TOGGLE: u32 = 0x8000 + 42;

/// Subclass WNDPROC that intercepts WM_HOTKEY_TOGGLE messages.
unsafe extern "system" fn overlay_subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::{CallWindowProcW, WNDPROC};

    if msg == WM_HOTKEY_TOGGLE {
        let show = wparam.0 != 0;
        let sc_hwnd_val = lparam.0 as isize;

        if let Some(app) = SUBCLASS_APP.get() {
            if show {
                info!("WM_HOTKEY_TOGGLE: showing overlay (main thread)");
                if sc_hwnd_val != 0 {
                    show_overlay(app, sc_hwnd_val);
                } else {
                    use tauri::Manager;
                    if let Some(w) = app.get_webview_window("overlay") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                    let _ = app.emit("overlay-shown", ());
                }
            } else {
                info!("WM_HOTKEY_TOGGLE: hiding overlay (main thread)");
                if sc_hwnd_val != 0 {
                    hide_overlay(app, sc_hwnd_val);
                } else {
                    use tauri::Manager;
                    if let Some(w) = app.get_webview_window("overlay") {
                        let _ = w.hide();
                    }
                }
            }
        }
        return windows::Win32::Foundation::LRESULT(0);
    }

    // Chain to original WNDPROC for all other messages.
    let original = ORIGINAL_WNDPROC.load(std::sync::atomic::Ordering::SeqCst);
    let original_proc: WNDPROC = std::mem::transmute(original);
    CallWindowProcW(original_proc, hwnd, msg, wparam, lparam)
}

/// Post a hotkey toggle message to the overlay window from any thread.
/// This is safe to call from the LL keyboard hook callback — it just enqueues
/// a message, never blocks on the main thread.
/// `show`: true to show, false to hide. `sc_hwnd_val`: SC HWND as isize (0 if unknown).
pub fn post_hotkey_toggle(show: bool, sc_hwnd_val: isize) {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::PostMessageW;

    let overlay = get_overlay_hwnd_raw();
    if overlay == 0 {
        log::warn!("post_hotkey_toggle: overlay HWND not yet cached");
        return;
    }
    let hwnd = crate::platform::hwnd_from_isize(overlay);
    unsafe {
        let _ = PostMessageW(
            hwnd,
            WM_HOTKEY_TOGGLE,
            WPARAM(if show { 1 } else { 0 }),
            LPARAM(sc_hwnd_val),
        );
    }
}

/// Helper: Get the Win32 HWND from a Tauri WebviewWindow.
/// Tauri 2 returns HWND from the `windows` crate version it depends on (0.61),
/// which may differ from our project's `windows` 0.58. Both define
/// HWND(*mut c_void), so we convert via the raw pointer.
fn get_hwnd(window: &WebviewWindow) -> Option<HWND> {
    match window.hwnd() {
        Ok(h) => Some(HWND(h.0)),
        Err(_) => None,
    }
}

/// Initialize the overlay window with WS_EX_TOOLWINDOW style.
/// Also installs a WNDPROC subclass to handle WM_HOTKEY_TOGGLE messages from
/// the LL keyboard hook thread.
/// Call this after the app is ready.
pub fn init_overlay_window(window: &WebviewWindow, app: &AppHandle) {
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
        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            (ex_style | WS_EX_TOOLWINDOW.0 as isize)
                & !(WS_EX_WINDOWEDGE.0 as isize)
                & !(WS_EX_CLIENTEDGE.0 as isize),
        );

        // Strip border-related base styles so no 1px frame is rendered.
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        SetWindowLongPtrW(
            hwnd,
            GWL_STYLE,
            style
                & !(WS_BORDER.0 as isize)
                & !(WS_THICKFRAME.0 as isize)
                & !(WS_DLGFRAME.0 as isize),
        );

        // Force the frame change to take effect immediately.
        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
        );

        // Install our subclass WNDPROC to handle hotkey toggle messages.
        let old_proc = SetWindowLongPtrW(
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::GWLP_WNDPROC,
            overlay_subclass_proc as *const () as isize,
        );
        ORIGINAL_WNDPROC.store(old_proc, std::sync::atomic::Ordering::SeqCst);
    }

    // Store the AppHandle so the subclass proc can access it.
    let _ = SUBCLASS_APP.set(app.clone());

    // Cache the HWND so the hotkey hook thread can PostMessage to it.
    OVERLAY_HWND.store(hwnd.0 as isize, std::sync::atomic::Ordering::SeqCst);

    info!("Overlay window initialized with WS_EX_TOOLWINDOW + subclass WNDPROC");
}

/// Show the overlay window, stealing focus from SC.
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

    let sc_hwnd = crate::platform::hwnd_from_isize(sc_hwnd_val);

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

    let _ = app.emit("overlay-shown", ());
    info!("Overlay shown");
}

/// Hide the overlay and return focus to SC.
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

    let sc_hwnd = crate::platform::hwnd_from_isize(sc_hwnd_val);

    unsafe {
        let _ = ShowWindow(overlay_hwnd, SW_HIDE);
        let _ = SetForegroundWindow(sc_hwnd);
    }

    info!("Overlay hidden, focus returned to SC");
}

/// Position and resize the overlay to match the SC window geometry.
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

/// Get the full screen rect of the monitor that contains the given HWND.
/// Falls back to the primary monitor if the HWND is invalid.
pub fn get_monitor_geometry_for_window(hwnd: HWND) -> (i32, i32, u32, u32) {
    unsafe {
        let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(hmonitor, &mut info).as_bool() {
            let r = info.rcMonitor;
            return (
                r.left,
                r.top,
                (r.right - r.left) as u32,
                (r.bottom - r.top) as u32,
            );
        }
    }
    // Last resort: primary monitor at origin with common resolution
    (0, 0, 1920, 1080)
}

/// Get the full screen rect of the primary monitor.
pub fn get_primary_monitor_geometry() -> (i32, i32, u32, u32) {
    unsafe {
        let hmonitor = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(hmonitor, &mut info).as_bool() {
            let r = info.rcMonitor;
            return (
                r.left,
                r.top,
                (r.right - r.left) as u32,
                (r.bottom - r.top) as u32,
            );
        }
    }
    (0, 0, 1920, 1080)
}

/// Check if the overlay window is currently visible.
pub fn is_overlay_visible(app: &AppHandle) -> bool {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("overlay") {
        return window.is_visible().unwrap_or(false);
    }
    false
}
