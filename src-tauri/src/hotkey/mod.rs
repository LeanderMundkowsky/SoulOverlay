/// Hotkey management using a WH_KEYBOARD_LL low-level keyboard hook.
///
/// `RegisterHotKey` / tauri-plugin-global-shortcut do NOT fire when a full-screen
/// or exclusive-foreground application (like Star Citizen) has focus.
/// A low-level keyboard hook is injected at the OS level and fires regardless of
/// which window is in the foreground, matching how Steam/Discord overlays work.
mod keymap;

use log::{info, warn};
use std::sync::{Arc, Mutex};

/// Handle to the running hook thread. Dropping it stops the thread.
pub struct HookHandle {
    thread_id: u32,
}

impl Drop for HookHandle {
    fn drop(&mut self) {
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
    }
}

/// Global re-entrancy guard shared between the hook callback and registration.
/// We use a Mutex<()> because SetWindowsHookExW only accepts a bare fn pointer,
/// not a closure, so we need a static.
static HOOK_GUARD: std::sync::OnceLock<Arc<Mutex<()>>> = std::sync::OnceLock::new();

/// Tracks whether the overlay is currently visible. Maintained by the hotkey
/// handler so the LL hook callback can read it without touching Tauri APIs
/// (which require the main thread).
/// Starts as `false` because the overlay is hidden on launch.
static OVERLAY_VISIBLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Self-tracked modifier key states. Updated by the LL hook itself on every
/// key event — avoids GetAsyncKeyState races when the overlay is focused.
static MOD_CTRL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static MOD_ALT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static MOD_SHIFT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Target VK code stored as an atomic so the hook can fast-reject without
/// locking HOOK_GUARD. Avoids mutex contention inside the time-critical callback.
static TARGET_VK: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
/// Required modifiers as packed bits: bit 0 = ctrl, bit 1 = alt, bit 2 = shift.
static TARGET_MODS: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

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

pub fn register_hotkey(
    hotkey: &str,
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

    // Initialise the re-entrancy guard once.
    let _ = HOOK_GUARD.get_or_init(|| Arc::new(Mutex::new(())));

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
                    // Use the mutex as a re-entrancy gate — if already locked, skip.
                    if let Some(guard) = HOOK_GUARD.get() {
                        if let Ok(_lock) = guard.try_lock() {
                            // Grant our process the foreground privilege so the
                            // main thread's SetForegroundWindow call succeeds.
                            let _ = windows::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow(
                                windows::Win32::System::Threading::GetCurrentProcessId(),
                            );

                            let was_visible = OVERLAY_VISIBLE.fetch_xor(true, SeqCst);
                            info!("LL hook: hotkey matched, was_visible={}", was_visible);
                            handle_hotkey_press(was_visible);

                            // Swallow the keypress — do NOT pass it to the game.
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

/// Notify the hotkey module that the overlay was hidden by means other than
/// the hotkey (e.g. ESC key from the frontend). Keeps the OVERLAY_VISIBLE
/// atomic in sync so the next hotkey press shows rather than double-hides.
pub fn notify_overlay_hidden() {
    OVERLAY_VISIBLE.store(false, std::sync::atomic::Ordering::SeqCst);
}

/// Notify the hotkey module that the overlay was shown by means other than
/// the hotkey. Keeps the OVERLAY_VISIBLE atomic in sync.
pub fn notify_overlay_shown() {
    OVERLAY_VISIBLE.store(true, std::sync::atomic::Ordering::SeqCst);
}

fn handle_hotkey_press(was_visible: bool) {
    let show = !was_visible;
    info!("Hotkey: posting WM_HOTKEY_TOGGLE (show={})", show);
    crate::window::post_hotkey_toggle(show);
}
