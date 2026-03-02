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
    /// Maximum number of search results returned by api_search (default: 50)
    #[serde(default = "default_max_search_results")]
    pub max_search_results: u32,
    /// Cache TTL in seconds for commodity/price collections (default: 600 = 10 min)
    #[serde(default = "default_ttl_prices")]
    pub cache_ttl_prices_secs: u32,
    /// Cache TTL in seconds for catalog collections: vehicles, items, locations (default: 86400 = 24 h)
    #[serde(default = "default_ttl_catalog")]
    pub cache_ttl_catalog_secs: u32,
}

fn default_true() -> bool {
    true
}

fn default_max_search_results() -> u32 {
    50
}

fn default_ttl_prices() -> u32 {
    600
}

fn default_ttl_catalog() -> u32 {
    86400
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
            max_search_results: 50,
            cache_ttl_prices_secs: 600,
            cache_ttl_catalog_secs: 86400,
        }
    }
}
