use serde::{Deserialize, Serialize};
use specta::Type;

/// Persisted state for a single self-timer (keycard or compboard countdown).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CzSelfTimer {
    /// Unique timer ID, e.g. "checkmate-terminal1"
    pub id: String,
    /// Zone name, e.g. "Checkmate"
    pub zone: String,
    /// Timer label, e.g. "Terminal 1"
    pub label: String,
    /// Category: "keycard" or "compboard"
    pub category: String,
    /// Default countdown duration in seconds
    pub default_seconds: u32,
    /// Remaining seconds (only meaningful when status is "idle" after partial use)
    pub remaining_seconds: u32,
    /// Unix epoch (seconds) when the timer will reach zero, if running. 0 if not running.
    pub end_epoch: u32,
    /// "idle" | "running" | "done"
    pub status: String,
}

/// A ship available in the Executive Hangar (scraped from contestedzonetimers.com).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CzShip {
    pub name: String,
    pub ship_type: String,
    pub image_url: String,
    pub wiki_url: Option<String>,
    pub pledge_url: Option<String>,
    pub credit: String,
}

/// A contested zone map image (scraped from contestedzonetimers.com).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CzMap {
    pub name: String,
    pub image_url: String,
}
