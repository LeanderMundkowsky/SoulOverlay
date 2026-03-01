/// Hotkey management using a WH_KEYBOARD_LL low-level keyboard hook.
///
/// `RegisterHotKey` / tauri-plugin-global-shortcut do NOT fire when a full-screen
/// or exclusive-foreground application (like Star Citizen) has focus.
/// A low-level keyboard hook is injected at the OS level and fires regardless of
/// which window is in the foreground, matching how Steam/Discord overlays work.
use log::{info, warn};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

use crate::game_tracker::SharedGameState;

/// Handle to the running hook thread. Dropping it stops the thread.
pub struct HookHandle {
    thread_id: u32,
}

impl Drop for HookHandle {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            // Post WM_QUIT to the hook thread's message loop so it exits cleanly.
            windows::Win32::UI::WindowsAndMessaging::PostThreadMessageW(
                self.thread_id,
                windows::Win32::UI::WindowsAndMessaging::WM_QUIT,
                windows::Win32::Foundation::WPARAM(0),
                windows::Win32::Foundation::LPARAM(0),
            )
            .ok();
        }
        let _ = self.thread_id; // suppress unused on non-windows
    }
}

/// Global slot for the current hook state, shared between the hook callback and
/// the thread that owns it.  We need a static because `SetWindowsHookExW` only
/// accepts a bare fn pointer, not a closure.
#[cfg(windows)]
static HOOK_STATE: std::sync::OnceLock<Arc<Mutex<HookState>>> = std::sync::OnceLock::new();

#[cfg(windows)]
struct HookState {
    app: AppHandle,
    game_state: SharedGameState,
    vk: u32,
    require_alt: bool,
    require_ctrl: bool,
    require_shift: bool,
}

/// Parse a hotkey string like "Alt+Shift+S" or "Ctrl+Alt+F9" into a
/// (virtual_key_code, requires_ctrl, requires_alt, requires_shift) tuple.
/// Returns None if the string is empty or the key token is unrecognised.
fn parse_hotkey(hotkey: &str) -> Option<(u32, bool, bool, bool)> {
    #[cfg(windows)]
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut vk: Option<u32> = None;

    for part in hotkey.split('+') {
        match part.trim().to_lowercase().as_str() {
            "ctrl" | "control" => ctrl = true,
            "alt" => alt = true,
            "shift" => shift = true,
            token => {
                #[cfg(windows)]
                {
                    let code: u32 = match token {
                        "a" => VK_A.0 as u32,
                        "b" => VK_B.0 as u32,
                        "c" => VK_C.0 as u32,
                        "d" => VK_D.0 as u32,
                        "e" => VK_E.0 as u32,
                        "f" => VK_F.0 as u32,
                        "g" => VK_G.0 as u32,
                        "h" => VK_H.0 as u32,
                        "i" => VK_I.0 as u32,
                        "j" => VK_J.0 as u32,
                        "k" => VK_K.0 as u32,
                        "l" => VK_L.0 as u32,
                        "m" => VK_M.0 as u32,
                        "n" => VK_N.0 as u32,
                        "o" => VK_O.0 as u32,
                        "p" => VK_P.0 as u32,
                        "q" => VK_Q.0 as u32,
                        "r" => VK_R.0 as u32,
                        "s" => VK_S.0 as u32,
                        "t" => VK_T.0 as u32,
                        "u" => VK_U.0 as u32,
                        "v" => VK_V.0 as u32,
                        "w" => VK_W.0 as u32,
                        "x" => VK_X.0 as u32,
                        "y" => VK_Y.0 as u32,
                        "z" => VK_Z.0 as u32,
                        "0" => VK_0.0 as u32,
                        "1" => VK_1.0 as u32,
                        "2" => VK_2.0 as u32,
                        "3" => VK_3.0 as u32,
                        "4" => VK_4.0 as u32,
                        "5" => VK_5.0 as u32,
                        "6" => VK_6.0 as u32,
                        "7" => VK_7.0 as u32,
                        "8" => VK_8.0 as u32,
                        "9" => VK_9.0 as u32,
                        "f1" => VK_F1.0 as u32,
                        "f2" => VK_F2.0 as u32,
                        "f3" => VK_F3.0 as u32,
                        "f4" => VK_F4.0 as u32,
                        "f5" => VK_F5.0 as u32,
                        "f6" => VK_F6.0 as u32,
                        "f7" => VK_F7.0 as u32,
                        "f8" => VK_F8.0 as u32,
                        "f9" => VK_F9.0 as u32,
                        "f10" => VK_F10.0 as u32,
                        "f11" => VK_F11.0 as u32,
                        "f12" => VK_F12.0 as u32,
                        "space" => VK_SPACE.0 as u32,
                        "tab" => VK_TAB.0 as u32,
                        "escape" | "esc" => VK_ESCAPE.0 as u32,
                        "insert" => VK_INSERT.0 as u32,
                        "delete" => VK_DELETE.0 as u32,
                        "home" => VK_HOME.0 as u32,
                        "end" => VK_END.0 as u32,
                        "pageup" => VK_PRIOR.0 as u32,
                        "pagedown" => VK_NEXT.0 as u32,
                        "up" => VK_UP.0 as u32,
                        "down" => VK_DOWN.0 as u32,
                        "left" => VK_LEFT.0 as u32,
                        "right" => VK_RIGHT.0 as u32,
                        _ => {
                            warn!("Unrecognised key token '{}' in hotkey '{}'", token, hotkey);
                            return None;
                        }
                    };
                    vk = Some(code);
                }
                #[cfg(not(windows))]
                {
                    // On non-Windows we just need something non-None so the parse succeeds.
                    let _ = token;
                    vk = Some(0);
                }
            }
        }
    }

    vk.map(|v| (v, ctrl, alt, shift))
}

// ---------------------------------------------------------------------------
// Windows implementation
// ---------------------------------------------------------------------------

#[cfg(windows)]
pub fn register_hotkey(
    app: &AppHandle,
    hotkey: &str,
    game_state: SharedGameState,
) -> Result<HookHandle, String> {
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_MENU, VK_RCONTROL,
        VK_RMENU, VK_RSHIFT, VK_SHIFT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
        KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
    };

    let (vk, require_ctrl, require_alt, require_shift) =
        parse_hotkey(hotkey).ok_or_else(|| format!("Could not parse hotkey: '{}'", hotkey))?;

    // Store state in the global slot so the bare-fn callback can access it.
    // If the OnceLock is already set we replace the inner value.
    let state = Arc::new(Mutex::new(HookState {
        app: app.clone(),
        game_state,
        vk,
        require_alt,
        require_ctrl,
        require_shift,
    }));

    // HOOK_STATE may already be initialised from a previous call; update in place.
    match HOOK_STATE.get() {
        Some(existing) => {
            *existing.lock().unwrap() = HookState {
                app: app.clone(),
                game_state: {
                    // We can't move game_state twice; re-extract from the Arc we just made.
                    let s = state.lock().unwrap();
                    s.game_state.clone()
                },
                vk,
                require_alt,
                require_ctrl,
                require_shift,
            };
            // Also update app since we have a new AppHandle.
            existing.lock().unwrap().app = app.clone();
        }
        None => {
            let _ = HOOK_STATE.set(state);
        }
    }

    // The low-level keyboard hook callback (bare fn — no captures allowed).
    unsafe extern "system" fn ll_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        use windows::Win32::UI::WindowsAndMessaging::HC_ACTION;

        if code == HC_ACTION as i32 {
            let msg = wparam.0 as u32;
            if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);

                if let Some(slot) = HOOK_STATE.get() {
                    let s = slot.lock().unwrap();

                    if kb.vkCode == s.vk {
                        // Check modifier state via GetAsyncKeyState (high bit = pressed).
                        let ctrl_down = (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16) & 0x8000
                            != 0
                            || (GetAsyncKeyState(VK_LCONTROL.0 as i32) as u16) & 0x8000 != 0
                            || (GetAsyncKeyState(VK_RCONTROL.0 as i32) as u16) & 0x8000 != 0;
                        let alt_down = (GetAsyncKeyState(VK_MENU.0 as i32) as u16) & 0x8000 != 0
                            || (GetAsyncKeyState(VK_LMENU.0 as i32) as u16) & 0x8000 != 0
                            || (GetAsyncKeyState(VK_RMENU.0 as i32) as u16) & 0x8000 != 0;
                        let shift_down = (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16) & 0x8000 != 0
                            || (GetAsyncKeyState(VK_LSHIFT.0 as i32) as u16) & 0x8000 != 0
                            || (GetAsyncKeyState(VK_RSHIFT.0 as i32) as u16) & 0x8000 != 0;

                        let modifiers_match = ctrl_down == s.require_ctrl
                            && alt_down == s.require_alt
                            && shift_down == s.require_shift;

                        if modifiers_match {
                            let app = s.app.clone();
                            let game_state = s.game_state.clone();
                            drop(s); // release lock before calling into Tauri
                            handle_hotkey_press(&app, &game_state);
                        }
                    }
                }
            }
        }

        CallNextHookEx(HHOOK::default(), code, wparam, lparam)
    }

    // Channel to receive the thread ID after the hook is installed.
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

            // Pump messages — required for the hook callback to fire.
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                // No TranslateMessage/DispatchMessage needed for LL hooks.
            }

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

#[cfg(windows)]
pub fn unregister_hotkey(handle: HookHandle) {
    // Dropping the handle posts WM_QUIT to the hook thread.
    drop(handle);
    info!("Unregistered LL keyboard hook");
}

// ---------------------------------------------------------------------------
// Non-Windows stubs
// ---------------------------------------------------------------------------

#[cfg(not(windows))]
pub fn register_hotkey(
    _app: &AppHandle,
    hotkey: &str,
    _game_state: SharedGameState,
) -> Result<HookHandle, String> {
    info!("register_hotkey: no-op on non-Windows (hotkey: {})", hotkey);
    Ok(HookHandle { thread_id: 0 })
}

#[cfg(not(windows))]
pub fn unregister_hotkey(_handle: HookHandle) {
    info!("unregister_hotkey: no-op on non-Windows");
}

// ---------------------------------------------------------------------------
// Shared toggle logic (same on all platforms, guarded internally)
// ---------------------------------------------------------------------------

fn handle_hotkey_press(app: &AppHandle, game_state: &SharedGameState) {
    let state = game_state.lock().unwrap();
    let is_visible = crate::window::is_overlay_visible(app);

    if is_visible {
        #[cfg(windows)]
        {
            if let Some(sc_hwnd_val) = state.sc_hwnd {
                drop(state);
                crate::window::hide_overlay(app, sc_hwnd_val);
            } else {
                drop(state);
                use tauri::Manager;
                if let Some(w) = app.get_webview_window("overlay") {
                    let _ = w.hide();
                }
            }
        }
        #[cfg(not(windows))]
        {
            drop(state);
            crate::window::hide_overlay(app, ());
        }
    } else {
        #[cfg(windows)]
        {
            if let Some(sc_hwnd_val) = state.sc_hwnd {
                drop(state);
                crate::window::show_overlay(app, sc_hwnd_val);
            } else {
                info!("Hotkey pressed but SC not detected — showing overlay anyway");
                drop(state);
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
            crate::window::show_overlay(app, ());
        }
    }
}
