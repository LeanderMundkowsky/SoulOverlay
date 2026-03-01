/// Hotkey management using a WH_KEYBOARD_LL low-level keyboard hook.
///
/// `RegisterHotKey` / tauri-plugin-global-shortcut do NOT fire when a full-screen
/// or exclusive-foreground application (like Star Citizen) has focus.
/// A low-level keyboard hook is injected at the OS level and fires regardless of
/// which window is in the foreground, matching how Steam/Discord overlays work.
mod keymap;

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

/// Tracks whether the overlay is currently visible. Maintained by the hotkey
/// handler so the LL hook callback can read it without touching Tauri APIs
/// (which require the main thread).
/// Starts as `false` because the overlay is hidden on launch.
#[cfg(windows)]
static OVERLAY_VISIBLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Self-tracked modifier key states. Updated by the LL hook itself on every
/// key event — avoids GetAsyncKeyState races when the overlay is focused.
#[cfg(windows)]
static MOD_CTRL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[cfg(windows)]
static MOD_ALT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[cfg(windows)]
static MOD_SHIFT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Target VK code stored as an atomic so the hook can fast-reject without
/// locking HOOK_STATE. Avoids mutex contention inside the time-critical callback.
#[cfg(windows)]
static TARGET_VK: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
/// Required modifiers as packed bits: bit 0 = ctrl, bit 1 = alt, bit 2 = shift.
#[cfg(windows)]
static TARGET_MODS: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

#[cfg(windows)]
struct HookState {
    app: AppHandle,
    game_state: SharedGameState,
}

/// Parse a hotkey string like "Alt+Shift+S" or "Ctrl+Alt+F9" into a
/// (virtual_key_code, requires_ctrl, requires_alt, requires_shift) tuple.
/// Returns None if the string is empty or the key token is unrecognised.
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
        VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_MENU, VK_RCONTROL, VK_RMENU, VK_RSHIFT,
        VK_SHIFT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
        KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
    };

    let (vk, require_ctrl, require_alt, require_shift) =
        parse_hotkey(hotkey).ok_or_else(|| format!("Could not parse hotkey: '{}'", hotkey))?;

    // Store VK and modifier requirements in atomics for lock-free access in the hook.
    TARGET_VK.store(vk, std::sync::atomic::Ordering::SeqCst);
    let mods: u8 = (require_ctrl as u8) | ((require_alt as u8) << 1) | ((require_shift as u8) << 2);
    TARGET_MODS.store(mods, std::sync::atomic::Ordering::SeqCst);

    // Reset tracked modifier state — the old hook may have seen a key-down
    // for a modifier that was part of the previous hotkey combo, and the
    // corresponding key-up was lost during re-registration.
    MOD_CTRL.store(false, std::sync::atomic::Ordering::SeqCst);
    MOD_ALT.store(false, std::sync::atomic::Ordering::SeqCst);
    MOD_SHIFT.store(false, std::sync::atomic::Ordering::SeqCst);

    // Store state in the global slot so the bare-fn callback can access it.
    // If the OnceLock is already set we replace the inner value.
    let state = Arc::new(Mutex::new(HookState {
        app: app.clone(),
        game_state,
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
            };
        }
        None => {
            let _ = HOOK_STATE.set(state);
        }
    }

    // The low-level keyboard hook callback (bare fn — no captures allowed).
    unsafe extern "system" fn ll_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        use std::sync::atomic::Ordering::SeqCst;
        use windows::Win32::UI::WindowsAndMessaging::HC_ACTION;

        if code == HC_ACTION as i32 {
            let msg = wparam.0 as u32;
            let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
            let vk = kb.vkCode;

            // Track modifier key state ourselves — avoids GetAsyncKeyState races
            // when the overlay window is focused.
            let is_ctrl = vk == VK_CONTROL.0 as u32
                || vk == VK_LCONTROL.0 as u32
                || vk == VK_RCONTROL.0 as u32;
            let is_alt =
                vk == VK_MENU.0 as u32 || vk == VK_LMENU.0 as u32 || vk == VK_RMENU.0 as u32;
            let is_shift =
                vk == VK_SHIFT.0 as u32 || vk == VK_LSHIFT.0 as u32 || vk == VK_RSHIFT.0 as u32;

            let pressed = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;

            if is_ctrl {
                MOD_CTRL.store(pressed, SeqCst);
            } else if is_alt {
                MOD_ALT.store(pressed, SeqCst);
            } else if is_shift {
                MOD_SHIFT.store(pressed, SeqCst);
            }

            // Fast path: check VK and modifiers via atomics — no mutex needed.
            let target_vk = TARGET_VK.load(SeqCst);
            if pressed && vk == target_vk {
                let mods = TARGET_MODS.load(SeqCst);
                let need_ctrl = (mods & 1) != 0;
                let need_alt = (mods & 2) != 0;
                let need_shift = (mods & 4) != 0;

                let got_ctrl = MOD_CTRL.load(SeqCst);
                let got_alt = MOD_ALT.load(SeqCst);
                let got_shift = MOD_SHIFT.load(SeqCst);

                let modifiers_match =
                    got_ctrl == need_ctrl && got_alt == need_alt && got_shift == need_shift;

                if !modifiers_match {
                    info!(
                        "LL hook: VK matched but mods wrong (ctrl={}/{}, alt={}/{}, shift={}/{})",
                        got_ctrl, need_ctrl, got_alt, need_alt, got_shift, need_shift
                    );
                }

                if modifiers_match {
                    // Only lock the mutex to grab app + game_state handles.
                    if let Some(slot) = HOOK_STATE.get() {
                        if let Ok(s) = slot.try_lock() {
                            let app = s.app.clone();
                            let game_state = s.game_state.clone();
                            drop(s);

                            let was_visible = OVERLAY_VISIBLE.fetch_xor(true, SeqCst);
                            info!("LL hook: hotkey matched, was_visible={}", was_visible);
                            handle_hotkey_press(&app, &game_state, was_visible);

                            // Swallow the keypress — do NOT pass it to the game.
                            return LRESULT(1);
                        } else {
                            info!("LL hook: hotkey matched but HOOK_STATE locked, skipping");
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

/// Notify the hotkey module that the overlay was hidden by means other than
/// the hotkey (e.g. ESC key from the frontend). Keeps the OVERLAY_VISIBLE
/// atomic in sync so the next hotkey press shows rather than double-hides.
pub fn notify_overlay_hidden() {
    #[cfg(windows)]
    OVERLAY_VISIBLE.store(false, std::sync::atomic::Ordering::SeqCst);
}

/// Notify the hotkey module that the overlay was shown by means other than
/// the hotkey. Keeps the OVERLAY_VISIBLE atomic in sync.
pub fn notify_overlay_shown() {
    #[cfg(windows)]
    OVERLAY_VISIBLE.store(true, std::sync::atomic::Ordering::SeqCst);
}

// ---------------------------------------------------------------------------
// Shared toggle logic (same on all platforms, guarded internally)
// ---------------------------------------------------------------------------

fn handle_hotkey_press(_app: &AppHandle, game_state: &SharedGameState, was_visible: bool) {
    // Snapshot the SC hwnd while we hold the lock, then release immediately.
    let sc_hwnd_val: isize = {
        let state = game_state.lock().unwrap();
        state.sc_hwnd.unwrap_or(0)
    };

    // Post a custom message to the overlay window. This is fully async —
    // it enqueues a message and returns immediately. The overlay's subclass
    // WNDPROC on the main thread will process it after the current keyboard
    // event finishes, avoiding the deadlock that run_on_main_thread caused
    // when the overlay was focused.
    let show = !was_visible;
    info!(
        "Hotkey: posting WM_HOTKEY_TOGGLE (show={}, sc_hwnd={})",
        show, sc_hwnd_val
    );
    crate::window::post_hotkey_toggle(show, sc_hwnd_val);
}
