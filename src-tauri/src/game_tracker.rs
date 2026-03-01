/// Game window tracker for Star Citizen.
///
/// Uses a polling thread to monitor:
/// - SC window creation/destruction
/// - SC window move/resize
/// - SC window focus changes
use log::info;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Instant;
use tauri::{AppHandle, Emitter};

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::FindWindowW;

/// Shared state for the game tracker.
/// We store HWND as `isize` instead of `HWND` because HWND contains a raw pointer
/// (*mut c_void) which is !Send. Storing as isize (which is Send+Sync) allows this
/// struct to be used in Tauri managed state.
pub struct GameTrackerState {
    /// The SC window handle stored as isize for Send+Sync safety.
    /// Convert to HWND via `HWND(val as *mut std::ffi::c_void)` at call sites.
    pub sc_hwnd: Option<isize>,
    pub is_focused: bool,
    pub is_running: bool,
    pub window_x: i32,
    pub window_y: i32,
    pub window_w: u32,
    pub window_h: u32,
}

// SAFETY: The isize stored in sc_hwnd is a window handle value that is safe to send
// between threads. We only use it to call Win32 APIs on the main thread or from
// the polling thread where it was obtained.
unsafe impl Send for GameTrackerState {}
unsafe impl Sync for GameTrackerState {}

impl Default for GameTrackerState {
    fn default() -> Self {
        Self {
            sc_hwnd: None,
            is_focused: false,
            is_running: false,
            window_x: 0,
            window_y: 0,
            window_w: 1920,
            window_h: 1080,
        }
    }
}

pub type SharedGameState = Arc<Mutex<GameTrackerState>>;

pub struct GameTracker {
    app: AppHandle,
    state: SharedGameState,
    running: Arc<AtomicBool>,
}

impl GameTracker {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            state: Arc::new(Mutex::new(GameTrackerState::default())),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn state(&self) -> SharedGameState {
        Arc::clone(&self.state)
    }

    /// Start the game tracker. Spawns a thread with a polling loop.
    pub fn start(&self) {
        let app = self.app.clone();
        let state = Arc::clone(&self.state);
        let running = Arc::clone(&self.running);

        // Try to find SC window immediately
        if let Some(hwnd) = find_sc_window() {
            let hwnd_val = crate::platform::hwnd_to_isize(hwnd);
            let mut s = state.lock().unwrap();
            s.sc_hwnd = Some(hwnd_val);
            s.is_running = true;

            // Use the monitor that contains the SC window so the overlay always
            // fills the full display, regardless of SC's window mode.
            let (x, y, w, h) = crate::window::get_monitor_geometry_for_window(hwnd);
            s.window_x = x;
            s.window_y = y;
            s.window_w = w;
            s.window_h = h;
            crate::window::set_window_geometry(&app, x, y, w, h);

            let _ = app.emit("sc-window-found", ());
            info!("Star Citizen window found on startup");
        }

        // Spawn polling thread for SC window detection and tracking
        std::thread::spawn(move || {
            use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, IsWindow};

            let mut last_geometry_check = Instant::now();
            let geometry_interval = std::time::Duration::from_millis(120);

            while running.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));

                let mut s = state.lock().unwrap();

                if let Some(hwnd_val) = s.sc_hwnd {
                    let hwnd = crate::platform::hwnd_from_isize(hwnd_val);

                    // Check if window is still valid
                    let still_valid = unsafe { IsWindow(hwnd).as_bool() };

                    if !still_valid {
                        info!("Star Citizen window lost");
                        s.sc_hwnd = None;
                        s.is_running = false;
                        s.is_focused = false;
                        let _ = app.emit("sc-window-lost", ());
                        continue;
                    }

                    // Check focus
                    let fg = unsafe { GetForegroundWindow() };
                    let was_focused = s.is_focused;
                    s.is_focused = fg == hwnd;

                    if s.is_focused != was_focused {
                        info!(
                            "SC focus changed: {}",
                            if s.is_focused { "focused" } else { "unfocused" }
                        );
                    }

                    // Check geometry (debounced) — track which monitor SC is on
                    // so the overlay stays fullscreen on that monitor even if SC moves.
                    if last_geometry_check.elapsed() >= geometry_interval {
                        last_geometry_check = Instant::now();
                        let (x, y, w, h) = crate::window::get_monitor_geometry_for_window(hwnd);
                        if x != s.window_x || y != s.window_y || w != s.window_w || h != s.window_h
                        {
                            s.window_x = x;
                            s.window_y = y;
                            s.window_w = w;
                            s.window_h = h;

                            let app_for_thread = app.clone();
                            let app_for_closure = app_for_thread.clone();
                            // Must set window geometry on main thread
                            let _ = app_for_thread.run_on_main_thread(move || {
                                crate::window::set_window_geometry(&app_for_closure, x, y, w, h);
                            });
                        }
                    }
                } else {
                    // Try to find SC window
                    if let Some(hwnd) = find_sc_window() {
                        let hwnd_val = crate::platform::hwnd_to_isize(hwnd);
                        s.sc_hwnd = Some(hwnd_val);
                        s.is_running = true;

                        let (x, y, w, h) = crate::window::get_monitor_geometry_for_window(hwnd);
                        s.window_x = x;
                        s.window_y = y;
                        s.window_w = w;
                        s.window_h = h;

                        let app_for_thread = app.clone();
                        let app_for_closure = app_for_thread.clone();
                        let _ = app_for_thread.run_on_main_thread(move || {
                            crate::window::set_window_geometry(&app_for_closure, x, y, w, h);
                        });

                        let _ = app.emit("sc-window-found", ());
                        info!("Star Citizen window found");
                    }
                }
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

impl Drop for GameTracker {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Find the Star Citizen window by title only.
/// We pass None for the class name because the SC window class is not "Star Citizen".
/// Returns the HWND if found, None otherwise.
fn find_sc_window() -> Option<HWND> {
    use windows::core::w;

    unsafe {
        // First arg = class name (None = match any class), second arg = window title.
        // Star Citizen's window title is "Star Citizen" but its class name is not.
        match FindWindowW(None, w!("Star Citizen")) {
            Ok(hwnd) => Some(hwnd),
            Err(_) => None,
        }
    }
}
