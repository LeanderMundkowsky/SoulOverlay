use log::{error, info, warn};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, OnceLock,
};
use tauri::{AppHandle, Emitter};

/// Compiled regexes for log parsing, initialised once on first use.
struct LogRegexes {
    location: Regex,
    death: Regex,
    kill: Regex,
    ship: Regex,
}

fn log_regexes() -> &'static LogRegexes {
    static INSTANCE: OnceLock<LogRegexes> = OnceLock::new();
    INSTANCE.get_or_init(|| LogRegexes {
        location: Regex::new(r"<Location:\s*(.+?)>").unwrap(),
        death: Regex::new(r"<Actor Death>.+?killed by (.+)").unwrap(),
        kill: Regex::new(r"<Actor Death>\s*(.+?) killed (.+)").unwrap(),
        ship: Regex::new(r"\[Ship\]\s*(.+)").unwrap(),
    })
}

pub struct LogWatcher {
    app: AppHandle,
    log_path: PathBuf,
    offset: Arc<Mutex<u64>>,
    running: Arc<AtomicBool>,
    _watcher: Option<RecommendedWatcher>,
}

type WatcherParts = (RecommendedWatcher, Arc<Mutex<u64>>, Arc<AtomicBool>);

/// Create a new file watcher and initialise the offset to the current file size.
/// Returns `(watcher, offset, running)` on success.
fn create_watcher(
    app: &AppHandle,
    log_path: &Path,
) -> Result<WatcherParts, String> {
    let offset = Arc::new(Mutex::new(0u64));
    let running = Arc::new(AtomicBool::new(true));

    // If file exists, seek to end (we only care about new events)
    if log_path.exists() {
        if let Ok(metadata) = std::fs::metadata(log_path) {
            let mut off = offset.lock().unwrap();
            *off = metadata.len();
        }
    }

    let offset_clone = Arc::clone(&offset);
    let app_clone = app.clone();
    let path_clone = log_path.to_path_buf();
    let running_clone = Arc::clone(&running);

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if !running_clone.load(Ordering::Relaxed) {
            return;
        }
        match res {
            Ok(event) => {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    LogWatcher::process_new_lines(&app_clone, &path_clone, &offset_clone);
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
        } else {
            warn!(
                "Log directory does not exist yet: {:?}. Watcher not started.",
                parent
            );
        }
    }

    Ok((watcher, offset, running))
}

impl LogWatcher {
    /// Start watching the game log file
    pub fn start(app: AppHandle, log_path: PathBuf) -> Result<Self, String> {
        let (watcher, offset, running) = create_watcher(&app, &log_path)?;
        info!("Log watcher started for: {:?}", log_path);

        Ok(Self {
            app,
            log_path,
            offset,
            running,
            _watcher: Some(watcher),
        })
    }

    /// Update the log file path (e.g., from settings change)
    pub fn update_path(&mut self, new_path: PathBuf) -> Result<(), String> {
        // Stop existing watcher
        self.running.store(false, Ordering::Relaxed);
        self._watcher = None;

        let (watcher, offset, running) = create_watcher(&self.app, &new_path)?;

        self.log_path = new_path;
        self.offset = offset;
        self.running = running;
        self._watcher = Some(watcher);

        info!("Log watcher updated to: {:?}", self.log_path);
        Ok(())
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
        let re = log_regexes();

        for line in text.lines() {
            if let Some(caps) = re.location.captures(line) {
                let location = caps[1].trim().to_string();
                info!("Log: location changed to {}", location);
                let _ = app.emit("sc-location", serde_json::json!({ "location": location }));
            }

            if let Some(caps) = re.death.captures(line) {
                let killer = caps[1].trim().to_string();
                info!("Log: player killed by {}", killer);
                let _ = app.emit("sc-death", serde_json::json!({ "killer": killer }));
            } else if let Some(caps) = re.kill.captures(line) {
                let victim = caps[2].trim().to_string();
                info!("Log: player killed {}", victim);
                let _ = app.emit("sc-kill", serde_json::json!({ "victim": victim }));
            }

            if let Some(caps) = re.ship.captures(line) {
                let ship = caps[1].trim().to_string();
                info!("Log: ship changed to {}", ship);
                let _ = app.emit("sc-ship-changed", serde_json::json!({ "ship": ship }));
            }
        }
    }
}

impl Drop for LogWatcher {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Get the default log path for Star Citizen.
///
/// On Windows: `%APPDATA%\..\Local\Star Citizen\StarCitizen\LIVE\game.log`
/// On Linux: searches common Wine/Proton prefix locations used by Lutris
/// and manual Wine installs.
pub fn default_log_path() -> PathBuf {
    #[cfg(windows)]
    {
        // APPDATA is Roaming; go up one level to reach AppData\Local
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata)
                .join("..")
                .join("Local")
                .join("Star Citizen")
                .join("StarCitizen")
                .join("LIVE")
                .join("game.log");
        }
        PathBuf::from(
            r"C:\Users\Default\AppData\Local\Star Citizen\StarCitizen\LIVE\game.log",
        )
    }

    #[cfg(not(windows))]
    {
        // Star Citizen on Linux runs under Wine or Proton.
        // The game log lives inside the Wine prefix at the Windows AppData\Local path.
        let home = std::env::var("HOME").unwrap_or_default();
        let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());

        // Relative path inside any Wine prefix drive_c root
        let sc_rel = format!(
            "drive_c/users/{}/AppData/Local/Star Citizen/StarCitizen/LIVE/game.log",
            user
        );

        // Common Wine prefix locations to probe, in priority order
        let candidates: &[String] = &[
            // Lutris default Wine prefix
            format!("{}/.wine/{}", home, sc_rel),
            // Lutris SC-specific game directory
            format!("{}/Games/star-citizen/{}", home, sc_rel),
            // Another common Lutris path
            format!("{}/Games/StarCitizen/{}", home, sc_rel),
            // XDG_DATA_HOME based
            format!(
                "{}/.local/share/lutris/runners/wine/starcitizen/{}",
                home, sc_rel
            ),
        ];

        for path in candidates {
            let p = PathBuf::from(path);
            if p.exists() {
                return p;
            }
        }

        // Return the Lutris default even if it doesn't exist yet,
        // so the user sees a sensible default in the Settings panel.
        PathBuf::from(format!("{}/.wine/{}", home, sc_rel))
    }
}
