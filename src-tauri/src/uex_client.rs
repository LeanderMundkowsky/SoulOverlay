use serde::{Deserialize, Serialize};

/// A search result from UEX API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UexResult {
    pub id: String,
    pub name: String,
    pub kind: String, // "commodity", "ship", "item", etc.
    pub slug: String,
}

impl UexResult {
    /// Parse a single JSON object from the UEX API `data` array into a `UexResult`.
    fn from_json(item: &serde_json::Value) -> Self {
        let id = item
            .get("id")
            .and_then(|v| v.as_u64())
            .map(|v| v.to_string())
            .or_else(|| item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .unwrap_or_default();

        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let slug = item
            .get("slug")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let kind = item
            .get("type")
            .or_else(|| item.get("kind"))
            .and_then(|v| v.as_str())
            .unwrap_or("commodity")
            .to_string();

        Self { id, name, kind, slug }
    }
}

/// A price entry from UEX API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceEntry {
    pub location: String,
    pub terminal: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub scu_available: Option<f64>,
    pub date_updated: String,
}

impl PriceEntry {
    /// Parse a single JSON object from the UEX API `data` array into a `PriceEntry`.
    fn from_json(item: &serde_json::Value) -> Self {
        let location = item
            .get("star_system_name")
            .or_else(|| item.get("planet_name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let terminal = item
            .get("terminal_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let buy_price = item
            .get("price_buy")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let sell_price = item
            .get("price_sell")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let scu_available = item
            .get("scu_buy")
            .or_else(|| item.get("scu_sell"))
            .and_then(|v| v.as_f64());

        let date_updated = item
            .get("date_modified")
            .or_else(|| item.get("date_added"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Self {
            location,
            terminal,
            buy_price,
            sell_price,
            scu_available,
            date_updated,
        }
    }
}

const UEX_BASE_URL: &str = "https://uexcorp.space/api/2.0";

/// Send a GET request to the UEX API and return the parsed JSON body.
async fn uex_get(url: &str, query: &[(&str, &str)], api_key: &str) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let mut req = client.get(url).query(query);
    if !api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", api_key));
    }

    // Build a display URL with query params for logging
    let display_url = if query.is_empty() {
        url.to_string()
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
        // Read the body to surface the UEX error message
        let body_text = resp.text().await.unwrap_or_else(|_| "<unreadable body>".to_string());
        log::warn!(
            "UEX GET {} → {} | body: {}",
            display_url,
            status,
            body_text
        );
        return Err(format!("UEX API returned status: {} — {}", status, body_text));
    }

    resp.json()
        .await
        .map_err(|e| format!("Failed to parse UEX response: {}", e))
}

/// Extract an array of items from the `data` field of a UEX API response.
fn extract_data_array(body: &serde_json::Value) -> Vec<&serde_json::Value> {
    body.get("data")
        .and_then(|d| d.as_array())
        .map(|a| a.iter().collect())
        .unwrap_or_default()
}

/// Extract results from a UEX API response body, applying a name filter client-side.
fn extract_results(body: &serde_json::Value, query_lower: &str, kind_override: Option<&str>) -> Vec<UexResult> {
    body.get("data")
        .and_then(|d| d.as_array())
        .map(|data| {
            data.iter()
                .map(|item| {
                    let mut r = UexResult::from_json(item);
                    if let Some(k) = kind_override {
                        r.kind = k.to_string();
                    }
                    r
                })
                .filter(|r| r.name.to_lowercase().contains(query_lower))
                .collect()
        })
        .unwrap_or_default()
}

// ── Fetch-all functions (full collection download for cache) ───────────────

/// Fetch ALL commodities from UEX. Returns parsed `UexResult` list.
pub async fn fetch_all_commodities(api_key: &str) -> Result<Vec<UexResult>, String> {
    let url = format!("{}/commodities", UEX_BASE_URL);
    let body = uex_get(&url, &[], api_key).await?;
    let results = extract_data_array(&body)
        .into_iter()
        .map(|item| {
            let mut r = UexResult::from_json(item);
            r.kind = "commodity".to_string();
            r
        })
        .collect();
    Ok(results)
}

/// Fetch ALL vehicles from UEX. Returns parsed `UexResult` list.
pub async fn fetch_all_vehicles(api_key: &str) -> Result<Vec<UexResult>, String> {
    let url = format!("{}/vehicles", UEX_BASE_URL);
    let body = uex_get(&url, &[], api_key).await?;
    let results = extract_data_array(&body)
        .into_iter()
        .map(|item| {
            let id = item
                .get("id")
                .and_then(|v| v.as_u64())
                .map(|v| v.to_string())
                .or_else(|| item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .unwrap_or_default();

            let name = item
                .get("name_full")
                .or_else(|| item.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            let slug = item
                .get("slug")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let kind = if item.get("is_ground_vehicle").and_then(|v| v.as_u64()).unwrap_or(0) == 1 {
                "ground vehicle"
            } else {
                "vehicle"
            };

            UexResult { id, name, kind: kind.to_string(), slug }
        })
        .collect();
    Ok(results)
}

/// Fetch ALL items from UEX. Returns parsed `UexResult` list.
pub async fn fetch_all_items(api_key: &str) -> Result<Vec<UexResult>, String> {
    let url = format!("{}/items", UEX_BASE_URL);
    let body = uex_get(&url, &[], api_key).await?;
    let results = extract_data_array(&body)
        .into_iter()
        .map(|item| {
            let mut r = UexResult::from_json(item);
            r.kind = "item".to_string();
            r
        })
        .collect();
    Ok(results)
}

/// Fetch ALL locations (terminals) from UEX. Returns parsed `UexResult` list.
pub async fn fetch_all_locations(api_key: &str) -> Result<Vec<UexResult>, String> {
    let url = format!("{}/terminals", UEX_BASE_URL);
    let body = uex_get(&url, &[], api_key).await?;
    let results = extract_data_array(&body)
        .into_iter()
        .map(|item| {
            let id = item
                .get("id")
                .and_then(|v| v.as_u64())
                .map(|v| v.to_string())
                .or_else(|| item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .unwrap_or_default();

            let name = item
                .get("displayname")
                .or_else(|| item.get("fullname"))
                .or_else(|| item.get("name"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .or_else(|| item.get("name").and_then(|v| v.as_str()))
                .unwrap_or("Unknown")
                .to_string();

            let slug = item
                .get("code")
                .or_else(|| item.get("slug"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            UexResult { id, name, kind: "location".to_string(), slug }
        })
        .collect();
    Ok(results)
}

// ── Search functions (query-based, used by commands) ───────────────────────

/// Search commodities from a cached collection by filtering in-memory.
pub fn search_in_collection(collection: &[UexResult], query: &str) -> Vec<UexResult> {
    let query_lower = query.to_lowercase();
    collection
        .iter()
        .filter(|r| r.name.to_lowercase().contains(&query_lower))
        .cloned()
        .collect()
}

/// Search UEX for commodities by query string (direct API call, no cache).
pub async fn search_commodities(query: &str, api_key: &str) -> Result<Vec<UexResult>, String> {
    let url = format!("{}/commodities", UEX_BASE_URL);
    let body = uex_get(&url, &[("name_filter", query)], api_key).await?;
    let query_lower = query.to_lowercase();
    Ok(extract_results(&body, &query_lower, Some("commodity")))
}

/// Search UEX for vehicles (ships + ground vehicles) by query string (direct API call, no cache).
pub async fn search_vehicles(query: &str, api_key: &str) -> Result<Vec<UexResult>, String> {
    let url = format!("{}/vehicles", UEX_BASE_URL);
    let body = uex_get(&url, &[], api_key).await?;
    let query_lower = query.to_lowercase();

    let results = body
        .get("data")
        .and_then(|d| d.as_array())
        .map(|data| {
            data.iter()
                .map(|item| {
                    let id = item
                        .get("id")
                        .and_then(|v| v.as_u64())
                        .map(|v| v.to_string())
                        .or_else(|| item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                        .unwrap_or_default();

                    let name = item
                        .get("name_full")
                        .or_else(|| item.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    let slug = item
                        .get("slug")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let kind = if item.get("is_ground_vehicle").and_then(|v| v.as_u64()).unwrap_or(0) == 1 {
                        "ground vehicle"
                    } else {
                        "vehicle"
                    };

                    UexResult { id, name, kind: kind.to_string(), slug }
                })
                .filter(|r| r.name.to_lowercase().contains(&query_lower))
                .collect()
        })
        .unwrap_or_default();

    Ok(results)
}

/// Search UEX for items by query string (direct API call, no cache).
pub async fn search_items(query: &str, api_key: &str) -> Result<Vec<UexResult>, String> {
    let url = format!("{}/items", UEX_BASE_URL);
    let body = uex_get(&url, &[("name_filter", query)], api_key).await?;
    let query_lower = query.to_lowercase();
    Ok(extract_results(&body, &query_lower, Some("item")))
}

/// Search UEX for locations (terminals) by query string (direct API call, no cache).
pub async fn search_locations(query: &str, api_key: &str) -> Result<Vec<UexResult>, String> {
    let url = format!("{}/terminals", UEX_BASE_URL);
    let body = uex_get(&url, &[("name_filter", query)], api_key).await?;
    let query_lower = query.to_lowercase();

    let results = body
        .get("data")
        .and_then(|d| d.as_array())
        .map(|data| {
            data.iter()
                .map(|item| {
                    let id = item
                        .get("id")
                        .and_then(|v| v.as_u64())
                        .map(|v| v.to_string())
                        .or_else(|| item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                        .unwrap_or_default();

                    let name = item
                        .get("displayname")
                        .or_else(|| item.get("fullname"))
                        .or_else(|| item.get("name"))
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .or_else(|| item.get("name").and_then(|v| v.as_str()))
                        .unwrap_or("Unknown")
                        .to_string();

                    let slug = item
                        .get("code")
                        .or_else(|| item.get("slug"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    UexResult { id, name, kind: "location".to_string(), slug }
                })
                .filter(|r| r.name.to_lowercase().contains(&query_lower))
                .collect()
        })
        .unwrap_or_default();

    Ok(results)
}

/// Get prices for a specific commodity from UEX (direct API call, no cache).
pub async fn get_prices(commodity_id: &str, api_key: &str) -> Result<Vec<PriceEntry>, String> {
    let url = format!("{}/commodities_prices", UEX_BASE_URL);
    let body = uex_get(&url, &[("id_commodity", commodity_id)], api_key).await?;

    let entries = body
        .get("data")
        .and_then(|d| d.as_array())
        .map(|data| data.iter().map(PriceEntry::from_json).collect())
        .unwrap_or_default();

    Ok(entries)
}
