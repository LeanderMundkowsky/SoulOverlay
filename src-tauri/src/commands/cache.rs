use log::{error, info};
use serde::Serialize;
use std::sync::Arc;
use tauri::State;

use crate::cache_store::{CacheStore, Collection, CollectionStatus};
use crate::settings::Settings;
use crate::state::AppState;
use crate::uex_client;
use crate::uex_client::PriceEntry;

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
    let result = refresh_collection_by_name(&collection, &settings.uex_api_key, &settings, &state).await;
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
            let result = refresh_collection_by_name(&key, &settings.uex_api_key, &settings, &state).await;
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
        let result = refresh_collection_by_name(&name, &settings.uex_api_key, &settings, &state).await;
        results.push(result);
    }
    Ok(results)
}

// ── Split helper ───────────────────────────────────────────────────────────

/// Split a flat vec of price entries by entity_id and store each group
/// as a separate per-entity cache entry (e.g. `commodity_prices:42`).
fn store_prices_split(
    cache: &CacheStore,
    entries: &[PriceEntry],
    collection: Collection,
    ttl: i64,
) -> Result<(), String> {
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

/// Internal helper: refresh a collection by its storage key name.
async fn refresh_collection_by_name(
    name: &str,
    api_key: &str,
    settings: &Settings,
    state: &AppState,
) -> CacheRefreshResult {
    let prices_ttl = settings.cache_ttl_prices_secs as i64;
    let catalog_ttl = settings.cache_ttl_catalog_secs as i64;

    let result = match name {
        "commodities" => {
            match uex_client::fetch_all_commodities(api_key).await {
                Ok(data) => {
                    info!("Refreshed commodities: {} entries", data.len());
                    state.cache.put(&Collection::Commodities.storage_key(), prices_ttl, &data)
                }
                Err(e) => Err(e),
            }
        }
        "vehicles" => {
            match uex_client::fetch_all_vehicles(api_key).await {
                Ok(data) => {
                    info!("Refreshed vehicles: {} entries", data.len());
                    state.cache.put(&Collection::Vehicles.storage_key(), catalog_ttl, &data)
                }
                Err(e) => Err(e),
            }
        }
        "items" => {
            match uex_client::fetch_all_items(api_key).await {
                Ok(data) => {
                    info!("Refreshed items: {} entries", data.len());
                    state.cache.put(&Collection::Items.storage_key(), catalog_ttl, &data)
                }
                Err(e) => Err(e),
            }
        }
        "locations" => {
            match uex_client::fetch_all_locations(api_key).await {
                Ok(data) => {
                    info!("Refreshed locations: {} entries", data.len());
                    state.cache.put(&Collection::Locations.storage_key(), catalog_ttl, &data)
                }
                Err(e) => Err(e),
            }
        }
        "commodity_prices" => {
            match uex_client::fetch_all_commodity_prices(api_key).await {
                Ok(data) => {
                    info!("Refreshed commodity prices: {} rows", data.len());
                    store_prices_split(&state.cache, &data, Collection::CommodityPrices, prices_ttl)
                }
                Err(e) => Err(e),
            }
        }
        "raw_commodity_prices" => {
            match uex_client::fetch_all_raw_commodity_prices(api_key).await {
                Ok(data) => {
                    info!("Refreshed raw commodity prices: {} rows", data.len());
                    store_prices_split(&state.cache, &data, Collection::RawCommodityPrices, prices_ttl)
                }
                Err(e) => Err(e),
            }
        }
        "item_prices" => {
            match uex_client::fetch_all_item_prices(api_key).await {
                Ok(data) => {
                    info!("Refreshed item prices: {} rows", data.len());
                    store_prices_split(&state.cache, &data, Collection::ItemPrices, prices_ttl)
                }
                Err(e) => Err(e),
            }
        }
        "vehicle_purchase_prices" => {
            match uex_client::fetch_all_vehicle_purchase_prices(api_key).await {
                Ok(data) => {
                    info!("Refreshed vehicle purchase prices: {} rows", data.len());
                    store_prices_split(&state.cache, &data, Collection::VehiclePurchasePrices, prices_ttl)
                }
                Err(e) => Err(e),
            }
        }
        "vehicle_rental_prices" => {
            match uex_client::fetch_all_vehicle_rental_prices(api_key).await {
                Ok(data) => {
                    info!("Refreshed vehicle rental prices: {} rows", data.len());
                    store_prices_split(&state.cache, &data, Collection::VehicleRentalPrices, prices_ttl)
                }
                Err(e) => Err(e),
            }
        }
        "fuel_prices" => {
            match uex_client::fetch_all_fuel_prices(api_key).await {
                Ok(data) => {
                    info!("Refreshed fuel prices: {} rows", data.len());
                    store_prices_split(&state.cache, &data, Collection::FuelPrices, prices_ttl)
                }
                Err(e) => Err(e),
            }
        }
        _ => Err(format!("Unknown collection: {}", name)),
    };

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
            let result = refresh_collection_by_name(&key, &settings.uex_api_key, &settings, state).await;
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
        let coll = *collection;
        let ttl = prices_ttl;

        handles.push(tokio::spawn(async move {
            let key = coll.storage_key();
            let result: Result<(), String> = match key.as_str() {
                "commodity_prices" => uex_client::fetch_all_commodity_prices(&api_key).await
                    .and_then(|data| { info!("Prefetched commodity prices: {} rows", data.len()); store_prices_split_arc(&cache, &data, coll, ttl) }),
                "raw_commodity_prices" => uex_client::fetch_all_raw_commodity_prices(&api_key).await
                    .and_then(|data| { info!("Prefetched raw commodity prices: {} rows", data.len()); store_prices_split_arc(&cache, &data, coll, ttl) }),
                "item_prices" => uex_client::fetch_all_item_prices(&api_key).await
                    .and_then(|data| { info!("Prefetched item prices: {} rows", data.len()); store_prices_split_arc(&cache, &data, coll, ttl) }),
                "vehicle_purchase_prices" => uex_client::fetch_all_vehicle_purchase_prices(&api_key).await
                    .and_then(|data| { info!("Prefetched vehicle purchase prices: {} rows", data.len()); store_prices_split_arc(&cache, &data, coll, ttl) }),
                "vehicle_rental_prices" => uex_client::fetch_all_vehicle_rental_prices(&api_key).await
                    .and_then(|data| { info!("Prefetched vehicle rental prices: {} rows", data.len()); store_prices_split_arc(&cache, &data, coll, ttl) }),
                "fuel_prices" => uex_client::fetch_all_fuel_prices(&api_key).await
                    .and_then(|data| { info!("Prefetched fuel prices: {} rows", data.len()); store_prices_split_arc(&cache, &data, coll, ttl) }),
                _ => Ok(()),
            };
            (key, result)
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
