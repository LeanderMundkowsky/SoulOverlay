use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

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

fn http_client() -> Result<Client, String> {
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
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .ok()?;

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
pub async fn fetch_account_on_startup(handle: &tauri::AppHandle) {
    use tauri::Manager;
    let state = handle.state::<AppState>();
    let token = state.current_settings.lock().unwrap().backend_api_token.clone();
    if token.is_empty() {
        return;
    }
    if let Some(account) = fetch_account_with_token(&token).await {
        info!("Backend session restored on startup: {}", account.username);
        *state.backend_account.lock().unwrap() = Some(account);
    } else {
        info!("Backend token present but account fetch failed — token may be expired");
    }
}

/// Check backend health. Returns true if the backend is reachable and healthy.
pub async fn check_backend_status() -> bool {
    let Ok(client) = http_client() else { return false; };
    let url = format!("{}/api/status", BACKEND_URL);
    match client.get(&url).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

fn extract_error_message(json: &serde_json::Value) -> String {
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
    "Unknown error".to_string()
}

fn save_token(state: &AppState, token: String) {
    let mut settings = state.current_settings.lock().unwrap();
    settings.backend_api_token = token;
    if let Err(e) = state.paths.save_settings(&settings) {
        error!("Failed to persist backend token: {}", e);
    }
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

    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "username": username, "password": password }))
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

    let token = json["data"]["api_token"]
        .as_str()
        .ok_or("Invalid response: missing api_token")?
        .to_string();

    let account = fetch_account_with_token(&token)
        .await
        .ok_or("Login succeeded but failed to fetch account")?;

    save_token(&state, token);
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

    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "username": username, "email": email, "password": password }))
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

    let token = json["data"]["api_token"]
        .as_str()
        .ok_or("Invalid response: missing api_token")?
        .to_string();

    let account = fetch_account_with_token(&token)
        .await
        .ok_or("Registration succeeded but failed to fetch account")?;

    save_token(&state, token);
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
    let token = state.current_settings.lock().unwrap().backend_api_token.clone();
    if token.is_empty() {
        return Err("Not logged in".to_string());
    }

    let client = http_client()?;
    let url = format!("{}/api/account", BACKEND_URL);

    let resp = client
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/merge-patch+json")
        .json(&serde_json::json!({ "uex_secret_key": uex_secret_key }))
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

    // Re-fetch to ensure local state matches backend
    let account = fetch_account_with_token(&token)
        .await
        .ok_or("Secret key updated but failed to refresh account")?;
    *state.backend_account.lock().unwrap() = Some(account.clone());

    Ok(account)
}

/// Log out: clears the stored token and in-memory account.
#[tauri::command]
#[specta::specta]
pub async fn backend_logout(state: State<'_, AppState>) -> Result<(), String> {
    save_token(&state, String::new());
    *state.backend_account.lock().unwrap() = None;
    info!("User logged out");
    Ok(())
}
