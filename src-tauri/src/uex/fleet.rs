use serde::Deserialize;

use super::client::UexClient;
use super::types::{deserialize_flexible_id, deserialize_bool_flag, deserialize_nonempty_string, HangarVehicle};

// ── API DTO ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct FleetVehicleDto {
    #[serde(deserialize_with = "deserialize_flexible_id")]
    id: String,
    #[serde(default, deserialize_with = "deserialize_flexible_id")]
    id_vehicle: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    model_name: String,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    serial: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty_string")]
    organization_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    is_hidden: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_flag")]
    is_pledged: Option<bool>,
    #[serde(default)]
    date_added: Option<serde_json::Value>,
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

// ── Public function ────────────────────────────────────────────────────────

/// Fetch the authenticated user's fleet from UEX.
pub async fn fetch_fleet(
    client: &UexClient,
    api_key: &str,
    secret_key: &str,
) -> Result<Vec<HangarVehicle>, String> {
    let dtos: Vec<FleetVehicleDto> = client
        .get_with_secret("/fleet", &[], api_key, secret_key)
        .await?;
    Ok(dtos.iter().map(HangarVehicle::from).collect())
}
