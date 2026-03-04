use serde::Deserialize;

use crate::uex::types::{
    deserialize_flexible_id, deserialize_nonempty_string, location_string, timestamp_string,
    EntityInfo, PriceEntry, UexResult,
};

// ── Category DTO ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CategoryDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default, rename = "type")]
    pub category_type: Option<String>,
}

// ── Item DTO ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ItemDto {
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

// ── Item Price DTO ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ItemPriceDto {
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
            rent_price: 0.0, scu_available: None,
            date_updated: timestamp_string(&dto.date_modified, &dto.date_added),
            orbit: dto.orbit_name.clone().unwrap_or_default(),
            system: dto.star_system_name.clone().unwrap_or_default(),
            faction: dto.faction_name.clone().unwrap_or_default(),
            scu_last: 0.0, scu_users: 0.0, scu_avg: 0.0, scu_min: 0.0, scu_max: 0.0,
            price_last: dto.price_buy.or(dto.price_sell).unwrap_or(0.0),
            price_users: 0.0, price_avg: 0.0, price_min: 0.0, price_max: 0.0,
            inventory_status: 0.0, inventory_status_avg: 0.0,
            container_sizes: String::new(),
            is_buy_location: dto.price_buy.unwrap_or(0.0) > 0.0,
        }
    }
}
