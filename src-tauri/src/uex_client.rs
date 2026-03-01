use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// In-memory cache entry
struct CacheEntry {
    data: serde_json::Value,
    inserted_at: Instant,
}

/// Simple TTL cache for UEX API responses
pub struct UexCache {
    entries: HashMap<String, CacheEntry>,
    ttl_secs: u64,
}

impl UexCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            entries: HashMap::new(),
            ttl_secs,
        }
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.entries.get(key).and_then(|entry| {
            if entry.inserted_at.elapsed().as_secs() < self.ttl_secs {
                Some(&entry.data)
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, key: String, data: serde_json::Value) {
        self.entries.insert(
            key,
            CacheEntry {
                data,
                inserted_at: Instant::now(),
            },
        );
    }

    /// Remove expired entries
    pub fn cleanup(&mut self) {
        self.entries
            .retain(|_, entry| entry.inserted_at.elapsed().as_secs() < self.ttl_secs);
    }
}

/// A search result from UEX API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UexResult {
    pub id: String,
    pub name: String,
    pub kind: String, // "commodity", "ship", "item", etc.
    pub slug: String,
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

/// UEX API response wrapper
#[derive(Debug, Deserialize)]
struct UexApiResponse<T> {
    pub status: Option<String>,
    pub data: Option<T>,
}

const UEX_BASE_URL: &str = "https://uexcorp.space/api/2.0";

/// Search UEX for commodities/items by query string
pub async fn search(query: &str, api_key: &str) -> Result<Vec<UexResult>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/commodities", UEX_BASE_URL);

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .query(&[("name_filter", query)])
        .send()
        .await
        .map_err(|e| format!("UEX request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("UEX API returned status: {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse UEX response: {}", e))?;

    // Parse the response - UEX API returns data in various formats
    let mut results = Vec::new();

    if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
        for item in data {
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

            // Filter by query (case insensitive)
            if name.to_lowercase().contains(&query.to_lowercase()) {
                results.push(UexResult {
                    id,
                    name,
                    kind,
                    slug,
                });
            }
        }
    }

    Ok(results)
}

/// Get prices for a specific commodity from UEX
pub async fn get_prices(commodity_id: &str, api_key: &str) -> Result<Vec<PriceEntry>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/commodities_prices", UEX_BASE_URL);

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .query(&[("id_commodity", commodity_id)])
        .send()
        .await
        .map_err(|e| format!("UEX request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("UEX API returned status: {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse UEX response: {}", e))?;

    let mut entries = Vec::new();

    if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
        for item in data {
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

            entries.push(PriceEntry {
                location,
                terminal,
                buy_price,
                sell_price,
                scu_available,
                date_updated,
            });
        }
    }

    Ok(entries)
}
