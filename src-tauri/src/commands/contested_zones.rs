use log::{debug, info, warn};
use tauri::State;

use crate::commands::api::ApiResponse;
use crate::providers::contested_zones::scraper;
use crate::providers::contested_zones::types::{CzMap, CzSelfTimer, CzShip};
use crate::state::AppState;

/// Default self-timer definitions seeded on first load.
fn default_timers() -> Vec<CzSelfTimer> {
    let keycard = |id: &str, zone: &str, label: &str| CzSelfTimer {
        id: id.to_string(),
        zone: zone.to_string(),
        label: label.to_string(),
        category: "keycard".to_string(),
        default_seconds: 15 * 60,
        remaining_seconds: 15 * 60,
        end_epoch: 0,
        status: "idle".to_string(),
    };
    let compboard = |id: &str, zone: &str, label: &str| CzSelfTimer {
        id: id.to_string(),
        zone: zone.to_string(),
        label: label.to_string(),
        category: "compboard".to_string(),
        default_seconds: 30 * 60,
        remaining_seconds: 30 * 60,
        end_epoch: 0,
        status: "idle".to_string(),
    };

    vec![
        // Checkmate
        keycard("checkmate-terminal1", "Checkmate", "Terminal 1"),
        keycard("checkmate-terminal2", "Checkmate", "Terminal 2"),
        keycard("checkmate-terminal3", "Checkmate", "Terminal 3"),
        compboard("checkmate-tablet1", "Checkmate", "Tablet 1"),
        compboard("checkmate-tablet2", "Checkmate", "Tablet 2"),
        compboard("checkmate-tablet3", "Checkmate", "Tablet 3"),
        // Orbituary
        keycard("orbituary-terminal1", "Orbituary", "Terminal 1"),
        keycard("orbituary-terminal2", "Orbituary", "Terminal 2"),
        compboard("orbituary-tablet4", "Orbituary", "Tablet 4"),
        compboard("orbituary-tablet7", "Orbituary", "Tablet 7"),
        // Ruin Station
        keycard("ruinstation-crypt", "Ruin Station", "The Crypt"),
        keycard("ruinstation-lastresort", "Ruin Station", "The Last Resort"),
        keycard("ruinstation-wasteland", "Ruin Station", "The Wasteland"),
        compboard("ruinstation-tablet5", "Ruin Station", "Tablet 5"),
        compboard("ruinstation-tablet6", "Ruin Station", "Tablet 6"),
        // PYAM-SUPVISR
        compboard("pyam-3-4", "PYAM-SUPVISR", "-3-4"),
        compboard("pyam-3-5", "PYAM-SUPVISR", "-3-5"),
    ]
}

/// Fetch the contested zone cycle start epoch from contestedzonetimers.com.
#[tauri::command]
#[specta::specta]
pub async fn cz_get_cycle_start(state: State<'_, AppState>) -> Result<ApiResponse<u32>, String> {
    let client = state.uex.client();
    match scraper::fetch_cycle_start(client).await {
        Ok(epoch) => {
            info!("[contested_zones] Cycle start epoch: {}", epoch);
            Ok(ApiResponse::ok(epoch))
        }
        Err(e) => {
            warn!("[contested_zones] Failed to fetch cycle start: {}", e);
            Ok(ApiResponse::err(e))
        }
    }
}

/// Fetch and scrape contested zone ship data from the website.
#[tauri::command]
#[specta::specta]
pub async fn cz_get_ships(state: State<'_, AppState>) -> Result<ApiResponse<Vec<CzShip>>, String> {
    let client = state.uex.client();
    match scraper::fetch_ships(client).await {
        Ok(ships) => Ok(ApiResponse::ok(ships)),
        Err(e) => {
            warn!("[contested_zones] Failed to fetch ships: {}", e);
            Ok(ApiResponse::err(e))
        }
    }
}

/// Fetch and scrape contested zone map data from the website.
#[tauri::command]
#[specta::specta]
pub async fn cz_get_maps(state: State<'_, AppState>) -> Result<ApiResponse<Vec<CzMap>>, String> {
    let client = state.uex.client();
    match scraper::fetch_maps(client).await {
        Ok(maps) => Ok(ApiResponse::ok(maps)),
        Err(e) => {
            warn!("[contested_zones] Failed to fetch maps: {}", e);
            Ok(ApiResponse::err(e))
        }
    }
}

/// Load all self-timers from SQLite. Seeds defaults if the table is empty.
#[tauri::command]
#[specta::specta]
pub async fn cz_load_self_timers(
    state: State<'_, AppState>,
) -> Result<Vec<CzSelfTimer>, String> {
    let db = state.cache.db().lock().unwrap();

    // Check if we have any timers
    let count: i64 = db
        .query_row("SELECT COUNT(*) FROM cz_self_timers", [], |row| row.get(0))
        .map_err(|e| format!("Failed to count self timers: {}", e))?;

    // Seed defaults on first use
    if count == 0 {
        debug!("[contested_zones] Seeding default self-timers");
        for t in default_timers() {
            db.execute(
                "INSERT INTO cz_self_timers (id, zone, label, category, default_seconds, remaining_seconds, end_epoch, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    t.id, t.zone, t.label, t.category,
                    t.default_seconds, t.remaining_seconds, t.end_epoch, t.status
                ],
            )
            .map_err(|e| format!("Failed to seed timer {}: {}", t.id, e))?;
        }
    }

    // Load all timers
    let mut stmt = db
        .prepare(
            "SELECT id, zone, label, category, default_seconds, remaining_seconds, end_epoch, status
             FROM cz_self_timers ORDER BY rowid",
        )
        .map_err(|e| format!("Failed to prepare self timers query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(CzSelfTimer {
                id: row.get(0)?,
                zone: row.get(1)?,
                label: row.get(2)?,
                category: row.get(3)?,
                default_seconds: row.get(4)?,
                remaining_seconds: row.get(5)?,
                end_epoch: row.get(6)?,
                status: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query self timers: {}", e))?;

    let mut timers = Vec::new();
    for row in rows {
        timers.push(row.map_err(|e| format!("Failed to read timer row: {}", e))?);
    }
    Ok(timers)
}

/// Save (upsert) a single self-timer's state.
#[tauri::command]
#[specta::specta]
pub async fn cz_save_self_timer(
    timer: CzSelfTimer,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.cache.db().lock().unwrap();
    db.execute(
        "INSERT OR REPLACE INTO cz_self_timers
         (id, zone, label, category, default_seconds, remaining_seconds, end_epoch, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            timer.id, timer.zone, timer.label, timer.category,
            timer.default_seconds, timer.remaining_seconds, timer.end_epoch, timer.status
        ],
    )
    .map_err(|e| format!("Failed to save timer {}: {}", timer.id, e))?;
    Ok(())
}

/// Reset all self-timers to their default state.
#[tauri::command]
#[specta::specta]
pub async fn cz_reset_all_self_timers(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.cache.db().lock().unwrap();
    db.execute(
        "UPDATE cz_self_timers SET remaining_seconds = default_seconds, end_epoch = 0, status = 'idle'",
        [],
    )
    .map_err(|e| format!("Failed to reset self timers: {}", e))?;
    info!("[contested_zones] All self-timers reset to defaults");
    Ok(())
}
