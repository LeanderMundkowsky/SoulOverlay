use serde::Deserialize;

use super::client::UexClient;
use super::types::{
    deserialize_flexible_id, deserialize_nonempty_string, deserialize_positive_f64,
    location_string, timestamp_string, EntityInfo, PriceEntry, UexResult,
};

// ── API DTOs ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct VehicleDto {
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
    // Role flags
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
    fn kind_str(&self) -> &'static str {
        if self.is_ground_vehicle.unwrap_or(0) == 1 {
            "ground vehicle"
        } else {
            "vehicle"
        }
    }

    fn roles(&self) -> Vec<String> {
        let flags: &[(&Option<u8>, &str)] = &[
            (&self.is_boarding, "Boarding"),
            (&self.is_bomber, "Bomber"),
            (&self.is_cargo, "Cargo"),
            (&self.is_carrier, "Carrier"),
            (&self.is_civilian, "Civilian"),
            (&self.is_construction, "Construction"),
            (&self.is_datarunner, "Data Runner"),
            (&self.is_exploration, "Exploration"),
            (&self.is_industrial, "Industrial"),
            (&self.is_interdiction, "Interdiction"),
            (&self.is_medical, "Medical"),
            (&self.is_military, "Military"),
            (&self.is_mining, "Mining"),
            (&self.is_passenger, "Passenger"),
            (&self.is_racing, "Racing"),
            (&self.is_refinery, "Refinery"),
            (&self.is_refuel, "Refuel"),
            (&self.is_repair, "Repair"),
            (&self.is_research, "Research"),
            (&self.is_salvage, "Salvage"),
            (&self.is_scanning, "Scanning"),
            (&self.is_science, "Science"),
            (&self.is_stealth, "Stealth"),
        ];
        flags
            .iter()
            .filter(|(val, _)| val.unwrap_or(0) == 1)
            .map(|(_, label)| label.to_string())
            .collect()
    }

    /// Display name preferring name_full over name.
    fn display_name(&self) -> &str {
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
        // Filter crew "0" as empty
        let crew = dto
            .crew
            .as_deref()
            .filter(|s| !s.is_empty() && *s != "0")
            .map(|s| s.to_string());

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

#[derive(Deserialize)]
pub(crate) struct VehiclePurchasePriceDto {
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
            sell_price: 0.0,
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
            price_last: dto.price_buy.unwrap_or(0.0),
            price_users: 0.0,
            price_avg: 0.0,
            price_min: 0.0,
            price_max: 0.0,
            inventory_status: 0.0,
            inventory_status_avg: 0.0,
            container_sizes: String::new(),
            is_buy_location: true,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct VehicleRentalPriceDto {
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
            buy_price: 0.0,
            sell_price: 0.0,
            rent_price: dto.price_rent.unwrap_or(0.0),
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
            price_last: 0.0,
            price_users: 0.0,
            price_avg: 0.0,
            price_min: 0.0,
            price_max: 0.0,
            inventory_status: 0.0,
            inventory_status_avg: 0.0,
            container_sizes: String::new(),
            is_buy_location: false,
        }
    }
}

// ── Public functions ───────────────────────────────────────────────────────

/// Fetch ALL vehicles from UEX.
pub async fn fetch_all_vehicles(
    client: &UexClient,
    api_key: &str,
) -> Result<Vec<UexResult>, String> {
    let dtos: Vec<VehicleDto> = client.get("/vehicles", &[], api_key).await?;
    Ok(dtos.iter().map(UexResult::from).collect())
}

/// Search UEX for vehicles by query string (direct API call, no cache).
pub async fn search_vehicles(
    client: &UexClient,
    query: &str,
    api_key: &str,
) -> Result<Vec<UexResult>, String> {
    let dtos: Vec<VehicleDto> = client.get("/vehicles", &[], api_key).await?;
    let query_lower = query.to_lowercase();
    Ok(dtos
        .iter()
        .map(UexResult::from)
        .filter(|r| r.name.to_lowercase().contains(&query_lower))
        .collect())
}

/// Fetch vehicle details by id.
pub async fn get_vehicle_info(
    client: &UexClient,
    vehicle_id: &str,
    api_key: &str,
) -> Result<EntityInfo, String> {
    let dtos: Vec<VehicleDto> = client.get("/vehicles", &[], api_key).await?;
    dtos.iter()
        .find(|dto| dto.id == vehicle_id)
        .map(EntityInfo::from)
        .ok_or_else(|| format!("Vehicle {} not found", vehicle_id))
}

/// Get vehicle purchase prices for a specific vehicle (direct API call).
pub async fn get_vehicle_purchase_prices(
    client: &UexClient,
    vehicle_id: &str,
    api_key: &str,
) -> Result<Vec<PriceEntry>, String> {
    let dtos: Vec<VehiclePurchasePriceDto> = client
        .get(
            "/vehicles_purchases_prices",
            &[("id_vehicle", vehicle_id)],
            api_key,
        )
        .await?;
    Ok(dtos.iter().map(PriceEntry::from).collect())
}

/// Get vehicle rental prices for a specific vehicle (direct API call).
pub async fn get_vehicle_rental_prices(
    client: &UexClient,
    vehicle_id: &str,
    api_key: &str,
) -> Result<Vec<PriceEntry>, String> {
    let dtos: Vec<VehicleRentalPriceDto> = client
        .get(
            "/vehicles_rentals_prices",
            &[("id_vehicle", vehicle_id)],
            api_key,
        )
        .await?;
    Ok(dtos.iter().map(PriceEntry::from).collect())
}

/// Fetch vehicle purchase prices for each ID in `vehicle_ids` using per-entity API calls
/// in parallel. Returns the full rich dataset including orbit, system, and faction.
pub async fn fetch_all_vehicle_purchase_prices_per_entity(
    client: &UexClient,
    vehicle_ids: &[String],
    api_key: &str,
) -> Vec<PriceEntry> {
    let handles: Vec<_> = vehicle_ids
        .iter()
        .map(|id| {
            let client = client.clone();
            let id = id.clone();
            let key = api_key.to_string();
            tokio::spawn(async move { get_vehicle_purchase_prices(&client, &id, &key).await })
        })
        .collect();

    let mut all = Vec::new();
    for handle in handles {
        if let Ok(Ok(prices)) = handle.await {
            all.extend(prices);
        }
    }
    all
}

/// Fetch vehicle rental prices for each ID in `vehicle_ids` using per-entity API calls
/// in parallel. Returns the full rich dataset including orbit, system, and faction.
pub async fn fetch_all_vehicle_rental_prices_per_entity(
    client: &UexClient,
    vehicle_ids: &[String],
    api_key: &str,
) -> Vec<PriceEntry> {
    let handles: Vec<_> = vehicle_ids
        .iter()
        .map(|id| {
            let client = client.clone();
            let id = id.clone();
            let key = api_key.to_string();
            tokio::spawn(async move { get_vehicle_rental_prices(&client, &id, &key).await })
        })
        .collect();

    let mut all = Vec::new();
    for handle in handles {
        if let Ok(Ok(prices)) = handle.await {
            all.extend(prices);
        }
    }
    all
}


pub async fn fetch_all_vehicle_purchase_prices(
    client: &UexClient,
    api_key: &str,
) -> Result<Vec<PriceEntry>, String> {
    let dtos: Vec<VehiclePurchasePriceDto> = client
        .get("/vehicles_purchases_prices_all", &[], api_key)
        .await?;
    Ok(dtos.iter().map(PriceEntry::from).collect())
}

/// Fetch ALL vehicle rental prices from UEX (bulk).
pub async fn fetch_all_vehicle_rental_prices(
    client: &UexClient,
    api_key: &str,
) -> Result<Vec<PriceEntry>, String> {
    let dtos: Vec<VehicleRentalPriceDto> = client
        .get("/vehicles_rentals_prices_all", &[], api_key)
        .await?;
    Ok(dtos.iter().map(PriceEntry::from).collect())
}
