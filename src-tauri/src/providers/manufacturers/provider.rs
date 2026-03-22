use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::cache_store::Collection;
use crate::providers::{store_blob, BlobProvider, RefreshContext};
use crate::wiki::client;

/// Cached manufacturer entry (stored in the Manufacturers blob collection).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ManufacturerEntry {
    pub uuid: String,
    pub name: String,
    pub code: String,
}

pub struct ManufacturersCatalog;

#[async_trait]
impl BlobProvider for ManufacturersCatalog {
    fn collection(&self) -> Collection { Collection::Manufacturers }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let http = ctx.client.client();
        let dtos = client::list_all_manufacturers(http).await?;
        let data: Vec<ManufacturerEntry> = dtos
            .into_iter()
            .filter_map(|dto| {
                Some(ManufacturerEntry {
                    uuid: dto.uuid?,
                    name: dto.name.unwrap_or_default(),
                    code: dto.code.unwrap_or_default(),
                })
            })
            .collect();
        let count = data.len() as u32;
        let ttl = self.collection().ttl_for(ctx.settings);
        store_blob(ctx.cache, self.collection(), ttl, &data, count)
    }
}
