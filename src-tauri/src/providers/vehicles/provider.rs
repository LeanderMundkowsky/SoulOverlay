use async_trait::async_trait;

use super::dto::{VehicleDto, VehiclePurchasePriceDto, VehicleRentalPriceDto};
use crate::cache_store::Collection;
use crate::providers::{
    enrich_locations_from_hierarchy, store_blob,
    store_prices_by_terminal, store_prices_split,
    BlobProvider, PerEntityProvider, RefreshContext,
};
use crate::uex::types::{PriceEntry, UexResult};
use crate::uex::UexClient;
use crate::wiki::client;
use crate::wiki::dto::WikiVehicleDto;

/// Cache key for persisted EntityMapper snapshot.
pub const MAPPER_CACHE_KEY: &str = "entity_mapper_snapshot";

// ── Catalog provider ───────────────────────────────────────────────────────

pub struct VehiclesCatalog;

#[async_trait]
impl BlobProvider for VehiclesCatalog {
    fn collection(&self) -> Collection { Collection::Vehicles }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        // 1. Fetch full vehicle catalog from Wiki API
        let http = ctx.client.client();
        let wiki_vehicles = client::list_all_vehicles(http).await?;

        // 2. Convert Wiki vehicles to UexResult (uuid as primary ID)
        let data: Vec<UexResult> = wiki_vehicles.iter().map(wiki_vehicle_to_result).collect();
        let count = data.len() as u32;

        // 3. Fetch UEX vehicle list for EntityMapper (name → UEX ID mapping)
        let uex_dtos: Vec<VehicleDto> = ctx.client.get("/vehicles", &[], ctx.api_key).await
            .unwrap_or_else(|e| {
                log::warn!("Failed to fetch UEX vehicles for mapper: {}", e);
                vec![]
            });

        // 4. Build EntityMapper
        let wiki_pairs: Vec<(String, String)> = wiki_vehicles.iter()
            .filter_map(|v| {
                let uuid = v.uuid.as_deref()?.to_string();
                let name = v.name.as_deref()?.to_string();
                Some((uuid, name))
            })
            .collect();
        let uex_pairs: Vec<(String, String)> = uex_dtos.iter()
            .map(|v| (v.id.clone(), v.name.clone()))
            .collect();

        let ttl = self.collection().ttl_for(ctx.settings);
        if let Ok(mut mapper) = ctx.entity_mapper.lock() {
            mapper.set_vehicle_maps(&wiki_pairs, &uex_pairs);
            // Persist mapper to cache so it survives app restarts
            let snap = mapper.snapshot();
            let _ = ctx.cache.put(MAPPER_CACHE_KEY, ttl, &snap);
        }

        // 5. Store catalog blob
        store_blob(ctx.cache, self.collection(), ttl, &data, count)
    }
}

/// Convert a Wiki vehicle DTO to a search result entry.
fn wiki_vehicle_to_result(dto: &WikiVehicleDto) -> UexResult {
    let uuid = dto.uuid.clone().unwrap_or_default();
    let name = dto.name.clone().unwrap_or_default();
    let kind = match dto.vehicle_type.as_deref() {
        Some("vehicle") => "ground vehicle",
        Some("gravlev") => "ground vehicle",
        _ => "vehicle",
    };
    UexResult {
        id: uuid.clone(),
        name,
        kind: kind.to_string(),
        slug: dto.slug.clone().unwrap_or_default(),
        uuid,
        source: "wiki".to_string(),
    }
}

// ── Vehicle purchase prices provider ───────────────────────────────────────

pub struct VehiclePurchasePrices;

#[async_trait]
impl PerEntityProvider for VehiclePurchasePrices {
    fn collection(&self) -> Collection { Collection::VehiclePurchasePrices }
    fn depends_on(&self) -> &[Collection] { &[Collection::Vehicles, Collection::Locations] }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        // Get UEX IDs from EntityMapper (populated during vehicle catalog refresh)
        let uex_ids = ctx.entity_mapper.lock()
            .map_err(|e| format!("EntityMapper lock failed: {}", e))?
            .all_mapped_vehicle_uex_ids();
        if uex_ids.is_empty() {
            return Err("No vehicle UEX ID mappings; refresh vehicles first".to_string());
        }
        let mut data = fetch_all_vehicle_purchase_prices_per_entity(ctx.client, &uex_ids, ctx.api_key).await;

        // Remap entity_id from UEX ID → Wiki UUID so cache keys match catalog IDs
        if let Ok(mapper) = ctx.entity_mapper.lock() {
            for entry in &mut data {
                if let Some(uuid) = mapper.vehicle_uex_id_to_uuid(&entry.entity_id) {
                    entry.entity_id = uuid.to_string();
                }
            }
        }

        enrich_locations_from_hierarchy(ctx.cache, &mut data);
        let ttl = self.collection().ttl_for(ctx.settings);
        let count = store_prices_split(ctx.cache, &data, self.collection(), ttl)?;
        if let Err(e) = store_prices_by_terminal(ctx.cache, &data, self.collection(), ttl) {
            log::warn!("Failed to store vehicle purchase prices by terminal: {}", e);
        }
        Ok(count)
    }
}

// ── Vehicle rental prices provider ─────────────────────────────────────────

pub struct VehicleRentalPrices;

#[async_trait]
impl PerEntityProvider for VehicleRentalPrices {
    fn collection(&self) -> Collection { Collection::VehicleRentalPrices }
    fn depends_on(&self) -> &[Collection] { &[Collection::Vehicles, Collection::Locations] }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let uex_ids = ctx.entity_mapper.lock()
            .map_err(|e| format!("EntityMapper lock failed: {}", e))?
            .all_mapped_vehicle_uex_ids();
        if uex_ids.is_empty() {
            return Err("No vehicle UEX ID mappings; refresh vehicles first".to_string());
        }
        let mut data = fetch_all_vehicle_rental_prices_per_entity(ctx.client, &uex_ids, ctx.api_key).await;

        // Remap entity_id from UEX ID → Wiki UUID so cache keys match catalog IDs
        if let Ok(mapper) = ctx.entity_mapper.lock() {
            for entry in &mut data {
                if let Some(uuid) = mapper.vehicle_uex_id_to_uuid(&entry.entity_id) {
                    entry.entity_id = uuid.to_string();
                }
            }
        }

        enrich_locations_from_hierarchy(ctx.cache, &mut data);
        let ttl = self.collection().ttl_for(ctx.settings);
        let count = store_prices_split(ctx.cache, &data, self.collection(), ttl)?;
        if let Err(e) = store_prices_by_terminal(ctx.cache, &data, self.collection(), ttl) {
            log::warn!("Failed to store vehicle rental prices by terminal: {}", e);
        }
        Ok(count)
    }
}

// ── Standalone functions ───────────────────────────────────────────────────

pub async fn search_vehicles(
    client: &UexClient,
    query: &str,
    api_key: &str,
) -> Result<Vec<UexResult>, String> {
    let dtos: Vec<VehicleDto> = client.get("/vehicles", &[], api_key).await?;
    let query_lower = query.to_lowercase();
    Ok(dtos.iter().map(UexResult::from).filter(|r| r.name.to_lowercase().contains(&query_lower)).collect())
}

#[allow(dead_code)]
pub async fn fetch_vehicle_photo_map(
    client: &UexClient,
    api_key: &str,
) -> Result<std::collections::HashMap<String, String>, String> {
    let dtos: Vec<VehicleDto> = client.get("/vehicles", &[], api_key).await?;
    let mut map = std::collections::HashMap::new();
    for dto in &dtos {
        if let Some(ref url) = dto.url_photo {
            map.insert(dto.id.clone(), url.clone());
        }
    }
    Ok(map)
}

pub async fn fetch_all_vehicle_infos(
    client: &UexClient,
    api_key: &str,
) -> Result<Vec<crate::uex::types::EntityInfo>, String> {
    let dtos: Vec<VehicleDto> = client.get("/vehicles", &[], api_key).await?;
    Ok(dtos.iter().map(crate::uex::types::EntityInfo::from).collect())
}

async fn get_vehicle_purchase_prices(
    client: &UexClient,
    vehicle_id: &str,
    api_key: &str,
) -> Result<Vec<PriceEntry>, String> {
    let dtos: Vec<VehiclePurchasePriceDto> = client
        .get("/vehicles_purchases_prices", &[("id_vehicle", vehicle_id)], api_key)
        .await?;
    Ok(dtos.iter().map(PriceEntry::from).collect())
}

async fn get_vehicle_rental_prices(
    client: &UexClient,
    vehicle_id: &str,
    api_key: &str,
) -> Result<Vec<PriceEntry>, String> {
    let dtos: Vec<VehicleRentalPriceDto> = client
        .get("/vehicles_rentals_prices", &[("id_vehicle", vehicle_id)], api_key)
        .await?;
    Ok(dtos.iter().map(PriceEntry::from).collect())
}

async fn fetch_all_vehicle_purchase_prices_per_entity(
    client: &UexClient,
    vehicle_ids: &[String],
    api_key: &str,
) -> Vec<PriceEntry> {
    let handles: Vec<_> = vehicle_ids.iter().map(|id| {
        let client = client.clone();
        let id = id.clone();
        let key = api_key.to_string();
        tokio::spawn(async move { get_vehicle_purchase_prices(&client, &id, &key).await })
    }).collect();
    let mut all = Vec::new();
    for handle in handles {
        if let Ok(Ok(prices)) = handle.await { all.extend(prices); }
    }
    all
}

async fn fetch_all_vehicle_rental_prices_per_entity(
    client: &UexClient,
    vehicle_ids: &[String],
    api_key: &str,
) -> Vec<PriceEntry> {
    let handles: Vec<_> = vehicle_ids.iter().map(|id| {
        let client = client.clone();
        let id = id.clone();
        let key = api_key.to_string();
        tokio::spawn(async move { get_vehicle_rental_prices(&client, &id, &key).await })
    }).collect();
    let mut all = Vec::new();
    for handle in handles {
        if let Ok(Ok(prices)) = handle.await { all.extend(prices); }
    }
    all
}
