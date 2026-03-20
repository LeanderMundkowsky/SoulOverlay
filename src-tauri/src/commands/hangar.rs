use tauri::State;

use crate::cache_store::{CacheResult, Collection};
use crate::commands::api::ApiResponse;
use crate::providers::fleet::fetch_fleet;
use crate::providers::vehicles::fetch_vehicle_photo_map;
use crate::state::AppState;
use crate::uex::types::HangarVehicle;

/// Enrich fleet vehicles with photo URLs from the vehicles API.
async fn enrich_with_photos(
    fleet: &mut [HangarVehicle],
    state: &AppState,
    api_key: &str,
) {
    if let Ok(photo_map) = fetch_vehicle_photo_map(&state.uex, api_key).await {
        for ship in fleet.iter_mut() {
            if let Some(url) = photo_map.get(&ship.id_vehicle) {
                ship.url_photo = Some(url.clone());
            }
        }
    }
}

/// Fetch the authenticated user's fleet from UEX.
/// Requires a logged-in backend account with a UEX secret key.
#[tauri::command]
#[specta::specta]
pub async fn hangar_get_fleet(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<HangarVehicle>>, String> {
    let api_key = state.fetched_api_key.lock().unwrap().clone();
    let secret_key = state.backend_account.lock().unwrap()
        .as_ref()
        .and_then(|a| a.uex_secret_key.clone())
        .unwrap_or_default();
    let settings = state.current_settings.lock().unwrap().clone();

    if secret_key.is_empty() {
        return Ok(ApiResponse::err(
            "Log in and set your UEX Secret Key in your Profile to access your hangar."
        ));
    }

    let cache_key = Collection::Fleet.storage_key();
    let ttl = Collection::Fleet.ttl_for(&settings);

    // Serve from cache if fresh
    match state.cache.get::<Vec<HangarVehicle>>(&cache_key) {
        CacheResult::Fresh(mut data) => {
            enrich_with_photos(&mut data, &state, &api_key).await;
            return Ok(ApiResponse::ok(data));
        }
        CacheResult::Stale(mut data) => {
            enrich_with_photos(&mut data, &state, &api_key).await;
            // Return stale data immediately, refresh in background
            let uex = state.uex.clone();
            let cache = state.cache_arc();
            let ak = api_key.clone();
            let sk = secret_key.clone();
            tokio::spawn(async move {
                if let Ok(fleet) = fetch_fleet(&uex, &ak, &sk).await {
                    let key = Collection::Fleet.storage_key();
                    let _ = cache.put(&key, ttl, &fleet);
                }
            });
            let mut resp = ApiResponse::ok(data);
            resp.stale = true;
            return Ok(resp);
        }
        CacheResult::Missing => {}
    }

    // No cache — fetch directly
    match fetch_fleet(&state.uex, &api_key, &secret_key).await {
        Ok(mut fleet) => {
            let _ = state.cache.put(&cache_key, ttl, &fleet);
            enrich_with_photos(&mut fleet, &state, &api_key).await;
            Ok(ApiResponse::ok(fleet))
        }
        Err(e) => Ok(ApiResponse::err(e)),
    }
}
