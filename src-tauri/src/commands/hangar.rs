use tauri::State;

use crate::cache_store::{CacheResult, Collection};
use crate::commands::api::ApiResponse;
use crate::state::AppState;
use crate::uex::{self, HangarVehicle};

/// Enrich fleet vehicles with photo URLs from the vehicles API.
async fn enrich_with_photos(
    fleet: &mut [HangarVehicle],
    state: &AppState,
    api_key: &str,
) {
    if let Ok(photo_map) = uex::fetch_vehicle_photo_map(&state.uex, api_key).await {
        for ship in fleet.iter_mut() {
            if let Some(url) = photo_map.get(&ship.id_vehicle) {
                ship.url_photo = Some(url.clone());
            }
        }
    }
}

/// Fetch the authenticated user's fleet from UEX.
/// Requires both `uex_api_key` and `uex_secret_key` to be configured.
#[tauri::command]
#[specta::specta]
pub async fn hangar_get_fleet(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<HangarVehicle>>, String> {
    let (api_key, secret_key, settings) = {
        let s = state.current_settings.lock().unwrap();
        (s.uex_api_key.clone(), s.uex_secret_key.clone(), s.clone())
    };

    if api_key.is_empty() {
        return Ok(ApiResponse::err("UEX API key not configured. Set it in Settings."));
    }
    if secret_key.is_empty() {
        return Ok(ApiResponse::err("UEX secret key not configured. Set it in Settings."));
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
                if let Ok(fleet) = uex::fetch_fleet(&uex, &ak, &sk).await {
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
    match uex::fetch_fleet(&state.uex, &api_key, &secret_key).await {
        Ok(mut fleet) => {
            let _ = state.cache.put(&cache_key, ttl, &fleet);
            enrich_with_photos(&mut fleet, &state, &api_key).await;
            Ok(ApiResponse::ok(fleet))
        }
        Err(e) => Ok(ApiResponse::err(e)),
    }
}
