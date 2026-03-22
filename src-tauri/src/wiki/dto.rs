use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

/// Deserializes a field that can be either a plain string (list endpoint)
/// or a localized object like `{"en_EN": "value"}` (detail endpoint).
/// Returns the English value in either case.
fn deserialize_localized_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Option::<serde_json::Value>::deserialize(deserializer)?;
    match val {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(s)) => Ok(Some(s)),
        Some(serde_json::Value::Object(map)) => Ok(map
            .get("en_EN")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())),
        _ => Ok(None),
    }
}

// ── Response envelopes ─────────────────────────────────────────────────────

/// Single-item response: `{ "data": T }`
#[derive(Debug, Deserialize)]
pub struct WikiApiResponse<T> {
    pub data: T,
}

/// Paginated list response: `{ "data": [...], "meta": { ... } }`
#[derive(Debug, Deserialize)]
pub struct WikiSearchResponse<T> {
    pub data: Vec<T>,
    #[serde(default)]
    #[allow(dead_code)]
    pub meta: WikiPaginationMeta,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiPaginationMeta {
    pub current_page: u32,
    pub last_page: u32,
    pub per_page: u32,
    pub total: u32,
}

// ── Shared nested types ────────────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiManufacturer {
    pub name: Option<String>,
    pub code: Option<String>,
    pub uuid: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiDescription {
    #[serde(rename = "en_EN")]
    pub en_en: Option<String>,
    #[serde(rename = "de_DE")]
    pub de_de: Option<String>,
    #[serde(rename = "zh_CN")]
    pub zh_cn: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiDimension {
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub length: Option<f64>,
    pub volume: Option<f64>,
}

// ── Item-specific nested types ─────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiPowerPlant {
    pub power_output: Option<f64>,
    pub power_segment_generation: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiHeat {
    pub temperature_to_ir: Option<f64>,
    pub ir_temperature_threshold: Option<f64>,
    pub overpower_heat: Option<f64>,
    pub overclock_threshold_min: Option<f64>,
    pub overclock_threshold_max: Option<f64>,
    pub thermal_energy_base: Option<f64>,
    pub thermal_energy_draw: Option<f64>,
    pub thermal_conductivity: Option<f64>,
    pub specific_heat_capacity: Option<f64>,
    pub start_cooling_temperature: Option<f64>,
    pub max_cooling_rate: Option<f64>,
    pub max_temperature: Option<f64>,
    pub overheat_temperature: Option<f64>,
    pub recovery_temperature: Option<f64>,
    pub misfire_min_temperature: Option<f64>,
    pub misfire_max_temperature: Option<f64>,
    pub ir_emission: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiPower {
    pub power_base: Option<f64>,
    pub power_draw: Option<f64>,
    pub throttleable: Option<f64>,
    pub overclockable: Option<f64>,
    pub overclock_threshold_min: Option<f64>,
    pub overclock_threshold_max: Option<f64>,
    pub overpower_performance: Option<f64>,
    pub overclock_performance: Option<f64>,
    pub power_to_em: Option<f64>,
    pub decay_rate_em: Option<f64>,
    pub em_min: Option<f64>,
    pub em_max: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiResistance {
    pub physical: Option<f64>,
    pub energy: Option<f64>,
    pub thermal: Option<f64>,
    pub distortion: Option<f64>,
    pub biochemical: Option<f64>,
    pub stun: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiDurability {
    pub health: Option<f64>,
    pub repairable: Option<bool>,
    pub salvageable: Option<bool>,
    pub resistance: Option<WikiResistance>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiDistortion {
    pub decay_rate: Option<f64>,
    pub decay_delay: Option<f64>,
    pub maximum: Option<f64>,
    pub warning_ratio: Option<f64>,
    pub shutdown_time: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiItemEmission {
    pub ir: Option<f64>,
    pub em_min: Option<f64>,
    pub em_max: Option<f64>,
    pub em_decay: Option<f64>,
}

// ── Weapon nested types ────────────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiWeaponDps {
    pub physical: Option<f64>,
    pub energy: Option<f64>,
    pub distortion: Option<f64>,
    pub thermal: Option<f64>,
    pub biochemical: Option<f64>,
    pub stun: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiWeaponDamage {
    pub dps: Option<WikiWeaponDps>,
    pub alpha: Option<WikiWeaponDps>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiFireMode {
    pub mode: Option<String>,
    #[serde(rename = "type")]
    pub fire_type: Option<String>,
    pub rounds_per_minute: Option<f64>,
    pub ammo_per_shot: Option<f64>,
    pub pellets_per_shot: Option<f64>,
    pub damage_per_second: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiWeaponSpread {
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub first_attack: Option<f64>,
    pub per_attack: Option<f64>,
    pub decay: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiCapacitor {
    pub max_ammo_load: Option<f64>,
    pub regen_per_second: Option<f64>,
    pub cooldown: Option<f64>,
    pub costs_per_shot: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiAmmunition {
    pub uuid: Option<String>,
    pub size: Option<f64>,
    pub lifetime: Option<f64>,
    pub speed: Option<f64>,
    pub range: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiVehicleWeapon {
    pub class: Option<String>,
    #[serde(rename = "type")]
    pub weapon_type: Option<String>,
    pub range: Option<f64>,
    pub damage_per_shot: Option<f64>,
    pub rpm: Option<f64>,
    pub damage: Option<WikiWeaponDamage>,
    pub modes: Option<Vec<WikiFireMode>>,
    pub spread: Option<WikiWeaponSpread>,
    pub capacitor: Option<WikiCapacitor>,
    pub ammunition: Option<WikiAmmunition>,
}

// ── Vehicle nested types ───────────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiSpeed {
    pub scm: Option<f64>,
    pub max: Option<f64>,
    pub boost_forward: Option<f64>,
    pub boost_backward: Option<f64>,
    pub zero_to_scm: Option<f64>,
    pub zero_to_max: Option<f64>,
    pub scm_to_zero: Option<f64>,
    pub max_to_zero: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiAcceleration {
    pub main: Option<f64>,
    pub retro: Option<f64>,
    pub vtol: Option<f64>,
    pub main_g: Option<f64>,
    pub retro_g: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiAgility {
    pub pitch: Option<f64>,
    pub yaw: Option<f64>,
    pub roll: Option<f64>,
    pub pitch_boosted: Option<f64>,
    pub yaw_boosted: Option<f64>,
    pub roll_boosted: Option<f64>,
    pub acceleration: Option<WikiAcceleration>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiQuantum {
    pub quantum_speed: Option<f64>,
    pub quantum_spool_time: Option<f64>,
    pub quantum_fuel_capacity: Option<f64>,
    pub quantum_range: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiFuel {
    pub capacity: Option<f64>,
    pub intake_rate: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiShield {
    pub hp: Option<f64>,
    pub regeneration: Option<f64>,
    pub face_type: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiDamageMultipliers {
    pub physical: Option<f64>,
    pub energy: Option<f64>,
    pub distortion: Option<f64>,
    pub thermal: Option<f64>,
    pub biochemical: Option<f64>,
    pub stun: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiArmor {
    pub health: Option<f64>,
    pub damage_multipliers: Option<WikiDamageMultipliers>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiInsurance {
    pub claim_time: Option<f64>,
    pub expedite_time: Option<f64>,
    pub expedite_cost: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiCrew {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

// ── Manufacturer list item ──────────────────────────────────────────────────

/// Manufacturer from `GET /api/manufacturers` (list endpoint).
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct WikiManufacturerListItem {
    pub name: Option<String>,
    pub code: Option<String>,
    pub uuid: Option<String>,
}

// ── Main DTOs ──────────────────────────────────────────────────────────────

/// Full item detail from `GET /api/items/{uuid}` or list endpoints.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiItemDto {
    pub uuid: Option<String>,
    pub name: Option<String>,
    pub class_name: Option<String>,
    pub classification: Option<String>,
    pub description: Option<WikiDescription>,
    pub size: Option<f64>,
    pub mass: Option<f64>,
    pub grade: Option<String>,
    pub class: Option<String>,
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    pub sub_type: Option<String>,
    pub manufacturer: Option<WikiManufacturer>,
    pub web_url: Option<String>,
    pub version: Option<String>,
    // Type-specific nested data
    pub power_plant: Option<WikiPowerPlant>,
    pub heat: Option<WikiHeat>,
    pub power: Option<WikiPower>,
    pub durability: Option<WikiDurability>,
    pub distortion: Option<WikiDistortion>,
    pub emission: Option<WikiItemEmission>,
    pub dimension: Option<WikiDimension>,
    pub vehicle_weapon: Option<WikiVehicleWeapon>,
    pub ammunition: Option<WikiAmmunition>,
}

/// Full vehicle detail from `GET /api/vehicles/{uuid}` or list endpoints.
/// Note: Some fields (career, role, size, type) return plain strings in the
/// list endpoint but localized objects in the detail endpoint. We use
/// `deserialize_localized_string` to handle both formats.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WikiVehicleDto {
    pub uuid: Option<String>,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub class_name: Option<String>,
    pub description: Option<HashMap<String, String>>,
    pub mass: Option<f64>,
    pub cargo_capacity: Option<f64>,
    pub vehicle_inventory: Option<f64>,
    pub crew: Option<WikiCrew>,
    pub health: Option<f64>,
    pub shield_hp: Option<f64>,
    pub speed: Option<WikiSpeed>,
    pub agility: Option<WikiAgility>,
    pub quantum: Option<WikiQuantum>,
    pub fuel: Option<WikiFuel>,
    pub shield: Option<WikiShield>,
    pub armor: Option<WikiArmor>,
    pub insurance: Option<WikiInsurance>,
    pub manufacturer: Option<WikiManufacturer>,
    pub dimension: Option<WikiDimension>,
    pub msrp: Option<f64>,
    pub pledge_url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_localized_string")]
    #[allow(dead_code)]
    pub career: Option<String>,
    #[serde(default, deserialize_with = "deserialize_localized_string")]
    #[allow(dead_code)]
    pub role: Option<String>,
    pub web_url: Option<String>,
    pub version: Option<String>,
    #[serde(default, deserialize_with = "deserialize_localized_string")]
    pub size: Option<String>,
    #[serde(default, rename = "type", deserialize_with = "deserialize_localized_string")]
    pub vehicle_type: Option<String>,
    /// Localized array of objects, e.g. `[{"en_EN": "Light Freight"}]`
    #[serde(default)]
    #[allow(dead_code)]
    pub foci: Vec<serde_json::Value>,
}
