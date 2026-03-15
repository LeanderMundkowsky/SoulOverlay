use async_trait::async_trait;

use super::dto::{CommodityDto, CommodityPriceDto};
use crate::cache_store::Collection;
use crate::providers::{
    catalog_ids_from_cache, enrich_locations_from_hierarchy, store_blob,
    store_prices_by_terminal, store_prices_split,
    BlobProvider, PerEntityProvider, RefreshContext,
};
use crate::uex::types::{PriceEntry, UexResult};
use crate::uex::UexClient;

// ── Catalog provider ───────────────────────────────────────────────────────

pub struct CommoditiesCatalog;

#[async_trait]
impl BlobProvider for CommoditiesCatalog {
    fn collection(&self) -> Collection { Collection::Commodities }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let dtos: Vec<CommodityDto> = ctx.client.get("/commodities", &[], ctx.api_key).await?;
        let data: Vec<UexResult> = dtos.iter().map(UexResult::from).collect();
        let count = data.len() as u32;
        let ttl = self.collection().ttl_for(ctx.settings);
        store_blob(ctx.cache, self.collection(), ttl, &data, count)
    }
}

// ── Commodity prices provider ──────────────────────────────────────────────

pub struct CommodityPrices;

#[async_trait]
impl PerEntityProvider for CommodityPrices {
    fn collection(&self) -> Collection { Collection::CommodityPrices }
    fn depends_on(&self) -> &[Collection] { &[Collection::Commodities, Collection::Locations] }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let ids = catalog_ids_from_cache(ctx.cache, Collection::Commodities);
        if ids.is_empty() {
            return Err("Commodities not in cache; refresh commodities first".to_string());
        }
        let mut data = fetch_all_commodity_prices_per_entity(ctx.client, &ids, ctx.api_key).await;
        enrich_locations_from_hierarchy(ctx.cache, &mut data);
        let ttl = self.collection().ttl_for(ctx.settings);
        let count = store_prices_split(ctx.cache, &data, self.collection(), ttl)?;
        if let Err(e) = store_prices_by_terminal(ctx.cache, &data, self.collection(), ttl) {
            log::warn!("Failed to store commodity prices by terminal: {}", e);
        }
        Ok(count)
    }
}

// ── Raw commodity prices provider ──────────────────────────────────────────

pub struct RawCommodityPrices;

#[async_trait]
impl PerEntityProvider for RawCommodityPrices {
    fn collection(&self) -> Collection { Collection::RawCommodityPrices }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let dtos: Vec<CommodityPriceDto> = ctx.client
            .get("/commodities_raw_prices_all", &[], ctx.api_key)
            .await?;
        let mut data: Vec<PriceEntry> = dtos.iter().map(|d| d.to_price_entry("raw_commodity")).collect();
        enrich_locations_from_hierarchy(ctx.cache, &mut data);
        let ttl = self.collection().ttl_for(ctx.settings);
        let count = store_prices_split(ctx.cache, &data, self.collection(), ttl)?;
        if let Err(e) = store_prices_by_terminal(ctx.cache, &data, self.collection(), ttl) {
            log::warn!("Failed to store raw commodity prices by terminal: {}", e);
        }
        Ok(count)
    }
}

// ── Standalone functions (used by command fallbacks) ────────────────────────

/// Search commodities via direct API call (cache-miss fallback).
pub async fn search_commodities(
    client: &UexClient,
    query: &str,
    api_key: &str,
) -> Result<Vec<UexResult>, String> {
    let dtos: Vec<CommodityDto> = client
        .get("/commodities", &[("name_filter", query)], api_key)
        .await?;
    let query_lower = query.to_lowercase();
    Ok(dtos
        .iter()
        .map(UexResult::from)
        .filter(|r| r.name.to_lowercase().contains(&query_lower))
        .collect())
}

/// Fetch prices for a single commodity (used by legacy uex_prices command).
pub async fn get_commodity_prices(
    client: &UexClient,
    commodity_id: &str,
    api_key: &str,
) -> Result<Vec<PriceEntry>, String> {
    let dtos: Vec<CommodityPriceDto> = client
        .get("/commodities_prices", &[("id_commodity", commodity_id)], api_key)
        .await?;
    Ok(dtos.iter().map(|d| d.to_price_entry("commodity")).collect())
}

/// Fetch commodity prices per entity in parallel.
async fn fetch_all_commodity_prices_per_entity(
    client: &UexClient,
    commodity_ids: &[String],
    api_key: &str,
) -> Vec<PriceEntry> {
    let handles: Vec<_> = commodity_ids
        .iter()
        .map(|id| {
            let client = client.clone();
            let id = id.clone();
            let key = api_key.to_string();
            tokio::spawn(async move { get_commodity_prices(&client, &id, &key).await })
        })
        .collect();

    let mut all = Vec::new();
    for handle in handles {
        if let Ok(Ok(prices)) = handle.await {
            all.extend(prices);
        }
    }
    all
}

/// Fetch all commodity EntityInfo from UEX.
pub async fn fetch_all_commodity_infos(
    client: &UexClient,
    api_key: &str,
) -> Result<Vec<crate::uex::types::EntityInfo>, String> {
    let dtos: Vec<CommodityDto> = client.get("/commodities", &[], api_key).await?;
    Ok(dtos.iter().map(crate::uex::types::EntityInfo::from).collect())
}
