use serde::{Deserialize, Serialize};

use crate::uex::types::{deserialize_bool_flag, deserialize_flexible_id, deserialize_nonempty_string, deserialize_optional_id, LocationTerminal, UexResult};

/// Trait for location DTOs that have an `is_available_live` field.
pub(crate) trait LiveAvailable {
    fn is_available_live(&self) -> bool;
}

/// Internal cache struct for terminal hierarchy data (not an IPC type).
/// Stored alongside the Locations cache to resolve terminals by parent location.
#[derive(Serialize, Deserialize)]
pub struct TerminalHierarchy {
    pub id: String,
    pub name: String,
    pub nickname: String,
    pub system_name: String,
    pub planet_name: String,
    pub orbit_name: String,
    /// Most specific parent location (station > city > outpost > orbit)
    pub location_name: String,
    pub id_star_system: String,
    pub id_planet: String,
    pub id_moon: String,
    pub id_orbit: String,
    pub id_space_station: String,
    pub id_outpost: String,
    pub id_city: String,
    pub id_poi: String,
}

impl TerminalHierarchy {
    pub fn from_dto(dto: &TerminalDto) -> Self {
        // Resolve most specific parent: station > city > outpost > orbit
        let location_name = dto.space_station_name.clone()
            .or_else(|| dto.city_name.clone())
            .or_else(|| dto.outpost_name.clone())
            .or_else(|| dto.orbit_name.clone())
            .unwrap_or_default();

        Self {
            id: dto.id.clone(),
            name: dto.name.clone(),
            nickname: dto.nickname.clone().unwrap_or_default(),
            system_name: dto.star_system_name.clone().unwrap_or_default(),
            planet_name: dto.planet_name.clone().unwrap_or_default(),
            orbit_name: dto.orbit_name.clone().unwrap_or_default(),
            location_name,
            id_star_system: dto.id_star_system.clone().unwrap_or_default(),
            id_planet: dto.id_planet.clone().unwrap_or_default(),
            id_moon: dto.id_moon.clone().unwrap_or_default(),
            id_orbit: dto.id_orbit.clone().unwrap_or_default(),
            id_space_station: dto.id_space_station.clone().unwrap_or_default(),
            id_outpost: dto.id_outpost.clone().unwrap_or_default(),
            id_city: dto.id_city.clone().unwrap_or_default(),
            id_poi: dto.id_poi.clone().unwrap_or_default(),
        }
    }

    pub fn to_location_terminal(&self) -> LocationTerminal {
        LocationTerminal {
            id: self.id.clone(),
            name: self.name.clone(),
            nickname: self.nickname.clone(),
            system_name: self.system_name.clone(),
            planet_name: self.planet_name.clone(),
            orbit_name: self.orbit_name.clone(),
        }
    }

    /// Check if this terminal belongs to the given location (slug + raw ID).
    pub fn matches_location(&self, slug: &str, raw_id: &str) -> bool {
        match slug {
            "star_system" => self.id_star_system == raw_id,
            "planet" => self.id_planet == raw_id,
            "moon" => self.id_moon == raw_id,
            "orbit" => self.id_orbit == raw_id,
            "space_station" => self.id_space_station == raw_id,
            "outpost" => self.id_outpost == raw_id,
            "city" => self.id_city == raw_id,
            "poi" => self.id_poi == raw_id,
            "terminal" => self.id == raw_id,
            _ => false,
        }
    }
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
    pub moon_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub space_station_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub city_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub outpost_name: Option<String>,
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
        }
    }
}