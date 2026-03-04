use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;

// ── Public app-level types ─────────────────────────────────────────────────

/// A search result from UEX API.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UexResult {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub slug: String,
    /// UUID (items only). Used to fetch item details from the UEX API.
    #[serde(default)]
    pub uuid: String,
}

/// Detailed entity metadata from UEX API, with type-specific optional fields.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Type)]
pub struct EntityInfo {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub slug: String,
    // Common
    pub code: Option<String>,
    pub company_name: Option<String>,
    pub wiki: Option<String>,
    pub game_version: Option<String>,
    // Commodity
    pub commodity_kind: Option<String>,
    pub weight_scu: Option<f64>,
    pub avg_buy: Option<f64>,
    pub avg_sell: Option<f64>,
    pub is_illegal: Option<bool>,
    pub is_buyable: Option<bool>,
    pub is_sellable: Option<bool>,
    pub is_mineral: Option<bool>,
    pub is_raw: Option<bool>,
    pub is_refined: Option<bool>,
    pub is_harvestable: Option<bool>,
    // Item
    pub section: Option<String>,
    pub category: Option<String>,
    pub size: Option<String>,
    pub color: Option<String>,
    // Vehicle
    pub name_full: Option<String>,
    pub scu: Option<f64>,
    pub crew: Option<String>,
    pub length: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub mass: Option<f64>,
    pub pad_type: Option<String>,
    pub url_photo: Option<String>,
    pub url_store: Option<String>,
    pub roles: Vec<String>,
}

/// A price entry from UEX API.
/// Unified across all price types — entity metadata identifies the source.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PriceEntry {
    pub entity_id: String,
    pub entity_name: String,
    pub price_type: String,
    pub location: String,
    pub terminal: String,
    pub terminal_id: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub rent_price: f64,
    pub scu_available: Option<f64>,
    pub date_updated: String,
    // Rich fields (populated by per-entity endpoints, zeroed for bulk/other types)
    #[serde(default)]
    pub orbit: String,
    #[serde(default)]
    pub system: String,
    #[serde(default)]
    pub faction: String,
    #[serde(default)]
    pub scu_last: f64,
    #[serde(default)]
    pub scu_users: f64,
    #[serde(default)]
    pub scu_avg: f64,
    #[serde(default)]
    pub scu_min: f64,
    #[serde(default)]
    pub scu_max: f64,
    #[serde(default)]
    pub price_last: f64,
    #[serde(default)]
    pub price_users: f64,
    #[serde(default)]
    pub price_avg: f64,
    #[serde(default)]
    pub price_min: f64,
    #[serde(default)]
    pub price_max: f64,
    #[serde(default)]
    pub inventory_status: f64,
    #[serde(default)]
    pub inventory_status_avg: f64,
    #[serde(default)]
    pub container_sizes: String,
    #[serde(default)]
    pub is_buy_location: bool,
}

/// Authenticated user profile from the UEX API `GET /user` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UexUserProfile {
    pub id: u32,
    pub name: String,
    pub username: String,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub website_url: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub discord_username: Option<String>,
    pub twitch_username: Option<String>,
    pub day_availability: Vec<String>,
    pub time_availability: Vec<String>,
    pub specializations: Vec<String>,
    pub languages: Vec<String>,
    pub archetypes: Vec<String>,
    pub is_datarunner: bool,
    pub is_datarunner_banned: bool,
    pub is_staff: bool,
    pub is_away_game: bool,
    pub date_added: Option<String>,
    pub date_modified: Option<String>,
    pub date_rsi_verified: Option<String>,
    pub date_twitch_verified: Option<String>,
}

/// A vehicle in the user's hangar/fleet from UEX API.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HangarVehicle {
    pub id: String,
    pub id_vehicle: String,
    pub name: String,
    pub model_name: String,
    pub serial: Option<String>,
    pub description: Option<String>,
    pub organization_name: Option<String>,
    pub is_hidden: bool,
    pub is_pledged: bool,
    pub date_added: String,
    pub url_photo: Option<String>,
}

// ── Serde helpers ──────────────────────────────────────────────────────────

/// Deserialize a field that may be a JSON number or a JSON string into a `String`.
pub fn deserialize_flexible_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let val = serde_json::Value::deserialize(deserializer)?;
    match val {
        serde_json::Value::Number(n) => Ok(n.to_string()),
        serde_json::Value::String(s) => Ok(s),
        _ => Ok(String::new()),
    }
}

/// Deserialize a UEX boolean flag (0/1 integer) into `Option<bool>`.
pub fn deserialize_bool_flag<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Option::<u8>::deserialize(deserializer)?;
    Ok(val.map(|v| v == 1))
}

/// Deserialize a field that may be absent/null/zero into `Option<f64>`,
/// treating 0.0 as None.
pub fn deserialize_positive_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Option::<f64>::deserialize(deserializer)?;
    Ok(val.filter(|&v| v > 0.0))
}

/// Deserialize a string field, returning None for empty strings.
pub fn deserialize_nonempty_string<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Option::<String>::deserialize(deserializer)?;
    Ok(val.filter(|s| !s.is_empty()))
}

/// Extract a location string from star_system_name or planet_name.
pub fn location_string(star_system: &Option<String>, planet: &Option<String>) -> String {
    star_system
        .as_deref()
        .or(planet.as_deref())
        .unwrap_or("Unknown")
        .to_string()
}

/// Extract a timestamp string from date_modified/date_added fields.
pub fn timestamp_string(date_modified: &Option<serde_json::Value>, date_added: &Option<serde_json::Value>) -> String {
    date_modified
        .as_ref()
        .or(date_added.as_ref())
        .map(|v| match v {
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s.clone(),
            _ => String::new(),
        })
        .unwrap_or_default()
}
