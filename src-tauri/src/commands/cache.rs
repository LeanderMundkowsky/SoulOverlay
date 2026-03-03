use log::{error, info};
use serde::Serialize;
use specta::Type;
use std::time::Instant;
use tauri::State;

use crate::activity::FetchEvent;
use crate::cache_store::{CacheResult, CacheStore, Collection, CollectionStatus};
use crate::settings::Settings;
use crate::state::AppState;
use crate::uex;
use crate::uex::PriceEntry;

/// Response from cache refresh operations.
#[derive(Debug, Serialize, Type)]
pub struct CacheRefreshResult {
    pub ok: bool,
    pub collection: String,
    pub error: Option<String>,
}

/// Get status for all cached collections.
#[tauri::command]
#[specta::specta]
pub async fn cache_status(
    state: State<'_, AppState>,
) -> Result<Vec<CollectionStatus>, String> {
    Ok(state.cache.status())
}

/// Refresh a single collection by name.
#[tauri::command]
#[specta::specta]
pub async fn cache_refresh(
    collection: String,
    state: State<'_, AppState>,
) -> Result<CacheRefreshResult, String> {
    let settings = state.current_settings.lock().unwrap().clone();
    let result = refresh_collection_by_name(&collection, &settings.uex_api_key, &settings, &state, "manual").await;
    Ok(result)
}

/// Refresh all prefetchable collections, but only those whose TTL has expired.
/// This is the same logic used on startup.
#[tauri::command]
#[specta::specta]
pub async fn cache_refresh_expired(
    state: State<'_, AppState>,
) -> Result<Vec<CacheRefreshResult>, String> {
    let settings = state.current_settings.lock().unwrap().clone();
    let mut results = Vec::new();
    for collection in Collection::all() {
        let key = collection.storage_key();
        if state.cache.is_expired(&key) {
            let result = refresh_collection_by_name(&key, &settings.uex_api_key, &settings, &state, "manual").await;
            results.push(result);
        }
    }
    Ok(results)
}

/// Refresh all prefetchable collections.
#[tauri::command]
#[specta::specta]
pub async fn cache_refresh_all(
    state: State<'_, AppState>,
) -> Result<Vec<CacheRefreshResult>, String> {
    let settings = state.current_settings.lock().unwrap().clone();
    let mut results = Vec::new();
    for collection in Collection::all() {
        let name = collection.storage_key();
        let result = refresh_collection_by_name(&name, &settings.uex_api_key, &settings, &state, "manual").await;
        results.push(result);
    }
    Ok(results)
}

// ── Split helper ───────────────────────────────────────────────────────────

/// Split a flat vec of price entries by entity_id and store each group
/// as a separate per-entity cache entry (e.g. `commodity_prices:42`).
/// Invalidates old sub-entries first so stale entity IDs don't linger.
fn store_prices_split(
    cache: &CacheStore,
    entries: &[PriceEntry],
    collection: Collection,
    ttl: i64,
) -> Result<(), String> {
    // Remove old sub-entries so entity IDs no longer in the API response don't
    // cause the whole collection to appear stale.
    cache.invalidate_collection(collection);

    let mut groups: std::collections::HashMap<&str, Vec<PriceEntry>> = std::collections::HashMap::new();
    for entry in entries {
        groups.entry(&entry.entity_id).or_default().push(entry.clone());
    }

    let base_key = collection.storage_key();
    let mut errors = Vec::new();
    for (entity_id, group) in &groups {
        let key = format!("{}:{}", base_key, entity_id);
        if let Err(e) = cache.put(&key, ttl, group) {
            errors.push(format!("{}: {}", key, e));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

/// Read entity IDs from a catalog collection in cache (Fresh or Stale).
/// Returns an empty vec if the catalog is not cached yet.
fn catalog_ids_from_cache(cache: &CacheStore, collection: Collection) -> Vec<String> {
    let key = collection.storage_key();
    match cache.get::<Vec<uex::UexResult>>(&key) {
        CacheResult::Fresh(items) | CacheResult::Stale(items) => {
            items.into_iter().map(|i| i.id).collect()
        }
        CacheResult::Missing => vec![],
    }
}

/// Internal helper: refresh a collection by its storage key name.
pub(crate) async fn refresh_collection_by_name(
    name: &str,
    api_key: &str,
    settings: &Settings,
    state: &AppState,
    triggered_by: &str,
) -> CacheRefreshResult {
    let client = &state.uex;
    let start = Instant::now();

    let result = match name {
        "commodities" => {
            let ttl = Collection::Commodities.ttl_for(settings);
            match uex::fetch_all_commodities(client, api_key).await {
                Ok(data) => {
                    info!("Refreshed commodities: {} entries", data.len());
                    state.cache.put(&Collection::Commodities.storage_key(), ttl, &data)
                }
                Err(e) => Err(e),
            }
        }
        "vehicles" => {
            let ttl = Collection::Vehicles.ttl_for(settings);
            match uex::fetch_all_vehicles(client, api_key).await {
                Ok(data) => {
                    info!("Refreshed vehicles: {} entries", data.len());
                    state.cache.put(&Collection::Vehicles.storage_key(), ttl, &data)
                }
                Err(e) => Err(e),
            }
        }
        "items" => {
            let ttl = Collection::Items.ttl_for(settings);
            match uex::fetch_all_items(client, api_key).await {
                Ok(data) => {
                    info!("Refreshed items: {} entries", data.len());
                    state.cache.put(&Collection::Items.storage_key(), ttl, &data)
                }
                Err(e) => Err(e),
            }
        }
        "locations" => {
            let ttl = Collection::Locations.ttl_for(settings);
            match uex::fetch_all_locations(client, api_key).await {
                Ok(data) => {
                    info!("Refreshed locations: {} entries", data.len());
                    state.cache.put(&Collection::Locations.storage_key(), ttl, &data)
                }
                Err(e) => Err(e),
            }
        }
        "commodity_prices" => {
            let ttl = Collection::CommodityPrices.ttl_for(settings);
            let ids = catalog_ids_from_cache(&state.cache, Collection::Commodities);
            if ids.is_empty() {
                return CacheRefreshResult {
                    ok: false,
                    collection: name.to_string(),
                    error: Some("Commodities not in cache; refresh commodities first".to_string()),
                };
            }
            let data = uex::fetch_all_commodity_prices_per_entity(client, &ids, api_key).await;
            info!("Refreshed commodity prices: {} rows across {} commodities", data.len(), ids.len());
            store_prices_split(&state.cache, &data, Collection::CommodityPrices, ttl)
        }
        "raw_commodity_prices" => {
            let ttl = Collection::RawCommodityPrices.ttl_for(settings);
            match uex::fetch_all_raw_commodity_prices(client, api_key).await {
                Ok(data) => {
                    info!("Refreshed raw commodity prices: {} rows", data.len());
                    store_prices_split(&state.cache, &data, Collection::RawCommodityPrices, ttl)
                }
                Err(e) => Err(e),
            }
        }
        "item_prices" => {
            let ttl = Collection::ItemPrices.ttl_for(settings);
            match uex::fetch_all_item_prices(client, api_key).await {
                Ok(data) => {
                    info!("Refreshed item prices: {} rows", data.len());
                    store_prices_split(&state.cache, &data, Collection::ItemPrices, ttl)
                }
                Err(e) => Err(e),
            }
        }
        "vehicle_purchase_prices" => {
            let ttl = Collection::VehiclePurchasePrices.ttl_for(settings);
            let ids = catalog_ids_from_cache(&state.cache, Collection::Vehicles);
            if ids.is_empty() {
                return CacheRefreshResult {
                    ok: false,
                    collection: name.to_string(),
                    error: Some("Vehicles not in cache; refresh vehicles first".to_string()),
                };
            }
            let data = uex::fetch_all_vehicle_purchase_prices_per_entity(client, &ids, api_key).await;
            info!("Refreshed vehicle purchase prices: {} rows across {} vehicles", data.len(), ids.len());
            store_prices_split(&state.cache, &data, Collection::VehiclePurchasePrices, ttl)
        }
        "vehicle_rental_prices" => {
            let ttl = Collection::VehicleRentalPrices.ttl_for(settings);
            let ids = catalog_ids_from_cache(&state.cache, Collection::Vehicles);
            if ids.is_empty() {
                return CacheRefreshResult {
                    ok: false,
                    collection: name.to_string(),
                    error: Some("Vehicles not in cache; refresh vehicles first".to_string()),
                };
            }
            let data = uex::fetch_all_vehicle_rental_prices_per_entity(client, &ids, api_key).await;
            info!("Refreshed vehicle rental prices: {} rows across {} vehicles", data.len(), ids.len());
            store_prices_split(&state.cache, &data, Collection::VehicleRentalPrices, ttl)
        }
        "fuel_prices" => {
            let ttl = Collection::FuelPrices.ttl_for(settings);
            match uex::fetch_all_fuel_prices(client, api_key).await {
                Ok(data) => {
                    info!("Refreshed fuel prices: {} rows", data.len());
                    store_prices_split(&state.cache, &data, Collection::FuelPrices, ttl)
                }
                Err(e) => Err(e),
            }
        }
        "fleet" => {
            if settings.uex_secret_key.is_empty() {
                return CacheRefreshResult {
                    ok: false,
                    collection: name.to_string(),
                    error: Some("UEX secret key not configured; skipping fleet refresh".to_string()),
                };
            }
            let ttl = Collection::Fleet.ttl_for(settings);
            match uex::fetch_fleet(client, api_key, &settings.uex_secret_key).await {
                Ok(data) => {
                    info!("Refreshed fleet: {} entries", data.len());
                    state.cache.put(&Collection::Fleet.storage_key(), ttl, &data)
                }
                Err(e) => Err(e),
            }
        }
        _ => Err(format!("Unknown collection: {}", name)),
    };

    let duration_ms = start.elapsed().as_millis() as u32;
    let (ok, err_msg) = match &result {
        Ok(()) => (true, None),
        Err(e) => (false, Some(e.clone())),
    };

    let endpoint = match name {
        "commodity_prices" => "/commodities_prices (per-entity)".to_string(),
        "raw_commodity_prices" => "/commodities_raw_prices_all".to_string(),
        "item_prices" => "/items_prices_all".to_string(),
        "vehicle_purchase_prices" => "/vehicles_purchases_prices (per-entity)".to_string(),
        "vehicle_rental_prices" => "/vehicles_rentals_prices (per-entity)".to_string(),
        "fuel_prices" => "/fuel_prices_all".to_string(),
        "commodities" => "/commodities".to_string(),
        "vehicles" => "/vehicles".to_string(),
        "items" => "/items (per-category)".to_string(),
        "locations" => "/terminals".to_string(),
        _ => format!("/{}", name),
    };

    let event = FetchEvent {
        timestamp: chrono::Utc::now().to_rfc3339(),
        collection: name.to_string(),
        endpoint,
        row_count: 0, // not tracked here; per-collection count is logged above
        duration_ms,
        triggered_by: triggered_by.to_string(),
        ok,
        error: err_msg.clone(),
    };
    if let Ok(mut log) = state.activity.lock() {
        log.push_fetch(event);
    }

    match result {
        Ok(()) => CacheRefreshResult {
            ok: true,
            collection: name.to_string(),
            error: None,
        },
        Err(e) => {
            error!("Failed to refresh collection '{}': {}", name, e);
            CacheRefreshResult {
                ok: false,
                collection: name.to_string(),
                error: Some(e),
            }
        }
    }
}

/// Public helper used by app_setup to prefetch all collections on startup.
/// Catalog collections run first (prices depend on their IDs), then the rest.
pub async fn prefetch_all(state: &AppState) {
    let settings = state.current_settings.lock().unwrap().clone();

    // Catalog collections first (prices need their IDs for per-entity fetching).
    let catalogs = [
        Collection::Commodities,
        Collection::Vehicles,
        Collection::Items,
        Collection::Locations,
    ];
    for collection in &catalogs {
        let key = collection.storage_key();
        if state.cache.is_expired(&key) {
            info!("Prefetching expired collection: {}", key);
            let result = refresh_collection_by_name(&key, &settings.uex_api_key, &settings, state, "startup").await;
            if !result.ok {
                if let Some(e) = &result.error {
                    error!("Prefetch failed for {}: {}", key, e);
                }
            }
        } else {
            info!("Collection '{}' is still fresh, skipping prefetch", key);
        }
    }

    // Remaining collections (prices, fleet, etc.)
    for collection in Collection::all() {
        if catalogs.contains(collection) {
            continue;
        }
        let key = collection.storage_key();
        if state.cache.is_expired(&key) {
            info!("Prefetching expired collection: {}", key);
            let result = refresh_collection_by_name(&key, &settings.uex_api_key, &settings, state, "startup").await;
            if !result.ok {
                if let Some(e) = &result.error {
                    error!("Prefetch failed for {}: {}", key, e);
                }
            }
        } else {
            info!("Collection '{}' is still fresh, skipping prefetch", key);
        }
    }
}

/// Try to acquire the refresh guard for `collection_key`. Returns `true` if the
/// guard was acquired (caller should refresh), `false` if already in-flight.
pub(crate) fn try_acquire_refresh(state: &AppState, collection_key: &str) -> bool {
    let mut guard = state.refreshing_collections.lock().unwrap();
    guard.insert(collection_key.to_string())
}

/// Release the refresh guard for `collection_key`.
pub(crate) fn release_refresh(state: &AppState, collection_key: &str) {
    let mut guard = state.refreshing_collections.lock().unwrap();
    guard.remove(collection_key);
}

/// Refresh a single collection if expired, respecting the in-flight guard.
/// Returns `true` if a refresh was performed.
pub(crate) async fn guarded_refresh(state: &AppState, collection: &Collection) -> bool {
    let key = collection.storage_key();
    if !state.cache.is_expired(&key) {
        return false;
    }
    if !try_acquire_refresh(state, &key) {
        return false;
    }
    let settings = state.current_settings.lock().unwrap().clone();
    let result = refresh_collection_by_name(&key, &settings.uex_api_key, &settings, state, "timer").await;
    release_refresh(state, &key);
    if !result.ok {
        if let Some(e) = &result.error {
            error!("Background refresh failed for '{}': {}", key, e);
        }
    }
    result.ok
}
