use log::{error, info, warn};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tauri::{AppHandle, Emitter};

/// Events emitted from log parsing
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum LogEvent {
    Location { location: String },
    Death { killer: String },
    Kill { victim: String },
    ShipChanged { ship: String },
}

pub struct LogWatcher {
    app: AppHandle,
    log_path: PathBuf,
    offset: Arc<Mutex<u64>>,
    running: Arc<AtomicBool>,
    _watcher: Option<RecommendedWatcher>,
}

impl LogWatcher {
    /// Start watching the game log file
    pub fn start(app: AppHandle, log_path: PathBuf) -> Result<Self, String> {
        let offset = Arc::new(Mutex::new(0u64));
        let running = Arc::new(AtomicBool::new(true));

        // If file exists, seek to end (we only care about new events)
        if log_path.exists() {
            if let Ok(metadata) = std::fs::metadata(&log_path) {
                let mut off = offset.lock().unwrap();
                *off = metadata.len();
            }
        }

        let offset_clone = Arc::clone(&offset);
        let app_clone = app.clone();
        let path_clone = log_path.clone();
        let running_clone = Arc::clone(&running);

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if !running_clone.load(Ordering::Relaxed) {
                return;
            }
            match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        Self::process_new_lines(&app_clone, &path_clone, &offset_clone);
                    }
                }
                Err(e) => {
                    error!("Log watcher error: {}", e);
                }
            }
        })
        .map_err(|e| format!("Failed to create file watcher: {}", e))?;

        // Watch the parent directory (handles file rotation)
        if let Some(parent) = log_path.parent() {
            if parent.exists() {
                watcher
                    .watch(parent, RecursiveMode::NonRecursive)
                    .map_err(|e| format!("Failed to watch log directory: {}", e))?;
                info!("Log watcher started for: {:?}", log_path);
            } else {
                warn!(
                    "Log directory does not exist yet: {:?}. Watcher not started.",
                    parent
                );
            }
        }

        Ok(Self {
            app,
            log_path,
            offset,
            running,
            _watcher: Some(watcher),
        })
    }

    fn process_new_lines(app: &AppHandle, path: &Path, offset: &Arc<Mutex<u64>>) {
        let mut off = match offset.lock() {
            Ok(o) => o,
            Err(_) => return,
        };

        let metadata = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return,
        };

        let file_size = metadata.len();

        // Handle log rotation: file got smaller
        if file_size < *off {
            info!("Log file rotated, resetting offset");
            *off = 0;
        }

        if file_size == *off {
            return; // No new data
        }

        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to open log file: {}", e);
                return;
            }
        };

        if let Err(e) = file.seek(SeekFrom::Start(*off)) {
            error!("Failed to seek in log file: {}", e);
            return;
        }

        let bytes_to_read = (file_size - *off) as usize;
        let mut buffer = vec![0u8; bytes_to_read];

        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to read log file: {}", e);
                return;
            }
        }

        *off = file_size;

        let text = String::from_utf8_lossy(&buffer);
        Self::parse_and_emit(app, &text);
    }

    fn parse_and_emit(app: &AppHandle, text: &str) {
        // Compile regexes (in a real hot path, you'd cache these)
        let location_re = Regex::new(r"<Location:\s*(.+?)>").unwrap();
        let death_re = Regex::new(r"<Actor Death>.+?killed by (.+)").unwrap();
        let kill_re = Regex::new(r"<Actor Death>\s*(.+?) killed (.+)").unwrap();
        let ship_re = Regex::new(r"\[Ship\]\s*(.+)").unwrap();

        for line in text.lines() {
            if let Some(caps) = location_re.captures(line) {
                let location = caps[1].trim().to_string();
                info!("Log: location changed to {}", location);
                let _ = app.emit("sc-location", serde_json::json!({ "location": location }));
            }

            if let Some(caps) = death_re.captures(line) {
                let killer = caps[1].trim().to_string();
                info!("Log: player killed by {}", killer);
                let _ = app.emit("sc-death", serde_json::json!({ "killer": killer }));
            } else if let Some(caps) = kill_re.captures(line) {
                let victim = caps[2].trim().to_string();
                info!("Log: player killed {}", victim);
                let _ = app.emit("sc-kill", serde_json::json!({ "victim": victim }));
            }

            if let Some(caps) = ship_re.captures(line) {
                let ship = caps[1].trim().to_string();
                info!("Log: ship changed to {}", ship);
                let _ = app.emit("sc-ship-changed", serde_json::json!({ "ship": ship }));
            }
        }
    }

    /// Update the log file path (e.g., from settings change)
    pub fn update_path(&mut self, new_path: PathBuf) -> Result<(), String> {
        self.running.store(false, Ordering::Relaxed);
        self._watcher = None;

        // Re-create watcher with new path
        let offset = Arc::new(Mutex::new(0u64));
        let running = Arc::new(AtomicBool::new(true));

        if new_path.exists() {
            if let Ok(metadata) = std::fs::metadata(&new_path) {
                let mut off = offset.lock().unwrap();
                *off = metadata.len();
            }
        }

        let offset_clone = Arc::clone(&offset);
        let app_clone = self.app.clone();
        let path_clone = new_path.clone();
        let running_clone = Arc::clone(&running);

        let watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if !running_clone.load(Ordering::Relaxed) {
                return;
            }
            match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        Self::process_new_lines(&app_clone, &path_clone, &offset_clone);
                    }
                }
                Err(e) => {
                    error!("Log watcher error: {}", e);
                }
            }
        })
        .map_err(|e| format!("Failed to create file watcher: {}", e))?;

        self.log_path = new_path;
        self.offset = offset;
        self.running = running;
        self._watcher = Some(watcher);

        // Watch the parent directory
        if let Some(parent) = self.log_path.parent() {
            if parent.exists() {
                if let Some(ref mut w) = self._watcher {
                    w.watch(parent, RecursiveMode::NonRecursive)
                        .map_err(|e| format!("Failed to watch log directory: {}", e))?;
                }
            }
        }

        info!("Log watcher updated to: {:?}", self.log_path);
        Ok(())
    }
}

impl Drop for LogWatcher {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Get the default log path for Star Citizen
pub fn default_log_path() -> PathBuf {
    // On Windows: C:\Users\<USERNAME>\AppData\Local\..
    // Roberts Space Industries\StarCitizen\LIVE\game.log
    if let Ok(appdata) = std::env::var("APPDATA") {
        // APPDATA is Roaming, go up one level to get to Local's parent
        let base = PathBuf::from(appdata);
        // Actually the path in the spec uses APPDATA\..\Roberts Space Industries
        // which resolves to C:\Users\<USERNAME>\AppData\Roberts Space Industries
        base.join("..")
            .join("Local")
            .join("Star Citizen")
            .join("StarCitizen")
            .join("LIVE")
            .join("game.log")
    } else {
        // Fallback
        PathBuf::from(r"C:\Users\Default\AppData\Local\Star Citizen\StarCitizen\LIVE\game.log")
    }
}
