/// api.rs — Structured API wrapper over all UEX data endpoints.
///
/// Every command returns `ApiResponse<T>`, a consistent envelope:
///
/// ```json
/// { "ok": true,  "data": <T>,   "error": null }
/// { "ok": false, "data": null,  "error": "…"  }
/// ```
///
/// Endpoints
/// ---------
/// | Tauri command            | JS invoke string           | Description                                |
/// |--------------------------|----------------------------|--------------------------------------------|
/// | `api_search`             | `"api_search"`             | Search all entity types by query string    |
/// | `api_search_commodities` | `"api_search_commodities"` | Search commodities only                    |
/// | `api_search_vehicles`    | `"api_search_vehicles"`    | Search vehicles / ground vehicles          |
/// | `api_search_items`       | `"api_search_items"`       | Search items                               |
/// | `api_search_locations`   | `"api_search_locations"`   | Search locations (terminals)               |
/// | `api_commodity_prices`   | `"api_commodity_prices"`   | Fetch buy/sell prices for a commodity ID   |
use serde::Serialize;
use tauri::State;

use crate::cache_store::{CacheResult, Collection};
use crate::state::AppState;
use crate::uex_client::{self, PriceEntry, UexResult};

// ── Response envelope ──────────────────────────────────────────────────────

/// Uniform response wrapper returned by every API command.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    /// Indicates the data was served from an expired cache entry.
    /// Frontend can show a "refreshing..." indicator when this is true.
    pub stale: bool,
    /// Total number of matches before any limit was applied (None when not applicable).
    pub total: Option<usize>,
}

impl<T: Serialize> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self { ok: true, data: Some(data), error: None, stale: false, total: None }
    }

    fn ok_stale(data: T) -> Self {
        Self { ok: true, data: Some(data), error: None, stale: true, total: None }
    }

    fn err(msg: impl Into<String>) -> Self {
        Self { ok: false, data: None, error: Some(msg.into()), stale: false, total: None }
    }
}

// ── Helper ─────────────────────────────────────────────────────────────────

/// Pull the UEX API key from current settings.
fn api_key(state: &AppState) -> String {
    state.current_settings.lock().unwrap().uex_api_key.clone()
}

/// Search a single collection from cache. Returns (results, is_stale).
/// Falls back to a direct API call if the cache is empty.
async fn search_cached_or_fetch(
    state: &AppState,
    collection: Collection,
    query: &str,
    api_key: &str,
) -> Result<(Vec<UexResult>, bool), String> {
    let key = collection.storage_key();
    match state.cache.get::<Vec<UexResult>>(&key) {
        CacheResult::Fresh(data) => {
            Ok((uex_client::search_in_collection(&data, query), false))
        }
        CacheResult::Stale(data) => {
            Ok((uex_client::search_in_collection(&data, query), true))
        }
        CacheResult::Missing => {
            // Fallback: direct API call for this specific query
            let results = match collection {
                Collection::Commodities => uex_client::search_commodities(query, api_key).await?,
                Collection::Vehicles => uex_client::search_vehicles(query, api_key).await?,
                Collection::Items => uex_client::search_items(query, api_key).await?,
                Collection::Locations => uex_client::search_locations(query, api_key).await?,
                Collection::CommodityPrices => return Err("Use api_commodity_prices for prices".to_string()),
            };
            Ok((results, false))
        }
    }
}

// ── Search endpoints ───────────────────────────────────────────────────────

/// Search all UEX entity types (commodities, vehicles, items, locations)
/// from the local cache, falling back to direct API calls.
///
/// Parameters:
/// - `query`: free-text search string
#[tauri::command]
pub async fn api_search(
    query: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<UexResult>>, String> {
    if query.trim().is_empty() {
        return Ok(ApiResponse::ok(vec![]));
    }

    let api_key = api_key(&state);
    let max_results = state.current_settings.lock().unwrap().max_search_results as usize;
    let mut all_results = Vec::new();
    let mut any_stale = false;

    let collections = [
        Collection::Commodities,
        Collection::Vehicles,
        Collection::Items,
        Collection::Locations,
    ];

    for collection in &collections {
        match search_cached_or_fetch(&state, *collection, &query, &api_key).await {
            Ok((mut results, stale)) => {
                all_results.append(&mut results);
                if stale { any_stale = true; }
            }
            Err(e) => {
                // Log but don't fail the entire search
                log::warn!("Search failed for {:?} (query={:?}): {}", collection, query, e);
            }
        }
    }

    let total = all_results.len();
    all_results.truncate(max_results);
    let mut resp = if any_stale {
        ApiResponse::ok_stale(all_results)
    } else {
        ApiResponse::ok(all_results)
    };
    resp.total = Some(total);
    Ok(resp)
}

/// Search UEX commodities only.
#[tauri::command]
pub async fn api_search_commodities(
    query: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<UexResult>>, String> {
    if query.trim().is_empty() {
        return Ok(ApiResponse::ok(vec![]));
    }

    let api_key = api_key(&state);
    match search_cached_or_fetch(&state, Collection::Commodities, &query, &api_key).await {
        Ok((results, stale)) => {
            if stale { Ok(ApiResponse::ok_stale(results)) } else { Ok(ApiResponse::ok(results)) }
        }
        Err(e) => Ok(ApiResponse::err(e)),
    }
}

/// Search UEX vehicles (ships + ground vehicles).
#[tauri::command]
pub async fn api_search_vehicles(
    query: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<UexResult>>, String> {
    if query.trim().is_empty() {
        return Ok(ApiResponse::ok(vec![]));
    }

    let api_key = api_key(&state);
    match search_cached_or_fetch(&state, Collection::Vehicles, &query, &api_key).await {
        Ok((results, stale)) => {
            if stale { Ok(ApiResponse::ok_stale(results)) } else { Ok(ApiResponse::ok(results)) }
        }
        Err(e) => Ok(ApiResponse::err(e)),
    }
}

/// Search UEX items.
#[tauri::command]
pub async fn api_search_items(
    query: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<UexResult>>, String> {
    if query.trim().is_empty() {
        return Ok(ApiResponse::ok(vec![]));
    }

    let api_key = api_key(&state);
    match search_cached_or_fetch(&state, Collection::Items, &query, &api_key).await {
        Ok((results, stale)) => {
            if stale { Ok(ApiResponse::ok_stale(results)) } else { Ok(ApiResponse::ok(results)) }
        }
        Err(e) => Ok(ApiResponse::err(e)),
    }
}

/// Search UEX locations (terminals / stations).
#[tauri::command]
pub async fn api_search_locations(
    query: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<UexResult>>, String> {
    if query.trim().is_empty() {
        return Ok(ApiResponse::ok(vec![]));
    }

    let api_key = api_key(&state);
    match search_cached_or_fetch(&state, Collection::Locations, &query, &api_key).await {
        Ok((results, stale)) => {
            if stale { Ok(ApiResponse::ok_stale(results)) } else { Ok(ApiResponse::ok(results)) }
        }
        Err(e) => Ok(ApiResponse::err(e)),
    }
}

// ── Price endpoint ─────────────────────────────────────────────────────────

/// Fetch buy/sell prices for a commodity by its UEX ID.
/// Uses per-commodity cache key (e.g. `commodity_prices:42`).
#[tauri::command]
pub async fn api_commodity_prices(
    commodity_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<PriceEntry>>, String> {
    if commodity_id.trim().is_empty() {
        return Ok(ApiResponse::err("commodity_id must not be empty"));
    }

    let cache_key = Collection::CommodityPrices.storage_key_with_id(&commodity_id);
    let api_key = api_key(&state);

    // Check cache first
    match state.cache.get::<Vec<PriceEntry>>(&cache_key) {
        CacheResult::Fresh(prices) => {
            return Ok(ApiResponse::ok(prices));
        }
        CacheResult::Stale(prices) => {
            // Return stale data immediately; caller can trigger a refresh
            // We still try to fetch fresh data below, but return stale first
            // Actually — to keep the command simple and avoid complexity,
            // we return stale and let the frontend decide whether to refresh.
            return Ok(ApiResponse::ok_stale(prices));
        }
        CacheResult::Missing => {
            // Fall through to fetch
        }
    }

    // Fetch from API
    match uex_client::get_prices(&commodity_id, &api_key).await {
        Ok(prices) => {
            // Cache the result
            let _ = state.cache.put(&cache_key, Collection::CommodityPrices, &prices);
            Ok(ApiResponse::ok(prices))
        }
        Err(e) => Ok(ApiResponse::err(e)),
    }
}
