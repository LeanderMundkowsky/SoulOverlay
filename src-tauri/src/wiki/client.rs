use super::dto::{
    WikiApiResponse, WikiManufacturerListItem, WikiSearchResponse, WikiItemDto, WikiVehicleDto,
};

const WIKI_BASE_URL: &str = "https://api.star-citizen.wiki";

fn user_agent() -> &'static str {
    concat!("SoulOverlay/", env!("CARGO_PKG_VERSION"))
}

/// Fetch a single item by UUID (accepts both UEX and Wiki UUIDs).
pub async fn get_item(
    client: &reqwest::Client,
    uuid: &str,
) -> Result<WikiItemDto, String> {
    let url = format!("{}/api/items/{}", WIKI_BASE_URL, uuid);
    let resp: WikiApiResponse<WikiItemDto> = client
        .get(&url)
        .header("User-Agent", user_agent())
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
        .header("User-Agent", user_agent())
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
        .header("User-Agent", user_agent())
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
        .header("User-Agent", user_agent())
        .send()
        .await
        .map_err(|e| format!("Wiki API request failed: {}", e))?
        .json::<WikiSearchResponse<WikiVehicleDto>>()
        .await
        .map_err(|e| format!("Wiki API parse failed: {}", e))?;
    Ok(resp)
}

/// Fetch a page of vehicles (paginated). Returns data + pagination meta.
pub async fn list_vehicles(
    client: &reqwest::Client,
    page: u32,
    per_page: u32,
) -> Result<WikiSearchResponse<WikiVehicleDto>, String> {
    let url = format!("{}/api/vehicles", WIKI_BASE_URL);
    let resp = client
        .get(&url)
        .query(&[
            ("page[number]", &page.to_string()),
            ("page[size]", &per_page.to_string()),
        ])
        .header("User-Agent", user_agent())
        .send()
        .await
        .map_err(|e| format!("Wiki API request failed: {}", e))?
        .json::<WikiSearchResponse<WikiVehicleDto>>()
        .await
        .map_err(|e| format!("Wiki API parse failed: {}", e))?;
    Ok(resp)
}

/// Fetch ALL vehicles across all pages.
pub async fn list_all_vehicles(
    client: &reqwest::Client,
) -> Result<Vec<WikiVehicleDto>, String> {
    let per_page = 100u32;
    let first_page = list_vehicles(client, 1, per_page).await?;
    let total_pages = first_page.meta.last_page;
    let mut all = first_page.data;

    if total_pages > 1 {
        let handles: Vec<_> = (2..=total_pages)
            .map(|page| {
                let c = client.clone();
                tokio::spawn(async move { list_vehicles(&c, page, per_page).await })
            })
            .collect();
        for handle in handles {
            match handle.await {
                Ok(Ok(resp)) => all.extend(resp.data),
                Ok(Err(e)) => log::warn!("Failed to fetch vehicle page: {}", e),
                Err(e) => log::warn!("Vehicle page fetch task panicked: {}", e),
            }
        }
    }

    log::info!("Wiki: fetched {} vehicles across {} pages", all.len(), total_pages);
    Ok(all)
}

/// Fetch ALL manufacturers (small dataset, typically ≤ 2 pages).
pub async fn list_all_manufacturers(
    client: &reqwest::Client,
) -> Result<Vec<WikiManufacturerListItem>, String> {
    let per_page = 100u32;
    let url = format!("{}/api/manufacturers", WIKI_BASE_URL);

    let first_page: WikiSearchResponse<WikiManufacturerListItem> = client
        .get(&url)
        .query(&[
            ("page[number]", "1"),
            ("page[size]", &per_page.to_string()),
        ])
        .header("User-Agent", user_agent())
        .send()
        .await
        .map_err(|e| format!("Wiki API request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Wiki API parse failed: {}", e))?;

    let total_pages = first_page.meta.last_page;
    let mut all = first_page.data;

    for page in 2..=total_pages {
        let resp: WikiSearchResponse<WikiManufacturerListItem> = client
            .get(&url)
            .query(&[
                ("page[number]", &page.to_string()),
                ("page[size]", &per_page.to_string()),
            ])
            .header("User-Agent", user_agent())
            .send()
            .await
            .map_err(|e| format!("Wiki API request failed: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Wiki API parse failed: {}", e))?;
        all.extend(resp.data);
    }

    log::info!("Wiki: fetched {} manufacturers across {} pages", all.len(), total_pages);
    Ok(all)
}
