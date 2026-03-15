use serde::Deserialize;

use crate::uex::types::{
    deserialize_flexible_id, deserialize_nonempty_string, deserialize_positive_f64,
    location_string, timestamp_string, EntityInfo, PriceEntry, UexResult,
};

// ── Vehicle DTO ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct VehicleDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub name_full: Option<String>,
    #[serde(default)]
    pub slug: String,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub company_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_positive_f64")]
    pub scu: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub crew: Option<String>,
    #[serde(default, deserialize_with = "deserialize_positive_f64")]
    pub length: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_positive_f64")]
    pub width: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_positive_f64")]
    pub height: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_positive_f64")]
    pub mass: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub pad_type: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub url_photo: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub url_store: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub game_version: Option<String>,
    #[serde(default)]
    pub is_ground_vehicle: Option<u8>,
    #[serde(default)]
    pub is_boarding: Option<u8>,
    #[serde(default)]
    pub is_bomber: Option<u8>,
    #[serde(default)]
    pub is_cargo: Option<u8>,
    #[serde(default)]
    pub is_carrier: Option<u8>,
    #[serde(default)]
    pub is_civilian: Option<u8>,
    #[serde(default)]
    pub is_construction: Option<u8>,
    #[serde(default)]
    pub is_datarunner: Option<u8>,
    #[serde(default)]
    pub is_exploration: Option<u8>,
    #[serde(default)]
    pub is_industrial: Option<u8>,
    #[serde(default)]
    pub is_interdiction: Option<u8>,
    #[serde(default)]
    pub is_medical: Option<u8>,
    #[serde(default)]
    pub is_military: Option<u8>,
    #[serde(default)]
    pub is_mining: Option<u8>,
    #[serde(default)]
    pub is_passenger: Option<u8>,
    #[serde(default)]
    pub is_racing: Option<u8>,
    #[serde(default)]
    pub is_refinery: Option<u8>,
    #[serde(default)]
    pub is_refuel: Option<u8>,
    #[serde(default)]
    pub is_repair: Option<u8>,
    #[serde(default)]
    pub is_research: Option<u8>,
    #[serde(default)]
    pub is_salvage: Option<u8>,
    #[serde(default)]
    pub is_scanning: Option<u8>,
    #[serde(default)]
    pub is_science: Option<u8>,
    #[serde(default)]
    pub is_stealth: Option<u8>,
}

impl VehicleDto {
    pub fn kind_str(&self) -> &'static str {
        if self.is_ground_vehicle.unwrap_or(0) == 1 { "ground vehicle" } else { "vehicle" }
    }

    pub fn roles(&self) -> Vec<String> {
        let flags: &[(&Option<u8>, &str)] = &[
            (&self.is_boarding, "Boarding"), (&self.is_bomber, "Bomber"),
            (&self.is_cargo, "Cargo"), (&self.is_carrier, "Carrier"),
            (&self.is_civilian, "Civilian"), (&self.is_construction, "Construction"),
            (&self.is_datarunner, "Data Runner"), (&self.is_exploration, "Exploration"),
            (&self.is_industrial, "Industrial"), (&self.is_interdiction, "Interdiction"),
            (&self.is_medical, "Medical"), (&self.is_military, "Military"),
            (&self.is_mining, "Mining"), (&self.is_passenger, "Passenger"),
            (&self.is_racing, "Racing"), (&self.is_refinery, "Refinery"),
            (&self.is_refuel, "Refuel"), (&self.is_repair, "Repair"),
            (&self.is_research, "Research"), (&self.is_salvage, "Salvage"),
            (&self.is_scanning, "Scanning"), (&self.is_science, "Science"),
            (&self.is_stealth, "Stealth"),
        ];
        flags.iter().filter(|(val, _)| val.unwrap_or(0) == 1).map(|(_, label)| label.to_string()).collect()
    }

    pub fn display_name(&self) -> &str {
        self.name_full.as_deref().unwrap_or(&self.name)
    }
}

impl From<&VehicleDto> for UexResult {
    fn from(dto: &VehicleDto) -> Self {
        Self {
            id: dto.id.clone(),
            name: dto.display_name().to_string(),
            kind: dto.kind_str().to_string(),
            slug: dto.slug.clone(),
            uuid: String::new(),
        }
    }
}

impl From<&VehicleDto> for EntityInfo {
    fn from(dto: &VehicleDto) -> Self {
        let crew = dto.crew.as_deref().filter(|s| !s.is_empty() && *s != "0").map(|s| s.to_string());
        Self {
            id: dto.id.clone(),
            name: dto.name.clone(),
            kind: dto.kind_str().to_string(),
            slug: dto.slug.clone(),
            name_full: dto.name_full.clone(),
            company_name: dto.company_name.clone(),
            scu: dto.scu,
            crew,
            length: dto.length,
            width: dto.width,
            height: dto.height,
            mass: dto.mass,
            pad_type: dto.pad_type.clone(),
            url_photo: dto.url_photo.clone(),
            url_store: dto.url_store.clone(),
            game_version: dto.game_version.clone(),
            roles: dto.roles(),
            ..Default::default()
        }
    }
}

// ── Vehicle Purchase Price DTO ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct VehiclePurchasePriceDto {
    #[serde(default, deserialize_with = "deserialize_flexible_id")]
    pub id_vehicle: String,
    #[serde(default)]
    pub vehicle_name: Option<String>,
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

impl From<&VehiclePurchasePriceDto> for PriceEntry {
    fn from(dto: &VehiclePurchasePriceDto) -> Self {
        Self {
            entity_id: dto.id_vehicle.clone(),
            entity_name: dto.vehicle_name.clone().unwrap_or_default(),
            price_type: "vehicle_purchase".to_string(),
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
            category: String::new(),
        }
    }
}

// ── Vehicle Rental Price DTO───────────────────────────────────────────────

#[derive(Deserialize)]
pub struct VehicleRentalPriceDto {
    #[serde(default, deserialize_with = "deserialize_flexible_id")]
    pub id_vehicle: String,
    #[serde(default)]
    pub vehicle_name: Option<String>,
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
    pub price_rent: Option<f64>,
    #[serde(default)]
    pub date_modified: Option<serde_json::Value>,
    #[serde(default)]
    pub date_added: Option<serde_json::Value>,
}

impl From<&VehicleRentalPriceDto> for PriceEntry {
    fn from(dto: &VehicleRentalPriceDto) -> Self {
        Self {
            entity_id: dto.id_vehicle.clone(),
            entity_name: dto.vehicle_name.clone().unwrap_or_default(),
            price_type: "vehicle_rental".to_string(),
            location: location_string(&dto.star_system_name, &dto.planet_name),
            terminal: dto.terminal_name.clone().unwrap_or_else(|| "Unknown".to_string()),
            terminal_id: dto.id_terminal.clone(),
            buy_price: 0.0, sell_price: 0.0,
            rent_price: dto.price_rent.unwrap_or(0.0),
            scu_available: None,
            date_updated: timestamp_string(&dto.date_modified, &dto.date_added),
            orbit: dto.orbit_name.clone().unwrap_or_default(),
            system: dto.star_system_name.clone().unwrap_or_default(),
            faction: dto.faction_name.clone().unwrap_or_default(),
            scu_last: 0.0, scu_users: 0.0, scu_avg: 0.0, scu_min: 0.0, scu_max: 0.0,
            price_last: 0.0, price_users: 0.0, price_avg: 0.0, price_min: 0.0, price_max: 0.0,
            inventory_status: 0.0, inventory_status_avg: 0.0,
            container_sizes: String::new(), is_buy_location: false,
            category: String::new(),
        }
    }
}
