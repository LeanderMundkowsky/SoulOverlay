use tauri::State;

use crate::commands::api::ApiResponse;
use crate::providers::wiki_specs::provider::fetch_wiki_specs;
use crate::state::AppState;
use crate::uex::types::UexResult;
use crate::wiki::types::WikiEntitySpecs;
use crate::wiki::client;

// ── Wiki Search ────────────────────────────────────────────────────────────

/// Search the Star Citizen Wiki for items and vehicles by name.
/// Returns results formatted as UexResult with `source: "wiki"`.
#[tauri::command]
#[specta::specta]
pub async fn wiki_search(
    query: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<UexResult>>, String> {
    let http = state.uex.client();
    let q = query.trim();

    if q.is_empty() {
        return Ok(ApiResponse::ok(vec![]));
    }

    // Search items and vehicles in parallel
    let (items_res, vehicles_res) = tokio::join!(
        client::search_items(http, q, 20),
        client::search_vehicles(http, q, 10),
    );

    let mut results: Vec<UexResult> = Vec::new();

    // Convert item results
    if let Ok(resp) = items_res {
        for dto in resp.data {
            let uuid = dto.uuid.clone().unwrap_or_default();
            if uuid.is_empty() {
                continue;
            }
            results.push(UexResult {
                id: uuid.clone(),
                name: dto.name.unwrap_or_default(),
                kind: "item".to_string(),
                slug: dto.classification.clone().unwrap_or_default(),
                source: "wiki".to_string(),
                uuid,
            });
        }
    } else if let Err(e) = items_res {
        log::warn!("Wiki item search failed: {}", e);
    }

    // Convert vehicle results
    if let Ok(resp) = vehicles_res {
        for dto in resp.data {
            let uuid = dto.uuid.clone().unwrap_or_default();
            if uuid.is_empty() {
                continue;
            }
            results.push(UexResult {
                id: uuid.clone(),
                name: dto.name.unwrap_or_default(),
                kind: "vehicle".to_string(),
                slug: dto.vehicle_type.unwrap_or_default(),
                source: "wiki".to_string(),
                uuid,
            });
        }
    } else if let Err(e) = vehicles_res {
        log::warn!("Wiki vehicle search failed: {}", e);
    }

    Ok(ApiResponse::ok(results))
}

// ── Wiki Entity Specs ──────────────────────────────────────────────────────

/// Fetch detailed specifications for an entity from the Wiki API.
/// Results are cached for 7 days.
#[tauri::command]
#[specta::specta]
pub async fn wiki_entity_specs(
    kind: String,
    entity_id: String,
    entity_name: String,
    uuid: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<WikiEntitySpecs>, String> {
    let http = state.uex.client();
    let cache = &state.cache;

    match fetch_wiki_specs(http, cache, &kind, &entity_id, &entity_name, &uuid).await {
        Ok(specs) => Ok(ApiResponse::ok(specs)),
        Err(e) => Ok(ApiResponse::err(e)),
    }
}
