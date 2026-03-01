use serde::{Deserialize, Serialize};

/// Application settings persisted via tauri-plugin-store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Global hotkey string (e.g., "Alt+Shift+S")
    pub hotkey: String,
    /// UEX Corp API key
    pub uex_api_key: String,
    /// Optional custom log file path (None = use default)
    pub log_path: Option<String>,
    /// Overlay opacity (0.0 - 1.0)
    pub overlay_opacity: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: "Alt+Shift+S".to_string(),
            uex_api_key: String::new(),
            log_path: None,
            overlay_opacity: 0.6,
        }
    }
}
