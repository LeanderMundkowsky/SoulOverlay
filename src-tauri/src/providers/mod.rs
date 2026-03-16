pub mod commodities;
pub mod contested_zones;
pub mod entity_info;
pub mod fleet;
pub mod fuel;
pub mod items;
pub mod locations;
pub mod user;
pub mod vehicles;

use async_trait::async_trait;

use crate::cache_store::{CacheResult, CacheStore, Collection};
use crate::settings::Settings;
use crate::uex::types::{EntityInfo, PriceEntry, UexResult};
use crate::uex::UexClient;

// ── Refresh context ────────────────────────────────────────────────────────

/// Shared context passed to every provider's `refresh` method.
pub struct RefreshContext<'a> {
    pub client: &'a UexClient,
    pub cache: &'a CacheStore,
    pub api_key: &'a str,
    pub secret_key: Option<&'a str>,
    pub settings: &'a Settings,
}

// ── Provider traits ────────────────────────────────────────────────────────

/// Provider for collections stored as a single cache entry (e.g. `Vec<UexResult>`).
#[async_trait]
pub trait BlobProvider: Send + Sync {
    fn collection(&self) -> Collection;
    fn requires_secret(&self) -> bool { false }
    fn depends_on(&self) -> &[Collection] { &[] }

    /// Fetch from API and store as a single blob. Returns item count.
    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String>;
}

/// Provider for collections stored as per-entity sub-keys (e.g. `commodity_prices:42`).
#[async_trait]
pub trait PerEntityProvider: Send + Sync {
    fn collection(&self) -> Collection;
    fn requires_secret(&self) -> bool { false }
    fn depends_on(&self) -> &[Collection] { &[] }

    /// Fetch from API and store per-entity. Returns total item count.
    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String>;
}

// ── Unified dispatch ───────────────────────────────────────────────────────

/// Wraps both provider types for uniform dispatch.
pub enum AnyProvider {
    Blob(Box<dyn BlobProvider>),
    PerEntity(Box<dyn PerEntityProvider>),
}

impl AnyProvider {
    pub fn collection(&self) -> Collection {
        match self {
            Self::Blob(p) => p.collection(),
            Self::PerEntity(p) => p.collection(),
        }
    }

    pub fn requires_secret(&self) -> bool {
        match self {
            Self::Blob(p) => p.requires_secret(),
            Self::PerEntity(p) => p.requires_secret(),
        }
    }

    pub fn depends_on(&self) -> &[Collection] {
        match self {
            Self::Blob(p) => p.depends_on(),
            Self::PerEntity(p) => p.depends_on(),
        }
    }

    pub async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        match self {
            Self::Blob(p) => p.refresh(ctx).await,
            Self::PerEntity(p) => p.refresh(ctx).await,
        }
    }
}

// ── Provider registry ──────────────────────────────────────────────────────

/// Returns all registered collection providers.
pub fn all_providers() -> Vec<AnyProvider> {
    vec![
        // Catalogs (no dependencies)
        AnyProvider::Blob(Box::new(commodities::provider::CommoditiesCatalog)),
        AnyProvider::Blob(Box::new(vehicles::provider::VehiclesCatalog)),
        AnyProvider::Blob(Box::new(items::provider::ItemsCatalog)),
        AnyProvider::Blob(Box::new(locations::provider::LocationsCatalog)),
        // Prices (depend on catalogs)
        AnyProvider::PerEntity(Box::new(commodities::provider::CommodityPrices)),
        AnyProvider::PerEntity(Box::new(commodities::provider::RawCommodityPrices)),
        AnyProvider::PerEntity(Box::new(items::provider::ItemPrices)),
        AnyProvider::PerEntity(Box::new(vehicles::provider::VehiclePurchasePrices)),
        AnyProvider::PerEntity(Box::new(vehicles::provider::VehicleRentalPrices)),
        AnyProvider::PerEntity(Box::new(fuel::provider::FuelPrices)),
        // Entity info (no dependencies — does its own API calls)
        AnyProvider::PerEntity(Box::new(entity_info::provider::EntityInfoProvider)),
        // Auth-required
        AnyProvider::Blob(Box::new(fleet::provider::FleetProvider)),
        AnyProvider::Blob(Box::new(user::provider::UserProfileProvider)),
    ]
}

/// Look up a provider by its collection storage key.
pub fn provider_for(name: &str) -> Option<AnyProvider> {
    all_providers().into_iter().find(|p| p.collection().storage_key() == name)
}

// ── Shared helpers ─────────────────────────────────────────────────────────

/// Store data as a single blob cache entry. Returns item count.
pub fn store_blob<T: serde::Serialize>(
    cache: &CacheStore,
    collection: Collection,
    ttl: i64,
    data: &T,
    count: u32,
) -> Result<u32, String> {
    cache.put(&collection.storage_key(), ttl, data)?;
    Ok(count)
}

/// Invalidate old sub-entries, then store price entries grouped by entity_id.
pub fn store_prices_split(
    cache: &CacheStore,
    entries: &[PriceEntry],
    collection: Collection,
    ttl: i64,
) -> Result<u32, String> {
    cache.invalidate_collection(collection);

    let mut groups: std::collections::HashMap<&str, Vec<PriceEntry>> =
        std::collections::HashMap::new();
    for entry in entries {
        groups.entry(&entry.entity_id).or_default().push(entry.clone());
    }

    let base_key = collection.storage_key();
    let total = entries.len() as u32;
    let mut errors = Vec::new();
    for (entity_id, group) in &groups {
        let key = format!("{}:{}", base_key, entity_id);
        if let Err(e) = cache.put(&key, ttl, group) {
            errors.push(format!("{}: {}", key, e));
        }
    }

    if errors.is_empty() {
        Ok(total)
    } else {
        Err(errors.join("; "))
    }
}

/// Store price entries grouped by terminal_id under a "{collection}_by_terminal:{tid}" key.
/// Used for lookups like "show me all prices at terminal X".
pub fn store_prices_by_terminal(
    cache: &CacheStore,
    entries: &[PriceEntry],
    collection: Collection,
    ttl: i64,
) -> Result<(), String> {
    let mut groups: std::collections::HashMap<&str, Vec<PriceEntry>> =
        std::collections::HashMap::new();
    for entry in entries {
        if !entry.terminal_id.is_empty() {
            groups.entry(&entry.terminal_id).or_default().push(entry.clone());
        }
    }

    let base_key = format!("{}_by_terminal", collection.storage_key());
    let mut errors = Vec::new();
    for (terminal_id, group) in &groups {
        let key = format!("{}:{}", base_key, terminal_id);
        if let Err(e) = cache.put(&key, ttl, group) {
            errors.push(format!("{}: {}", key, e));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

/// Store entity infos as per-entity sub-keys (e.g. `entity_info:commodity:6`).
pub fn store_entity_infos(
    cache: &CacheStore,
    infos: &[EntityInfo],
    ttl: i64,
) -> Result<u32, String> {
    let base_key = Collection::EntityInfo.storage_key();
    let mut errors = Vec::new();
    for info in infos {
        let key = format!("{}:{}:{}", base_key, info.kind, info.id);
        if let Err(e) = cache.put(&key, ttl, info) {
            errors.push(format!("{}: {}", key, e));
        }
    }
    if errors.is_empty() {
        Ok(infos.len() as u32)
    } else {
        Err(errors.join("; "))
    }
}

/// Read entity IDs from a catalog collection in cache.
pub fn catalog_ids_from_cache(cache: &CacheStore, collection: Collection) -> Vec<String> {
    let key = collection.storage_key();
    match cache.get::<Vec<UexResult>>(&key) {
        CacheResult::Fresh(items) | CacheResult::Stale(items) => {
            items.into_iter().map(|i| i.id).collect()
        }
        CacheResult::Missing => vec![],
    }
}

/// Enrich PriceEntry location fields from the terminal hierarchy cache.
/// Sets orbit to the most specific parent (station > city > outpost > orbit),
/// location to planet_name, and system to system_name.
pub fn enrich_locations_from_hierarchy(cache: &CacheStore, entries: &mut [PriceEntry]) {
    use crate::providers::locations::dto::TerminalHierarchy;
    use crate::providers::locations::TERMINAL_HIERARCHY_KEY;

    let hierarchy: Vec<TerminalHierarchy> = match cache.get(TERMINAL_HIERARCHY_KEY) {
        CacheResult::Fresh(h) | CacheResult::Stale(h) => h,
        CacheResult::Missing => return,
    };
    let map: std::collections::HashMap<&str, &TerminalHierarchy> =
        hierarchy.iter().map(|t| (t.id.as_str(), t)).collect();

    for entry in entries.iter_mut() {
        if let Some(th) = map.get(entry.terminal_id.as_str()) {
            if !th.location_name.is_empty() {
                entry.orbit = th.location_name.clone();
            } else if !th.orbit_name.is_empty() {
                entry.orbit = th.orbit_name.clone();
            }
            if !th.planet_name.is_empty() {
                entry.location = th.planet_name.clone();
            }
            if !th.system_name.is_empty() {
                entry.system = th.system_name.clone();
            }
        }
    }
}

/// Search a pre-fetched collection by name substring match.
pub fn search_in_collection(collection: &[UexResult], query: &str) -> Vec<UexResult> {
    let query_lower = query.to_lowercase();
    collection
        .iter()
        .filter(|r| r.name.to_lowercase().contains(&query_lower))
        .cloned()
        .collect()
}
