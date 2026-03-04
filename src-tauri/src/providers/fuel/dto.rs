use serde::Deserialize;

use crate::uex::types::{
    deserialize_flexible_id, location_string, timestamp_string, PriceEntry,
};

#[derive(Deserialize)]
pub struct FuelPriceDto {
    #[serde(default, deserialize_with = "deserialize_flexible_id")]
    pub id_commodity: String,
    #[serde(default)]
    pub commodity_name: Option<String>,
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
    pub date_modified: Option<serde_json::Value>,
    #[serde(default)]
    pub date_added: Option<serde_json::Value>,
}

impl From<&FuelPriceDto> for PriceEntry {
    fn from(dto: &FuelPriceDto) -> Self {
        Self {
            entity_id: dto.id_commodity.clone(),
            entity_name: dto.commodity_name.clone().unwrap_or_default(),
            price_type: "fuel".to_string(),
            location: location_string(&dto.star_system_name, &dto.planet_name),
            terminal: dto.terminal_name.clone().unwrap_or_else(|| "Unknown".to_string()),
            terminal_id: dto.id_terminal.clone(),
            buy_price: dto.price_buy.unwrap_or(0.0),
            sell_price: 0.0, rent_price: 0.0, scu_available: None,
            date_updated: timestamp_string(&dto.date_modified, &dto.date_added),
            orbit: dto.orbit_name.clone().unwrap_or_default(),
            system: dto.star_system_name.clone().unwrap_or_default(),
            faction: dto.faction_name.clone().unwrap_or_default(),
            scu_last: 0.0, scu_users: 0.0, scu_avg: 0.0, scu_min: 0.0, scu_max: 0.0,
            price_last: dto.price_buy.unwrap_or(0.0),
            price_users: 0.0, price_avg: 0.0, price_min: 0.0, price_max: 0.0,
            inventory_status: 0.0, inventory_status_avg: 0.0,
            container_sizes: String::new(), is_buy_location: true,
        }
    }
}
