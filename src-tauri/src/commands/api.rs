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
use specta::Type;
use tauri::State;

use crate::activity::LastUserAction;
use crate::cache_store::{CacheResult, Collection};
use crate::providers::search_in_collection;
use crate::providers::commodities::search_commodities;
use crate::providers::vehicles::search_vehicles;
use crate::providers::locations::{search_locations, TERMINAL_HIERARCHY_KEY};
use crate::state::AppState;
use crate::uex::types::{EntityInfo, LocationTerminal, PriceEntry, UexResult};

// ── Response envelope ──────────────────────────────────────────────────────

/// Uniform response wrapper returned by every API command.
#[derive(Debug, Serialize, Type)]
pub struct ApiResponse<T: Serialize> {
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    /// Indicates the data was served from an expired cache entry.
    /// Frontend can show a "refreshing..." indicator when this is true.
    pub stale: bool,
    /// Total number of matches before any limit was applied (None when not applicable).
    pub total: Option<u32>,
}

impl<T: Serialize> ApiResponse<T> {
    pub(crate) fn ok(data: T) -> Self {
        Self { ok: true, data: Some(data), error: None, stale: false, total: None }
    }

    pub(crate) fn ok_stale(data: T) -> Self {
        Self { ok: true, data: Some(data), error: None, stale: true, total: None }
    }

    pub(crate) fn err(msg: impl Into<String>) -> Self {
        Self { ok: false, data: None, error: Some(msg.into()), stale: false, total: None }
    }
}

// ── Helper ─────────────────────────────────────────────────────────────────

/// Pull the UEX API key from the fetched backend key.
fn api_key(state: &AppState) -> String {
    state.fetched_api_key.lock().unwrap().clone()
}

/// Search a single collection from cache. Returns (results, is_stale).
/// Falls back to a direct API call if the cache is empty.
/// Note: Items are NOT in cache — use `search_items_wiki` for item search.
async fn search_cached_or_fetch(
    state: &AppState,
    collection: Collection,
    query: &str,
    api_key: &str,
) -> Result<(Vec<UexResult>, bool), String> {
    let key = collection.storage_key();
    match state.cache.get::<Vec<UexResult>>(&key) {
        CacheResult::Fresh(data) => {
            Ok((search_in_collection(&data, query), false))
        }
        CacheResult::Stale(data) => {
            Ok((search_in_collection(&data, query), true))
        }
        CacheResult::Missing => {
            // Fallback: direct API call for this specific query
            let results = match collection {
                Collection::Commodities => search_commodities(&state.uex, query, api_key).await?,
                Collection::Vehicles => search_vehicles(&state.uex, query, api_key).await?,
                Collection::Locations => search_locations(&state.uex, query, api_key).await?,
                _ => return Err("Use specific price commands".to_string()),
            };
            Ok((results, false))
        }
    }
}

// ── Search endpoints ───────────────────────────────────────────────────────

/// Search all entity types (commodities, vehicles, locations) from local cache.
/// Items are NOT included here — the frontend supplements with Wiki API search
/// via the separate `wiki_search` command (with deduplication).
///
/// Parameters:
/// - `query`: free-text search string
#[tauri::command]
#[specta::specta]
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

    // Local cache searches (instant)
    let collections = [
        Collection::Commodities,
        Collection::Vehicles,
        Collection::Locations,
    ];

    for collection in &collections {
        match search_cached_or_fetch(&state, *collection, &query, &api_key).await {
            Ok((mut results, stale)) => {
                all_results.append(&mut results);
                if stale { any_stale = true; }
            }
            Err(e) => {
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
    resp.total = Some(total as u32);
    Ok(resp)
}

/// Search UEX commodities only.
#[tauri::command]
#[specta::specta]
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
#[specta::specta]
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

/// Search items via Wiki API (on-demand search, not cached catalog).
#[tauri::command]
#[specta::specta]
pub async fn api_search_items(
    query: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<UexResult>>, String> {
    if query.trim().is_empty() {
        return Ok(ApiResponse::ok(vec![]));
    }

    let hide_untranslated = state.current_settings.lock().unwrap().hide_untranslated_items;
    let http = state.uex.client();
    match crate::providers::items::provider::search_items_wiki(http, &query, 100).await {
        Ok(mut results) => {
            if hide_untranslated {
                results.retain(|r| !r.name.starts_with('@'));
            }
            Ok(ApiResponse::ok(results))
        }
        Err(e) => Ok(ApiResponse::err(e)),
    }
}

/// Search UEX locations (terminals / stations).
#[tauri::command]
#[specta::specta]
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

/// Lookup prices from cache only. Records the lookup in the activity log.
fn price_lookup_cached(
    entity_id: &str,
    kind: &str,
    collection: Collection,
    state: &AppState,
) -> ApiResponse<Vec<PriceEntry>> {
    let cache_key = collection.storage_key_with_id(entity_id);
    let (response, source, row_count) = match state.cache.get::<Vec<PriceEntry>>(&cache_key) {
        CacheResult::Fresh(prices) => {
            let n = prices.len() as u32;
            (ApiResponse::ok(prices), "fresh", n)
        }
        CacheResult::Stale(prices) => {
            let n = prices.len() as u32;
            (ApiResponse::ok_stale(prices), "stale", n)
        }
        CacheResult::Missing => (ApiResponse::ok(vec![]), "missing", 0),
    };

    if let Ok(mut log) = state.activity.lock() {
        log.last_user_action = Some(LastUserAction {
            timestamp: chrono::Utc::now().to_rfc3339(),
            kind: kind.to_string(),
            entity_id: entity_id.to_string(),
            collection: collection.storage_key(),
            source: source.to_string(),
            row_count,
        });
    }

    response
}

/// Fetch buy/sell prices for a commodity by its UEX ID.
/// Serves from per-commodity cache key (e.g. `commodity_prices:42`).
#[tauri::command]
#[specta::specta]
pub async fn api_commodity_prices(
    commodity_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<PriceEntry>>, String> {
    if commodity_id.trim().is_empty() {
        return Ok(ApiResponse::err("commodity_id must not be empty"));
    }
    Ok(price_lookup_cached(&commodity_id, "commodity", Collection::CommodityPrices, &state))
}

/// Fetch raw commodity prices by commodity ID.
#[tauri::command]
#[specta::specta]
pub async fn api_raw_commodity_prices(
    commodity_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<PriceEntry>>, String> {
    if commodity_id.trim().is_empty() {
        return Ok(ApiResponse::err("commodity_id must not be empty"));
    }
    Ok(price_lookup_cached(&commodity_id, "raw_commodity", Collection::RawCommodityPrices, &state))
}

/// Fetch item prices by item ID.
#[tauri::command]
#[specta::specta]
pub async fn api_item_prices(
    item_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<PriceEntry>>, String> {
    if item_id.trim().is_empty() {
        return Ok(ApiResponse::err("item_id must not be empty"));
    }
    Ok(price_lookup_cached(&item_id, "item", Collection::ItemPrices, &state))
}

/// Fetch vehicle purchase prices by vehicle ID.
#[tauri::command]
#[specta::specta]
pub async fn api_vehicle_purchase_prices(
    vehicle_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<PriceEntry>>, String> {
    if vehicle_id.trim().is_empty() {
        return Ok(ApiResponse::err("vehicle_id must not be empty"));
    }
    Ok(price_lookup_cached(&vehicle_id, "vehicle", Collection::VehiclePurchasePrices, &state))
}

/// Fetch vehicle rental prices by vehicle ID.
#[tauri::command]
#[specta::specta]
pub async fn api_vehicle_rental_prices(
    vehicle_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<PriceEntry>>, String> {
    if vehicle_id.trim().is_empty() {
        return Ok(ApiResponse::err("vehicle_id must not be empty"));
    }
    Ok(price_lookup_cached(&vehicle_id, "vehicle_rental", Collection::VehicleRentalPrices, &state))
}

/// Fetch fuel prices by terminal ID.
#[tauri::command]
#[specta::specta]
pub async fn api_fuel_prices(
    terminal_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<PriceEntry>>, String> {
    if terminal_id.trim().is_empty() {
        return Ok(ApiResponse::err("terminal_id must not be empty"));
    }
    // Fuel prices are stored by terminal under "fuel_prices_by_terminal:{tid}"
    let cache_key = format!("fuel_prices_by_terminal:{}", terminal_id);
    let (response, source, row_count) = match state.cache.get::<Vec<PriceEntry>>(&cache_key) {
        CacheResult::Fresh(prices) => {
            let n = prices.len() as u32;
            (ApiResponse::ok(prices), "fresh", n)
        }
        CacheResult::Stale(prices) => {
            let n = prices.len() as u32;
            (ApiResponse::ok_stale(prices), "stale", n)
        }
        CacheResult::Missing => (ApiResponse::ok(vec![]), "missing", 0),
    };

    if let Ok(mut log) = state.activity.lock() {
        log.last_user_action = Some(LastUserAction {
            timestamp: chrono::Utc::now().to_rfc3339(),
            kind: "fuel".to_string(),
            entity_id: terminal_id,
            collection: "fuel_prices".to_string(),
            source: source.to_string(),
            row_count,
        });
    }

    Ok(response)
}

// ── Entity info endpoint ──────────────────────────────────────────────────

/// Fetch detailed entity metadata by kind and id.
/// - Items: fetched on-demand from Wiki API (not bulk-cached).
/// - Vehicles: from bulk entity_info cache (Wiki vehicle data).
/// - Commodities/locations: from bulk entity_info cache (UEX data).
#[tauri::command]
#[specta::specta]
pub async fn api_entity_info(
    kind: String,
    entity_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<EntityInfo>, String> {
    if entity_id.trim().is_empty() {
        return Ok(ApiResponse::err("entity_id must not be empty"));
    }

    let cache_key = Collection::EntityInfo.storage_key_with_id(&format!("{}:{}", kind, entity_id));

    // Check cache first (all kinds)
    match state.cache.get::<EntityInfo>(&cache_key) {
        CacheResult::Fresh(info) if info.id == entity_id => return Ok(ApiResponse::ok(info)),
        CacheResult::Stale(info) if info.id == entity_id => return Ok(ApiResponse::ok_stale(info)),
        _ => {}
    }

    // On-demand fetch for items from Wiki API
    if kind == "item" {
        let http = state.uex.client();
        match crate::wiki::client::get_item(http, &entity_id).await {
            Ok(dto) => {
                let info = wiki_item_to_entity_info(&dto);
                let ttl = Collection::EntityInfo.ttl_secs();
                let _ = state.cache.put(&cache_key, ttl, &info);
                return Ok(ApiResponse::ok(info));
            }
            Err(e) => {
                log::warn!("Wiki item fetch failed for {}: {}", entity_id, e);
            }
        }
    }

    // On-demand fetch for vehicles from Wiki API
    if kind == "vehicle" || kind == "ground vehicle" {
        let http = state.uex.client();
        match crate::wiki::client::get_vehicle(http, &entity_id).await {
            Ok(dto) => {
                let info = wiki_vehicle_to_entity_info(&dto);
                let ttl = Collection::EntityInfo.ttl_secs();
                let _ = state.cache.put(&cache_key, ttl, &info);
                return Ok(ApiResponse::ok(info));
            }
            Err(e) => {
                log::warn!("Wiki vehicle fetch failed for {}: {}", entity_id, e);
            }
        }
    }

    Ok(ApiResponse::err(format!(
        "Entity info for {} {} not in cache. Wait for cache to finish loading.",
        kind, entity_id
    )))
}

/// Convert Wiki item DTO to EntityInfo for cache storage.
fn wiki_item_to_entity_info(dto: &crate::wiki::dto::WikiItemDto) -> EntityInfo {
    let manufacturer = dto.manufacturer.as_ref();
    EntityInfo {
        id: dto.uuid.clone().unwrap_or_default(),
        name: dto.name.clone().unwrap_or_default(),
        kind: "item".to_string(),
        slug: String::new(),
        uuid: dto.uuid.clone(),
        company_name: manufacturer.and_then(|m| m.name.clone()),
        game_version: dto.version.clone(),
        section: dto.item_type.clone(),
        category: dto.sub_type.clone(),
        size: dto.size.map(|s| s.to_string()),
        mass: dto.mass,
        ..Default::default()
    }
}

/// Convert Wiki vehicle DTO to EntityInfo for cache storage.
fn wiki_vehicle_to_entity_info(dto: &crate::wiki::dto::WikiVehicleDto) -> EntityInfo {
    let kind = match dto.vehicle_type.as_deref() {
        Some("vehicle") | Some("gravlev") => "ground vehicle",
        _ => "vehicle",
    };
    let dim = dto.dimension.as_ref();
    EntityInfo {
        id: dto.uuid.clone().unwrap_or_default(),
        name: dto.name.clone().unwrap_or_default(),
        kind: kind.to_string(),
        slug: dto.slug.clone().unwrap_or_default(),
        uuid: dto.uuid.clone(),
        name_full: dto.name.clone(),
        company_name: dto.manufacturer.as_ref().and_then(|m| m.name.clone()),
        scu: dto.cargo_capacity,
        crew: dto.crew.as_ref().and_then(|c| c.max).map(|v| v.to_string()).filter(|s| s != "0"),
        length: dim.and_then(|d| d.length),
        width: dim.and_then(|d| d.width),
        height: dim.and_then(|d| d.height),
        mass: dto.mass,
        game_version: dto.version.clone(),
        ..Default::default()
    }
}

// ── Location terminals endpoint ───────────────────────────────────────────

/// Strip the type prefix from a location ID (e.g. "sys_1" → "1", "planet_3" → "3").
/// Terminal IDs have no prefix and are returned as-is.
fn strip_location_id_prefix(slug: &str, id: &str) -> String {
    let prefix = match slug {
        "star_system" => "sys_",
        "planet" => "planet_",
        "moon" => "moon_",
        "orbit" => "orbit_",
        "space_station" => "station_",
        "outpost" => "outpost_",
        "city" => "city_",
        "poi" => "poi_",
        "faction" => "faction_",
        "company" => "company_",
        _ => "",
    };
    if prefix.is_empty() {
        id.to_string()
    } else {
        id.strip_prefix(prefix).unwrap_or(id).to_string()
    }
}

/// Fetch all terminals belonging to a location (by slug and prefixed ID).
/// Uses cached terminal hierarchy data to filter by the appropriate parent ID.
#[tauri::command]
#[specta::specta]
pub async fn api_location_terminals(
    slug: String,
    id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<LocationTerminal>>, String> {
    use crate::providers::locations::dto::TerminalHierarchy;

    if id.trim().is_empty() {
        return Ok(ApiResponse::err("id must not be empty"));
    }

    let raw_id = strip_location_id_prefix(&slug, &id);

    let hierarchy: Vec<TerminalHierarchy> = match state.cache.get::<Vec<TerminalHierarchy>>(TERMINAL_HIERARCHY_KEY) {
        CacheResult::Fresh(h) => h,
        CacheResult::Stale(h) => h,
        CacheResult::Missing => {
            return Ok(ApiResponse::err("Terminal data not loaded yet. Wait for cache to finish loading."));
        }
    };

    let terminals: Vec<LocationTerminal> = hierarchy
        .iter()
        .filter(|t| t.matches_location(&slug, &raw_id))
        .map(|t| t.to_location_terminal())
        .collect();

    let count = terminals.len() as u32;
    let mut resp = ApiResponse::ok(terminals);
    resp.total = Some(count);
    Ok(resp)
}

// ── Terminal prices endpoint (all price types at a terminal) ──────────────

/// Fetch all prices available at a specific terminal, aggregating across
/// commodities, raw commodities, items, vehicles, and fuel.
#[tauri::command]
#[specta::specta]
pub async fn api_terminal_prices(
    terminal_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<PriceEntry>>, String> {
    if terminal_id.trim().is_empty() {
        return Ok(ApiResponse::err("terminal_id must not be empty"));
    }

    let collections = [
        Collection::CommodityPrices,
        Collection::RawCommodityPrices,
        Collection::ItemPrices,
        Collection::VehiclePurchasePrices,
        Collection::VehicleRentalPrices,
        Collection::FuelPrices,
    ];

    let mut all_prices: Vec<PriceEntry> = Vec::new();
    let mut any_stale = false;

    for collection in &collections {
        let key = format!("{}_by_terminal:{}", collection.storage_key(), terminal_id);
        match state.cache.get::<Vec<PriceEntry>>(&key) {
            CacheResult::Fresh(entries) => all_prices.extend(entries),
            CacheResult::Stale(entries) => {
                any_stale = true;
                all_prices.extend(entries);
            }
            CacheResult::Missing => {}
        }
    }

    let count = all_prices.len() as u32;
    let mut resp = if any_stale {
        ApiResponse::ok_stale(all_prices)
    } else {
        ApiResponse::ok(all_prices)
    };
    resp.total = Some(count);
    Ok(resp)
}
