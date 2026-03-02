use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::VecDeque;

pub const ACTIVITY_LOG_MAX: usize = 100;

/// A single backend fetch event (prefetch, timer refresh, or manual cache refresh).
#[derive(Debug, Clone, Serialize)]
pub struct FetchEvent {
    /// RFC3339 timestamp when the fetch completed.
    pub timestamp: String,
    /// Collection storage key, e.g. "commodity_prices".
    pub collection: String,
    /// Human-readable API endpoint summary, e.g. "/commodities_prices (per-entity, 120 calls)".
    pub endpoint: String,
    /// Number of rows/entries fetched.
    pub row_count: usize,
    /// Wall-clock duration of the fetch in milliseconds.
    pub duration_ms: u64,
    /// What triggered this fetch: "startup", "timer", or "manual".
    pub triggered_by: String,
    pub ok: bool,
    pub error: Option<String>,
}

/// The last price/entity lookup issued by the user (via a price command).
#[derive(Debug, Clone, Serialize)]
pub struct LastUserAction {
    pub timestamp: String,
    /// Entity kind, e.g. "commodity", "vehicle".
    pub kind: String,
    pub entity_id: String,
    /// Cache collection key, e.g. "commodity_prices".
    pub collection: String,
    /// "fresh", "stale", or "missing".
    pub source: String,
    pub row_count: usize,
}

/// Central activity tracker stored on AppState.
pub struct ActivityLog {
    pub events: VecDeque<FetchEvent>,
    pub last_bg_check_at: Option<DateTime<Utc>>,
    pub last_user_action: Option<LastUserAction>,
}

impl ActivityLog {
    pub fn new() -> Self {
        Self {
            events: VecDeque::with_capacity(ACTIVITY_LOG_MAX),
            last_bg_check_at: None,
            last_user_action: None,
        }
    }

    pub fn push_fetch(&mut self, event: FetchEvent) {
        if self.events.len() >= ACTIVITY_LOG_MAX {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }
}
