use log::info;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::cache_store::{CacheResult, Collection};
use crate::state::AppState;
use crate::uex::types::{HangarVehicle, UexResult};

/// Allowed location slugs for inventory storage.
const STORAGE_SLUGS: &[&str] = &["space_station", "city", "outpost", "poi"];

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InventoryEntry {
    pub id: i32,
    pub entity_id: String,
    pub entity_name: String,
    pub entity_kind: String,
    pub location_id: String,
    pub location_name: String,
    pub location_slug: String,
    pub quantity: i32,
    pub collection: String,
    pub added_at: String,
    pub updated_at: String,
}

// ── Queries ────────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn get_inventory(state: State<'_, AppState>) -> Result<Vec<InventoryEntry>, String> {
    let db = state.cache.db().lock().unwrap();
    let mut stmt = db
        .prepare(
            "SELECT id, entity_id, entity_name, entity_kind, location_id, location_name,
                    location_slug, quantity, collection, added_at, updated_at
             FROM inventory ORDER BY location_name, entity_name",
        )
        .map_err(|e| format!("Failed to prepare inventory query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(InventoryEntry {
                id: row.get(0)?,
                entity_id: row.get(1)?,
                entity_name: row.get(2)?,
                entity_kind: row.get(3)?,
                location_id: row.get(4)?,
                location_name: row.get(5)?,
                location_slug: row.get(6)?,
                quantity: row.get(7)?,
                collection: row.get(8)?,
                added_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| format!("Failed to query inventory: {}", e))?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| format!("Failed to read inventory row: {}", e))?);
    }
    Ok(entries)
}

#[tauri::command]
#[specta::specta]
pub async fn add_inventory_entry(
    entity_id: String,
    entity_name: String,
    entity_kind: String,
    location_id: String,
    location_name: String,
    location_slug: String,
    quantity: i32,
    collection: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.cache.db().lock().unwrap();
    db.execute(
        "INSERT INTO inventory (entity_id, entity_name, entity_kind, location_id, location_name, location_slug, quantity, collection)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(entity_id, location_id, collection) DO UPDATE SET
             quantity = quantity + excluded.quantity,
             updated_at = datetime('now')",
        rusqlite::params![entity_id, entity_name, entity_kind, location_id, location_name, location_slug, quantity, collection],
    )
    .map_err(|e| format!("Failed to add inventory entry: {}", e))?;

    info!(
        "Added {}x {} to {} (collection: {})",
        quantity,
        entity_name,
        location_name,
        if collection.is_empty() { "none" } else { &collection }
    );
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn update_inventory_quantity(
    id: i32,
    quantity: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.cache.db().lock().unwrap();
    db.execute(
        "UPDATE inventory SET quantity = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![quantity, id],
    )
    .map_err(|e| format!("Failed to update inventory quantity: {}", e))?;

    info!("Updated inventory entry {} to quantity {}", id, quantity);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn remove_inventory_entry(id: i32, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.cache.db().lock().unwrap();
    db.execute("DELETE FROM inventory WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| format!("Failed to remove inventory entry: {}", e))?;

    info!("Removed inventory entry {}", id);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn remove_inventory_quantity(
    id: i32,
    quantity: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.cache.db().lock().unwrap();

    let current: i32 = db
        .query_row(
            "SELECT quantity FROM inventory WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to read inventory entry: {}", e))?;

    if quantity >= current {
        db.execute("DELETE FROM inventory WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| format!("Failed to remove inventory entry: {}", e))?;
        info!("Removed inventory entry {} (quantity {} >= {})", id, quantity, current);
    } else {
        db.execute(
            "UPDATE inventory SET quantity = quantity - ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![quantity, id],
        )
        .map_err(|e| format!("Failed to decrease inventory quantity: {}", e))?;
        info!("Decreased inventory entry {} by {} (was {})", id, quantity, current);
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn transfer_inventory(
    id: i32,
    quantity: i32,
    target_location_id: String,
    target_location_name: String,
    target_location_slug: String,
    target_collection: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.cache.db().lock().unwrap();

    // Read source entry
    let src: InventoryEntry = db
        .query_row(
            "SELECT id, entity_id, entity_name, entity_kind, location_id, location_name,
                    location_slug, quantity, collection, added_at, updated_at
             FROM inventory WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(InventoryEntry {
                    id: row.get(0)?,
                    entity_id: row.get(1)?,
                    entity_name: row.get(2)?,
                    entity_kind: row.get(3)?,
                    location_id: row.get(4)?,
                    location_name: row.get(5)?,
                    location_slug: row.get(6)?,
                    quantity: row.get(7)?,
                    collection: row.get(8)?,
                    added_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            },
        )
        .map_err(|e| format!("Failed to read source inventory entry: {}", e))?;

    let transfer_qty = if quantity <= 0 || quantity >= src.quantity {
        src.quantity
    } else {
        quantity
    };

    // Decrease or remove source
    if transfer_qty >= src.quantity {
        db.execute("DELETE FROM inventory WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| format!("Failed to remove source entry: {}", e))?;
    } else {
        db.execute(
            "UPDATE inventory SET quantity = quantity - ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![transfer_qty, id],
        )
        .map_err(|e| format!("Failed to decrease source entry: {}", e))?;
    }

    // Upsert destination (merge if same entity+location+collection exists)
    db.execute(
        "INSERT INTO inventory (entity_id, entity_name, entity_kind, location_id, location_name, location_slug, quantity, collection)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(entity_id, location_id, collection) DO UPDATE SET
             quantity = quantity + excluded.quantity,
             updated_at = datetime('now')",
        rusqlite::params![
            src.entity_id,
            src.entity_name,
            src.entity_kind,
            target_location_id,
            target_location_name,
            target_location_slug,
            transfer_qty,
            target_collection,
        ],
    )
    .map_err(|e| format!("Failed to upsert destination entry: {}", e))?;

    info!(
        "Transferred {}x {} from {} to {} (collection: {})",
        transfer_qty,
        src.entity_name,
        src.location_name,
        target_location_name,
        if target_collection.is_empty() { "none" } else { &target_collection }
    );
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_inventory_collections(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let db = state.cache.db().lock().unwrap();
    let mut stmt = db
        .prepare("SELECT DISTINCT collection FROM inventory WHERE collection != '' ORDER BY collection")
        .map_err(|e| format!("Failed to prepare collections query: {}", e))?;

    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("Failed to query collections: {}", e))?;

    let mut seen = std::collections::HashSet::new();
    let mut collections = Vec::new();
    for row in rows {
        let coll_str = row.map_err(|e| format!("Failed to read collection: {}", e))?;
        for c in coll_str.split(',') {
            let c = c.trim().to_string();
            if !c.is_empty() && seen.insert(c.clone()) {
                collections.push(c);
            }
        }
    }
    collections.sort();
    Ok(collections)
}

#[tauri::command]
#[specta::specta]
pub async fn update_inventory_entry(
    id: i32,
    entity_id: String,
    entity_name: String,
    entity_kind: String,
    location_id: String,
    location_name: String,
    location_slug: String,
    quantity: i32,
    collection: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.cache.db().lock().unwrap();

    // Delete the old entry, then upsert with the new values.
    // The upsert merges with any existing conflicting entry (same entity+location+collection)
    // and fully replaces its fields with the edited values.
    db.execute("DELETE FROM inventory WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| format!("Failed to remove old inventory entry: {}", e))?;

    db.execute(
        "INSERT INTO inventory (entity_id, entity_name, entity_kind, location_id, location_name, location_slug, quantity, collection)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(entity_id, location_id, collection) DO UPDATE SET
             entity_name    = excluded.entity_name,
             entity_kind    = excluded.entity_kind,
             location_name  = excluded.location_name,
             location_slug  = excluded.location_slug,
             quantity       = excluded.quantity,
             updated_at     = datetime('now')",
        rusqlite::params![entity_id, entity_name, entity_kind, location_id, location_name, location_slug, quantity, collection],
    )
    .map_err(|e| format!("Failed to save updated inventory entry: {}", e))?;

    info!(
        "Updated inventory entry {} → {}x {} at {} (collection: {})",
        id,
        quantity,
        entity_name,
        location_name,
        if collection.is_empty() { "none" } else { &collection }
    );
    Ok(())
}

// ── Storage location search ────────────────────────────────────────────────

/// Convert a fleet vehicle into a UexResult for the location picker.
fn fleet_to_location(v: &HangarVehicle) -> UexResult {
    let display_name = if v.name.is_empty() || v.name == v.model_name {
        format!("[Ship] {}", v.model_name)
    } else {
        format!("[Ship] {} ({})", v.name, v.model_name)
    };

    UexResult {
        id: format!("fleet_{}", v.id),
        name: display_name,
        kind: "location".to_string(),
        slug: "fleet_vehicle".to_string(),
        ..Default::default()
    }
}

/// Search storage-capable locations + fleet vehicles for the inventory location picker.
#[tauri::command]
#[specta::specta]
pub async fn get_storage_locations(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<UexResult>, String> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    // Search cached locations, filtered to storage-capable slugs
    let locations_key = Collection::Locations.storage_key();
    match state.cache.get::<Vec<UexResult>>(&locations_key) {
        CacheResult::Fresh(data) | CacheResult::Stale(data) => {
            for loc in &data {
                if STORAGE_SLUGS.contains(&loc.slug.as_str())
                    && (query_lower.is_empty()
                        || loc.name.to_lowercase().contains(&query_lower))
                {
                    results.push(loc.clone());
                }
            }
        }
        CacheResult::Missing => {}
    }

    // Search fleet vehicles (gracefully skip if unavailable)
    let fleet_key = Collection::Fleet.storage_key();
    match state.cache.get::<Vec<HangarVehicle>>(&fleet_key) {
        CacheResult::Fresh(data) | CacheResult::Stale(data) => {
            for vehicle in &data {
                let loc = fleet_to_location(vehicle);
                if query_lower.is_empty() || loc.name.to_lowercase().contains(&query_lower) {
                    results.push(loc);
                }
            }
        }
        CacheResult::Missing => {}
    }

    Ok(results)
}
