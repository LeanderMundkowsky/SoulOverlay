/// Window management for the overlay.
///
/// Uses Win32 API calls to:
/// - Set WS_EX_TOOLWINDOW (no taskbar entry)
/// - Position the overlay on the active monitor
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
    BringWindowToTop, GetForegroundWindow, GetWindowLongPtrW, GetWindowThreadProcessId,
    SetForegroundWindow, SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_EXSTYLE, GWL_STYLE,
    HWND_TOPMOST, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    SWP_SHOWWINDOW, SW_HIDE, WS_BORDER, WS_DLGFRAME, WS_EX_CLIENTEDGE, WS_EX_TOOLWINDOW,
    WS_EX_WINDOWEDGE, WS_THICKFRAME,
};

/// Cached overlay HWND as isize so any thread can call Win32 show/hide directly.
/// Set once during init_overlay_window and never changes.
static OVERLAY_HWND: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

/// Get the cached overlay HWND. Returns 0 if not yet initialised.
pub fn get_overlay_hwnd_raw() -> isize {
    OVERLAY_HWND.load(std::sync::atomic::Ordering::SeqCst)
}

/// HWND of the window that was foreground before we showed the overlay.
/// Restored on hide so focus returns to the previously active application.
static PREV_FOREGROUND_HWND: std::sync::atomic::AtomicIsize =
    std::sync::atomic::AtomicIsize::new(0);

/// HWND of the window that was foreground before our app launched.
/// Used to restore focus after setup so we don't steal it on startup.
static PRE_LAUNCH_HWND: std::sync::atomic::AtomicIsize =
    std::sync::atomic::AtomicIsize::new(0);

/// Original WNDPROC of the overlay window, saved during subclassing.
static ORIGINAL_WNDPROC: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

/// Global AppHandle stored for the subclass proc to use.
static SUBCLASS_APP: std::sync::OnceLock<AppHandle> = std::sync::OnceLock::new();

/// Custom message used by the hotkey hook to request show/hide.
/// WPARAM: 0 = hide, 1 = show. LPARAM: unused.
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

        if let Some(app) = SUBCLASS_APP.get() {
            if show {
                info!("WM_HOTKEY_TOGGLE: showing overlay (main thread)");
                show_overlay(app);
            } else {
                info!("WM_HOTKEY_TOGGLE: hiding overlay (main thread)");
                hide_overlay(app);
            }
        }
        return windows::Win32::Foundation::LRESULT(0);
    }

    // Chain to original WNDPROC for all other messages.
    let original = ORIGINAL_WNDPROC.load(std::sync::atomic::Ordering::SeqCst);
    let original_proc: WNDPROC = std::mem::transmute(original);
    CallWindowProcW(original_proc, hwnd, msg, wparam, lparam)
}

/// Remember whichever window has focus right now.
/// Call once before Tauri creates any windows so we can give focus back after setup.
pub fn capture_pre_launch_foreground() {
    unsafe {
        let fg = GetForegroundWindow();
        if !fg.is_invalid() {
            PRE_LAUNCH_HWND.store(
                crate::platform::hwnd_to_isize(fg),
                std::sync::atomic::Ordering::SeqCst,
            );
        }
    }
}

/// Give focus back to the window that was active before the app launched.
pub fn restore_pre_launch_foreground() {
    let prev = PRE_LAUNCH_HWND.load(std::sync::atomic::Ordering::SeqCst);
    if prev != 0 {
        let hwnd = crate::platform::hwnd_from_isize(prev);
        unsafe {
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

/// Post a hotkey toggle message to the overlay window from any thread.
/// This is safe to call from the LL keyboard hook callback — it just enqueues
/// a message, never blocks on the main thread.
pub fn post_hotkey_toggle(show: bool) {
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
            LPARAM(0),
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
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED | SWP_NOACTIVATE,
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

/// Show the overlay on the currently active monitor and take focus.
pub fn show_overlay(app: &AppHandle) {
    use tauri::Manager;

    // Dismiss the startup hint if still visible
    if let Some(hint) = app.get_webview_window("startup-hint") {
        let _ = hint.close();
    }

    let window = match app.get_webview_window("overlay") {
        Some(w) => w,
        None => return,
    };

    let overlay_hwnd = match get_hwnd(&window) {
        Some(h) => h,
        None => return,
    };

    unsafe {
        // Remember the currently focused window so we can restore it on hide.
        let fg = GetForegroundWindow();
        if fg != overlay_hwnd && !fg.is_invalid() {
            PREV_FOREGROUND_HWND.store(
                crate::platform::hwnd_to_isize(fg),
                std::sync::atomic::Ordering::SeqCst,
            );
        }

        // Position the overlay on whichever monitor contains the foreground window.
        let (mx, my, mw, mh) = if !fg.is_invalid() {
            get_monitor_geometry_for_window(fg)
        } else {
            get_primary_monitor_geometry()
        };
        let _ = SetWindowPos(
            overlay_hwnd,
            HWND_TOPMOST,
            mx,
            my,
            mw as i32,
            mh as i32,
            SWP_SHOWWINDOW,
        );

        // Try to claim foreground. AllowSetForegroundWindow was called in
        // the LL hook callback, so this should usually succeed.
        if !SetForegroundWindow(overlay_hwnd).as_bool() {
            info!("SetForegroundWindow failed, using AttachThreadInput fallback");
            // Fallback: briefly join our input queue to the current foreground
            // thread so SetForegroundWindow is guaranteed to succeed. The
            // attach + detach is immediate — no sustained link is created.
            let fg_now = GetForegroundWindow();
            if !fg_now.is_invalid() && fg_now != overlay_hwnd {
                let fg_thread = GetWindowThreadProcessId(fg_now, None);
                let our_thread = GetCurrentThreadId();
                if fg_thread != 0 && fg_thread != our_thread {
                    let _ = AttachThreadInput(our_thread, fg_thread, BOOL::from(true));
                    let _ = BringWindowToTop(overlay_hwnd);
                    let _ = SetForegroundWindow(overlay_hwnd);
                    let _ = AttachThreadInput(our_thread, fg_thread, BOOL::from(false));
                }
            }
        }
    }

    let _ = app.emit("overlay-shown", ());
    info!("Overlay shown");
}

/// Hide the overlay and return focus to the previously active window.
pub fn hide_overlay(app: &AppHandle) {
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
        let _ = ShowWindow(overlay_hwnd, SW_HIDE);

        // Restore focus to whatever was active before we showed the overlay.
        let prev = PREV_FOREGROUND_HWND.swap(0, std::sync::atomic::Ordering::SeqCst);
        if prev != 0 {
            let prev_hwnd = crate::platform::hwnd_from_isize(prev);
            let _ = SetForegroundWindow(prev_hwnd);
        }
    }

    info!("Overlay hidden");
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
            SWP_NOACTIVATE,
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
