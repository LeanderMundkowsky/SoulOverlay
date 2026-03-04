use serde::Deserialize;

use crate::uex::types::{
    deserialize_bool_flag, deserialize_flexible_id, deserialize_nonempty_string,
    deserialize_positive_f64, location_string, timestamp_string, EntityInfo, PriceEntry, UexResult,
};

// ── Commodity DTO ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CommodityDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub wiki: Option<String>,
    #[serde(default)]
    pub weight_scu: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_positive_f64")]
    pub price_buy: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_positive_f64")]
    pub price_sell: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_illegal: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_buyable: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_sellable: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_mineral: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_raw: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_refined: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_harvestable: Option<bool>,
}

impl From<&CommodityDto> for UexResult {
    fn from(dto: &CommodityDto) -> Self {
        Self {
            id: dto.id.clone(),
            name: dto.name.clone(),
            kind: "commodity".to_string(),
            slug: dto.slug.clone(),
            uuid: String::new(),
        }
    }
}

impl From<&CommodityDto> for EntityInfo {
    fn from(dto: &CommodityDto) -> Self {
        Self {
            id: dto.id.clone(),
            name: dto.name.clone(),
            kind: "commodity".to_string(),
            slug: dto.slug.clone(),
            code: dto.code.clone(),
            wiki: dto.wiki.clone(),
            commodity_kind: dto.kind.clone(),
            weight_scu: dto.weight_scu,
            avg_buy: dto.price_buy,
            avg_sell: dto.price_sell,
            is_illegal: dto.is_illegal,
            is_buyable: dto.is_buyable,
            is_sellable: dto.is_sellable,
            is_mineral: dto.is_mineral,
            is_raw: dto.is_raw,
            is_refined: dto.is_refined,
            is_harvestable: dto.is_harvestable,
            ..Default::default()
        }
    }
}

// ── Commodity Price DTO ────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CommodityPriceDto {
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
    pub terminal_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_id")]
    pub id_terminal: String,
    #[serde(default)]
    pub faction_name: Option<String>,
    #[serde(default)]
    pub price_buy: Option<f64>,
    #[serde(default)]
    pub price_buy_min: Option<f64>,
    #[serde(default)]
    pub price_buy_max: Option<f64>,
    #[serde(default)]
    pub price_buy_avg: Option<f64>,
    #[serde(default)]
    pub price_buy_users: Option<f64>,
    #[serde(default)]
    pub price_sell: Option<f64>,
    #[serde(default)]
    pub price_sell_min: Option<f64>,
    #[serde(default)]
    pub price_sell_max: Option<f64>,
    #[serde(default)]
    pub price_sell_avg: Option<f64>,
    #[serde(default)]
    pub price_sell_users: Option<f64>,
    #[serde(default)]
    pub scu_buy: Option<f64>,
    #[serde(default)]
    pub scu_buy_min: Option<f64>,
    #[serde(default)]
    pub scu_buy_max: Option<f64>,
    #[serde(default)]
    pub scu_buy_avg: Option<f64>,
    #[serde(default)]
    pub scu_buy_users: Option<f64>,
    #[serde(default)]
    pub scu_sell: Option<f64>,
    #[serde(default)]
    pub scu_sell_stock: Option<f64>,
    #[serde(default)]
    pub scu_sell_stock_avg: Option<f64>,
    #[serde(default)]
    pub status_buy: Option<f64>,
    #[serde(default)]
    pub status_buy_avg: Option<f64>,
    #[serde(default)]
    pub status_sell: Option<f64>,
    #[serde(default)]
    pub status_sell_avg: Option<f64>,
    #[serde(default)]
    pub container_sizes: Option<String>,
    #[serde(default)]
    pub date_modified: Option<serde_json::Value>,
    #[serde(default)]
    pub date_added: Option<serde_json::Value>,
}

impl CommodityPriceDto {
    pub fn to_price_entry(&self, price_type: &str) -> PriceEntry {
        let is_buy = self.price_buy.unwrap_or(0.0) > 0.0;

        let (scu_last, scu_min, scu_max, scu_avg, scu_users) = if is_buy {
            (self.scu_buy, self.scu_buy_min, self.scu_buy_max, self.scu_buy_avg, self.scu_buy_users)
        } else {
            (self.scu_sell_stock, None, self.scu_sell, self.scu_sell_stock_avg, None)
        };

        let (price_last, price_min, price_max, price_avg, price_users) = if is_buy {
            (self.price_buy, self.price_buy_min, self.price_buy_max, self.price_buy_avg, self.price_buy_users)
        } else {
            (self.price_sell, self.price_sell_min, self.price_sell_max, self.price_sell_avg, self.price_sell_users)
        };

        let (status, status_avg) = if is_buy {
            (self.status_buy, self.status_buy_avg)
        } else {
            (self.status_sell, self.status_sell_avg)
        };

        let container_sizes_display = self.container_sizes.as_deref().map(|cs| {
            let nums: Vec<u32> = cs.split(',').filter_map(|s| s.trim().parse().ok()).collect();
            if nums.is_empty() {
                String::new()
            } else {
                let min = nums.iter().min().unwrap();
                let max = nums.iter().max().unwrap();
                if min == max { format!("{}", min) } else { format!("{}-{}", min, max) }
            }
        }).unwrap_or_default();

        PriceEntry {
            entity_id: self.id_commodity.clone(),
            entity_name: self.commodity_name.clone().unwrap_or_default(),
            price_type: price_type.to_string(),
            location: location_string(&self.star_system_name, &self.planet_name),
            terminal: self.terminal_name.clone().unwrap_or_else(|| "Unknown".to_string()),
            terminal_id: self.id_terminal.clone(),
            buy_price: self.price_buy.unwrap_or(0.0),
            sell_price: self.price_sell.unwrap_or(0.0),
            rent_price: 0.0,
            scu_available: self.scu_buy.or(self.scu_sell),
            date_updated: timestamp_string(&self.date_modified, &self.date_added),
            orbit: self.orbit_name.clone().unwrap_or_default(),
            system: self.star_system_name.clone().unwrap_or_default(),
            faction: self.faction_name.clone().unwrap_or_default(),
            scu_last: scu_last.unwrap_or(0.0),
            scu_users: scu_users.unwrap_or(0.0),
            scu_avg: scu_avg.unwrap_or(0.0),
            scu_min: scu_min.unwrap_or(0.0),
            scu_max: scu_max.unwrap_or(0.0),
            price_last: price_last.unwrap_or(0.0),
            price_users: price_users.unwrap_or(0.0),
            price_avg: price_avg.unwrap_or(0.0),
            price_min: price_min.unwrap_or(0.0),
            price_max: price_max.unwrap_or(0.0),
            inventory_status: status.unwrap_or(0.0),
            inventory_status_avg: status_avg.unwrap_or(0.0),
            container_sizes: container_sizes_display,
            is_buy_location: is_buy,
        }
    }
}
