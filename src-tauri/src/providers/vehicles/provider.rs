use async_trait::async_trait;

use super::dto::{VehicleDto, VehiclePurchasePriceDto, VehicleRentalPriceDto};
use crate::cache_store::Collection;
use crate::providers::{
    catalog_ids_from_cache, enrich_locations_from_hierarchy, store_blob,
    store_prices_by_terminal, store_prices_split,
    BlobProvider, PerEntityProvider, RefreshContext,
};
use crate::uex::types::{EntityInfo, PriceEntry, UexResult};
use crate::uex::UexClient;

// ── Catalog provider ───────────────────────────────────────────────────────

pub struct VehiclesCatalog;

#[async_trait]
impl BlobProvider for VehiclesCatalog {
    fn collection(&self) -> Collection { Collection::Vehicles }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let dtos: Vec<VehicleDto> = ctx.client.get("/vehicles", &[], ctx.api_key).await?;
        let data: Vec<UexResult> = dtos.iter().map(UexResult::from).collect();
        let count = data.len() as u32;
        let ttl = self.collection().ttl_for(ctx.settings);
        store_blob(ctx.cache, self.collection(), ttl, &data, count)
    }
}

// ── Vehicle purchase prices provider ───────────────────────────────────────

pub struct VehiclePurchasePrices;

#[async_trait]
impl PerEntityProvider for VehiclePurchasePrices {
    fn collection(&self) -> Collection { Collection::VehiclePurchasePrices }
    fn depends_on(&self) -> &[Collection] { &[Collection::Vehicles, Collection::Locations] }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let ids = catalog_ids_from_cache(ctx.cache, Collection::Vehicles);
        if ids.is_empty() {
            return Err("Vehicles not in cache; refresh vehicles first".to_string());
        }
        let mut data = fetch_all_vehicle_purchase_prices_per_entity(ctx.client, &ids, ctx.api_key).await;
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
        let ids = catalog_ids_from_cache(ctx.cache, Collection::Vehicles);
        if ids.is_empty() {
            return Err("Vehicles not in cache; refresh vehicles first".to_string());
        }
        let mut data = fetch_all_vehicle_rental_prices_per_entity(ctx.client, &ids, ctx.api_key).await;
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
) -> Result<Vec<EntityInfo>, String> {
    let dtos: Vec<VehicleDto> = client.get("/vehicles", &[], api_key).await?;
    Ok(dtos.iter().map(EntityInfo::from).collect())
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
