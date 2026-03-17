use async_trait::async_trait;

use crate::cache_store::{CacheResult, CacheStore, Collection};
use crate::providers::{PerEntityProvider, RefreshContext};
use crate::wiki::client;
use crate::wiki::types::WikiEntitySpecs;

pub struct WikiSpecsProvider;

#[async_trait]
impl PerEntityProvider for WikiSpecsProvider {
    fn collection(&self) -> Collection {
        Collection::WikiSpecs
    }

    /// No-op: wiki specs are lazy-fetched per entity, not bulk-refreshed.
    async fn refresh(&self, _ctx: &RefreshContext<'_>) -> Result<u32, String> {
        Ok(0)
    }
}

/// Fetch wiki specs for an entity, using cache if available.
///
/// Lookup strategy:
/// 1. Check cache `wiki_specs:{kind}:{entity_id}`
/// 2. If UUID provided → direct UUID lookup via Wiki API
/// 3. Fallback → search by name, take first match
pub async fn fetch_wiki_specs(
    http_client: &reqwest::Client,
    cache: &CacheStore,
    kind: &str,
    entity_id: &str,
    entity_name: &str,
    uuid: &str,
) -> Result<WikiEntitySpecs, String> {
    let cache_key = Collection::WikiSpecs.storage_key_with_id(&format!("{}:{}", kind, entity_id));

    // 1. Check cache
    match cache.get::<WikiEntitySpecs>(&cache_key) {
        CacheResult::Fresh(specs) | CacheResult::Stale(specs) => return Ok(specs),
        CacheResult::Missing => {}
    }

    // 2. Fetch from Wiki API
    let specs = match kind {
        "vehicle" => fetch_vehicle_specs(http_client, uuid, entity_name).await?,
        _ => fetch_item_specs(http_client, uuid, entity_name).await?,
    };

    // 3. Cache with 7-day TTL
    let ttl = Collection::WikiSpecs.ttl_secs();
    let _ = cache.put(&cache_key, ttl, &specs);

    Ok(specs)
}

/// Fetch item specs: try UUID lookup first, fall back to name search.
async fn fetch_item_specs(
    http_client: &reqwest::Client,
    uuid: &str,
    name: &str,
) -> Result<WikiEntitySpecs, String> {
    // Try direct UUID lookup
    if !uuid.is_empty() {
        match client::get_item(http_client, uuid).await {
            Ok(dto) => return Ok(WikiEntitySpecs::from(dto)),
            Err(e) => log::debug!("Wiki item UUID lookup failed ({}): {}", uuid, e),
        }
    }

    // Fallback: search by name
    if !name.is_empty() {
        match client::search_items(http_client, name, 5).await {
            Ok(resp) => {
                // Try exact match first, then containment (handles manufacturer prefixes)
                let name_lower = name.to_lowercase();
                if let Some(dto) = resp.data.into_iter().find(|d| {
                    d.name.as_deref().map(|n| {
                        let n_lower = n.to_lowercase();
                        n_lower == name_lower
                            || name_lower.contains(&n_lower)
                            || n_lower.contains(&name_lower)
                    }).unwrap_or(false)
                }) {
                    return Ok(WikiEntitySpecs::from(dto));
                }
            }
            Err(e) => log::debug!("Wiki item search failed for '{}': {}", name, e),
        }
    }

    Err(format!("No wiki specs found for item '{}'", name))
}

/// Fetch vehicle specs: try UUID lookup first, fall back to name search.
async fn fetch_vehicle_specs(
    http_client: &reqwest::Client,
    uuid: &str,
    name: &str,
) -> Result<WikiEntitySpecs, String> {
    if !uuid.is_empty() {
        match client::get_vehicle(http_client, uuid).await {
            Ok(dto) => return Ok(WikiEntitySpecs::from(dto)),
            Err(e) => log::debug!("Wiki vehicle UUID lookup failed ({}): {}", uuid, e),
        }
    }

    if !name.is_empty() {
        match client::search_vehicles(http_client, name, 5).await {
            Ok(resp) => {
                // Try exact match first, then containment (handles manufacturer prefixes)
                let name_lower = name.to_lowercase();
                if let Some(dto) = resp.data.into_iter().find(|d| {
                    d.name.as_deref().map(|n| {
                        let n_lower = n.to_lowercase();
                        n_lower == name_lower
                            || name_lower.contains(&n_lower)
                            || n_lower.contains(&name_lower)
                    }).unwrap_or(false)
                }) {
                    return Ok(WikiEntitySpecs::from(dto));
                }
            }
            Err(e) => log::debug!("Wiki vehicle search failed for '{}': {}", name, e),
        }
    }

    Err(format!("No wiki specs found for vehicle '{}'", name))
}
