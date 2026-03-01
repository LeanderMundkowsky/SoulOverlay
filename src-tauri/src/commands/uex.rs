use tauri::State;

use crate::state::AppState;
use crate::uex_client;

#[tauri::command]
pub async fn uex_search(
    query: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<Vec<uex_client::UexResult>, String> {
    let cache_key = format!("search:{}", query);
    uex_client::cached_fetch(&state.uex_cache, &cache_key, || {
        uex_client::search(&query, &api_key)
    })
    .await
}

#[tauri::command]
pub async fn uex_prices(
    commodity: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<Vec<uex_client::PriceEntry>, String> {
    let cache_key = format!("prices:{}", commodity);
    uex_client::cached_fetch(&state.uex_cache, &cache_key, || {
        uex_client::get_prices(&commodity, &api_key)
    })
    .await
}
