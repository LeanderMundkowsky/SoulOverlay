use async_trait::async_trait;

use super::dto::FuelPriceDto;
use crate::cache_store::Collection;
use crate::providers::{store_prices_split, PerEntityProvider, RefreshContext};
use crate::uex::types::PriceEntry;

pub struct FuelPrices;

#[async_trait]
impl PerEntityProvider for FuelPrices {
    fn collection(&self) -> Collection { Collection::FuelPrices }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let dtos: Vec<FuelPriceDto> = ctx.client.get("/fuel_prices_all", &[], ctx.api_key).await?;
        let data: Vec<PriceEntry> = dtos.iter().map(PriceEntry::from).collect();
        let ttl = self.collection().ttl_for(ctx.settings);
        store_prices_split(ctx.cache, &data, self.collection(), ttl)
    }
}
