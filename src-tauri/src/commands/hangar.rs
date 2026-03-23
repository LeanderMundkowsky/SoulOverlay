use log::info;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::commands::backend::{extract_error_message, http_client};
use crate::constants::BACKEND_URL;
use crate::state::AppState;

// ── IPC types ──────────────────────────────────────────────────────────────

/// A vehicle in the user's fleet managed by the backend.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HangarVehicle {
    pub id: u32,
    pub uex_id: String,
    pub uex_vehicle_id: String,
    pub name: String,
    pub model_name: String,
    pub serial: Option<String>,
    pub description: Option<String>,
    pub organization_name: Option<String>,
    pub is_hidden: bool,
    pub is_pledged: bool,
    pub url_photo: Option<String>,
    pub date_added: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct FleetImportResult {
    pub fleet: Vec<HangarVehicle>,
    pub imported: u32,
    pub created: u32,
    pub updated: u32,
    pub removed: u32,
}

// ── Private DTO ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackendVehicle {
    id: u32,
    uex_id: String,
    uex_vehicle_id: String,
    name: String,
    model_name: String,
    serial: Option<String>,
    description: Option<String>,
    organization_name: Option<String>,
    is_hidden: bool,
    is_pledged: bool,
    url_photo: Option<String>,
    date_added: Option<String>,
    created_at: String,
    updated_at: String,
}

impl From<BackendVehicle> for HangarVehicle {
    fn from(v: BackendVehicle) -> Self {
        HangarVehicle {
            id: v.id,
            uex_id: v.uex_id,
            uex_vehicle_id: v.uex_vehicle_id,
            name: v.name,
            model_name: v.model_name,
            serial: v.serial,
            description: v.description,
            organization_name: v.organization_name,
            is_hidden: v.is_hidden,
            is_pledged: v.is_pledged,
            url_photo: v.url_photo,
            date_added: v.date_added,
            created_at: v.created_at,
            updated_at: v.updated_at,
        }
    }
}

fn get_token(state: &State<AppState>) -> Result<String, String> {
    let token = state.current_settings.lock().unwrap().backend_api_token.clone();
    if token.is_empty() {
        Err("Not logged in".to_string())
    } else {
        Ok(token)
    }
}

fn parse_vehicle(val: &serde_json::Value) -> Result<HangarVehicle, String> {
    serde_json::from_value::<BackendVehicle>(val.clone())
        .map(HangarVehicle::from)
        .map_err(|e| format!("Failed to parse fleet vehicle: {}", e))
}

/// Fetch the authenticated user's fleet from the backend.
#[tauri::command]
#[specta::specta]
pub async fn hangar_get_fleet(
    state: State<'_, AppState>,
) -> Result<Vec<HangarVehicle>, String> {
    let token = get_token(&state)?;
    let client = http_client()?;
    let url = format!("{}/api/fleet", BACKEND_URL);

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse response: {}", e))?;

    if !status.is_success() {
        return Err(extract_error_message(&json));
    }

    let data = json["data"].as_array().ok_or("Unexpected response shape")?;
    Ok(data.iter().filter_map(|v| parse_vehicle(v).ok()).collect())
}

/// Import fleet from UEX Corp via the backend.
#[tauri::command]
#[specta::specta]
pub async fn hangar_import_fleet(
    state: State<'_, AppState>,
) -> Result<FleetImportResult, String> {
    let token = get_token(&state)?;
    let client = http_client()?;
    let url = format!("{}/api/fleet/import", BACKEND_URL);

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse response: {}", e))?;

    if !status.is_success() {
        return Err(extract_error_message(&json));
    }

    let data = json["data"].as_array().ok_or("Unexpected response shape")?;
    let fleet: Vec<HangarVehicle> = data.iter().filter_map(|v| parse_vehicle(v).ok()).collect();
    let imported = json["imported"].as_u64().unwrap_or(0) as u32;
    let created = json["created"].as_u64().unwrap_or(0) as u32;
    let updated = json["updated"].as_u64().unwrap_or(0) as u32;
    let removed = json["removed"].as_u64().unwrap_or(0) as u32;

    info!("Fleet import complete: {} imported ({} created, {} updated, {} removed)", imported, created, updated, removed);
    Ok(FleetImportResult { fleet, imported, created, updated, removed })
}

/// Update a fleet vehicle's name and/or description.
#[tauri::command]
#[specta::specta]
pub async fn hangar_update_vehicle(
    id: u32,
    name: Option<String>,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<HangarVehicle, String> {
    let token = get_token(&state)?;
    let client = http_client()?;
    let url = format!("{}/api/fleet/{}", BACKEND_URL, id);

    let mut body = serde_json::Map::new();
    if let Some(n) = name {
        body.insert("name".to_string(), serde_json::Value::String(n));
    }
    body.insert("description".to_string(), match description {
        Some(d) => serde_json::Value::String(d),
        None => serde_json::Value::Null,
    });

    let resp = client
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::Value::Object(body))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse response: {}", e))?;

    if !status.is_success() {
        return Err(extract_error_message(&json));
    }

    parse_vehicle(&json["data"])
}

/// Delete a fleet vehicle.
#[tauri::command]
#[specta::specta]
pub async fn hangar_delete_vehicle(
    id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let token = get_token(&state)?;
    let client = http_client()?;
    let url = format!("{}/api/fleet/{}", BACKEND_URL, id);

    let resp = client
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if resp.status().is_success() || resp.status().as_u16() == 204 {
        info!("Deleted fleet vehicle {}", id);
        Ok(())
    } else {
        let json: serde_json::Value = resp.json().await.unwrap_or_default();
        Err(extract_error_message(&json))
    }
}

/// Manually add a ship to the fleet (without importing from UEX).
#[tauri::command]
#[specta::specta]
pub async fn hangar_add_vehicle(
    model_name: String,
    uex_vehicle_id: Option<String>,
    name: Option<String>,
    serial: Option<String>,
    description: Option<String>,
    is_pledged: bool,
    is_hidden: bool,
    state: State<'_, AppState>,
) -> Result<HangarVehicle, String> {
    let token = get_token(&state)?;
    let client = http_client()?;
    let url = format!("{}/api/fleet", BACKEND_URL);

    // The frontend passes a Wiki UUID as the vehicle identifier. Resolve it to the
    // UEX numeric vehicle ID so the backend can auto-fetch the photo URL from UEX.
    let resolved_uex_vehicle_id: Option<String> = uex_vehicle_id.as_deref()
        .filter(|id| !id.is_empty())
        .map(|uuid| {
            state.entity_mapper.lock().ok()
                .and_then(|m| m.vehicle_uuid_to_uex_id(uuid).map(|s| s.to_string()))
                .unwrap_or_else(|| uuid.to_string())
        });

    let mut body = serde_json::Map::new();
    body.insert("modelName".to_string(), serde_json::Value::String(model_name));
    body.insert("uexVehicleId".to_string(), match resolved_uex_vehicle_id {
        Some(id) => serde_json::Value::String(id),
        None => serde_json::Value::Null,
    });
    if let Some(n) = name {
        body.insert("name".to_string(), serde_json::Value::String(n));
    }
    if let Some(s) = serial {
        body.insert("serial".to_string(), serde_json::Value::String(s));
    }
    body.insert("description".to_string(), match description {
        Some(d) => serde_json::Value::String(d),
        None => serde_json::Value::Null,
    });
    body.insert("isPledged".to_string(), serde_json::Value::Bool(is_pledged));
    body.insert("isHidden".to_string(), serde_json::Value::Bool(is_hidden));

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::Value::Object(body))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !status.is_success() {
        return Err(extract_error_message(&json));
    }

    let vehicle = parse_vehicle(&json["data"])?;
    info!("Added fleet vehicle: {} ({})", vehicle.model_name, vehicle.id);
    Ok(vehicle)
}
