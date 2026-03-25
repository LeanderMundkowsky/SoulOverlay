use log::{debug, error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use chrono::Utc;
use crate::backend_log::BackendCallEntry;
use crate::constants::BACKEND_URL;
use crate::state::AppState;

// ── IPC types ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BackendAccount {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub uex_secret_key: Option<String>,
    pub roles: Vec<String>,
    pub created_at: String,
}

/// Returned by `backend_get_account`.
/// Lets the frontend distinguish: never-logged-in (token_present=false),
/// session-expired (token_present=true, account=None), or logged-in (account=Some).
#[derive(Debug, Clone, Serialize, Type)]
pub struct BackendAccountStatus {
    pub account: Option<BackendAccount>,
    pub token_present: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct BackendAuthResult {
    pub account: BackendAccount,
}

// ── Private DTO types (not in IPC) ────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackendAccountDto {
    id: u32,
    username: String,
    email: String,
    uex_secret_key: Option<String>,
    roles: Vec<String>,
    created_at: String,
}

impl From<BackendAccountDto> for BackendAccount {
    fn from(dto: BackendAccountDto) -> Self {
        BackendAccount {
            id: dto.id,
            username: dto.username,
            email: dto.email,
            uex_secret_key: dto.uex_secret_key,
            roles: dto.roles,
            created_at: dto.created_at,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub(crate) fn http_client() -> Result<Client, String> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

/// Fetch account from the backend using a stored bearer token.
/// Returns None if the token is invalid/expired or the request fails.
pub async fn fetch_account_with_token(token: &str) -> Option<BackendAccount> {
    let client = http_client().ok()?;
    let url = format!("{}/api/account", BACKEND_URL);
    debug!("[backend] → GET /api/account");
    let t = std::time::Instant::now();
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .ok()?;
    debug!("[backend] ← GET /api/account {} ({}ms)", resp.status(), t.elapsed().as_millis());

    if resp.status() == 401 {
        info!("Backend token invalid or expired (401)");
        return None;
    }
    if !resp.status().is_success() {
        error!("Backend account fetch failed with status: {}", resp.status());
        return None;
    }

    #[derive(Deserialize)]
    struct Wrapper { data: BackendAccountDto }

    resp.json::<Wrapper>().await.ok().map(|w| BackendAccount::from(w.data))
}

/// Called on app startup to restore the session if a token is persisted.
/// If the access token is invalid/expired, silently attempts to refresh it.
pub async fn fetch_account_on_startup(handle: &tauri::AppHandle) {
    use tauri::Manager;
    let state = handle.state::<AppState>();

    let (token, refresh_token) = {
        let s = state.current_settings.lock().unwrap();
        (s.backend_api_token.clone(), s.backend_refresh_token.clone())
    };

    if token.is_empty() && refresh_token.is_empty() {
        return; // Never logged in
    }

    // Try the stored access token first
    if !token.is_empty() {
        if let Some(account) = fetch_account_with_token(&token).await {
            info!("Backend session restored on startup: {}", account.username);
            *state.backend_account.lock().unwrap() = Some(account);
            return;
        }
        info!("Backend access token invalid — attempting refresh");
    } else {
        info!("No access token — attempting session restore via refresh token");
    }

    // Access token absent or expired — try the refresh token
    if refresh_token.is_empty() {
        info!("No refresh token available — login required");
        return;
    }
    match try_refresh_tokens(&state).await {
        Ok(new_token) => {
            if let Some(account) = fetch_account_with_token(&new_token).await {
                info!("Backend session restored via refresh token: {}", account.username);
                *state.backend_account.lock().unwrap() = Some(account);
            } else {
                info!("Refresh succeeded but account fetch failed");
            }
        }
        Err(e) => info!("Session refresh failed on startup: {}", e),
    }
}

/// Check backend health. Returns true if the backend is reachable and healthy.
pub async fn check_backend_status() -> bool {
    let Ok(client) = http_client() else { return false; };
    let url = format!("{}/api/status", BACKEND_URL);
    debug!("[backend] → GET /api/status");
    let t = std::time::Instant::now();
    match client.get(&url).send().await {
        Ok(resp) => {
            debug!("[backend] ← GET /api/status {} ({}ms)", resp.status(), t.elapsed().as_millis());
            resp.status().is_success()
        }
        Err(e) => {
            debug!("[backend] ✗ GET /api/status: {} ({}ms)", e, t.elapsed().as_millis());
            false
        }
    }
}

pub(crate) fn extract_error_message(json: &serde_json::Value) -> String {
    // Symfony API Platform style: violations array
    if let Some(violations) = json.get("violations").and_then(|v| v.as_array()) {
        let msgs: Vec<String> = violations
            .iter()
            .filter_map(|v| {
                let field = v.get("propertyPath").and_then(|f| f.as_str()).unwrap_or("");
                let msg = v.get("message").and_then(|m| m.as_str())?;
                if field.is_empty() {
                    Some(msg.to_string())
                } else {
                    Some(format!("{}: {}", field, msg))
                }
            })
            .collect();
        if !msgs.is_empty() {
            return msgs.join(", ");
        }
    }
    // API Platform detail field
    if let Some(detail) = json.get("detail").and_then(|d| d.as_str()) {
        if !detail.is_empty() {
            return detail.to_string();
        }
    }
    // Generic message field
    if let Some(msg) = json.get("message").and_then(|m| m.as_str()) {
        if !msg.is_empty() {
            return msg.to_string();
        }
    }
    // Generic error field (used by our backend for simple error responses)
    if let Some(err) = json.get("error").and_then(|e| e.as_str()) {
        if !err.is_empty() {
            return err.to_string();
        }
    }
    "Unknown error".to_string()
}

fn save_tokens(state: &AppState, token: String, refresh_token: String) {
    let mut settings = state.current_settings.lock().unwrap();
    settings.backend_api_token = token;
    settings.backend_refresh_token = refresh_token;
    if let Err(e) = state.paths.save_settings(&settings) {
        error!("Failed to persist tokens: {}", e);
    }
}

/// Returns the stored access token or `Err("Not logged in")` if empty.
pub fn get_stored_token(state: &AppState) -> Result<String, String> {
    let token = state.current_settings.lock().unwrap().backend_api_token.clone();
    if token.is_empty() {
        Err("Not logged in".to_string())
    } else {
        Ok(token)
    }
}

/// Exchanges the stored refresh token for a new access+refresh token pair.
/// Saves the new tokens to settings and returns the new access token.
/// Clears `backend_account` and returns `Err` if the refresh token is missing or rejected.
pub async fn try_refresh_tokens(state: &AppState) -> Result<String, String> {
    let refresh_token = state.current_settings.lock().unwrap().backend_refresh_token.clone();
    if refresh_token.is_empty() {
        *state.backend_account.lock().unwrap() = None;
        return Err("Session expired. Please log in again.".to_string());
    }

    let client = http_client()?;
    let url = format!("{}/api/auth/refresh", BACKEND_URL);
    debug!("[backend] → POST /api/auth/refresh");
    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "refreshToken": refresh_token }))
        .send()
        .await
        .map_err(|e| format!("Refresh request failed: {}", e))?;

    if !resp.status().is_success() {
        info!("Token refresh rejected ({})", resp.status());
        *state.backend_account.lock().unwrap() = None;
        return Err("Session expired. Please log in again.".to_string());
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse refresh response: {}", e))?;

    let new_token = json["data"]["apiToken"]
        .as_str()
        .ok_or("Invalid refresh response: missing apiToken")?
        .to_string();
    let new_refresh = json["data"]["refreshToken"]
        .as_str()
        .ok_or("Invalid refresh response: missing refreshToken")?
        .to_string();

    save_tokens(state, new_token.clone(), new_refresh);
    info!("Access token refreshed silently");
    Ok(new_token)
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Login with username + password. Returns account info on success.
#[tauri::command]
#[specta::specta]
pub async fn backend_login(
    username: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<BackendAuthResult, String> {
    let client = http_client()?;
    let url = format!("{}/api/auth/login", BACKEND_URL);
    debug!("[backend] → POST /api/auth/login");
    let t = std::time::Instant::now();
    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "username": username, "password": password }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    debug!("[backend] ← POST /api/auth/login {} ({}ms)", status, t.elapsed().as_millis());
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    state.backend_call_log.record(BackendCallEntry {
        method: "POST".into(),
        path: "/api/auth/login".into(),
        status: Some(status.as_u16()),
        duration_ms: t.elapsed().as_millis() as u32,
        error: if !status.is_success() { Some(extract_error_message(&json)) } else { None },
        timestamp: Utc::now().to_rfc3339(),
    });

    if !status.is_success() {
        return Err(extract_error_message(&json));
    }

    let token = json["data"]["apiToken"]
        .as_str()
        .ok_or("Invalid response: missing apiToken")?
        .to_string();
    let refresh_token = json["data"]["refreshToken"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let account = fetch_account_with_token(&token)
        .await
        .ok_or("Login succeeded but failed to fetch account")?;

    save_tokens(&state, token, refresh_token);
    *state.backend_account.lock().unwrap() = Some(account.clone());

    info!("User logged in: {}", account.username);
    Ok(BackendAuthResult { account })
}

/// Register a new account. Returns account info on success.
#[tauri::command]
#[specta::specta]
pub async fn backend_register(
    username: String,
    email: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<BackendAuthResult, String> {
    let client = http_client()?;
    let url = format!("{}/api/auth/register", BACKEND_URL);
    debug!("[backend] → POST /api/auth/register");
    let t = std::time::Instant::now();
    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "username": username, "email": email, "password": password }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    debug!("[backend] ← POST /api/auth/register {} ({}ms)", status, t.elapsed().as_millis());
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    state.backend_call_log.record(BackendCallEntry {
        method: "POST".into(),
        path: "/api/auth/register".into(),
        status: Some(status.as_u16()),
        duration_ms: t.elapsed().as_millis() as u32,
        error: if !status.is_success() { Some(extract_error_message(&json)) } else { None },
        timestamp: Utc::now().to_rfc3339(),
    });

    if !status.is_success() {
        return Err(extract_error_message(&json));
    }

    let token = json["data"]["apiToken"]
        .as_str()
        .ok_or("Invalid response: missing apiToken")?
        .to_string();
    let refresh_token = json["data"]["refreshToken"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let account = fetch_account_with_token(&token)
        .await
        .ok_or("Registration succeeded but failed to fetch account")?;

    save_tokens(&state, token, refresh_token);
    *state.backend_account.lock().unwrap() = Some(account.clone());

    info!("User registered: {}", account.username);
    Ok(BackendAuthResult { account })
}

/// Returns the current account status (logged-in, session-expired, or never-logged-in).
#[tauri::command]
#[specta::specta]
pub async fn backend_get_account(
    state: State<'_, AppState>,
) -> Result<BackendAccountStatus, String> {
    let token_present = !state.current_settings.lock().unwrap().backend_api_token.is_empty();
    let account = state.backend_account.lock().unwrap().clone();
    Ok(BackendAccountStatus { account, token_present })
}

/// Update the UEX secret key stored on the backend account.
#[tauri::command]
#[specta::specta]
pub async fn backend_update_secret_key(
    uex_secret_key: Option<String>,
    state: State<'_, AppState>,
) -> Result<BackendAccount, String> {
    let mut token = get_stored_token(&state)?;

    let client = http_client()?;
    let url = format!("{}/api/account", BACKEND_URL);
    let body = serde_json::json!({ "uexSecretKey": uex_secret_key });

    debug!("[backend] → PATCH /api/account");
    let t = std::time::Instant::now();

    let mut refreshed = false;
    let (status, json) = loop {
        let resp = client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/merge-patch+json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        let s = resp.status();
        if s == reqwest::StatusCode::UNAUTHORIZED && !refreshed {
            token = try_refresh_tokens(&state).await?;
            refreshed = true;
            continue;
        }
        let j: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        break (s, j);
    };

    debug!("[backend] ← PATCH /api/account {} ({}ms)", status, t.elapsed().as_millis());
    state.backend_call_log.record(BackendCallEntry {
        method: "PATCH".into(),
        path: "/api/account".into(),
        status: Some(status.as_u16()),
        duration_ms: t.elapsed().as_millis() as u32,
        error: if !status.is_success() { Some(extract_error_message(&json)) } else { None },
        timestamp: Utc::now().to_rfc3339(),
    });

    if !status.is_success() {
        return Err(extract_error_message(&json));
    }

    // Re-fetch to ensure local state matches backend
    let account = fetch_account_with_token(&token)
        .await
        .ok_or("Secret key updated but failed to refresh account")?;
    *state.backend_account.lock().unwrap() = Some(account.clone());

    Ok(account)
}

/// Log out: clears both stored tokens and in-memory account.
#[tauri::command]
#[specta::specta]
pub async fn backend_logout(state: State<'_, AppState>) -> Result<(), String> {
    save_tokens(&state, String::new(), String::new());
    *state.backend_account.lock().unwrap() = None;
    info!("User logged out");
    Ok(())
}

// ── Home Locations ────────────────────────────────────────────────────────────

/// A curated home location available for selection.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HomeLocationOption {
    /// Backend HomeLocation entity ID.
    pub id: u32,
    /// Location name (e.g. "New Babbage").
    pub name: String,
    /// UEX platform ID as string (used as `location_id` in inventory). None if not mapped.
    pub uex_id: Option<String>,
    /// Location type name — matches inventory storage slugs (e.g. "city", "space_station").
    pub type_name: String,
    /// Star system name (e.g. "Stanton").
    pub system_name: String,
}

/// Fetch all active home locations from the backend (public endpoint, no auth required).
#[tauri::command]
#[specta::specta]
pub async fn backend_get_home_locations() -> Result<Vec<HomeLocationOption>, String> {
    let client = http_client()?;
    let url = format!("{}/api/home-locations", BACKEND_URL);
    debug!("[backend] → GET /api/home-locations");
    let t = std::time::Instant::now();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = resp.status();
    debug!("[backend] ← GET /api/home-locations {} ({}ms)", status, t.elapsed().as_millis());

    if !status.is_success() {
        return Err(format!("Failed to fetch home locations: HTTP {}", status));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let items = json["data"]
        .as_array()
        .ok_or("Invalid response: missing data array")?;

    let mut result = Vec::with_capacity(items.len());
    for item in items {
        let id = item["id"].as_u64().ok_or("Missing id")? as u32;
        let name = item["name"].as_str().ok_or("Missing name")?.to_string();
        let uex_id = item["uexId"].as_i64().map(|n| n.to_string());
        let type_name = item["type"]["name"].as_str().ok_or("Missing type.name")?.to_string();
        let system_name = item["system"]["name"].as_str().ok_or("Missing system.name")?.to_string();
        result.push(HomeLocationOption { id, name, uex_id, type_name, system_name });
    }

    Ok(result)
}

/// Returns the currently saved home location ID from settings.
#[tauri::command]
#[specta::specta]
pub async fn get_home_location_id(state: State<'_, AppState>) -> Result<Option<u32>, String> {
    Ok(state.current_settings.lock().unwrap().home_location_id)
}

/// Persists the chosen home location ID to settings.
#[tauri::command]
#[specta::specta]
pub async fn set_home_location_id(
    id: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut settings = state.current_settings.lock().unwrap();
    settings.home_location_id = id;
    state
        .paths
        .save_settings(&settings)
        .map_err(|e| format!("Failed to save settings: {}", e))
}
