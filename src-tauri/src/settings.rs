use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use specta::Type;

/// Configurable in-app keybinds (F-keys and combos that don't go through the Rust hook)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Keybinds {
    /// Key combo to toggle the Settings panel (default: "F12")
    pub toggle_settings: String,
    /// Key combo to toggle the Debug panel (default: "F11")
    pub toggle_debug: String,
    /// Key combo to pin/unpin a location in search (default: "Ctrl+Enter")
    pub pin_location: String,
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            toggle_settings: "F12".to_string(),
            toggle_debug:    "F11".to_string(),
            pin_location:    "Ctrl+Enter".to_string(),
        }
    }
}

/// Per-panel layout widths persisted alongside other settings
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct LayoutWidths {
    /// Width of the left column (Favorites + Debug) in pixels
    pub left_panel_px: u32,
    /// Width of the settings side panel in pixels
    pub settings_panel_px: u32,
    /// Width of the search column when detail panel is open, as a percentage (0–100)
    pub search_split_pct: u32,
    /// Width of the search panel when it is the only element (centered), as a percentage (0–100)
    pub search_solo_pct: u32,
}

impl Default for LayoutWidths {
    fn default() -> Self {
        Self {
            left_panel_px: 280,
            settings_panel_px: 448,
            search_split_pct: 50,
            search_solo_pct: 50,
        }
    }
}

/// Application settings persisted as JSON to `%APPDATA%\SoulOverlay\settings.json`
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Settings {
    /// Global hotkey string (e.g., "Alt+Shift+S")
    pub hotkey: String,
    /// SoulOverlay backend API token (persisted for session continuity)
    pub backend_api_token: String,
    /// Optional custom log file path (None = use default)
    pub log_path: Option<String>,
    /// Overlay opacity (0.0 - 1.0)
    pub overlay_opacity: f32,
    /// Whether pressing Escape closes the overlay (default: true)
    pub esc_closes_overlay: bool,
    /// Whether opening the overlay resets to the search tab and focuses the search bar (default: true)
    pub reset_on_open: bool,
    /// Maximum number of search results returned by api_search (default: 50)
    pub max_search_results: u32,
    /// Per-collection cache TTL overrides in seconds, keyed by Collection::storage_key().
    /// Missing entries fall back to Collection::ttl_secs().
    pub cache_ttls: HashMap<String, u32>,
    /// User-adjusted panel layout widths
    pub layout_widths: LayoutWidths,
    /// Base font size in pixels (default: 14)
    pub font_size: u32,
    /// In-app panel keybinds (not the global Rust hook hotkey)
    pub keybinds: Keybinds,
    /// Whether debug-level log output is enabled (default: false)
    pub debug_logging: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: "F6".to_string(),
            backend_api_token: String::new(),
            log_path: None,
            overlay_opacity: 0.6,
            esc_closes_overlay: true,
            reset_on_open: false,
            max_search_results: 50,
            cache_ttls: HashMap::new(),
            layout_widths: LayoutWidths::default(),
            font_size: 14,
            keybinds: Keybinds::default(),
            debug_logging: false,
        }
    }
}
