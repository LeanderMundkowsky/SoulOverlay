use async_trait::async_trait;

use crate::cache_store::Collection;
use crate::providers::commodities::provider::fetch_all_commodity_infos;
use crate::providers::items::provider::fetch_all_item_infos;
use crate::providers::vehicles::provider::fetch_all_vehicle_infos;
use crate::providers::{store_entity_infos, PerEntityProvider, RefreshContext};
use crate::uex::types::EntityInfo;

pub struct EntityInfoProvider;

#[async_trait]
impl PerEntityProvider for EntityInfoProvider {
    fn collection(&self) -> Collection { Collection::EntityInfo }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let mut all_infos: Vec<EntityInfo> = Vec::new();

        match fetch_all_commodity_infos(ctx.client, ctx.api_key).await {
            Ok(infos) => {
                log::info!("Fetched {} commodity infos for entity_info cache", infos.len());
                all_infos.extend(infos);
            }
            Err(e) => log::warn!("Failed to fetch commodity infos: {}", e),
        }

        match fetch_all_vehicle_infos(ctx.client, ctx.api_key).await {
            Ok(infos) => {
                log::info!("Fetched {} vehicle infos for entity_info cache", infos.len());
                all_infos.extend(infos);
            }
            Err(e) => log::warn!("Failed to fetch vehicle infos: {}", e),
        }

        match fetch_all_item_infos(ctx.client, ctx.api_key).await {
            Ok(infos) => {
                log::info!("Fetched {} item infos for entity_info cache", infos.len());
                all_infos.extend(infos);
            }
            Err(e) => log::warn!("Failed to fetch item infos: {}", e),
        }

        let ttl = self.collection().ttl_for(ctx.settings);
        store_entity_infos(ctx.cache, &all_infos, ttl)
    }
}
