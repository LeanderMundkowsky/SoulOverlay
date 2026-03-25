use std::collections::VecDeque;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

use serde::Serialize;
use specta::Type;

pub const MAX_ENTRIES: usize = 200;

#[derive(Debug, Clone, Serialize, Type)]
pub struct BackendCallEntry {
    pub method: String,
    pub path: String,
    pub status: Option<u16>,
    pub duration_ms: u32,
    pub error: Option<String>,
    pub timestamp: String,
}

pub struct BackendCallLog {
    pub enabled: Arc<AtomicBool>,
    entries: Arc<Mutex<VecDeque<BackendCallEntry>>>,
}

impl BackendCallLog {
    pub fn new(enabled: Arc<AtomicBool>) -> Self {
        Self {
            enabled,
            entries: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_ENTRIES))),
        }
    }

    pub fn record(&self, entry: BackendCallEntry) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }
        let mut log = self.entries.lock().unwrap();
        if log.len() >= MAX_ENTRIES {
            log.pop_front();
        }
        log.push_back(entry);
    }

    pub fn get_all(&self) -> Vec<BackendCallEntry> {
        self.entries.lock().unwrap().iter().cloned().rev().collect()
    }

    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }
}
