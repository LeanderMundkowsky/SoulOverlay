use serde::Deserialize;

use crate::uex::types::{
    deserialize_bool_flag, deserialize_flexible_id, deserialize_nonempty_string, HangarVehicle,
};

#[derive(Deserialize)]
pub struct FleetVehicleDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    #[serde(default, deserialize_with = "deserialize_flexible_id")]
    pub id_vehicle: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub model_name: String,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub serial: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    pub organization_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_hidden: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    pub is_pledged: Option<bool>,
    #[serde(default)]
    pub date_added: Option<serde_json::Value>,
}

impl From<&FleetVehicleDto> for HangarVehicle {
    fn from(dto: &FleetVehicleDto) -> Self {
        let date = dto
            .date_added
            .as_ref()
            .map(|v| match v {
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s.clone(),
                _ => String::new(),
            })
            .unwrap_or_default();

        Self {
            id: dto.id.clone(),
            id_vehicle: dto.id_vehicle.clone(),
            name: dto.name.clone(),
            model_name: dto.model_name.clone(),
            serial: dto.serial.clone(),
            description: dto.description.clone(),
            organization_name: dto.organization_name.clone(),
            is_hidden: dto.is_hidden.unwrap_or(false),
            is_pledged: dto.is_pledged.unwrap_or(false),
            date_added: date,
            url_photo: None,
        }
    }
}
