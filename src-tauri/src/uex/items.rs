use serde::Deserialize;

use super::client::UexClient;
use super::types::{
    deserialize_flexible_id, deserialize_nonempty_string, location_string, timestamp_string,
    EntityInfo, PriceEntry, UexResult,
};

// ── API DTOs ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct CategoryDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default, rename = "type")]
    pub category_type: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ItemDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub section: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub category: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub company_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub size: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub color: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub game_version: Option<String>,
}

impl From<&ItemDto> for UexResult {
    fn from(dto: &ItemDto) -> Self {
        Self {
            id: dto.id.clone(),
            name: dto.name.clone(),
            kind: "item".to_string(),
            slug: dto.slug.clone(),
            uuid: dto.uuid.clone().unwrap_or_default(),
        }
    }
}

impl From<&ItemDto> for EntityInfo {
    fn from(dto: &ItemDto) -> Self {
        Self {
            id: dto.id.clone(),
            name: dto.name.clone(),
            kind: "item".to_string(),
            slug: dto.slug.clone(),
            section: dto.section.clone(),
            category: dto.category.clone(),
            company_name: dto.company_name.clone(),
            size: dto.size.clone(),
            color: dto.color.clone(),
            game_version: dto.game_version.clone(),
            ..Default::default()
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct ItemPriceDto {
    #[serde(default, deserialize_with = "deserialize_flexible_id")]
    pub id_item: String,
    #[serde(default)]
    pub item_name: Option<String>,
    #[serde(default)]
    pub star_system_name: Option<String>,
    #[serde(default)]
    pub planet_name: Option<String>,
    #[serde(default)]
    pub orbit_name: Option<String>,
    #[serde(default)]
    pub faction_name: Option<String>,
    #[serde(default)]
    pub terminal_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_id")]
    pub id_terminal: String,
    #[serde(default)]
    pub price_buy: Option<f64>,
    #[serde(default)]
    pub price_sell: Option<f64>,
    #[serde(default)]
    pub date_modified: Option<serde_json::Value>,
    #[serde(default)]
    pub date_added: Option<serde_json::Value>,
}

impl From<&ItemPriceDto> for PriceEntry {
    fn from(dto: &ItemPriceDto) -> Self {
        Self {
            entity_id: dto.id_item.clone(),
            entity_name: dto.item_name.clone().unwrap_or_default(),
            price_type: "item".to_string(),
            location: location_string(&dto.star_system_name, &dto.planet_name),
            terminal: dto.terminal_name.clone().unwrap_or_else(|| "Unknown".to_string()),
            terminal_id: dto.id_terminal.clone(),
            buy_price: dto.price_buy.unwrap_or(0.0),
            sell_price: dto.price_sell.unwrap_or(0.0),
            rent_price: 0.0,
            scu_available: None,
            date_updated: timestamp_string(&dto.date_modified, &dto.date_added),
            orbit: dto.orbit_name.clone().unwrap_or_default(),
            system: dto.star_system_name.clone().unwrap_or_default(),
            faction: dto.faction_name.clone().unwrap_or_default(),
            scu_last: 0.0,
            scu_users: 0.0,
            scu_avg: 0.0,
            scu_min: 0.0,
            scu_max: 0.0,
            price_last: dto.price_buy.or(dto.price_sell).unwrap_or(0.0),
            price_users: 0.0,
            price_avg: 0.0,
            price_min: 0.0,
            price_max: 0.0,
            inventory_status: 0.0,
            inventory_status_avg: 0.0,
            container_sizes: String::new(),
            is_buy_location: dto.price_buy.unwrap_or(0.0) > 0.0,
        }
    }
}

// ── Public functions ───────────────────────────────────────────────────────

/// Fetch ALL items from UEX by iterating over all item categories in parallel.
///
/// The `/items` endpoint requires `id_category`, `id_company`, or `uuid` — there is no
/// "fetch all" variant. We first fetch `/categories` (type=item), then fan-out one request
/// per category and merge the results, deduplicating by item ID.
pub async fn fetch_all_items(
    client: &UexClient,
    api_key: &str,
) -> Result<Vec<UexResult>, String> {
    // Step 1: fetch all categories and collect item category IDs
    let categories: Vec<CategoryDto> = client.get("/categories", &[], api_key).await?;
    let category_ids: Vec<String> = categories
        .into_iter()
        .filter(|c| c.category_type.as_deref() == Some("item"))
        .map(|c| c.id)
        .collect();

    log::info!(
        "Fetching items across {} categories in parallel",
        category_ids.len()
    );

    // Step 2: spawn one task per category (UexClient is cheap to clone)
    let handles: Vec<_> = category_ids
        .into_iter()
        .map(|cat_id| {
            let client = client.clone();
            let key = api_key.to_string();
            tokio::spawn(async move {
                let dtos: Result<Vec<ItemDto>, String> = client
                    .get("/items", &[("id_category", &cat_id)], &key)
                    .await;
                dtos
            })
        })
        .collect();

    // Step 3: collect results, deduplicating by item ID
    let mut seen_ids = std::collections::HashSet::<String>::new();
    let mut all_items: Vec<UexResult> = Vec::new();

    for handle in handles {
        match handle.await {
            Ok(Ok(dtos)) => {
                for dto in &dtos {
                    if seen_ids.insert(dto.id.clone()) {
                        all_items.push(UexResult::from(dto));
                    }
                }
            }
            Ok(Err(e)) => log::warn!("Failed to fetch items for a category: {}", e),
            Err(e) => log::warn!("Item fetch task panicked: {}", e),
        }
    }

    log::info!("Total items fetched: {}", all_items.len());
    Ok(all_items)
}

/// Fetch ALL item EntityInfo from UEX by iterating over all item categories.
/// Same fan-out strategy as `fetch_all_items` but returns full EntityInfo.
pub async fn fetch_all_item_infos(
    client: &UexClient,
    api_key: &str,
) -> Result<Vec<EntityInfo>, String> {
    let categories: Vec<CategoryDto> = client.get("/categories", &[], api_key).await?;
    let category_ids: Vec<String> = categories
        .into_iter()
        .filter(|c| c.category_type.as_deref() == Some("item"))
        .map(|c| c.id)
        .collect();

    let handles: Vec<_> = category_ids
        .into_iter()
        .map(|cat_id| {
            let client = client.clone();
            let key = api_key.to_string();
            tokio::spawn(async move {
                let dtos: Result<Vec<ItemDto>, String> = client
                    .get("/items", &[("id_category", &cat_id)], &key)
                    .await;
                dtos
            })
        })
        .collect();

    let mut seen_ids = std::collections::HashSet::<String>::new();
    let mut all_infos: Vec<EntityInfo> = Vec::new();

    for handle in handles {
        match handle.await {
            Ok(Ok(dtos)) => {
                for dto in &dtos {
                    if seen_ids.insert(dto.id.clone()) {
                        all_infos.push(EntityInfo::from(dto));
                    }
                }
            }
            Ok(Err(e)) => log::warn!("Failed to fetch item infos for a category: {}", e),
            Err(e) => log::warn!("Item info fetch task panicked: {}", e),
        }
    }

    log::info!("Total item infos fetched: {}", all_infos.len());
    Ok(all_infos)
}

/// Search UEX for items by query string.
///
/// The `/items` endpoint does not support bare name search. This falls back to a
/// full fetch across all categories and filters client-side.
pub async fn search_items(
    client: &UexClient,
    query: &str,
    api_key: &str,
) -> Result<Vec<UexResult>, String> {
    let all = fetch_all_items(client, api_key).await?;
    Ok(super::search_in_collection(&all, query))
}

/// Fetch item details by uuid.
pub async fn get_item_info(
    client: &UexClient,
    uuid: &str,
    api_key: &str,
) -> Result<EntityInfo, String> {
    let dtos: Vec<ItemDto> = client
        .get("/items", &[("uuid", uuid)], api_key)
        .await?;
    dtos.first()
        .map(EntityInfo::from)
        .ok_or_else(|| format!("Item with uuid {} not found", uuid))
}

/// Get item prices for a specific item (direct API call).
pub async fn get_item_prices(
    client: &UexClient,
    item_id: &str,
    api_key: &str,
) -> Result<Vec<PriceEntry>, String> {
    let dtos: Vec<ItemPriceDto> = client
        .get("/items_prices", &[("id_item", item_id)], api_key)
        .await?;
    Ok(dtos.iter().map(PriceEntry::from).collect())
}

/// Fetch ALL item prices from UEX (bulk).
pub async fn fetch_all_item_prices(
    client: &UexClient,
    api_key: &str,
) -> Result<Vec<PriceEntry>, String> {
    let dtos: Vec<ItemPriceDto> = client.get("/items_prices_all", &[], api_key).await?;
    Ok(dtos.iter().map(PriceEntry::from).collect())
}
