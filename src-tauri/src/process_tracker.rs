/// Process tracker for Star Citizen.
///
/// Polls the process list every 2 seconds to detect whether StarCitizen.exe
/// is running. Emits `sc-window-found` / `sc-window-lost` Tauri events so the
/// frontend knows when to show the "Connected" indicator.
///
/// On Windows: uses the ToolHelp32 snapshot API.
/// On Linux: reads /proc to detect StarCitizen.exe running under Wine/Proton.
use log::info;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{AppHandle, Emitter};

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
    #[cfg(windows)]
    return is_sc_running_windows();

    #[cfg(not(windows))]
    return is_sc_running_linux();
}

/// Windows: scan the process list using the ToolHelp32 snapshot API.
#[cfg(windows)]
fn is_sc_running_windows() -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

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

/// Linux: scan /proc to find StarCitizen.exe running under Wine/Proton.
///
/// Star Citizen on Linux runs via Wine or Proton. The process name in /proc
/// may be truncated by the kernel (TASK_COMM_LEN = 15 chars), so we check
/// both /proc/<pid>/comm (for the truncated name) and /proc/<pid>/cmdline
/// (for the full command line that includes the .exe path).
#[cfg(not(windows))]
fn is_sc_running_linux() -> bool {
    let proc_dir = match std::fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return false,
    };

    for entry in proc_dir.flatten() {
        // Only consider numeric directories (PIDs)
        let name = entry.file_name();
        let pid_str = match name.to_str() {
            Some(s) => s,
            None => continue,
        };
        if !pid_str.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        // Check /proc/<pid>/cmdline for the full exe path
        let cmdline_path = format!("/proc/{}/cmdline", pid_str);
        if let Ok(cmdline) = std::fs::read_to_string(&cmdline_path) {
            if cmdline.to_ascii_lowercase().contains("starcitizen.exe") {
                return true;
            }
        }
    }

    false
}

