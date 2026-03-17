use log::info;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Favorite {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub slug: String,
    pub uuid: String,
    pub source: String,
    pub added_at: String,
}

#[tauri::command]
#[specta::specta]
pub async fn get_favorites(state: State<'_, AppState>) -> Result<Vec<Favorite>, String> {
    let db = state.cache.db().lock().unwrap();
    let mut stmt = db
        .prepare("SELECT id, name, kind, slug, uuid, source, added_at FROM favorites ORDER BY kind, name")
        .map_err(|e| format!("Failed to prepare favorites query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Favorite {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: row.get(2)?,
                slug: row.get(3)?,
                uuid: row.get(4)?,
                source: row.get(5)?,
                added_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query favorites: {}", e))?;

    let mut favorites = Vec::new();
    for row in rows {
        favorites.push(row.map_err(|e| format!("Failed to read favorite row: {}", e))?);
    }
    Ok(favorites)
}

#[tauri::command]
#[specta::specta]
pub async fn add_favorite(
    id: String,
    name: String,
    kind: String,
    slug: String,
    uuid: String,
    source: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.cache.db().lock().unwrap();
    db.execute(
        "INSERT OR IGNORE INTO favorites (id, name, kind, slug, uuid, source) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, name, kind, slug, uuid, source],
    )
    .map_err(|e| format!("Failed to add favorite: {}", e))?;

    info!("Added favorite: {} ({}, source={})", name, kind, source);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn remove_favorite(
    id: String,
    kind: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.cache.db().lock().unwrap();
    db.execute(
        "DELETE FROM favorites WHERE id = ?1 AND kind = ?2",
        rusqlite::params![id, kind],
    )
    .map_err(|e| format!("Failed to remove favorite: {}", e))?;

    info!("Removed favorite: {} ({})", id, kind);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn is_favorite(
    id: String,
    kind: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let db = state.cache.db().lock().unwrap();
    let count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM favorites WHERE id = ?1 AND kind = ?2",
            rusqlite::params![id, kind],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to check favorite: {}", e))?;
    Ok(count > 0)
}
