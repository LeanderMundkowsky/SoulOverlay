use log::{error, info};
use serde::Serialize;
use specta::Type;
use std::time::Instant;
use tauri::State;

use crate::activity::FetchEvent;
use crate::cache_store::{Collection, CollectionStatus};
use crate::providers::{self, RefreshContext};
use crate::settings::Settings;
use crate::state::AppState;

/// Response from cache refresh operations.
#[derive(Debug, Serialize, Type)]
pub struct CacheRefreshResult {
    pub ok: bool,
    pub collection: String,
    pub error: Option<String>,
}

/// Get status for all cached collections.
#[tauri::command]
#[specta::specta]
pub async fn cache_status(
    state: State<'_, AppState>,
) -> Result<Vec<CollectionStatus>, String> {
    Ok(state.cache.status())
}

/// Refresh a single collection by name.
#[tauri::command]
#[specta::specta]
pub async fn cache_refresh(
    collection: String,
    state: State<'_, AppState>,
) -> Result<CacheRefreshResult, String> {
    let settings = state.current_settings.lock().unwrap().clone();
    let result = refresh_collection_by_name(&collection, &settings.uex_api_key, &settings, &state, "manual").await;
    Ok(result)
}

/// Refresh all prefetchable collections, but only those whose TTL has expired.
/// This is the same logic used on startup.
#[tauri::command]
#[specta::specta]
pub async fn cache_refresh_expired(
    state: State<'_, AppState>,
) -> Result<Vec<CacheRefreshResult>, String> {
    let settings = state.current_settings.lock().unwrap().clone();
    let expired: Vec<String> = Collection::all()
        .iter()
        .map(|c| c.storage_key())
        .filter(|key| state.cache.is_expired(key))
        .collect();
    if !expired.is_empty() {
        info!("Refreshing {} expired collections in parallel: {}", expired.len(), expired.join(", "));
    }
    let futures: Vec<_> = expired
        .iter()
        .map(|key| refresh_collection_by_name(key, &settings.uex_api_key, &settings, &state, "manual"))
        .collect();
    Ok(futures::future::join_all(futures).await)
}

/// Refresh all prefetchable collections.
#[tauri::command]
#[specta::specta]
pub async fn cache_refresh_all(
    state: State<'_, AppState>,
) -> Result<Vec<CacheRefreshResult>, String> {
    let settings = state.current_settings.lock().unwrap().clone();

    // Phase 1: providers with no dependencies (catalogs)
    let all = providers::all_providers();
    let phase1_keys: Vec<String> = all.iter()
        .filter(|p| p.depends_on().is_empty())
        .map(|p| p.collection().storage_key())
        .collect();
    info!("Refreshing {} catalog collections in parallel: {}", phase1_keys.len(), phase1_keys.join(", "));
    let phase1_futures: Vec<_> = phase1_keys
        .iter()
        .map(|key| refresh_collection_by_name(key, &settings.uex_api_key, &settings, &state, "manual"))
        .collect();
    let mut results = futures::future::join_all(phase1_futures).await;

    // Phase 2: providers with dependencies (prices, entity_info, etc.)
    let phase2_keys: Vec<String> = all.iter()
        .filter(|p| !p.depends_on().is_empty())
        .map(|p| p.collection().storage_key())
        .collect();
    info!("Refreshing {} remaining collections in parallel: {}", phase2_keys.len(), phase2_keys.join(", "));
    let phase2_futures: Vec<_> = phase2_keys
        .iter()
        .map(|key| refresh_collection_by_name(key, &settings.uex_api_key, &settings, &state, "manual"))
        .collect();
    results.extend(futures::future::join_all(phase2_futures).await);
    Ok(results)
}

/// Internal helper: refresh a collection by its storage key name.
pub(crate) async fn refresh_collection_by_name(
    name: &str,
    api_key: &str,
    settings: &Settings,
    state: &AppState,
    triggered_by: &str,
) -> CacheRefreshResult {
    let start = Instant::now();

    let provider = match providers::provider_for(name) {
        Some(p) => p,
        None => {
            error!("Unknown collection: {}", name);
            return CacheRefreshResult {
                ok: false,
                collection: name.to_string(),
                error: Some(format!("Unknown collection: {}", name)),
            };
        }
    };

    if provider.requires_secret() && settings.uex_secret_key.is_empty() {
        return CacheRefreshResult {
            ok: false,
            collection: name.to_string(),
            error: Some(format!(
                "UEX secret key not configured; skipping {} refresh",
                name
            )),
        };
    }

    let secret = if settings.uex_secret_key.is_empty() {
        None
    } else {
        Some(settings.uex_secret_key.as_str())
    };

    let ctx = RefreshContext {
        client: &state.uex,
        cache: &state.cache,
        api_key,
        secret_key: secret,
        settings,
    };

    let result = provider.refresh(&ctx).await;

    let duration_ms = start.elapsed().as_millis() as u32;
    let (ok, row_count, err_msg) = match &result {
        Ok(count) => {
            info!("Refreshed '{}': {} entries in {}ms", name, count, duration_ms);
            (true, *count, None)
        }
        Err(e) => {
            error!("Failed to refresh collection '{}': {}", name, e);
            (false, 0, Some(e.clone()))
        }
    };

    let event = FetchEvent {
        timestamp: chrono::Utc::now().to_rfc3339(),
        collection: name.to_string(),
        endpoint: format!("/{}", name),
        row_count,
        duration_ms,
        triggered_by: triggered_by.to_string(),
        ok,
        error: err_msg.clone(),
    };
    if let Ok(mut log) = state.activity.lock() {
        log.push_fetch(event);
    }

    CacheRefreshResult {
        ok,
        collection: name.to_string(),
        error: err_msg,
    }
}

/// Public helper used by app_setup to prefetch all collections on startup.
/// Phase 1: providers with no dependencies, Phase 2: the rest.
pub async fn prefetch_all(state: &AppState) {
    let settings = state.current_settings.lock().unwrap().clone();
    let all = providers::all_providers();

    // Phase 1: no-dependency providers (catalogs)
    let phase1_keys: Vec<String> = all.iter()
        .filter(|p| p.depends_on().is_empty())
        .filter(|p| {
            let key = p.collection().storage_key();
            if state.cache.is_expired(&key) {
                true
            } else {
                info!("Collection '{}' is still fresh, skipping prefetch", key);
                false
            }
        })
        .map(|p| p.collection().storage_key())
        .collect();
    if !phase1_keys.is_empty() {
        info!("Prefetching {} catalog collections in parallel: {}", phase1_keys.len(), phase1_keys.join(", "));
    }
    let phase1_futures: Vec<_> = phase1_keys
        .iter()
        .map(|key| refresh_collection_by_name(key, &settings.uex_api_key, &settings, state, "startup"))
        .collect();
    for result in futures::future::join_all(phase1_futures).await {
        if !result.ok {
            if let Some(e) = &result.error {
                error!("Prefetch failed for {}: {}", result.collection, e);
            }
        }
    }

    // Phase 2: dependent providers (prices, entity_info, secret-required, etc.)
    let phase2_keys: Vec<String> = all.iter()
        .filter(|p| !p.depends_on().is_empty())
        .filter(|p| {
            let key = p.collection().storage_key();
            if state.cache.is_expired(&key) {
                true
            } else {
                info!("Collection '{}' is still fresh, skipping prefetch", key);
                false
            }
        })
        .map(|p| p.collection().storage_key())
        .collect();
    if !phase2_keys.is_empty() {
        info!("Prefetching {} remaining collections in parallel: {}", phase2_keys.len(), phase2_keys.join(", "));
    }
    let phase2_futures: Vec<_> = phase2_keys
        .iter()
        .map(|key| refresh_collection_by_name(key, &settings.uex_api_key, &settings, state, "startup"))
        .collect();
    for result in futures::future::join_all(phase2_futures).await {
        if !result.ok {
            if let Some(e) = &result.error {
                error!("Prefetch failed for {}: {}", result.collection, e);
            }
        }
    }
}

/// Try to acquire the refresh guard for `collection_key`. Returns `true` if the
/// guard was acquired (caller should refresh), `false` if already in-flight.
pub(crate) fn try_acquire_refresh(state: &AppState, collection_key: &str) -> bool {
    let mut guard = state.refreshing_collections.lock().unwrap();
    guard.insert(collection_key.to_string())
}

/// Release the refresh guard for `collection_key`.
pub(crate) fn release_refresh(state: &AppState, collection_key: &str) {
    let mut guard = state.refreshing_collections.lock().unwrap();
    guard.remove(collection_key);
}

/// Refresh a single collection if expired, respecting the in-flight guard.
/// Returns `true` if a refresh was performed.
pub(crate) async fn guarded_refresh(state: &AppState, collection: &Collection) -> bool {
    let key = collection.storage_key();
    if !state.cache.is_expired(&key) {
        return false;
    }
    if !try_acquire_refresh(state, &key) {
        return false;
    }
    let settings = state.current_settings.lock().unwrap().clone();
    let result = refresh_collection_by_name(&key, &settings.uex_api_key, &settings, state, "timer").await;
    release_refresh(state, &key);
    if !result.ok {
        if let Some(e) = &result.error {
            error!("Background refresh failed for '{}': {}", key, e);
        }
    }
    result.ok
}
