use async_trait::async_trait;

use super::dto::FleetVehicleDto;
use crate::cache_store::Collection;
use crate::commands::hangar::HangarVehicle;
use crate::providers::{store_blob, BlobProvider, RefreshContext};
use crate::uex::UexClient;

pub struct FleetProvider;

#[async_trait]
impl BlobProvider for FleetProvider {
    fn collection(&self) -> Collection { Collection::Fleet }
    fn requires_secret(&self) -> bool { true }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let secret = ctx.secret_key.ok_or("Secret key required for fleet")?;
        let dtos: Vec<FleetVehicleDto> = ctx.client
            .get_with_secret("/fleet", &[], ctx.api_key, secret)
            .await?;
        let data: Vec<HangarVehicle> = dtos.iter().map(HangarVehicle::from).collect();
        let count = data.len() as u32;
        let ttl = self.collection().ttl_for(ctx.settings);
        store_blob(ctx.cache, self.collection(), ttl, &data, count)
    }
}

/// Fetch the authenticated user's fleet (for direct use by commands).
pub async fn fetch_fleet(
    client: &UexClient,
    api_key: &str,
    secret_key: &str,
) -> Result<Vec<HangarVehicle>, String> {
    let dtos: Vec<FleetVehicleDto> = client
        .get_with_secret("/fleet", &[], api_key, secret_key)
        .await?;
    Ok(dtos.iter().map(HangarVehicle::from).collect())
}
