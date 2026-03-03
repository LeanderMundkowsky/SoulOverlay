use tauri::State;

use crate::cache_store::{CacheResult, Collection};
use crate::state::AppState;
use crate::uex;

#[tauri::command]
pub async fn uex_search(
    query: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<Vec<uex::UexResult>, String> {
    // Try cache first
    let key = Collection::Commodities.storage_key();
    match state.cache.get::<Vec<uex::UexResult>>(&key) {
        CacheResult::Fresh(data) | CacheResult::Stale(data) => {
            Ok(uex::search_in_collection(&data, &query))
        }
        CacheResult::Missing => {
            // Fallback to direct API call
            uex::search_commodities(&state.uex, &query, &api_key).await
        }
    }
}

#[tauri::command]
pub async fn uex_search_all(
    query: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<Vec<uex::UexResult>, String> {
    let mut results = Vec::new();

    let collections = [
        Collection::Commodities,
        Collection::Vehicles,
        Collection::Items,
        Collection::Locations,
    ];

    for collection in &collections {
        let key = collection.storage_key();
        match state.cache.get::<Vec<uex::UexResult>>(&key) {
            CacheResult::Fresh(data) | CacheResult::Stale(data) => {
                let mut filtered = uex::search_in_collection(&data, &query);
                results.append(&mut filtered);
            }
            CacheResult::Missing => {
                // Fallback to direct API call for this collection
                let fetched = match collection {
                    Collection::Commodities => uex::search_commodities(&state.uex, &query, &api_key).await,
                    Collection::Vehicles => uex::search_vehicles(&state.uex, &query, &api_key).await,
                    Collection::Items => uex::search_items(&state.uex, &query, &api_key).await,
                    Collection::Locations => uex::search_locations(&state.uex, &query, &api_key).await,
                    Collection::CommodityPrices
                    | Collection::RawCommodityPrices
                    | Collection::ItemPrices
                    | Collection::VehiclePurchasePrices
                    | Collection::VehicleRentalPrices
                    | Collection::FuelPrices
                    | Collection::Fleet => Ok(vec![]),
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
) -> Result<Vec<uex::PriceEntry>, String> {
    let cache_key = Collection::CommodityPrices.storage_key_with_id(&commodity);

    // Check cache first
    match state.cache.get::<Vec<uex::PriceEntry>>(&cache_key) {
        CacheResult::Fresh(prices) | CacheResult::Stale(prices) => {
            return Ok(prices);
        }
        CacheResult::Missing => {}
    }

    // Fetch from API and cache
    let prices = uex::get_commodity_prices(&state.uex, &commodity, &api_key).await?;
    let ttl = state.current_settings.lock().unwrap().cache_ttl_prices_secs as i64;
    let _ = state.cache.put(&cache_key, ttl, &prices);
    Ok(prices)
}
