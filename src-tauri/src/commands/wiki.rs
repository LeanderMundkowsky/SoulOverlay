use tauri::State;

use crate::cache_store::{CacheResult, Collection};
use crate::commands::api::ApiResponse;
use crate::providers::wiki_specs::provider::fetch_wiki_specs;
use crate::state::AppState;
use crate::uex::types::{PriceEntry, UexResult};
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

    // Convert item results.
    // Deduplicate by name, preferring the variant that has UEX prices in cache.
    // This handles `_hair_extension` variants: the wiki stores e.g. "Deadhead Helmet" as
    // both `srvl_helmet_01_01_01` (base) and `srvl_helmet_01_01_14_hair_extension` (hair ext).
    // UEX may only sell one of them — we want the one that has actual price data.
    // Priority: (1) has prices in cache, (2) not a _hair_extension, (3) first seen.
    if let Ok(resp) = items_res {
        // Collect candidates keyed by name
        struct Candidate { uuid: String, class_name: String, classification: String }
        let mut by_name: std::collections::HashMap<String, Candidate> = std::collections::HashMap::new();

        for dto in resp.data {
            let uuid = dto.uuid.clone().unwrap_or_default();
            if uuid.is_empty() { continue; }
            let name = dto.name.clone().unwrap_or_default();
            if name.is_empty() { continue; }
            let class_name = dto.class_name.clone().unwrap_or_default();
            let classification = dto.classification.clone().unwrap_or_default();

            let candidate_has_prices = {
                let key = Collection::ItemPrices.storage_key_with_id(&uuid);
                !matches!(state.cache.get::<Vec<PriceEntry>>(&key), CacheResult::Missing)
            };
            let candidate_is_hair_ext = class_name.ends_with("_hair_extension");

            let replace = match by_name.get(&name) {
                None => true,
                Some(existing) => {
                    let existing_has_prices = {
                        let key = Collection::ItemPrices.storage_key_with_id(&existing.uuid);
                        !matches!(state.cache.get::<Vec<PriceEntry>>(&key), CacheResult::Missing)
                    };
                    let existing_is_hair_ext = existing.class_name.ends_with("_hair_extension");
                    // Prefer: has prices > not hair extension > first seen
                    (candidate_has_prices && !existing_has_prices)
                        || (!existing_has_prices && !candidate_has_prices && existing_is_hair_ext && !candidate_is_hair_ext)
                }
            };

            if replace {
                by_name.insert(name, Candidate { uuid, class_name, classification });
            }
        }

        for (name, c) in by_name {
            results.push(UexResult {
                id: c.uuid.clone(),
                name,
                kind: "item".to_string(),
                slug: c.classification,
                source: "wiki".to_string(),
                uuid: c.uuid,
            });
        }
    } else if let Err(e) = items_res {
        log::warn!("Wiki item search failed: {}", e);
    }

    // Convert vehicle results — deduplicate by name (wiki has store/edition variants
    // sharing the same name, e.g. "Polaris" + "Polaris Collector Military").
    if let Ok(resp) = vehicles_res {
        let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for dto in resp.data {
            let uuid = dto.uuid.clone().unwrap_or_default();
            if uuid.is_empty() { continue; }
            let name = dto.name.clone().unwrap_or_default();
            if !seen_names.insert(name.clone()) { continue; }
            results.push(UexResult {
                id: uuid.clone(),
                name,
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
