use async_trait::async_trait;

use super::dto::{CategoryDto, ItemDto, ItemPriceDto};
use crate::cache_store::Collection;
use crate::providers::{
    search_in_collection, store_blob, store_prices_split, BlobProvider, PerEntityProvider,
    RefreshContext,
};
use crate::uex::types::{EntityInfo, PriceEntry, UexResult};
use crate::uex::UexClient;

// ── Catalog provider ───────────────────────────────────────────────────────

pub struct ItemsCatalog;

#[async_trait]
impl BlobProvider for ItemsCatalog {
    fn collection(&self) -> Collection { Collection::Items }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let data = fetch_all_items_inner(ctx.client, ctx.api_key).await?;
        let count = data.len() as u32;
        let ttl = self.collection().ttl_for(ctx.settings);
        store_blob(ctx.cache, self.collection(), ttl, &data, count)
    }
}

// ── Item prices provider ───────────────────────────────────────────────────

pub struct ItemPrices;

#[async_trait]
impl PerEntityProvider for ItemPrices {
    fn collection(&self) -> Collection { Collection::ItemPrices }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let dtos: Vec<ItemPriceDto> = ctx.client.get("/items_prices_all", &[], ctx.api_key).await?;
        let data: Vec<PriceEntry> = dtos.iter().map(PriceEntry::from).collect();
        let ttl = self.collection().ttl_for(ctx.settings);
        store_prices_split(ctx.cache, &data, self.collection(), ttl)
    }
}

// ── Standalone functions ───────────────────────────────────────────────────

/// Search items (full fetch + client-side filter, since /items doesn't support name search).
pub async fn search_items(
    client: &UexClient,
    _query: &str,
    api_key: &str,
) -> Result<Vec<UexResult>, String> {
    let all = fetch_all_items_inner(client, api_key).await?;
    Ok(search_in_collection(&all, _query))
}

/// Fetch all item EntityInfo via category fan-out.
pub async fn fetch_all_item_infos(
    client: &UexClient,
    api_key: &str,
) -> Result<Vec<EntityInfo>, String> {
    let categories: Vec<CategoryDto> = client.get("/categories", &[], api_key).await?;
    let category_ids: Vec<String> = categories
        .into_iter()
        .filter(|c| c.category_type.as_deref() == Some("item"))
        .map(|c| c.id)
        .collect();

    let handles: Vec<_> = category_ids
        .into_iter()
        .map(|cat_id| {
            let client = client.clone();
            let key = api_key.to_string();
            tokio::spawn(async move {
                let dtos: Result<Vec<ItemDto>, String> =
                    client.get("/items", &[("id_category", &cat_id)], &key).await;
                dtos
            })
        })
        .collect();

    let mut seen_ids = std::collections::HashSet::<String>::new();
    let mut all_infos: Vec<EntityInfo> = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(Ok(dtos)) => {
                for dto in &dtos {
                    if seen_ids.insert(dto.id.clone()) {
                        all_infos.push(EntityInfo::from(dto));
                    }
                }
            }
            Ok(Err(e)) => log::warn!("Failed to fetch item infos for a category: {}", e),
            Err(e) => log::warn!("Item info fetch task panicked: {}", e),
        }
    }

    log::info!("Total item infos fetched: {}", all_infos.len());
    Ok(all_infos)
}

/// Internal: category fan-out to fetch all items as UexResult.
async fn fetch_all_items_inner(
    client: &UexClient,
    api_key: &str,
) -> Result<Vec<UexResult>, String> {
    let categories: Vec<CategoryDto> = client.get("/categories", &[], api_key).await?;
    let category_ids: Vec<String> = categories
        .into_iter()
        .filter(|c| c.category_type.as_deref() == Some("item"))
        .map(|c| c.id)
        .collect();

    log::info!("Fetching items across {} categories in parallel", category_ids.len());

    let handles: Vec<_> = category_ids
        .into_iter()
        .map(|cat_id| {
            let client = client.clone();
            let key = api_key.to_string();
            tokio::spawn(async move {
                let dtos: Result<Vec<ItemDto>, String> =
                    client.get("/items", &[("id_category", &cat_id)], &key).await;
                dtos
            })
        })
        .collect();

    let mut seen_ids = std::collections::HashSet::<String>::new();
    let mut all_items: Vec<UexResult> = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(Ok(dtos)) => {
                for dto in &dtos {
                    if seen_ids.insert(dto.id.clone()) {
                        all_items.push(UexResult::from(dto));
                    }
                }
            }
            Ok(Err(e)) => log::warn!("Failed to fetch items for a category: {}", e),
            Err(e) => log::warn!("Item fetch task panicked: {}", e),
        }
    }

    log::info!("Total items fetched: {}", all_items.len());
    Ok(all_items)
}
