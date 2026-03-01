use log::{error, info};
use serde::Serialize;
use tauri::State;

use crate::cache_store::{Collection, CollectionStatus};
use crate::state::AppState;
use crate::uex_client;

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
    let api_key = state.current_settings.lock().unwrap().uex_api_key.clone();

    let result = refresh_collection_by_name(&collection, &api_key, &state).await;
    Ok(result)
}

/// Refresh all prefetchable collections.
#[tauri::command]
pub async fn cache_refresh_all(
    state: State<'_, AppState>,
) -> Result<Vec<CacheRefreshResult>, String> {
    let api_key = state.current_settings.lock().unwrap().uex_api_key.clone();

    let mut results = Vec::new();
    for collection in Collection::prefetch_list() {
        let name = collection.storage_key();
        let result = refresh_collection_by_name(&name, &api_key, &state).await;
        results.push(result);
    }

    Ok(results)
}

/// Internal helper: refresh a collection by its storage key name.
async fn refresh_collection_by_name(
    name: &str,
    api_key: &str,
    state: &AppState,
) -> CacheRefreshResult {
    let result = match name {
        "commodities" => {
            match uex_client::fetch_all_commodities(api_key).await {
                Ok(data) => {
                    info!("Refreshed commodities: {} entries", data.len());
                    state.cache.put(&Collection::Commodities.storage_key(), Collection::Commodities, &data)
                }
                Err(e) => Err(e),
            }
        }
        "vehicles" => {
            match uex_client::fetch_all_vehicles(api_key).await {
                Ok(data) => {
                    info!("Refreshed vehicles: {} entries", data.len());
                    state.cache.put(&Collection::Vehicles.storage_key(), Collection::Vehicles, &data)
                }
                Err(e) => Err(e),
            }
        }
        "items" => {
            match uex_client::fetch_all_items(api_key).await {
                Ok(data) => {
                    info!("Refreshed items: {} entries", data.len());
                    state.cache.put(&Collection::Items.storage_key(), Collection::Items, &data)
                }
                Err(e) => Err(e),
            }
        }
        "locations" => {
            match uex_client::fetch_all_locations(api_key).await {
                Ok(data) => {
                    info!("Refreshed locations: {} entries", data.len());
                    state.cache.put(&Collection::Locations.storage_key(), Collection::Locations, &data)
                }
                Err(e) => Err(e),
            }
        }
        "commodity_prices" => {
            // Cannot refresh all prices without knowing which commodity IDs to fetch.
            // Invalidate existing cached prices instead.
            state.cache.invalidate_collection(Collection::CommodityPrices);
            info!("Invalidated all cached commodity prices");
            Ok(())
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
/// Not a tauri command — called directly from Rust.
pub async fn prefetch_all(state: &AppState) {
    let api_key = state.current_settings.lock().unwrap().uex_api_key.clone();

    for collection in Collection::prefetch_list() {
        let key = collection.storage_key();
        if state.cache.is_expired(&key) {
            info!("Prefetching expired collection: {}", key);
            let result = refresh_collection_by_name(&key, &api_key, state).await;
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
