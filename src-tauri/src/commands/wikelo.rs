use log::info;
use tauri::State;

use crate::cache_store::{CacheResult, Collection};
use crate::commands::api::ApiResponse;
use crate::providers::wikelo::{fetch_all_trades, WikeloTrade};
use crate::state::AppState;

// ── Get trades ────────────────────────────────────────────────────────────

/// Return all Wikelo contracts, fetching and caching from wikelotrades.com if needed.
#[tauri::command]
#[specta::specta]
pub async fn wikelo_get_trades(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<WikeloTrade>>, String> {
    let cache = &state.cache;
    let collection = Collection::WikiloTrades;

    // Try to load from cache first.
    match cache.get::<Vec<WikeloTrade>>(&collection.storage_key()) {
        CacheResult::Fresh(trades) => {
            info!("Wikelo trades: cache hit ({} trades)", trades.len());
            return Ok(ApiResponse::ok(trades));
        }
        CacheResult::Stale(trades) => {
            // Use stale data but trigger a refresh in the background.
            info!("Wikelo trades: cache expired, returning stale data and refreshing");
            let http = state.uex.client().clone();
            let cache_clone = state.cache_arc();
            let ttl = collection.ttl_for(&*state.current_settings.lock().unwrap());
            tokio::spawn(async move {
                match fetch_all_trades(&http).await {
                    Ok(fresh) => {
                        let _ = cache_clone.put(&collection.storage_key(), ttl, &fresh);
                        info!("Wikelo trades: background refresh complete ({} trades)", fresh.len());
                    }
                    Err(e) => log::warn!("Wikelo trades: background refresh failed: {}", e),
                }
            });
            return Ok(ApiResponse::ok(trades));
        }
        CacheResult::Missing => {}
    }

    // Cache miss: fetch synchronously.
    info!("Wikelo trades: cache miss, fetching from wikelotrades.com");
    let http = state.uex.client();
    let ttl = {
        let settings = state.current_settings.lock().unwrap();
        collection.ttl_for(&settings)
    };

    match fetch_all_trades(http).await {
        Ok(trades) => {
            let _ = cache.put(&collection.storage_key(), ttl, &trades);
            Ok(ApiResponse::ok(trades))
        }
        Err(e) => Ok(ApiResponse::err(e)),
    }
}

// ── Completion tracking ────────────────────────────────────────────────────

/// Return all completed contract mission IDs from the local DB.
#[tauri::command]
#[specta::specta]
pub async fn wikelo_get_completions(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let db = state.cache.db().lock().unwrap();
    let mut stmt = db
        .prepare("SELECT mission_id FROM wikelo_completions ORDER BY completed_at DESC")
        .map_err(|e| format!("Failed to prepare wikelo_completions query: {}", e))?;

    let ids = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("Failed to query wikelo_completions: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect wikelo_completions: {}", e))?;

    Ok(ids)
}

/// Toggle a contract's completion status. Returns `true` if now completed, `false` if cleared.
#[tauri::command]
#[specta::specta]
pub async fn wikelo_toggle_completion(
    mission_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let db = state.cache.db().lock().unwrap();

    // Check current state.
    let exists: bool = db
        .query_row(
            "SELECT COUNT(*) FROM wikelo_completions WHERE mission_id = ?1",
            rusqlite::params![mission_id],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .map_err(|e| format!("Failed to check completion: {}", e))?;

    if exists {
        db.execute(
            "DELETE FROM wikelo_completions WHERE mission_id = ?1",
            rusqlite::params![mission_id],
        )
        .map_err(|e| format!("Failed to remove completion: {}", e))?;
        Ok(false)
    } else {
        db.execute(
            "INSERT INTO wikelo_completions (mission_id) VALUES (?1)",
            rusqlite::params![mission_id],
        )
        .map_err(|e| format!("Failed to insert completion: {}", e))?;
        Ok(true)
    }
}
