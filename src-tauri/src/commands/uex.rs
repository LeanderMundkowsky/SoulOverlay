use tauri::State;

use crate::cache_store::{CacheResult, Collection};
use crate::providers::search_in_collection;
use crate::providers::commodities::{search_commodities, get_commodity_prices};
use crate::providers::vehicles::search_vehicles;
use crate::providers::items::search_items;
use crate::providers::locations::search_locations;
use crate::state::AppState;
use crate::uex::types::{PriceEntry, UexResult};

#[tauri::command]
#[specta::specta]
pub async fn uex_search(
    query: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<Vec<UexResult>, String> {
    // Try cache first
    let key = Collection::Commodities.storage_key();
    match state.cache.get::<Vec<UexResult>>(&key) {
        CacheResult::Fresh(data) | CacheResult::Stale(data) => {
            Ok(search_in_collection(&data, &query))
        }
        CacheResult::Missing => {
            // Fallback to direct API call
            search_commodities(&state.uex, &query, &api_key).await
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn uex_search_all(
    query: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<Vec<UexResult>, String> {
    let mut results = Vec::new();

    let collections = [
        Collection::Commodities,
        Collection::Vehicles,
        Collection::Items,
        Collection::Locations,
    ];

    for collection in &collections {
        let key = collection.storage_key();
        match state.cache.get::<Vec<UexResult>>(&key) {
            CacheResult::Fresh(data) | CacheResult::Stale(data) => {
                let mut filtered = search_in_collection(&data, &query);
                results.append(&mut filtered);
            }
            CacheResult::Missing => {
                // Fallback to direct API call for this collection
                let fetched = match collection {
                    Collection::Commodities => search_commodities(&state.uex, &query, &api_key).await,
                    Collection::Vehicles => search_vehicles(&state.uex, &query, &api_key).await,
                    Collection::Items => search_items(&state.uex, &query, &api_key).await,
                    Collection::Locations => search_locations(&state.uex, &query, &api_key).await,
                    Collection::CommodityPrices
                    | Collection::RawCommodityPrices
                    | Collection::ItemPrices
                    | Collection::VehiclePurchasePrices
                    | Collection::VehicleRentalPrices
                    | Collection::FuelPrices
                    | Collection::Fleet
                    | Collection::UserProfile
                    | Collection::EntityInfo => Ok(vec![]),
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
#[specta::specta]
pub async fn uex_prices(
    commodity: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<Vec<PriceEntry>, String> {
    let cache_key = Collection::CommodityPrices.storage_key_with_id(&commodity);

    // Check cache first
    match state.cache.get::<Vec<PriceEntry>>(&cache_key) {
        CacheResult::Fresh(prices) | CacheResult::Stale(prices) => {
            return Ok(prices);
        }
        CacheResult::Missing => {}
    }

    // Fetch from API and cache
    let prices = get_commodity_prices(&state.uex, &commodity, &api_key).await?;
    let settings = state.current_settings.lock().unwrap().clone();
    let ttl = Collection::CommodityPrices.ttl_for(&settings);
    let _ = state.cache.put(&cache_key, ttl, &prices);
    Ok(prices)
}
