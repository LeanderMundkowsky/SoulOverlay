use serde::Deserialize;

use crate::uex::types::{deserialize_bool_flag, deserialize_flexible_id, deserialize_nonempty_string, deserialize_optional_id, UexResult};

/// Trait for location DTOs that have an `is_available_live` field.
pub(crate) trait LiveAvailable {
    fn is_available_live(&self) -> bool;
}

// -- Terminal DTO -----------------------------------------------------------

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct TerminalDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub nickname: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub star_system_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub planet_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub orbit_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub faction_name: Option<String>,
    // Hierarchy IDs for future pin-feature terminal resolution
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id_star_system: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id_planet: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id_moon: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id_orbit: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id_space_station: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id_outpost: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id_city: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id_poi: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_available_live: Option<bool>,
}

impl LiveAvailable for TerminalDto {
    fn is_available_live(&self) -> bool { self.is_available_live.unwrap_or(false) }
}

impl From<&TerminalDto> for UexResult {
    fn from(dto: &TerminalDto) -> Self {
        let display = if let Some(nick) = &dto.nickname {
            format!("[Terminal] {} ({})", nick, dto.name)
        } else {
            format!("[Terminal] {}", dto.name)
        };
        Self {
            id: dto.id.clone(),
            name: display,
            kind: "location".to_string(),
            slug: "terminal".to_string(),
            uuid: String::new(),
        }
    }
}

// -- Star System DTO --------------------------------------------------------

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct StarSystemDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_available_live: Option<bool>,
}

impl LiveAvailable for StarSystemDto {
    fn is_available_live(&self) -> bool { self.is_available_live.unwrap_or(false) }
}

impl From<&StarSystemDto> for UexResult {
    fn from(dto: &StarSystemDto) -> Self {
        Self {
            id: format!("sys_{}", dto.id),
            name: format!("[System] {}", dto.name),
            kind: "location".to_string(),
            slug: "star_system".to_string(),
            uuid: String::new(),
        }
    }
}

// -- Planet DTO -------------------------------------------------------------

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct PlanetDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id_star_system: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub star_system_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_available_live: Option<bool>,
}

impl LiveAvailable for PlanetDto {
    fn is_available_live(&self) -> bool { self.is_available_live.unwrap_or(false) }
}

impl From<&PlanetDto> for UexResult {
    fn from(dto: &PlanetDto) -> Self {
        Self {
            id: format!("planet_{}", dto.id),
            name: format!("[Planet] {}", dto.name),
            kind: "location".to_string(),
            slug: "planet".to_string(),
            uuid: String::new(),
        }
    }
}

// -- Moon DTO ---------------------------------------------------------------

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct MoonDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id_planet: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub planet_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_available_live: Option<bool>,
}

impl LiveAvailable for MoonDto {
    fn is_available_live(&self) -> bool { self.is_available_live.unwrap_or(false) }
}

impl From<&MoonDto> for UexResult {
    fn from(dto: &MoonDto) -> Self {
        Self {
            id: format!("moon_{}", dto.id),
            name: format!("[Moon] {}", dto.name),
            kind: "location".to_string(),
            slug: "moon".to_string(),
            uuid: String::new(),
        }
    }
}

// -- Orbit DTO --------------------------------------------------------------

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct OrbitDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id_star_system: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_available_live: Option<bool>,
}

impl LiveAvailable for OrbitDto {
    fn is_available_live(&self) -> bool { self.is_available_live.unwrap_or(false) }
}

impl From<&OrbitDto> for UexResult {
    fn from(dto: &OrbitDto) -> Self {
        Self {
            id: format!("orbit_{}", dto.id),
            name: format!("[Orbit] {}", dto.name),
            kind: "location".to_string(),
            slug: "orbit".to_string(),
            uuid: String::new(),
        }
    }
}

// -- Space Station DTO ------------------------------------------------------

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct SpaceStationDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id_orbit: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub orbit_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id_star_system: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_available_live: Option<bool>,
}

impl LiveAvailable for SpaceStationDto {
    fn is_available_live(&self) -> bool { self.is_available_live.unwrap_or(false) }
}

impl From<&SpaceStationDto> for UexResult {
    fn from(dto: &SpaceStationDto) -> Self {
        Self {
            id: format!("station_{}", dto.id),
            name: format!("[Station] {}", dto.name),
            kind: "location".to_string(),
            slug: "space_station".to_string(),
            uuid: String::new(),
        }
    }
}

// -- Outpost DTO ------------------------------------------------------------

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct OutpostDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id_orbit: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub orbit_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_available_live: Option<bool>,
}

impl LiveAvailable for OutpostDto {
    fn is_available_live(&self) -> bool { self.is_available_live.unwrap_or(false) }
}

impl From<&OutpostDto> for UexResult {
    fn from(dto: &OutpostDto) -> Self {
        Self {
            id: format!("outpost_{}", dto.id),
            name: format!("[Outpost] {}", dto.name),
            kind: "location".to_string(),
            slug: "outpost".to_string(),
            uuid: String::new(),
        }
    }
}

// -- POI DTO ----------------------------------------------------------------

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct PoiDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_available_live: Option<bool>,
}

impl LiveAvailable for PoiDto {
    fn is_available_live(&self) -> bool { self.is_available_live.unwrap_or(false) }
}

impl From<&PoiDto> for UexResult {
    fn from(dto: &PoiDto) -> Self {
        Self {
            id: format!("poi_{}", dto.id),
            name: format!("[POI] {}", dto.name),
            kind: "location".to_string(),
            slug: "poi".to_string(),
            uuid: String::new(),
        }
    }
}

// -- City DTO ---------------------------------------------------------------

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CityDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id_planet: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub planet_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_available_live: Option<bool>,
}

impl LiveAvailable for CityDto {
    fn is_available_live(&self) -> bool { self.is_available_live.unwrap_or(false) }
}

impl From<&CityDto> for UexResult {
    fn from(dto: &CityDto) -> Self {
        Self {
            id: format!("city_{}", dto.id),
            name: format!("[City] {}", dto.name),
            kind: "location".to_string(),
            slug: "city".to_string(),
            uuid: String::new(),
        }
    }
}

// -- Faction DTO ------------------------------------------------------------

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct FactionDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_available_live: Option<bool>,
}

impl LiveAvailable for FactionDto {
    fn is_available_live(&self) -> bool { self.is_available_live.unwrap_or(false) }
}

impl From<&FactionDto> for UexResult {
    fn from(dto: &FactionDto) -> Self {
        Self {
            id: format!("faction_{}", dto.id),
            name: format!("[Faction] {}", dto.name),
            kind: "location".to_string(),
            slug: "faction".to_string(),
            uuid: String::new(),
        }
    }
}

// -- Company DTO ------------------------------------------------------------

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CompanyDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_available_live: Option<bool>,
}

impl LiveAvailable for CompanyDto {
    fn is_available_live(&self) -> bool { self.is_available_live.unwrap_or(false) }
}

impl From<&CompanyDto> for UexResult {
    fn from(dto: &CompanyDto) -> Self {
        Self {
            id: format!("company_{}", dto.id),
            name: format!("[Company] {}", dto.name),
            kind: "location".to_string(),
            slug: "company".to_string(),
            uuid: String::new(),
        }
    }
}