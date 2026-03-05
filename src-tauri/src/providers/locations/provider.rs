use async_trait::async_trait;

use super::dto::{
    CityDto, CompanyDto, FactionDto, LiveAvailable, MoonDto, OrbitDto, OutpostDto, PlanetDto,
    PoiDto, SpaceStationDto, StarSystemDto, TerminalDto,
};
use crate::cache_store::Collection;
use crate::providers::{store_blob, BlobProvider, RefreshContext};
use crate::uex::types::UexResult;
use crate::uex::UexClient;

pub struct LocationsCatalog;

/// Fetch a single location endpoint, filter to live-available entries, and convert to `Vec<UexResult>`.
/// Returns an empty vec on error (logged, never fails the whole refresh).
macro_rules! fetch_location_type {
    ($ctx:expr, $path:expr, $dto_type:ty) => {{
        match $ctx.client.get::<$dto_type>($path, &[], $ctx.api_key).await {
            Ok(dtos) => dtos.iter()
                .filter(|d| d.is_available_live())
                .map(UexResult::from)
                .collect::<Vec<_>>(),
            Err(e) => {
                log::warn!("Failed to fetch {}: {}", $path, e);
                Vec::new()
            }
        }
    }};
}

#[async_trait]
impl BlobProvider for LocationsCatalog {
    fn collection(&self) -> Collection { Collection::Locations }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        // Fetch all location types concurrently
        let (
            terminals,
            star_systems,
            planets,
            moons,
            orbits,
            space_stations,
            outposts,
            pois,
            cities,
            factions,
            companies,
        ) = tokio::join!(
            async { fetch_location_type!(ctx, "/terminals", TerminalDto) },
            async { fetch_location_type!(ctx, "/star_systems", StarSystemDto) },
            async { fetch_location_type!(ctx, "/planets", PlanetDto) },
            async { fetch_location_type!(ctx, "/moons", MoonDto) },
            async { fetch_location_type!(ctx, "/orbits", OrbitDto) },
            async { fetch_location_type!(ctx, "/space_stations", SpaceStationDto) },
            async { fetch_location_type!(ctx, "/outposts", OutpostDto) },
            async { fetch_location_type!(ctx, "/poi", PoiDto) },
            async { fetch_location_type!(ctx, "/cities", CityDto) },
            async { fetch_location_type!(ctx, "/factions", FactionDto) },
            async { fetch_location_type!(ctx, "/companies", CompanyDto) },
        );

        let mut data = Vec::with_capacity(
            terminals.len()
                + star_systems.len()
                + planets.len()
                + moons.len()
                + orbits.len()
                + space_stations.len()
                + outposts.len()
                + pois.len()
                + cities.len()
                + factions.len()
                + companies.len(),
        );

        data.extend(terminals);
        data.extend(star_systems);
        data.extend(planets);
        data.extend(moons);
        data.extend(orbits);
        data.extend(space_stations);
        data.extend(outposts);
        data.extend(pois);
        data.extend(cities);
        data.extend(factions);
        data.extend(companies);

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
    // Fallback: fetch only terminals (lightweight) for direct API search
    let dtos: Vec<TerminalDto> = client.get("/terminals", &[], api_key).await?;
    let query_lower = query.to_lowercase();
    Ok(dtos.iter().map(UexResult::from).filter(|r| r.name.to_lowercase().contains(&query_lower)).collect())
}
