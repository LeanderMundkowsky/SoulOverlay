use serde::{Deserialize, Serialize};

/// Application settings persisted as JSON to `%APPDATA%\SoulOverlay\settings.json`
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
    /// Whether pressing Escape closes the overlay (default: true)
    #[serde(default = "default_true")]
    pub esc_closes_overlay: bool,
    /// Whether opening the overlay resets to the search tab and focuses the search bar (default: true)
    #[serde(default = "default_true")]
    pub reset_on_open: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: "F6".to_string(),
            uex_api_key: String::new(),
            log_path: None,
            overlay_opacity: 0.6,
            esc_closes_overlay: true,
            reset_on_open: true,
        }
    }
}
