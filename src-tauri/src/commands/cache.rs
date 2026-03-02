use log::{error, info};
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;
use tauri::State;

use crate::activity::FetchEvent;
use crate::cache_store::{CacheResult, CacheStore, Collection, CollectionStatus};
use crate::settings::Settings;
use crate::state::AppState;
use crate::uex;
use crate::uex::PriceEntry;

/// Response from cache refresh operations.
#[derive(Debug, Serialize)]
pub struct CacheRefreshResult {
    pub ok: bool,
    pub collection: String,
    pub error: Option<String>,
}

/// Get status for all cached collections.
#[tauri::command]
pub async fn cache_status(
    state: State<'_, AppState>,
) -> Result<Vec<CollectionStatus>, String> {
    Ok(state.cache.status())
}

/// Refresh a single collection by name.
#[tauri::command]
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
pub async fn cache_refresh_expired(
    state: State<'_, AppState>,
) -> Result<Vec<CacheRefreshResult>, String> {
    let settings = state.current_settings.lock().unwrap().clone();
    let mut results = Vec::new();
    for collection in Collection::prefetch_list() {
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
pub async fn cache_refresh_all(
    state: State<'_, AppState>,
) -> Result<Vec<CacheRefreshResult>, String> {
    let settings = state.current_settings.lock().unwrap().clone();
    let mut results = Vec::new();
    for collection in Collection::prefetch_list() {
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

/// Arc variant for use in spawned tasks.
fn store_prices_split_arc(
    cache: &Arc<CacheStore>,
    entries: &[PriceEntry],
    collection: Collection,
    ttl: i64,
) -> Result<(), String> {
    store_prices_split(cache.as_ref(), entries, collection, ttl)
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
    let prices_ttl = settings.cache_ttl_prices_secs as i64;
    let catalog_ttl = settings.cache_ttl_catalog_secs as i64;
    let client = &state.uex;
    let start = Instant::now();

    let result = match name {
        "commodities" => {
            match uex::fetch_all_commodities(client, api_key).await {
                Ok(data) => {
                    info!("Refreshed commodities: {} entries", data.len());
                    state.cache.put(&Collection::Commodities.storage_key(), prices_ttl, &data)
                }
                Err(e) => Err(e),
            }
        }
        "vehicles" => {
            match uex::fetch_all_vehicles(client, api_key).await {
                Ok(data) => {
                    info!("Refreshed vehicles: {} entries", data.len());
                    state.cache.put(&Collection::Vehicles.storage_key(), catalog_ttl, &data)
                }
                Err(e) => Err(e),
            }
        }
        "items" => {
            match uex::fetch_all_items(client, api_key).await {
                Ok(data) => {
                    info!("Refreshed items: {} entries", data.len());
                    state.cache.put(&Collection::Items.storage_key(), catalog_ttl, &data)
                }
                Err(e) => Err(e),
            }
        }
        "locations" => {
            match uex::fetch_all_locations(client, api_key).await {
                Ok(data) => {
                    info!("Refreshed locations: {} entries", data.len());
                    state.cache.put(&Collection::Locations.storage_key(), catalog_ttl, &data)
                }
                Err(e) => Err(e),
            }
        }
        "commodity_prices" => {
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
            store_prices_split(&state.cache, &data, Collection::CommodityPrices, prices_ttl)
        }
        "raw_commodity_prices" => {
            match uex::fetch_all_raw_commodity_prices(client, api_key).await {
                Ok(data) => {
                    info!("Refreshed raw commodity prices: {} rows", data.len());
                    store_prices_split(&state.cache, &data, Collection::RawCommodityPrices, prices_ttl)
                }
                Err(e) => Err(e),
            }
        }
        "item_prices" => {
            match uex::fetch_all_item_prices(client, api_key).await {
                Ok(data) => {
                    info!("Refreshed item prices: {} rows", data.len());
                    store_prices_split(&state.cache, &data, Collection::ItemPrices, prices_ttl)
                }
                Err(e) => Err(e),
            }
        }
        "vehicle_purchase_prices" => {
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
            store_prices_split(&state.cache, &data, Collection::VehiclePurchasePrices, prices_ttl)
        }
        "vehicle_rental_prices" => {
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
            store_prices_split(&state.cache, &data, Collection::VehicleRentalPrices, prices_ttl)
        }
        "fuel_prices" => {
            match uex::fetch_all_fuel_prices(client, api_key).await {
                Ok(data) => {
                    info!("Refreshed fuel prices: {} rows", data.len());
                    store_prices_split(&state.cache, &data, Collection::FuelPrices, prices_ttl)
                }
                Err(e) => Err(e),
            }
        }
        _ => Err(format!("Unknown collection: {}", name)),
    };

    let duration_ms = start.elapsed().as_millis() as u64;
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
/// Catalog collections run sequentially, then price collections in parallel.
pub async fn prefetch_all(state: &AppState) {
    let settings = state.current_settings.lock().unwrap().clone();

    // Catalog collections first (sequential)
    let catalog = [
        Collection::Commodities,
        Collection::Vehicles,
        Collection::Items,
        Collection::Locations,
    ];
    for collection in &catalog {
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

    // Price collections in parallel
    // Read catalog IDs needed for per-entity price fetching (catalogs were just prefetched above).
    let commodity_ids: Vec<String> = catalog_ids_from_cache(&state.cache, Collection::Commodities);
    let vehicle_ids: Vec<String> = catalog_ids_from_cache(&state.cache, Collection::Vehicles);

    let prices_ttl = settings.cache_ttl_prices_secs as i64;
    let mut handles = Vec::new();

    for collection in Collection::price_collections() {
        let key = collection.storage_key();
        if !state.cache.is_expired(&key) {
            info!("Collection '{}' is still fresh, skipping prefetch", key);
            continue;
        }

        info!("Prefetching expired collection: {}", key);
        let api_key = settings.uex_api_key.clone();
        let cache = state.cache_arc();
        let activity = state.activity.clone();
        let client = state.uex.clone();
        let coll = *collection;
        let ttl = prices_ttl;
        let c_ids = commodity_ids.clone();
        let v_ids = vehicle_ids.clone();

        handles.push(tokio::spawn(async move {
            let key = coll.storage_key();
            let start = Instant::now();
            let (result, row_count, endpoint): (Result<(), String>, usize, &str) = match key.as_str() {
                "commodity_prices" => {
                    let data = uex::fetch_all_commodity_prices_per_entity(&client, &c_ids, &api_key).await;
                    let n = data.len();
                    info!("Prefetched commodity prices: {} rows across {} commodities", n, c_ids.len());
                    (store_prices_split_arc(&cache, &data, coll, ttl), n, "/commodities_prices (per-entity)")
                }
                "raw_commodity_prices" => {
                    match uex::fetch_all_raw_commodity_prices(&client, &api_key).await {
                        Ok(data) => { let n = data.len(); info!("Prefetched raw commodity prices: {} rows", n); (store_prices_split_arc(&cache, &data, coll, ttl), n, "/commodities_raw_prices_all") }
                        Err(e) => (Err(e), 0, "/commodities_raw_prices_all"),
                    }
                }
                "item_prices" => {
                    match uex::fetch_all_item_prices(&client, &api_key).await {
                        Ok(data) => { let n = data.len(); info!("Prefetched item prices: {} rows", n); (store_prices_split_arc(&cache, &data, coll, ttl), n, "/items_prices_all") }
                        Err(e) => (Err(e), 0, "/items_prices_all"),
                    }
                }
                "vehicle_purchase_prices" => {
                    let data = uex::fetch_all_vehicle_purchase_prices_per_entity(&client, &v_ids, &api_key).await;
                    let n = data.len();
                    info!("Prefetched vehicle purchase prices: {} rows across {} vehicles", n, v_ids.len());
                    (store_prices_split_arc(&cache, &data, coll, ttl), n, "/vehicles_purchases_prices (per-entity)")
                }
                "vehicle_rental_prices" => {
                    let data = uex::fetch_all_vehicle_rental_prices_per_entity(&client, &v_ids, &api_key).await;
                    let n = data.len();
                    info!("Prefetched vehicle rental prices: {} rows across {} vehicles", n, v_ids.len());
                    (store_prices_split_arc(&cache, &data, coll, ttl), n, "/vehicles_rentals_prices (per-entity)")
                }
                "fuel_prices" => {
                    match uex::fetch_all_fuel_prices(&client, &api_key).await {
                        Ok(data) => { let n = data.len(); info!("Prefetched fuel prices: {} rows", n); (store_prices_split_arc(&cache, &data, coll, ttl), n, "/fuel_prices_all") }
                        Err(e) => (Err(e), 0, "/fuel_prices_all"),
                    }
                }
                _ => (Ok(()), 0, ""),
            };
            let duration_ms = start.elapsed().as_millis() as u64;
            let ok = result.is_ok();
            let err = result.err();
            if !endpoint.is_empty() {
                let event = FetchEvent {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    collection: key.clone(),
                    endpoint: endpoint.to_string(),
                    row_count,
                    duration_ms,
                    triggered_by: "startup".to_string(),
                    ok,
                    error: err.clone(),
                };
                if let Ok(mut log) = activity.lock() {
                    log.push_fetch(event);
                }
            }
            (key, if ok { Ok(()) } else { Err(err.unwrap_or_default()) })
        }));
    }

    for handle in handles {
        match handle.await {
            Ok((key, Ok(()))) => info!("Prefetch complete for '{}'", key),
            Ok((key, Err(e))) => error!("Prefetch failed for '{}': {}", key, e),
            Err(e) => error!("Prefetch task panicked: {}", e),
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
