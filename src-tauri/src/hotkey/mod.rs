/// Hotkey management.
///
/// On Windows: uses a `WH_KEYBOARD_LL` low-level keyboard hook that fires
/// even when a full-screen or exclusive-foreground application has focus,
/// matching how Steam/Discord overlays work.
///
/// On Linux (KDE and other desktops): uses `tauri-plugin-global-shortcut`
/// which leverages XInput2 on X11. This works reliably with Star Citizen
/// running under Proton/Wine, as Wine does not hold exclusive keyboard focus.
mod keymap;

/// Tracks whether the overlay is currently visible. Maintained by the hotkey
/// handler so the callback can read it without touching Tauri APIs (which
/// require the main thread on Windows).
/// Starts as `false` because the overlay is hidden on launch.
pub(crate) static OVERLAY_VISIBLE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Notify the hotkey module that the overlay was hidden by means other than
/// the hotkey (e.g. ESC key from the frontend). Keeps OVERLAY_VISIBLE in sync.
pub fn notify_overlay_hidden() {
    OVERLAY_VISIBLE.store(false, std::sync::atomic::Ordering::SeqCst);
}

/// Notify the hotkey module that the overlay was shown by means other than
/// the hotkey. Keeps OVERLAY_VISIBLE in sync.
pub fn notify_overlay_shown() {
    OVERLAY_VISIBLE.store(true, std::sync::atomic::Ordering::SeqCst);
}

// ─── Windows implementation (WH_KEYBOARD_LL) ─────────────────────────────────

#[cfg(windows)]
mod windows_impl {
    use super::keymap;
    use log::{info, warn};
    use std::sync::{Arc, Mutex};

    pub struct HookHandle {
        pub thread_id: u32,
    }

    impl Drop for HookHandle {
        fn drop(&mut self) {
            unsafe {
                windows::Win32::UI::WindowsAndMessaging::PostThreadMessageW(
                    self.thread_id,
                    windows::Win32::UI::WindowsAndMessaging::WM_QUIT,
                    windows::Win32::Foundation::WPARAM(0),
                    windows::Win32::Foundation::LPARAM(0),
                )
                .ok();
            }
        }
    }

    static HOOK_GUARD: std::sync::OnceLock<Arc<Mutex<()>>> = std::sync::OnceLock::new();

    static MOD_CTRL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    static MOD_ALT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    static MOD_SHIFT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

    static TARGET_VK: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    static TARGET_MODS: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

    fn parse_hotkey(hotkey: &str) -> Option<(u32, bool, bool, bool)> {
        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;
        let mut vk: Option<u32> = None;

        for part in hotkey.split('+') {
            match part.trim().to_lowercase().as_str() {
                "ctrl" | "control" => ctrl = true,
                "alt" => alt = true,
                "shift" => shift = true,
                token => match keymap::token_to_vk(token) {
                    Some(code) => vk = Some(code),
                    None => {
                        warn!("Unrecognised key token '{}' in hotkey '{}'", token, hotkey);
                        return None;
                    }
                },
            }
        }

        vk.map(|v| (v, ctrl, alt, shift))
    }

    pub fn register_hotkey(
        hotkey: &str,
        _app: &tauri::AppHandle,
    ) -> Result<HookHandle, String> {
        use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_MENU, VK_RCONTROL, VK_RMENU,
            VK_RSHIFT, VK_SHIFT,
        };
        use windows::Win32::UI::WindowsAndMessaging::{
            CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
            KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
        };

        let (vk, require_ctrl, require_alt, require_shift) = parse_hotkey(hotkey)
            .ok_or_else(|| format!("Could not parse hotkey: '{}'", hotkey))?;

        TARGET_VK.store(vk, std::sync::atomic::Ordering::SeqCst);
        let mods: u8 =
            (require_ctrl as u8) | ((require_alt as u8) << 1) | ((require_shift as u8) << 2);
        TARGET_MODS.store(mods, std::sync::atomic::Ordering::SeqCst);

        MOD_CTRL.store(false, std::sync::atomic::Ordering::SeqCst);
        MOD_ALT.store(false, std::sync::atomic::Ordering::SeqCst);
        MOD_SHIFT.store(false, std::sync::atomic::Ordering::SeqCst);

        let _ = HOOK_GUARD.get_or_init(|| Arc::new(Mutex::new(())));

        unsafe extern "system" fn ll_hook(
            code: i32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT {
            use std::sync::atomic::Ordering::SeqCst;
            use windows::Win32::UI::WindowsAndMessaging::HC_ACTION;

            if code == HC_ACTION as i32 {
                let msg = wparam.0 as u32;
                let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
                let vk = kb.vkCode;

                let is_ctrl = vk == VK_CONTROL.0 as u32
                    || vk == VK_LCONTROL.0 as u32
                    || vk == VK_RCONTROL.0 as u32;
                let is_alt = vk == VK_MENU.0 as u32
                    || vk == VK_LMENU.0 as u32
                    || vk == VK_RMENU.0 as u32;
                let is_shift = vk == VK_SHIFT.0 as u32
                    || vk == VK_LSHIFT.0 as u32
                    || vk == VK_RSHIFT.0 as u32;

                let pressed = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;

                if is_ctrl {
                    MOD_CTRL.store(pressed, SeqCst);
                } else if is_alt {
                    MOD_ALT.store(pressed, SeqCst);
                } else if is_shift {
                    MOD_SHIFT.store(pressed, SeqCst);
                }

                let target_vk = TARGET_VK.load(SeqCst);
                if pressed && vk == target_vk {
                    let mods = TARGET_MODS.load(SeqCst);
                    let need_ctrl = (mods & 1) != 0;
                    let need_alt = (mods & 2) != 0;
                    let need_shift = (mods & 4) != 0;

                    let got_ctrl = MOD_CTRL.load(SeqCst);
                    let got_alt = MOD_ALT.load(SeqCst);
                    let got_shift = MOD_SHIFT.load(SeqCst);

                    let modifiers_match = got_ctrl == need_ctrl
                        && got_alt == need_alt
                        && got_shift == need_shift;

                    if !modifiers_match {
                        info!(
                            "LL hook: VK matched but mods wrong (ctrl={}/{}, alt={}/{}, shift={}/{})",
                            got_ctrl, need_ctrl, got_alt, need_alt, got_shift, need_shift
                        );
                    }

                    if modifiers_match {
                        if let Some(guard) = HOOK_GUARD.get() {
                            if let Ok(_lock) = guard.try_lock() {
                                let _ = windows::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow(
                                    windows::Win32::System::Threading::GetCurrentProcessId(),
                                );

                                let was_visible = super::OVERLAY_VISIBLE
                                    .fetch_xor(true, SeqCst);
                                info!("LL hook: hotkey matched, was_visible={}", was_visible);
                                handle_hotkey_press(was_visible);

                                return LRESULT(1);
                            } else {
                                info!("LL hook: hotkey matched but HOOK_GUARD locked, skipping");
                            }
                        }
                    }
                }
            }

            CallNextHookEx(HHOOK::default(), code, wparam, lparam)
        }

        let (tx, rx) = std::sync::mpsc::channel::<Result<u32, String>>();

        std::thread::spawn(move || {
            unsafe {
                let hook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(ll_hook), None, 0) {
                    Ok(h) => h,
                    Err(e) => {
                        let _ = tx.send(Err(format!("SetWindowsHookExW failed: {}", e)));
                        return;
                    }
                };

                let thread_id = windows::Win32::System::Threading::GetCurrentThreadId();
                let _ = tx.send(Ok(thread_id));
                info!("LL keyboard hook installed on thread {}", thread_id);

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).as_bool() {}

                UnhookWindowsHookEx(hook).ok();
                info!("LL keyboard hook removed");
            }
        });

        let thread_id = rx
            .recv()
            .map_err(|_| "Hook thread died before sending thread ID".to_string())??;

        info!("Registered LL keyboard hook for hotkey: {}", hotkey);
        Ok(HookHandle { thread_id })
    }

    fn handle_hotkey_press(was_visible: bool) {
        let show = !was_visible;
        info!("Hotkey: posting WM_HOTKEY_TOGGLE (show={})", show);
        crate::window::post_hotkey_toggle(show);
    }
}

// ─── Linux implementation (tauri-plugin-global-shortcut) ─────────────────────

#[cfg(target_os = "linux")]
mod linux_impl {
    use log::info;
    use tauri::AppHandle;
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    pub struct HookHandle {
        hotkey: String,
        app: AppHandle,
    }

    impl Drop for HookHandle {
        fn drop(&mut self) {
            if let Err(e) = self.app.global_shortcut().unregister(self.hotkey.as_str()) {
                log::warn!("Failed to unregister Linux shortcut '{}': {}", self.hotkey, e);
            } else {
                info!("Linux global shortcut '{}' unregistered", self.hotkey);
            }
        }
    }

    pub fn register_hotkey(hotkey: &str, app: &AppHandle) -> Result<HookHandle, String> {
        // Unregister any previously registered shortcut with this key string first
        // (best-effort, ignore errors in case it wasn't registered).
        let _ = app.global_shortcut().unregister(hotkey);

        app.global_shortcut()
            .on_shortcut(hotkey, move |_app, _shortcut, _event| {
                use std::sync::atomic::Ordering::SeqCst;
                let was_visible = super::OVERLAY_VISIBLE.fetch_xor(true, SeqCst);
                let show = !was_visible;
                info!("Global shortcut fired, was_visible={}", was_visible);
                crate::window::post_hotkey_toggle(show);
            })
            .map_err(|e| format!("Failed to register global shortcut '{}': {}", hotkey, e))?;

        info!("Registered Linux global shortcut: {}", hotkey);
        Ok(HookHandle {
            hotkey: hotkey.to_string(),
            app: app.clone(),
        })
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Opaque handle to a registered hotkey. Dropping it unregisters the hotkey.
#[allow(dead_code)]
pub struct HookHandle(
    #[cfg(windows)] windows_impl::HookHandle,
    #[cfg(target_os = "linux")] linux_impl::HookHandle,
    /// Placeholder so the struct is valid on other platforms (e.g. macOS).
    #[cfg(not(any(windows, target_os = "linux")))] (),
);

/// Register the global overlay toggle hotkey.
///
/// The `app` handle is used on Linux to register with `tauri-plugin-global-shortcut`
/// and is ignored on Windows (which uses a low-level keyboard hook instead).
///
/// Returns a `HookHandle` that unregisters the hotkey when dropped.
pub fn register_hotkey(hotkey: &str, app: &tauri::AppHandle) -> Result<HookHandle, String> {
    #[cfg(windows)]
    return windows_impl::register_hotkey(hotkey, app).map(HookHandle);

    #[cfg(target_os = "linux")]
    return linux_impl::register_hotkey(hotkey, app).map(HookHandle);

    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _ = (hotkey, app);
        log::warn!("Global hotkey not supported on this platform");
        Ok(HookHandle(()))
    }
}

