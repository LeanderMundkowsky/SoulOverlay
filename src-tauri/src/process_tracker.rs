/// Process tracker for Star Citizen.
///
/// Polls the Windows process list every 2 seconds to detect whether
/// StarCitizen.exe is running. Emits `sc-window-found` / `sc-window-lost`
/// Tauri events so the frontend knows when to show the "Connected" indicator.
use log::info;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{AppHandle, Emitter};

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

pub struct ProcessTracker {
    app: AppHandle,
    pub sc_running: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
}

impl ProcessTracker {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            sc_running: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn sc_running(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.sc_running)
    }

    /// Start the process tracker. Spawns a background thread with a polling loop.
    pub fn start(&self) {
        let app = self.app.clone();
        let sc_running = Arc::clone(&self.sc_running);
        let running = Arc::clone(&self.running);

        // Initial check before spawning thread
        let initially_running = is_sc_running();
        sc_running.store(initially_running, Ordering::Relaxed);
        if initially_running {
            let _ = app.emit("sc-window-found", ());
            info!("Star Citizen process detected on startup");
        }

        std::thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_secs(2));

                let currently = is_sc_running();
                let was = sc_running.load(Ordering::Relaxed);

                if currently && !was {
                    sc_running.store(true, Ordering::Relaxed);
                    let _ = app.emit("sc-window-found", ());
                    info!("Star Citizen process started");
                } else if !currently && was {
                    sc_running.store(false, Ordering::Relaxed);
                    let _ = app.emit("sc-window-lost", ());
                    info!("Star Citizen process stopped");
                }
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

impl Drop for ProcessTracker {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Returns true if a process named `StarCitizen.exe` is currently running.
fn is_sc_running() -> bool {
    unsafe {
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(h) => h,
            Err(_) => return false,
        };

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let found = if Process32FirstW(snapshot, &mut entry).is_ok() {
            let mut result = false;
            loop {
                let len = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
                if name.eq_ignore_ascii_case("StarCitizen.exe") {
                    result = true;
                    break;
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
            result
        } else {
            false
        };

        let _ = CloseHandle(snapshot);
        found
    }
}
