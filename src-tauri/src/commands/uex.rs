use tauri::State;

use crate::cache_store::{CacheResult, Collection};
use crate::state::AppState;
use crate::uex_client;

#[tauri::command]
pub async fn uex_search(
    query: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<Vec<uex_client::UexResult>, String> {
    // Try cache first
    let key = Collection::Commodities.storage_key();
    match state.cache.get::<Vec<uex_client::UexResult>>(&key) {
        CacheResult::Fresh(data) | CacheResult::Stale(data) => {
            Ok(uex_client::search_in_collection(&data, &query))
        }
        CacheResult::Missing => {
            // Fallback to direct API call
            uex_client::search_commodities(&query, &api_key).await
        }
    }
}

#[tauri::command]
pub async fn uex_search_all(
    query: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<Vec<uex_client::UexResult>, String> {
    let mut results = Vec::new();

    let collections = [
        Collection::Commodities,
        Collection::Vehicles,
        Collection::Items,
        Collection::Locations,
    ];

    for collection in &collections {
        let key = collection.storage_key();
        match state.cache.get::<Vec<uex_client::UexResult>>(&key) {
            CacheResult::Fresh(data) | CacheResult::Stale(data) => {
                let mut filtered = uex_client::search_in_collection(&data, &query);
                results.append(&mut filtered);
            }
            CacheResult::Missing => {
                // Fallback to direct API call for this collection
                let fetched = match collection {
                    Collection::Commodities => uex_client::search_commodities(&query, &api_key).await,
                    Collection::Vehicles => uex_client::search_vehicles(&query, &api_key).await,
                    Collection::Items => uex_client::search_items(&query, &api_key).await,
                    Collection::Locations => uex_client::search_locations(&query, &api_key).await,
                    Collection::CommodityPrices => Ok(vec![]),
                };
                if let Ok(mut v) = fetched {
                    results.append(&mut v);
                }
            }
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn uex_prices(
    commodity: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<Vec<uex_client::PriceEntry>, String> {
    let cache_key = Collection::CommodityPrices.storage_key_with_id(&commodity);

    // Check cache first
    match state.cache.get::<Vec<uex_client::PriceEntry>>(&cache_key) {
        CacheResult::Fresh(prices) | CacheResult::Stale(prices) => {
            return Ok(prices);
        }
        CacheResult::Missing => {}
    }

    // Fetch from API and cache
    let prices = uex_client::get_prices(&commodity, &api_key).await?;
    let ttl = state.current_settings.lock().unwrap().cache_ttl_prices_secs as i64;
    let _ = state.cache.put(&cache_key, ttl, &prices);
    Ok(prices)
}
