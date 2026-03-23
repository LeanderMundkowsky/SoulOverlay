use log::info;
use std::path::PathBuf;

/// Centralized path configuration for all application files.
///
/// The data directory varies by build profile on Windows:
/// - `tester` feature active  → `%APPDATA%\SoulOverlayTest`
/// - debug build (tauri dev)  → `%APPDATA%\SoulOverlayDev`
/// - release build            → `%APPDATA%\SoulOverlay`
///
/// On Linux, files live under `$XDG_DATA_HOME/souloverlay/` (default: `~/.local/share/souloverlay/`).
///
/// This struct is the single source of truth — no other module should
/// resolve platform paths directly.
#[derive(Debug, Clone)]
pub struct AppPaths {
    /// Root data directory (path depends on platform and build profile)
    pub data_dir: PathBuf,
    /// Log file: `<data_dir>/soul-overlay.log`
    pub log_file: PathBuf,
    /// SQLite database: `<data_dir>/soul_overlay.db`
    pub db_file: PathBuf,
    /// Settings JSON: `<data_dir>/settings.json`
    pub settings_file: PathBuf,
}

#[cfg(windows)]
fn get_data_dir() -> Result<PathBuf, String> {
    let app_data = std::env::var("APPDATA")
        .map_err(|_| "APPDATA environment variable not set".to_string())?;

    let dir_name = if cfg!(feature = "tester") {
        "SoulOverlayTest"
    } else if cfg!(debug_assertions) {
        "SoulOverlayDev"
    } else {
        "SoulOverlay"
    };
    Ok(PathBuf::from(app_data).join(dir_name))
}

#[cfg(not(windows))]
fn get_data_dir() -> Result<PathBuf, String> {
    // Follow the XDG Base Directory Specification
    let base = std::env::var("XDG_DATA_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".local").join("share"))
        })
        .ok_or_else(|| "Could not determine XDG_DATA_HOME or HOME".to_string())?;
    Ok(base.join("souloverlay"))
}

impl AppPaths {
    /// Resolve all paths from the platform-appropriate base directory.
    /// Creates the data directory if it doesn't exist yet.
    pub fn init() -> Result<Self, String> {
        let data_dir = get_data_dir()?;
        
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let paths = Self {
            log_file: data_dir.join("soul-overlay.log"),
            db_file: data_dir.join("soul_overlay.db"),
            settings_file: data_dir.join("settings.json"),
            data_dir,
        };

        info!("App paths initialized: {:?}", paths.data_dir);
        Ok(paths)
    }

    /// Load settings from the JSON file on disk.
    /// Returns `Settings::default()` if the file doesn't exist or can't be parsed.
    pub fn load_settings(&self) -> crate::settings::Settings {
        match std::fs::read_to_string(&self.settings_file) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
                log::warn!("Failed to parse settings file, using defaults: {}", e);
                crate::settings::Settings::default()
            }),
            Err(_) => {
                // File doesn't exist yet — perfectly normal on first launch
                crate::settings::Settings::default()
            }
        }
    }

    /// Persist settings to the JSON file on disk.
    pub fn save_settings(&self, settings: &crate::settings::Settings) -> Result<(), String> {
        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        std::fs::write(&self.settings_file, json)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;
        Ok(())
    }
}
