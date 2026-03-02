use serde::{Deserialize, Serialize};

/// Per-panel layout widths persisted alongside other settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutWidths {
    /// Width of the left column (Favorites + Debug) in pixels
    #[serde(default = "default_left_panel_px")]
    pub left_panel_px: u32,
    /// Width of the settings side panel in pixels
    #[serde(default = "default_settings_panel_px")]
    pub settings_panel_px: u32,
    /// Width of the search column when detail panel is open, as a percentage (0–100)
    #[serde(default = "default_search_split_pct")]
    pub search_split_pct: u32,
    /// Width of the search panel when it is the only element (centered), as a percentage (0–100)
    #[serde(default = "default_search_solo_pct")]
    pub search_solo_pct: u32,
}

fn default_left_panel_px() -> u32 {
    280
}
fn default_settings_panel_px() -> u32 {
    448
}
fn default_search_split_pct() -> u32 {
    50
}

fn default_search_solo_pct() -> u32 {
    50
}

impl Default for LayoutWidths {
    fn default() -> Self {
        Self {
            left_panel_px: default_left_panel_px(),
            settings_panel_px: default_settings_panel_px(),
            search_split_pct: default_search_split_pct(),
            search_solo_pct: default_search_solo_pct(),
        }
    }
}

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
    /// Cache TTL in seconds for price collections (default: 3600 = 1 h)
    #[serde(default = "default_ttl_prices")]
    pub cache_ttl_prices_secs: u32,
    /// Cache TTL in seconds for catalog collections: vehicles, items, locations (default: 86400 = 24 h)
    #[serde(default = "default_ttl_catalog")]
    pub cache_ttl_catalog_secs: u32,
    /// User-adjusted panel layout widths
    #[serde(default)]
    pub layout_widths: LayoutWidths,
    /// Base font size in pixels (default: 14)
    #[serde(default = "default_font_size")]
    pub font_size: u32,
}

fn default_true() -> bool {
    true
}

fn default_max_search_results() -> u32 {
    50
}

fn default_ttl_prices() -> u32 {
    3600
}

fn default_ttl_catalog() -> u32 {
    86400
}

fn default_font_size() -> u32 {
    14
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
            cache_ttl_prices_secs: 3600,
            cache_ttl_catalog_secs: 86400,
            layout_widths: LayoutWidths::default(),
            font_size: 14,
        }
    }
}
