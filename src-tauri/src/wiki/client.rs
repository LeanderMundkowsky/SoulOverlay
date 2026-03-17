use super::dto::{WikiApiResponse, WikiSearchResponse, WikiItemDto, WikiVehicleDto};

const WIKI_BASE_URL: &str = "https://api.star-citizen.wiki";

/// Fetch a single item by UUID (accepts both UEX and Wiki UUIDs).
pub async fn get_item(
    client: &reqwest::Client,
    uuid: &str,
) -> Result<WikiItemDto, String> {
    let url = format!("{}/api/items/{}", WIKI_BASE_URL, uuid);
    let resp: WikiApiResponse<WikiItemDto> = client
        .get(&url)
        .header("User-Agent", concat!("SoulOverlay/", env!("CARGO_PKG_VERSION")))
        .send()
        .await
        .map_err(|e| format!("Wiki API request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Wiki API parse failed: {}", e))?;
    Ok(resp.data)
}

/// Fetch a single vehicle by UUID.
pub async fn get_vehicle(
    client: &reqwest::Client,
    uuid: &str,
) -> Result<WikiVehicleDto, String> {
    let url = format!("{}/api/vehicles/{}", WIKI_BASE_URL, uuid);
    let resp: WikiApiResponse<WikiVehicleDto> = client
        .get(&url)
        .header("User-Agent", concat!("SoulOverlay/", env!("CARGO_PKG_VERSION")))
        .send()
        .await
        .map_err(|e| format!("Wiki API request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Wiki API parse failed: {}", e))?;
    Ok(resp.data)
}

/// Search items by name (case-insensitive substring match).
pub async fn search_items(
    client: &reqwest::Client,
    query: &str,
    limit: u32,
) -> Result<WikiSearchResponse<WikiItemDto>, String> {
    let url = format!("{}/api/items", WIKI_BASE_URL);
    let resp = client
        .get(&url)
        .query(&[
            ("filter[name]", query),
            ("page[size]", &limit.to_string()),
        ])
        .header("User-Agent", concat!("SoulOverlay/", env!("CARGO_PKG_VERSION")))
        .send()
        .await
        .map_err(|e| format!("Wiki API request failed: {}", e))?
        .json::<WikiSearchResponse<WikiItemDto>>()
        .await
        .map_err(|e| format!("Wiki API parse failed: {}", e))?;
    Ok(resp)
}

/// Search vehicles by name (case-insensitive substring match).
pub async fn search_vehicles(
    client: &reqwest::Client,
    query: &str,
    limit: u32,
) -> Result<WikiSearchResponse<WikiVehicleDto>, String> {
    let url = format!("{}/api/vehicles", WIKI_BASE_URL);
    let resp = client
        .get(&url)
        .query(&[
            ("filter[name]", query),
            ("page[size]", &limit.to_string()),
        ])
        .header("User-Agent", concat!("SoulOverlay/", env!("CARGO_PKG_VERSION")))
        .send()
        .await
        .map_err(|e| format!("Wiki API request failed: {}", e))?
        .json::<WikiSearchResponse<WikiVehicleDto>>()
        .await
        .map_err(|e| format!("Wiki API parse failed: {}", e))?;
    Ok(resp)
}
