use serde::de::DeserializeOwned;
use serde::Deserialize;

/// Base URL for the UEX Corp API v2.
/// Note: The swagger spec lists `https://api.uexcorp.space/2.0` but the
/// legacy URL below also works and is what the app has used historically.
const UEX_BASE_URL: &str = "https://uexcorp.space/api/2.0";

/// Shared HTTP client for UEX API requests.
/// Holds a `reqwest::Client` for connection pooling across all requests.
/// Clone is cheap — `reqwest::Client` is `Arc`-backed internally.
#[derive(Clone)]
pub struct UexClient {
    http: reqwest::Client,
}

impl UexClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    /// Expose the underlying reqwest client for non-UEX HTTP requests.
    pub fn client(&self) -> &reqwest::Client {
        &self.http
    }

    /// Send a GET request and deserialize the `data` array from the response.
    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
        api_key: &str,
    ) -> Result<Vec<T>, String> {
        let body = self.get_raw(path, query, api_key).await?;
        let wrapper: UexApiResponse<T> =
            serde_json::from_value(body).map_err(|e| format!("Failed to parse UEX response: {}", e))?;
        Ok(wrapper.data.unwrap_or_default())
    }

    /// Send a GET request and return the raw JSON body.
    pub(crate) async fn get_raw(
        &self,
        path: &str,
        query: &[(&str, &str)],
        api_key: &str,
    ) -> Result<serde_json::Value, String> {
        self.get_raw_inner(path, query, api_key, None).await
    }

    /// Send a GET request with secret-key header and return the raw JSON body.
    /// Checks the UEX `status` field for auth errors before returning.
    pub(crate) async fn get_raw_with_secret(
        &self,
        path: &str,
        query: &[(&str, &str)],
        api_key: &str,
        secret_key: &str,
    ) -> Result<serde_json::Value, String> {
        let body = self.get_raw_inner(path, query, api_key, Some(secret_key)).await?;
        if let Some(status) = body.get("status").and_then(|v| v.as_str()) {
            if status != "ok" {
                return Err(format!("UEX API error: {}", status));
            }
        }
        Ok(body)
    }

    /// Send a GET request with both Bearer token and secret-key header.
    /// Checks the UEX `status` field for auth errors before parsing data.
    pub(crate) async fn get_with_secret<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
        api_key: &str,
        secret_key: &str,
    ) -> Result<Vec<T>, String> {
        let body = self.get_raw_inner(path, query, api_key, Some(secret_key)).await?;

        // UEX returns 200 even for auth errors — check the status field
        if let Some(status) = body.get("status").and_then(|v| v.as_str()) {
            if status != "ok" {
                return Err(format!("UEX API error: {}", status));
            }
        }

        let wrapper: UexApiResponse<T> =
            serde_json::from_value(body).map_err(|e| format!("Failed to parse UEX response: {}", e))?;
        Ok(wrapper.data.unwrap_or_default())
    }

    /// Internal: shared GET logic with optional secret-key header.
    async fn get_raw_inner(
        &self,
        path: &str,
        query: &[(&str, &str)],
        api_key: &str,
        secret_key: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}{}", UEX_BASE_URL, path);

        let mut req = self.http.get(&url).query(query);
        if !api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        if let Some(sk) = secret_key {
            if !sk.is_empty() {
                req = req.header("secret-key", sk);
            }
        }

        // Build display URL for logging
        let display_url = if query.is_empty() {
            url.clone()
        } else {
            let params = query
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}?{}", url, params)
        };
        log::debug!("UEX GET {} (api_key={})", display_url, !api_key.is_empty());

        let resp = req
            .send()
            .await
            .map_err(|e| format!("UEX request failed: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let body_text = resp
                .text()
                .await
                .unwrap_or_else(|_| "<unreadable body>".to_string());
            log::warn!("UEX GET {} → {} | body: {}", display_url, status, body_text);
            return Err(format!(
                "UEX API returned status: {} — {}",
                status, body_text
            ));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse UEX response: {}", e))
    }
}

/// Generic UEX API response wrapper. All endpoints return `{ "data": [...] }`.
#[derive(Deserialize)]
struct UexApiResponse<T> {
    data: Option<Vec<T>>,
}
