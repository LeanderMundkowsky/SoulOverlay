/// Window management for the overlay.
///
/// On Windows: uses Win32 API calls for WS_EX_TOOLWINDOW, monitor positioning,
/// and WNDPROC subclassing for WM_HOTKEY_TOGGLE message routing.
///
/// On Linux (KDE): uses Tauri's cross-platform WebviewWindow API backed by
/// GTK, which already handles transparent, always-on-top, and skip-taskbar
/// via the window configuration in tauri.conf.json.

// ─── Windows implementation ──────────────────────────────────────────────────

#[cfg(windows)]
mod windows_impl {
    use log::info;
    use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

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
    pub(super) static OVERLAY_HWND: std::sync::atomic::AtomicIsize =
        std::sync::atomic::AtomicIsize::new(0);

    pub fn get_overlay_hwnd_raw() -> isize {
        OVERLAY_HWND.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub(super) static PREV_FOREGROUND_HWND: std::sync::atomic::AtomicIsize =
        std::sync::atomic::AtomicIsize::new(0);

    pub(super) static PRE_LAUNCH_HWND: std::sync::atomic::AtomicIsize =
        std::sync::atomic::AtomicIsize::new(0);

    pub(super) static ORIGINAL_WNDPROC: std::sync::atomic::AtomicIsize =
        std::sync::atomic::AtomicIsize::new(0);

    pub(super) static SUBCLASS_APP: std::sync::OnceLock<AppHandle> = std::sync::OnceLock::new();

    /// Custom message: WPARAM 0 = hide, 1 = show.
    pub(super) const WM_HOTKEY_TOGGLE: u32 = 0x8000 + 42;

    pub(super) unsafe extern "system" fn overlay_subclass_proc(
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

        let original = ORIGINAL_WNDPROC.load(std::sync::atomic::Ordering::SeqCst);
        let original_proc: WNDPROC = std::mem::transmute(original);
        CallWindowProcW(original_proc, hwnd, msg, wparam, lparam)
    }

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

    pub fn restore_pre_launch_foreground() {
        let prev = PRE_LAUNCH_HWND.load(std::sync::atomic::Ordering::SeqCst);
        if prev != 0 {
            let hwnd = crate::platform::hwnd_from_isize(prev);
            unsafe {
                let _ = SetForegroundWindow(hwnd);
            }
        }
    }

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

    fn get_hwnd(window: &WebviewWindow) -> Option<HWND> {
        match window.hwnd() {
            Ok(h) => Some(HWND(h.0)),
            Err(_) => None,
        }
    }

    pub fn init_overlay_window(window: &WebviewWindow, app: &AppHandle) {
        let hwnd = match get_hwnd(window) {
            Some(h) => h,
            None => {
                log::error!("Failed to get overlay HWND for init");
                return;
            }
        };

        unsafe {
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(
                hwnd,
                GWL_EXSTYLE,
                (ex_style | WS_EX_TOOLWINDOW.0 as isize)
                    & !(WS_EX_WINDOWEDGE.0 as isize)
                    & !(WS_EX_CLIENTEDGE.0 as isize),
            );

            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            SetWindowLongPtrW(
                hwnd,
                GWL_STYLE,
                style
                    & !(WS_BORDER.0 as isize)
                    & !(WS_THICKFRAME.0 as isize)
                    & !(WS_DLGFRAME.0 as isize),
            );

            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED | SWP_NOACTIVATE,
            );

            let old_proc = SetWindowLongPtrW(
                hwnd,
                windows::Win32::UI::WindowsAndMessaging::GWLP_WNDPROC,
                overlay_subclass_proc as *const () as isize,
            );
            ORIGINAL_WNDPROC.store(old_proc, std::sync::atomic::Ordering::SeqCst);
        }

        let _ = SUBCLASS_APP.set(app.clone());
        OVERLAY_HWND.store(hwnd.0 as isize, std::sync::atomic::Ordering::SeqCst);

        info!("Overlay window initialized with WS_EX_TOOLWINDOW + subclass WNDPROC");
    }

    pub fn show_overlay(app: &AppHandle) {
        let window = match app.get_webview_window("overlay") {
            Some(w) => w,
            None => return,
        };

        // Dismiss the startup hint if still visible
        if let Some(hint) = app.get_webview_window("startup-hint") {
            let _ = hint.close();
        }

        let overlay_hwnd = match get_hwnd(&window) {
            Some(h) => h,
            None => return,
        };

        unsafe {
            let fg = GetForegroundWindow();
            if fg != overlay_hwnd && !fg.is_invalid() {
                PREV_FOREGROUND_HWND.store(
                    crate::platform::hwnd_to_isize(fg),
                    std::sync::atomic::Ordering::SeqCst,
                );
            }

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

            if !SetForegroundWindow(overlay_hwnd).as_bool() {
                info!("SetForegroundWindow failed, using AttachThreadInput fallback");
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

    pub fn hide_overlay(app: &AppHandle) {
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

            let prev = PREV_FOREGROUND_HWND.swap(0, std::sync::atomic::Ordering::SeqCst);
            if prev != 0 {
                let prev_hwnd = crate::platform::hwnd_from_isize(prev);
                let _ = SetForegroundWindow(prev_hwnd);
            }
        }

        info!("Overlay hidden");
    }

    pub fn set_window_geometry(app: &AppHandle, x: i32, y: i32, w: u32, h: u32) {
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
        (0, 0, 1920, 1080)
    }

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
}

// ─── Linux / KDE implementation ──────────────────────────────────────────────
//
// Tauri's GTK backend already handles transparent, always-on-top, skip-taskbar,
// and decorations via tauri.conf.json. Show/hide use the cross-platform
// WebviewWindow API. A static AppHandle is stored at init time so that
// post_hotkey_toggle (called from the hotkey thread) can access the window.

#[cfg(not(windows))]
mod linux_impl {
    use log::info;
    use tauri::{AppHandle, Emitter, Manager};

    /// Stored AppHandle so `post_hotkey_toggle` can access the window from any thread.
    static LINUX_APP: std::sync::OnceLock<AppHandle> = std::sync::OnceLock::new();

    pub fn post_hotkey_toggle(show: bool) {
        if let Some(app) = LINUX_APP.get() {
            if show {
                show_overlay(app);
            } else {
                hide_overlay(app);
            }
        } else {
            log::warn!("post_hotkey_toggle: app handle not yet stored (Linux)");
        }
    }

    pub fn init_overlay_window(
        _window: &tauri::WebviewWindow,
        app: &AppHandle,
    ) {
        let _ = LINUX_APP.set(app.clone());
        info!("Overlay window initialized (Linux/KDE — Tauri GTK backend)");
    }

    pub fn show_overlay(app: &AppHandle) {
        // Dismiss the startup hint if still visible
        if let Some(hint) = app.get_webview_window("startup-hint") {
            let _ = hint.close();
        }

        if let Some(window) = app.get_webview_window("overlay") {
            // Position to cover the primary monitor before showing
            if let Ok(Some(monitor)) = app.primary_monitor() {
                let pos = monitor.position();
                let size = monitor.size();
                let _ = window.set_position(tauri::PhysicalPosition::new(pos.x, pos.y));
                let _ = window.set_size(tauri::PhysicalSize::new(size.width, size.height));
            }
            let _ = window.show();
            let _ = window.set_focus();
        }
        let _ = app.emit("overlay-shown", ());
        info!("Overlay shown (Linux)");
    }

    pub fn hide_overlay(app: &AppHandle) {
        if let Some(window) = app.get_webview_window("overlay") {
            let _ = window.hide();
        }
        info!("Overlay hidden (Linux)");
    }

    pub fn set_window_geometry(app: &AppHandle, x: i32, y: i32, w: u32, h: u32) {
        if let Some(window) = app.get_webview_window("overlay") {
            let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
            let _ = window.set_size(tauri::PhysicalSize::new(w, h));
        }
        info!("Overlay geometry set to {}x{} at ({}, {}) (Linux)", w, h, x, y);
    }

    pub fn get_primary_monitor_geometry() -> (i32, i32, u32, u32) {
        // Best-effort from the stored handle; falls back to 1920×1080.
        if let Some(app) = LINUX_APP.get() {
            if let Ok(Some(monitor)) = app.primary_monitor() {
                let pos = monitor.position();
                let size = monitor.size();
                return (pos.x, pos.y, size.width, size.height);
            }
        }
        (0, 0, 1920, 1080)
    }
}

// ─── Public API (unified, platform-dispatched) ───────────────────────────────

use tauri::{AppHandle, Manager, WebviewWindow};

/// Remember whichever window has focus right now (Windows only).
/// Call once before Tauri creates any windows so we can restore focus after setup.
#[cfg(windows)]
pub fn capture_pre_launch_foreground() {
    windows_impl::capture_pre_launch_foreground();
}

/// Give focus back to the window that was active before the app launched (Windows only).
pub fn restore_pre_launch_foreground() {
    #[cfg(windows)]
    windows_impl::restore_pre_launch_foreground();
}

/// Post a hotkey toggle to the overlay from any thread (safe to call from hotkey callback).
pub fn post_hotkey_toggle(show: bool) {
    #[cfg(windows)]
    windows_impl::post_hotkey_toggle(show);

    #[cfg(not(windows))]
    linux_impl::post_hotkey_toggle(show);
}

/// Initialize platform-specific overlay window properties.
pub fn init_overlay_window(window: &WebviewWindow, app: &AppHandle) {
    #[cfg(windows)]
    windows_impl::init_overlay_window(window, app);

    #[cfg(not(windows))]
    linux_impl::init_overlay_window(window, app);
}

/// Show the overlay and take focus.
pub fn show_overlay(app: &AppHandle) {
    #[cfg(windows)]
    windows_impl::show_overlay(app);

    #[cfg(not(windows))]
    linux_impl::show_overlay(app);
}

/// Hide the overlay and return focus to the previously active window.
pub fn hide_overlay(app: &AppHandle) {
    #[cfg(windows)]
    windows_impl::hide_overlay(app);

    #[cfg(not(windows))]
    linux_impl::hide_overlay(app);
}

/// Position and resize the overlay window.
pub fn set_window_geometry(app: &AppHandle, x: i32, y: i32, w: u32, h: u32) {
    #[cfg(windows)]
    windows_impl::set_window_geometry(app, x, y, w, h);

    #[cfg(not(windows))]
    linux_impl::set_window_geometry(app, x, y, w, h);
}

/// Get the full screen rect of the primary monitor.
pub fn get_primary_monitor_geometry() -> (i32, i32, u32, u32) {
    #[cfg(windows)]
    return windows_impl::get_primary_monitor_geometry();

    #[cfg(not(windows))]
    return linux_impl::get_primary_monitor_geometry();
}

/// Get the full screen rect of the monitor containing the given HWND (Windows only).
/// On Linux, returns the primary monitor geometry.
#[cfg(windows)]
pub fn get_monitor_geometry_for_window(
    hwnd: windows::Win32::Foundation::HWND,
) -> (i32, i32, u32, u32) {
    windows_impl::get_monitor_geometry_for_window(hwnd)
}

/// Check if the overlay window is currently visible.
pub fn is_overlay_visible(app: &AppHandle) -> bool {
    if let Some(window) = app.get_webview_window("overlay") {
        return window.is_visible().unwrap_or(false);
    }
    false
}

