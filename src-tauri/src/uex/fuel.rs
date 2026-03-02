use serde::Deserialize;

use super::client::UexClient;
use super::types::{
    deserialize_flexible_id, location_string, timestamp_string, PriceEntry,
};

// ── API DTOs ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct FuelPriceDto {
    #[serde(default, deserialize_with = "deserialize_flexible_id")]
    pub id_commodity: String,
    #[serde(default)]
    pub commodity_name: Option<String>,
    #[serde(default)]
    pub star_system_name: Option<String>,
    #[serde(default)]
    pub planet_name: Option<String>,
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
            sell_price: 0.0,
            rent_price: 0.0,
            scu_available: None,
            date_updated: timestamp_string(&dto.date_modified, &dto.date_added),
        }
    }
}

// ── Public functions ───────────────────────────────────────────────────────

/// Get fuel prices for a specific terminal (direct API call).
pub async fn get_fuel_prices(
    client: &UexClient,
    terminal_id: &str,
    api_key: &str,
) -> Result<Vec<PriceEntry>, String> {
    let dtos: Vec<FuelPriceDto> = client
        .get("/fuel_prices", &[("id_terminal", terminal_id)], api_key)
        .await?;
    Ok(dtos.iter().map(PriceEntry::from).collect())
}

/// Fetch ALL fuel prices from UEX (bulk).
pub async fn fetch_all_fuel_prices(
    client: &UexClient,
    api_key: &str,
) -> Result<Vec<PriceEntry>, String> {
    let dtos: Vec<FuelPriceDto> = client.get("/fuel_prices_all", &[], api_key).await?;
    Ok(dtos.iter().map(PriceEntry::from).collect())
}
