use async_trait::async_trait;

use super::dto::TerminalDto;
use crate::cache_store::Collection;
use crate::providers::{store_blob, BlobProvider, RefreshContext};
use crate::uex::types::UexResult;
use crate::uex::UexClient;

pub struct LocationsCatalog;

#[async_trait]
impl BlobProvider for LocationsCatalog {
    fn collection(&self) -> Collection { Collection::Locations }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let dtos: Vec<TerminalDto> = ctx.client.get("/terminals", &[], ctx.api_key).await?;
        let data: Vec<UexResult> = dtos.iter().map(UexResult::from).collect();
        let count = data.len() as u32;
        let ttl = self.collection().ttl_for(ctx.settings);
        store_blob(ctx.cache, self.collection(), ttl, &data, count)
    }
}

pub async fn search_locations(
    client: &UexClient,
    query: &str,
    api_key: &str,
) -> Result<Vec<UexResult>, String> {
    let dtos: Vec<TerminalDto> = client.get("/terminals", &[], api_key).await?;
    let query_lower = query.to_lowercase();
    Ok(dtos.iter().map(UexResult::from).filter(|r| r.name.to_lowercase().contains(&query_lower)).collect())
}
