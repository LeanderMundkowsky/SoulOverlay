use log::info;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WatchEntry {
    pub entity_id: String,
    pub entity_name: String,
    pub entity_kind: String,
    pub entity_slug: String,
    pub terminal_id: String,
    pub terminal_name: String,
    pub price_type: String,
    pub added_at: String,
}

#[tauri::command]
#[specta::specta]
pub async fn get_watchlist(state: State<'_, AppState>) -> Result<Vec<WatchEntry>, String> {
    let db = state.cache.db().lock().unwrap();
    let mut stmt = db
        .prepare(
            "SELECT entity_id, entity_name, entity_kind, entity_slug, terminal_id, terminal_name, price_type, added_at
             FROM watch_list ORDER BY added_at DESC",
        )
        .map_err(|e| format!("Failed to prepare watchlist query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(WatchEntry {
                entity_id: row.get(0)?,
                entity_name: row.get(1)?,
                entity_kind: row.get(2)?,
                entity_slug: row.get(3)?,
                terminal_id: row.get(4)?,
                terminal_name: row.get(5)?,
                price_type: row.get(6)?,
                added_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query watchlist: {}", e))?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| format!("Failed to read watchlist row: {}", e))?);
    }
    Ok(entries)
}

#[tauri::command]
#[specta::specta]
pub async fn add_watch_entry(
    entity_id: String,
    entity_name: String,
    entity_kind: String,
    entity_slug: String,
    terminal_id: String,
    terminal_name: String,
    price_type: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.cache.db().lock().unwrap();
    db.execute(
        "INSERT OR IGNORE INTO watch_list (entity_id, entity_name, entity_kind, entity_slug, terminal_id, terminal_name, price_type)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![entity_id, entity_name, entity_kind, entity_slug, terminal_id, terminal_name, price_type],
    )
    .map_err(|e| format!("Failed to add watch entry: {}", e))?;

    info!("Added watch entry: {} at {} ({})", entity_name, terminal_name, price_type);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn remove_watch_entry(
    entity_id: String,
    terminal_id: String,
    price_type: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.cache.db().lock().unwrap();
    db.execute(
        "DELETE FROM watch_list WHERE entity_id = ?1 AND terminal_id = ?2 AND price_type = ?3",
        rusqlite::params![entity_id, terminal_id, price_type],
    )
    .map_err(|e| format!("Failed to remove watch entry: {}", e))?;

    info!("Removed watch entry: {} at {} ({})", entity_id, terminal_id, price_type);
    Ok(())
}
