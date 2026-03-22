use async_trait::async_trait;

use super::dto::ItemPriceDto;
use crate::cache_store::{CacheResult, Collection};
use crate::providers::{
    enrich_locations_from_hierarchy, store_prices_by_terminal, store_prices_split,
    PerEntityProvider, RefreshContext,
};
use crate::uex::types::{EntityInfo, PriceEntry, UexResult};
use crate::wiki::client as wiki_client;
use crate::wiki::dto::WikiItemDto;

// ── Item prices provider ───────────────────────────────────────────────────

pub struct ItemPrices;

#[async_trait]
impl PerEntityProvider for ItemPrices {
    fn collection(&self) -> Collection { Collection::ItemPrices }
    fn depends_on(&self) -> &[Collection] { &[Collection::EntityInfo, Collection::Locations] }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let dtos: Vec<ItemPriceDto> = ctx.client.get("/items_prices_all", &[], ctx.api_key).await?;
        let mut data: Vec<PriceEntry> = dtos.iter().map(PriceEntry::from).collect();

        enrich_locations_from_hierarchy(ctx.cache, &mut data);

        // Enrich category from EntityInfo
        let info_base = Collection::EntityInfo.storage_key();
        for entry in &mut data {
            let key = format!("{}:item:{}", info_base, entry.entity_id);
            if let CacheResult::Fresh(info) | CacheResult::Stale(info) =
                ctx.cache.get::<EntityInfo>(&key)
            {
                if let Some(section) = info.section {
                    entry.category = section;
                }
            }
        }

        let ttl = self.collection().ttl_for(ctx.settings);
        let count = store_prices_split(ctx.cache, &data, self.collection(), ttl)?;
        if let Err(e) = store_prices_by_terminal(ctx.cache, &data, self.collection(), ttl) {
            log::warn!("Failed to store item prices by terminal: {}", e);
        }
        Ok(count)
    }
}

// ── Wiki-based item search ─────────────────────────────────────────────────

/// Search items via Wiki API (on-demand, not cached catalog).
pub async fn search_items_wiki(
    http_client: &reqwest::Client,
    query: &str,
    limit: u32,
) -> Result<Vec<UexResult>, String> {
    let resp = wiki_client::search_items(http_client, query, limit).await?;
    Ok(resp.data.iter().map(wiki_item_to_result).collect())
}

/// Convert a Wiki item DTO to a UexResult for unified search.
fn wiki_item_to_result(dto: &WikiItemDto) -> UexResult {
    let uuid = dto.uuid.clone().unwrap_or_default();
    UexResult {
        id: uuid.clone(),
        name: dto.name.clone().unwrap_or_default(),
        kind: "item".to_string(),
        slug: String::new(),
        uuid,
        source: "wiki".to_string(),
    }
}
