use log::{error, info};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::cache_store::{CacheResult, Collection};
use crate::commands::backend::{extract_error_message, http_client};
use crate::constants::BACKEND_URL;
use crate::state::AppState;
use crate::uex::types::{HangarVehicle, UexResult};

/// Allowed location slugs for inventory storage.
const STORAGE_SLUGS: &[&str] = &["space_station", "city", "outpost", "poi"];

// ── IPC types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InventoryCollection {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InventoryEntry {
    pub id: u32,
    pub entity_id: String,
    pub entity_name: String,
    pub entity_kind: String,
    pub location_id: String,
    pub location_name: String,
    pub location_slug: String,
    pub quantity: i32,
    pub collections: Vec<InventoryCollection>,
    pub added_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct TransferResult {
    pub source: Option<InventoryEntry>,
    pub target: InventoryEntry,
}

// ── Private DTO types (not exported to IPC) ────────────────────────────────

#[derive(Deserialize)]
struct BackendCollection {
    id: u32,
    name: String,
}

#[derive(Deserialize)]
struct BackendEntry {
    id: u32,
    entity_id: String,
    entity_name: String,
    entity_kind: String,
    location_id: String,
    location_name: String,
    location_slug: String,
    quantity: i32,
    collections: Vec<BackendCollection>,
    added_at: String,
    updated_at: String,
}

impl From<BackendCollection> for InventoryCollection {
    fn from(c: BackendCollection) -> Self {
        InventoryCollection { id: c.id, name: c.name }
    }
}

impl From<BackendEntry> for InventoryEntry {
    fn from(e: BackendEntry) -> Self {
        InventoryEntry {
            id: e.id,
            entity_id: e.entity_id,
            entity_name: e.entity_name,
            entity_kind: e.entity_kind,
            location_id: e.location_id,
            location_name: e.location_name,
            location_slug: e.location_slug,
            quantity: e.quantity,
            collections: e.collections.into_iter().map(|c| c.into()).collect(),
            added_at: e.added_at,
            updated_at: e.updated_at,
        }
    }
}

// ── Private helpers ────────────────────────────────────────────────────────

fn get_token(state: &State<AppState>) -> Result<String, String> {
    let token = state.current_settings.lock().unwrap().backend_api_token.clone();
    if token.is_empty() {
        Err("Not logged in".to_string())
    } else {
        Ok(token)
    }
}

fn cache_entry(db: &rusqlite::Connection, entry: &InventoryEntry) -> Result<(), rusqlite::Error> {
    let collections_json = serde_json::to_string(&entry.collections).unwrap_or_default();
    db.execute(
        "INSERT OR REPLACE INTO inventory
            (id, entity_id, entity_name, entity_kind, location_id, location_name,
             location_slug, quantity, collections_json, added_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            entry.id,
            entry.entity_id,
            entry.entity_name,
            entry.entity_kind,
            entry.location_id,
            entry.location_name,
            entry.location_slug,
            entry.quantity,
            collections_json,
            entry.added_at,
            entry.updated_at,
        ],
    )?;
    Ok(())
}

fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<InventoryEntry> {
    let collections_json: String = row.get(8)?;
    let collections: Vec<InventoryCollection> =
        serde_json::from_str(&collections_json).unwrap_or_default();
    Ok(InventoryEntry {
        id: row.get(0)?,
        entity_id: row.get(1)?,
        entity_name: row.get(2)?,
        entity_kind: row.get(3)?,
        location_id: row.get(4)?,
        location_name: row.get(5)?,
        location_slug: row.get(6)?,
        quantity: row.get(7)?,
        collections,
        added_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn load_all_entries(db: &rusqlite::Connection) -> Result<Vec<InventoryEntry>, String> {
    let mut stmt = db
        .prepare(
            "SELECT id, entity_id, entity_name, entity_kind, location_id, location_name,
                    location_slug, quantity, collections_json, added_at, updated_at
             FROM inventory ORDER BY location_name, entity_name",
        )
        .map_err(|e| format!("Failed to prepare inventory query: {}", e))?;

    let rows = stmt
        .query_map([], row_to_entry)
        .map_err(|e| format!("Failed to query inventory: {}", e))?;

    rows.map(|r| r.map_err(|e| format!("Failed to read row: {}", e))).collect()
}

fn parse_entry(val: &serde_json::Value) -> Result<InventoryEntry, String> {
    serde_json::from_value::<BackendEntry>(val.clone())
        .map(|e| e.into())
        .map_err(|e| format!("Failed to parse inventory entry: {}", e))
}

fn parse_collection(val: &serde_json::Value) -> Result<InventoryCollection, String> {
    let id = val["id"].as_u64().ok_or("Missing id in collection response")? as u32;
    let name = val["name"].as_str().ok_or("Missing name in collection response")?.to_string();
    Ok(InventoryCollection { id, name })
}

// ── Startup sync ───────────────────────────────────────────────────────────

/// Fetch all inventory entries from the backend and repopulate the SQLite cache.
/// Called on startup after the user session is restored.
pub async fn sync_inventory_from_backend(handle: &tauri::AppHandle) {
    use tauri::Manager;
    let state = handle.state::<AppState>();
    let token = state.current_settings.lock().unwrap().backend_api_token.clone();
    if token.is_empty() {
        return;
    }

    let client = match http_client() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to create HTTP client for inventory sync: {}", e);
            return;
        }
    };

    let url = format!("{}/api/inventory", BACKEND_URL);
    let resp = match client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Inventory sync failed: {}", e);
            return;
        }
    };

    if !resp.status().is_success() {
        error!("Inventory sync failed with status: {}", resp.status());
        return;
    }

    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(e) => {
            error!("Failed to parse inventory sync response: {}", e);
            return;
        }
    };

    let Some(data) = json["data"].as_array() else {
        error!("Inventory sync: unexpected response shape");
        return;
    };

    let entries: Vec<InventoryEntry> = data.iter().filter_map(|v| parse_entry(v).ok()).collect();

    let db = state.cache.db().lock().unwrap();
    if let Err(e) = db.execute("DELETE FROM inventory", []) {
        error!("Failed to clear inventory cache: {}", e);
        return;
    }
    for entry in &entries {
        if let Err(e) = cache_entry(&db, entry) {
            error!("Failed to cache inventory entry {}: {}", entry.id, e);
        }
    }
    info!("Synced {} inventory entries from backend", entries.len());
}

/// Migrate inventory entries from the legacy SQLite table to the backend API.
/// Called on startup (after account restore) and after first login.
/// Safe to call multiple times — drops the legacy table only on full success.
pub async fn migrate_legacy_inventory(handle: &tauri::AppHandle) {
    use tauri::Manager;
    let state = handle.state::<AppState>();

    // Check if the legacy table exists
    let legacy_count = {
        let db = state.cache.db().lock().unwrap();
        db.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='inventory_legacy'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
    };
    if legacy_count == 0 {
        return; // No legacy data
    }

    let token = state.current_settings.lock().unwrap().backend_api_token.clone();
    if token.is_empty() {
        return; // Not logged in — retry next time
    }

    info!("Found legacy inventory data — migrating to backend...");

    // Read all legacy entries
    #[derive(Debug)]
    struct LegacyEntry {
        entity_id: String,
        entity_name: String,
        entity_kind: String,
        location_id: String,
        location_name: String,
        location_slug: String,
        quantity: i32,
        collection: String,
    }

    let legacy_entries: Vec<LegacyEntry> = {
        let db = state.cache.db().lock().unwrap();
        let mut stmt = match db.prepare(
            "SELECT entity_id, entity_name, entity_kind, location_id, location_name,
                    location_slug, quantity, collection FROM inventory_legacy",
        ) {
            Ok(s) => s,
            Err(e) => { error!("Failed to prepare legacy inventory query: {}", e); return; }
        };
        let rows = stmt.query_map([], |row| {
            Ok(LegacyEntry {
                entity_id: row.get(0)?,
                entity_name: row.get(1)?,
                entity_kind: row.get(2)?,
                location_id: row.get(3)?,
                location_name: row.get(4)?,
                location_slug: row.get(5)?,
                quantity: row.get(6)?,
                collection: row.get(7)?,
            })
        });
        match rows {
            Ok(r) => r.filter_map(|e| e.ok()).collect(),
            Err(e) => { error!("Failed to query legacy inventory: {}", e); return; }
        }
    };

    if legacy_entries.is_empty() {
        let db = state.cache.db().lock().unwrap();
        let _ = db.execute("DROP TABLE IF EXISTS inventory_legacy", []);
        info!("Legacy inventory table was empty — dropped");
        return;
    }

    let client = match http_client() {
        Ok(c) => c,
        Err(e) => { error!("Migration HTTP client error: {}", e); return; }
    };

    // Collect unique collection names from legacy data
    let mut all_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for entry in &legacy_entries {
        for name in entry.collection.split(',') {
            let trimmed = name.trim().to_string();
            if !trimmed.is_empty() {
                all_names.insert(trimmed);
            }
        }
    }

    // Fetch existing backend collections to avoid duplicate creation
    let mut collection_map: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let coll_url = format!("{}/api/inventory/collections", BACKEND_URL);
    if let Ok(resp) = client
        .get(&coll_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            if let Some(arr) = json["data"].as_array() {
                for c in arr {
                    if let (Some(id), Some(name)) = (c["id"].as_u64(), c["name"].as_str()) {
                        collection_map.insert(name.to_string(), id as u32);
                    }
                }
            }
        }
    }

    // Create any collections that don't exist yet on the backend
    for name in &all_names {
        if collection_map.contains_key(name) {
            continue;
        }
        let create_url = format!("{}/api/inventory/collections", BACKEND_URL);
        match client
            .post(&create_url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({ "name": name }))
            .send()
            .await
        {
            Ok(resp) => {
                let status = resp.status();
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if status.is_success() {
                        if let (Some(id), Some(n)) =
                            (json["data"]["id"].as_u64(), json["data"]["name"].as_str())
                        {
                            collection_map.insert(n.to_string(), id as u32);
                        }
                    }
                }
            }
            Err(e) => { error!("Failed to create collection '{}': {}", name, e); }
        }
    }

    // Push each entry to the backend
    let inv_url = format!("{}/api/inventory", BACKEND_URL);
    let mut success_count = 0u32;
    let mut fail_count = 0u32;

    for entry in &legacy_entries {
        let collection_ids: Vec<u32> = entry
            .collection
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .filter_map(|name| collection_map.get(name).copied())
            .collect();

        match client
            .post(&inv_url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "entity_id": entry.entity_id,
                "entity_name": entry.entity_name,
                "entity_kind": entry.entity_kind,
                "location_id": entry.location_id,
                "location_name": entry.location_name,
                "location_slug": entry.location_slug,
                "quantity": entry.quantity,
                "collection_ids": collection_ids,
            }))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => { success_count += 1; }
            Ok(resp) => {
                error!(
                    "Failed to migrate '{}' at '{}': HTTP {}",
                    entry.entity_name, entry.location_name, resp.status()
                );
                fail_count += 1;
            }
            Err(e) => {
                error!("Network error migrating '{}': {}", entry.entity_name, e);
                fail_count += 1;
            }
        }
    }

    info!(
        "Legacy inventory migration complete: {} succeeded, {} failed",
        success_count, fail_count
    );

    if fail_count == 0 {
        let db = state.cache.db().lock().unwrap();
        if let Err(e) = db.execute("DROP TABLE IF EXISTS inventory_legacy", []) {
            error!("Failed to drop legacy inventory table: {}", e);
        } else {
            info!("Legacy inventory table dropped after successful migration");
        }
    } else {
        info!("Some entries failed — legacy table kept for retry on next startup");
    }
}

// ── Commands exposed for frontend post-login flow ──────────────────────────

/// Migrate legacy SQLite inventory data to the backend (one-time, idempotent).
/// The frontend calls this after login so users who weren't logged in on startup
/// still get their data migrated before the first inventory load.
#[tauri::command]
#[specta::specta]
pub async fn inventory_migrate_legacy(
    handle: tauri::AppHandle,
) -> Result<(), String> {
    migrate_legacy_inventory(&handle).await;
    Ok(())
}

/// Sync inventory from backend → SQLite cache and return the fresh entries.
/// Called by the frontend after login or after migrate_legacy.
#[tauri::command]
#[specta::specta]
pub async fn inventory_sync_from_backend(
    handle: tauri::AppHandle,
) -> Result<Vec<InventoryEntry>, String> {
    sync_inventory_from_backend(&handle).await;
    use tauri::Manager;
    let state = handle.state::<AppState>();
    let db = state.cache.db().lock().unwrap();
    load_all_entries(&db)
}

#[tauri::command]
#[specta::specta]
pub async fn get_inventory(state: State<'_, AppState>) -> Result<Vec<InventoryEntry>, String> {
    let db = state.cache.db().lock().unwrap();
    load_all_entries(&db)
}

#[tauri::command]
#[specta::specta]
pub async fn get_inventory_collections(
    state: State<'_, AppState>,
) -> Result<Vec<InventoryCollection>, String> {
    let token = get_token(&state)?;
    let client = http_client()?;
    let url = format!("{}/api/inventory/collections", BACKEND_URL);

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Failed to fetch collections ({})", resp.status()));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    let data = json["data"].as_array().ok_or("Unexpected response shape")?;
    data.iter().map(parse_collection).collect()
}

// ── Mutations ──────────────────────────────────────────────────────────────

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
    collection_ids: Vec<u32>,
    state: State<'_, AppState>,
) -> Result<InventoryEntry, String> {
    let token = get_token(&state)?;
    let client = http_client()?;
    let url = format!("{}/api/inventory", BACKEND_URL);

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "entity_id": entity_id,
            "entity_name": entity_name,
            "entity_kind": entity_kind,
            "location_id": location_id,
            "location_name": location_name,
            "location_slug": location_slug,
            "quantity": quantity,
            "collection_ids": collection_ids,
        }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    if !status.is_success() {
        return Err(extract_error_message(&json));
    }

    let entry = parse_entry(&json["data"])?;
    let db = state.cache.db().lock().unwrap();
    cache_entry(&db, &entry).map_err(|e| format!("Cache error: {}", e))?;
    info!("Added {}x {} to {}", entry.quantity, entry.entity_name, entry.location_name);
    Ok(entry)
}

#[tauri::command]
#[specta::specta]
pub async fn update_inventory_entry(
    id: u32,
    entity_id: String,
    entity_name: String,
    entity_kind: String,
    location_id: String,
    location_name: String,
    location_slug: String,
    quantity: i32,
    collection_ids: Vec<u32>,
    state: State<'_, AppState>,
) -> Result<InventoryEntry, String> {
    let token = get_token(&state)?;
    let client = http_client()?;
    let url = format!("{}/api/inventory/{}", BACKEND_URL, id);

    let resp = client
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "entity_id": entity_id,
            "entity_name": entity_name,
            "entity_kind": entity_kind,
            "location_id": location_id,
            "location_name": location_name,
            "location_slug": location_slug,
            "quantity": quantity,
            "collection_ids": collection_ids,
        }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    if !status.is_success() {
        return Err(extract_error_message(&json));
    }

    let entry = parse_entry(&json["data"])?;
    let db = state.cache.db().lock().unwrap();
    // Remove old entry (id may be same, but safe to re-insert via REPLACE)
    let _ = db.execute("DELETE FROM inventory WHERE id = ?1", rusqlite::params![id]);
    cache_entry(&db, &entry).map_err(|e| format!("Cache error: {}", e))?;
    info!("Updated inventory entry {} → {}x {} at {}", id, entry.quantity, entry.entity_name, entry.location_name);
    Ok(entry)
}

#[tauri::command]
#[specta::specta]
pub async fn update_inventory_quantity(
    id: u32,
    quantity: i32,
    state: State<'_, AppState>,
) -> Result<InventoryEntry, String> {
    let token = get_token(&state)?;
    let client = http_client()?;
    let url = format!("{}/api/inventory/{}", BACKEND_URL, id);

    let resp = client
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "quantity": quantity }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    if !status.is_success() {
        return Err(extract_error_message(&json));
    }

    let entry = parse_entry(&json["data"])?;
    let db = state.cache.db().lock().unwrap();
    cache_entry(&db, &entry).map_err(|e| format!("Cache error: {}", e))?;
    info!("Updated quantity for inventory entry {} to {}", id, quantity);
    Ok(entry)
}

#[tauri::command]
#[specta::specta]
pub async fn remove_inventory_entry(id: u32, state: State<'_, AppState>) -> Result<(), String> {
    let token = get_token(&state)?;
    let client = http_client()?;
    let url = format!("{}/api/inventory/{}", BACKEND_URL, id);

    let resp = client
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    if status.as_u16() == 204 || status.is_success() {
        let db = state.cache.db().lock().unwrap();
        let _ = db.execute("DELETE FROM inventory WHERE id = ?1", rusqlite::params![id]);
        info!("Removed inventory entry {}", id);
        return Ok(());
    }

    Err(format!("Failed to delete inventory entry ({})", status))
}

#[tauri::command]
#[specta::specta]
pub async fn remove_inventory_quantity(
    id: u32,
    quantity: i32,
    state: State<'_, AppState>,
) -> Result<Option<InventoryEntry>, String> {
    let token = get_token(&state)?;
    let client = http_client()?;
    let url = format!("{}/api/inventory/{}/remove-quantity", BACKEND_URL, id);

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "quantity": quantity }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    if !status.is_success() {
        return Err(extract_error_message(&json));
    }

    let db = state.cache.db().lock().unwrap();
    if json["data"].is_null() {
        let _ = db.execute("DELETE FROM inventory WHERE id = ?1", rusqlite::params![id]);
        info!("Removed inventory entry {} (quantity depleted)", id);
        Ok(None)
    } else {
        let entry = parse_entry(&json["data"])?;
        cache_entry(&db, &entry).map_err(|e| format!("Cache error: {}", e))?;
        info!("Decreased inventory entry {} by {}", id, quantity);
        Ok(Some(entry))
    }
}

#[tauri::command]
#[specta::specta]
pub async fn transfer_inventory(
    id: u32,
    quantity: i32,
    target_location_id: String,
    target_location_name: String,
    target_location_slug: String,
    target_collection_ids: Vec<u32>,
    state: State<'_, AppState>,
) -> Result<TransferResult, String> {
    let token = get_token(&state)?;
    let client = http_client()?;
    let url = format!("{}/api/inventory/{}/transfer", BACKEND_URL, id);

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "quantity": quantity,
            "target_location_id": target_location_id,
            "target_location_name": target_location_name,
            "target_location_slug": target_location_slug,
            "target_collection_ids": target_collection_ids,
        }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    if !status.is_success() {
        return Err(extract_error_message(&json));
    }

    let data = &json["data"];
    let source = if data["source"].is_null() {
        None
    } else {
        Some(parse_entry(&data["source"])?)
    };
    let target = parse_entry(&data["target"])?;

    let db = state.cache.db().lock().unwrap();
    if let Some(ref src) = source {
        cache_entry(&db, src).map_err(|e| format!("Cache error (source): {}", e))?;
    } else {
        let _ = db.execute("DELETE FROM inventory WHERE id = ?1", rusqlite::params![id]);
    }
    cache_entry(&db, &target).map_err(|e| format!("Cache error (target): {}", e))?;
    info!("Transferred {}x to {}", quantity, target_location_name);
    Ok(TransferResult { source, target })
}

// ── Collection commands ────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn inventory_collection_create(
    name: String,
    state: State<'_, AppState>,
) -> Result<InventoryCollection, String> {
    let token = get_token(&state)?;
    let client = http_client()?;
    let url = format!("{}/api/inventory/collections", BACKEND_URL);

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "name": name }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    if !status.is_success() {
        return Err(extract_error_message(&json));
    }

    parse_collection(&json["data"])
}

#[tauri::command]
#[specta::specta]
pub async fn inventory_collection_update(
    id: u32,
    name: String,
    state: State<'_, AppState>,
) -> Result<InventoryCollection, String> {
    let token = get_token(&state)?;
    let client = http_client()?;
    let url = format!("{}/api/inventory/collections/{}", BACKEND_URL, id);

    let resp = client
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "name": name }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    if !status.is_success() {
        return Err(extract_error_message(&json));
    }

    parse_collection(&json["data"])
}

#[tauri::command]
#[specta::specta]
pub async fn inventory_collection_delete(
    id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let token = get_token(&state)?;
    let client = http_client()?;
    let url = format!("{}/api/inventory/collections/{}", BACKEND_URL, id);

    let resp = client
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    if status.as_u16() == 204 || status.is_success() {
        info!("Deleted collection {}", id);
        return Ok(());
    }

    Err(format!("Failed to delete collection ({})", status))
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
