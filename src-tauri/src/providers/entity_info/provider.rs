use async_trait::async_trait;

use crate::cache_store::Collection;
use crate::providers::commodities::provider::fetch_all_commodity_infos;
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
            Ok(mut infos) => {
                // Remap vehicle IDs from UEX → Wiki UUID so cache keys match catalog
                if let Ok(mapper) = ctx.entity_mapper.lock() {
                    for info in &mut infos {
                        if let Some(uuid) = mapper.vehicle_uex_id_to_uuid(&info.id) {
                            info.uuid = Some(uuid.to_string());
                            info.id = uuid.to_string();
                        }
                    }
                }
                log::info!("Fetched {} vehicle infos for entity_info cache", infos.len());
                all_infos.extend(infos);
            }
            Err(e) => log::warn!("Failed to fetch vehicle infos: {}", e),
        }

        // Items are no longer bulk-fetched (19K items too many).
        // Item entity info is fetched on-demand from Wiki API.

        let ttl = self.collection().ttl_for(ctx.settings);
        store_entity_infos(ctx.cache, &all_infos, ttl)
    }
}
