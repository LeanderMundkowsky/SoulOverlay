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
use crate::uex_client::{self, EntityInfo, PriceEntry, UexResult};

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
                Collection::CommodityPrices
                | Collection::RawCommodityPrices
                | Collection::ItemPrices
                | Collection::VehiclePurchasePrices
                | Collection::VehicleRentalPrices
                | Collection::FuelPrices => return Err("Use specific price commands".to_string()),
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
    match uex_client::get_commodity_prices(&commodity_id, &api_key).await {
        Ok(prices) => {
            // Cache the result
            let ttl = state.current_settings.lock().unwrap().cache_ttl_prices_secs as i64;
            let _ = state.cache.put(&cache_key, ttl, &prices);
            Ok(ApiResponse::ok(prices))
        }
        Err(e) => Ok(ApiResponse::err(e)),
    }
}

// ── Generic price lookup helper ───────────────────────────────────────────

/// Lookup prices from cache, or live-fetch if stale/missing.
async fn price_lookup(
    entity_id: &str,
    collection: Collection,
    fetch_fn: impl std::future::Future<Output = Result<Vec<PriceEntry>, String>>,
    state: &AppState,
) -> Result<ApiResponse<Vec<PriceEntry>>, String> {
    let cache_key = collection.storage_key_with_id(entity_id);

    match state.cache.get::<Vec<PriceEntry>>(&cache_key) {
        CacheResult::Fresh(prices) => return Ok(ApiResponse::ok(prices)),
        CacheResult::Stale(prices) => return Ok(ApiResponse::ok_stale(prices)),
        CacheResult::Missing => {}
    }

    match fetch_fn.await {
        Ok(prices) => {
            let ttl = state.current_settings.lock().unwrap().cache_ttl_prices_secs as i64;
            let _ = state.cache.put(&cache_key, ttl, &prices);
            Ok(ApiResponse::ok(prices))
        }
        Err(e) => Ok(ApiResponse::err(e)),
    }
}

/// Fetch raw commodity prices by commodity ID.
#[tauri::command]
pub async fn api_raw_commodity_prices(
    commodity_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<PriceEntry>>, String> {
    if commodity_id.trim().is_empty() {
        return Ok(ApiResponse::err("commodity_id must not be empty"));
    }
    let key = api_key(&state);
    price_lookup(
        &commodity_id,
        Collection::RawCommodityPrices,
        uex_client::get_raw_commodity_prices(&commodity_id, &key),
        &state,
    ).await
}

/// Fetch item prices by item ID.
#[tauri::command]
pub async fn api_item_prices(
    item_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<PriceEntry>>, String> {
    if item_id.trim().is_empty() {
        return Ok(ApiResponse::err("item_id must not be empty"));
    }
    let key = api_key(&state);
    price_lookup(
        &item_id,
        Collection::ItemPrices,
        uex_client::get_item_prices(&item_id, &key),
        &state,
    ).await
}

/// Fetch vehicle purchase prices by vehicle ID.
#[tauri::command]
pub async fn api_vehicle_purchase_prices(
    vehicle_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<PriceEntry>>, String> {
    if vehicle_id.trim().is_empty() {
        return Ok(ApiResponse::err("vehicle_id must not be empty"));
    }
    let key = api_key(&state);
    price_lookup(
        &vehicle_id,
        Collection::VehiclePurchasePrices,
        uex_client::get_vehicle_purchase_prices(&vehicle_id, &key),
        &state,
    ).await
}

/// Fetch vehicle rental prices by vehicle ID.
#[tauri::command]
pub async fn api_vehicle_rental_prices(
    vehicle_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<PriceEntry>>, String> {
    if vehicle_id.trim().is_empty() {
        return Ok(ApiResponse::err("vehicle_id must not be empty"));
    }
    let key = api_key(&state);
    price_lookup(
        &vehicle_id,
        Collection::VehicleRentalPrices,
        uex_client::get_vehicle_rental_prices(&vehicle_id, &key),
        &state,
    ).await
}

/// Fetch fuel prices by terminal ID.
#[tauri::command]
pub async fn api_fuel_prices(
    terminal_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<PriceEntry>>, String> {
    if terminal_id.trim().is_empty() {
        return Ok(ApiResponse::err("terminal_id must not be empty"));
    }
    let key = api_key(&state);
    price_lookup(
        &terminal_id,
        Collection::FuelPrices,
        uex_client::get_fuel_prices(&terminal_id, &key),
        &state,
    ).await
}

// ── Entity info endpoint ──────────────────────────────────────────────────

const ENTITY_INFO_TTL_SECS: i64 = 86400; // 24 hours

/// Fetch detailed entity metadata by kind and id.
/// Caches results for 24 hours under `entity_info:{kind}:{id}`.
#[tauri::command]
pub async fn api_entity_info(
    kind: String,
    entity_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<EntityInfo>, String> {
    if entity_id.trim().is_empty() {
        return Ok(ApiResponse::err("entity_id must not be empty"));
    }

    let cache_key = format!("entity_info:{}:{}", kind, entity_id);

    // Check cache
    match state.cache.get::<EntityInfo>(&cache_key) {
        CacheResult::Fresh(info) => return Ok(ApiResponse::ok(info)),
        CacheResult::Stale(info) => return Ok(ApiResponse::ok_stale(info)),
        CacheResult::Missing => {}
    }

    let key = api_key(&state);

    let result = match kind.as_str() {
        "commodity" => uex_client::get_commodity_info(&entity_id, &key).await,
        "vehicle" | "ground vehicle" => uex_client::get_vehicle_info(&entity_id, &key).await,
        "item" => {
            // Items require uuid; look it up from the cached items collection
            let items_key = Collection::Items.storage_key();
            let uuid = match state.cache.get::<Vec<UexResult>>(&items_key) {
                CacheResult::Fresh(items) | CacheResult::Stale(items) => {
                    items.iter()
                        .find(|i| i.id == entity_id)
                        .map(|i| i.uuid.clone())
                        .unwrap_or_default()
                }
                CacheResult::Missing => String::new(),
            };
            if uuid.is_empty() {
                Err(format!("Item {} uuid not available in cache", entity_id))
            } else {
                uex_client::get_item_info(&uuid, &key).await
            }
        }
        _ => Err(format!("Entity info not supported for kind: {}", kind)),
    };

    match result {
        Ok(info) => {
            let _ = state.cache.put(&cache_key, ENTITY_INFO_TTL_SECS, &info);
            Ok(ApiResponse::ok(info))
        }
        Err(e) => Ok(ApiResponse::err(e)),
    }
}
