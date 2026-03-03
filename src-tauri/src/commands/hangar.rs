use tauri::State;

use crate::cache_store::{CacheResult, Collection};
use crate::commands::api::ApiResponse;
use crate::state::AppState;
use crate::uex::{self, HangarVehicle};

/// Fetch the authenticated user's fleet from UEX.
/// Requires both `uex_api_key` and `uex_secret_key` to be configured.
/// Results are cached in SQLite with a 10-minute TTL.
#[tauri::command]
pub async fn hangar_get_fleet(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<HangarVehicle>>, String> {
    let (api_key, secret_key) = {
        let settings = state.current_settings.lock().unwrap();
        (settings.uex_api_key.clone(), settings.uex_secret_key.clone())
    };

    if api_key.is_empty() {
        return Ok(ApiResponse::err("UEX API key not configured. Set it in Settings."));
    }
    if secret_key.is_empty() {
        return Ok(ApiResponse::err("UEX secret key not configured. Set it in Settings."));
    }

    let cache_key = Collection::Fleet.storage_key();

    // Serve from cache if fresh
    match state.cache.get::<Vec<HangarVehicle>>(&cache_key) {
        CacheResult::Fresh(data) => return Ok(ApiResponse::ok(data)),
        CacheResult::Stale(data) => {
            // Return stale data immediately, refresh in background
            let uex = state.uex.clone();
            let cache = state.cache_arc();
            let ak = api_key.clone();
            let sk = secret_key.clone();
            let ttl = Collection::Fleet.ttl_secs();
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
        Ok(fleet) => {
            let ttl = Collection::Fleet.ttl_secs();
            let _ = state.cache.put(&cache_key, ttl, &fleet);
            Ok(ApiResponse::ok(fleet))
        }
        Err(e) => Ok(ApiResponse::err(e)),
    }
}
