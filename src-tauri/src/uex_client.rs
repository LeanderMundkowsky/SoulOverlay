use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Mutex;
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

    /// Number of entries currently in the cache (including expired).
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Fetch-through cache: returns cached data if valid, otherwise calls `fetch_fn`,
/// stores the result, and returns it.
pub async fn cached_fetch<T, F, Fut>(
    cache: &Mutex<UexCache>,
    cache_key: &str,
    fetch_fn: F,
) -> Result<T, String>
where
    T: Serialize + for<'de> Deserialize<'de>,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    // Check cache
    {
        let c = cache.lock().unwrap();
        if let Some(cached) = c.get(cache_key) {
            if let Ok(results) = serde_json::from_value(cached.clone()) {
                return Ok(results);
            }
        }
    }

    let results = fetch_fn().await?;

    // Store in cache
    {
        let mut c = cache.lock().unwrap();
        if let Ok(json) = serde_json::to_value(&results) {
            c.insert(cache_key.to_string(), json);
        }
        c.cleanup();
    }

    Ok(results)
}

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

    let resp = req
        .send()
        .await
        .map_err(|e| format!("UEX request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("UEX API returned status: {}", resp.status()));
    }

    resp.json()
        .await
        .map_err(|e| format!("Failed to parse UEX response: {}", e))
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

/// Search UEX for commodities by query string
pub async fn search_commodities(query: &str, api_key: &str) -> Result<Vec<UexResult>, String> {
    let url = format!("{}/commodities", UEX_BASE_URL);
    let body = uex_get(&url, &[("name_filter", query)], api_key).await?;
    let query_lower = query.to_lowercase();
    Ok(extract_results(&body, &query_lower, Some("commodity")))
}

/// Search UEX for vehicles (ships + ground vehicles) by query string
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

/// Search UEX for items by query string
pub async fn search_items(query: &str, api_key: &str) -> Result<Vec<UexResult>, String> {
    let url = format!("{}/items", UEX_BASE_URL);
    let body = uex_get(&url, &[("name_filter", query)], api_key).await?;
    let query_lower = query.to_lowercase();
    Ok(extract_results(&body, &query_lower, Some("item")))
}

/// Search UEX for locations (terminals) by query string
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

                    // Prefer displayname → fullname → name
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

/// Search UEX across all entity types (commodities, vehicles, items, locations).
/// Runs all four queries in parallel and merges results, grouped by kind.
pub async fn search_all(query: &str, api_key: &str) -> Result<Vec<UexResult>, String> {
    let (commodities, vehicles, items, locations) = tokio::join!(
        search_commodities(query, api_key),
        search_vehicles(query, api_key),
        search_items(query, api_key),
        search_locations(query, api_key),
    );

    let mut results = Vec::new();
    // Collect each category; ignore individual failures so a broken endpoint
    // doesn't block the others.
    if let Ok(mut v) = commodities { results.append(&mut v); }
    if let Ok(mut v) = vehicles    { results.append(&mut v); }
    if let Ok(mut v) = items       { results.append(&mut v); }
    if let Ok(mut v) = locations   { results.append(&mut v); }

    Ok(results)
}

/// Get prices for a specific commodity from UEX
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
