use tauri::State;

use crate::cache_store::{CacheResult, Collection};
use crate::commands::api::ApiResponse;
use crate::providers::user::fetch_user_profile;
use crate::state::AppState;
use crate::uex::types::UexUserProfile;

/// Fetch the authenticated user's profile from UEX.
/// Requires `uex_secret_key` to be configured. The UEX API key is fetched from the backend at startup.
#[tauri::command]
#[specta::specta]
pub async fn user_get_profile(
    state: State<'_, AppState>,
) -> Result<ApiResponse<UexUserProfile>, String> {
    let api_key = state.fetched_api_key.lock().unwrap().clone();
    let (secret_key, settings) = {
        let s = state.current_settings.lock().unwrap();
        (s.uex_secret_key.clone(), s.clone())
    };

    if secret_key.is_empty() {
        return Ok(ApiResponse::err("UEX secret key not configured. Set it in Settings."));
    }

    let cache_key = Collection::UserProfile.storage_key();
    let ttl = Collection::UserProfile.ttl_for(&settings);

    // Serve from cache if fresh
    match state.cache.get::<UexUserProfile>(&cache_key) {
        CacheResult::Fresh(data) => {
            return Ok(ApiResponse::ok(data));
        }
        CacheResult::Stale(data) => {
            // Return stale data immediately, refresh in background
            let uex = state.uex.clone();
            let cache = state.cache_arc();
            let ak = api_key.clone();
            let sk = secret_key.clone();
            tokio::spawn(async move {
                if let Ok(profile) = fetch_user_profile(&uex, &ak, &sk).await {
                    let key = Collection::UserProfile.storage_key();
                    let _ = cache.put(&key, ttl, &profile);
                }
            });
            let mut resp = ApiResponse::ok(data);
            resp.stale = true;
            return Ok(resp);
        }
        CacheResult::Missing => {}
    }

    // No cache — fetch directly
    match fetch_user_profile(&state.uex, &api_key, &secret_key).await {
        Ok(profile) => {
            let _ = state.cache.put(&cache_key, ttl, &profile);
            Ok(ApiResponse::ok(profile))
        }
        Err(e) => Ok(ApiResponse::err(e)),
    }
}
