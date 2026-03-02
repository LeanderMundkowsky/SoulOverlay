use serde::{Deserialize, Serialize};

/// A search result from UEX API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UexResult {
    pub id: String,
    pub name: String,
    pub kind: String, // "commodity", "ship", "item", etc.
    pub slug: String,
    /// UUID (items only). Used to fetch item details from the UEX API.
    #[serde(default)]
    pub uuid: String,
}

/// Detailed entity metadata from UEX API, with type-specific optional fields.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityInfo {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub slug: String,
    // Common
    pub code: Option<String>,
    pub company_name: Option<String>,
    pub wiki: Option<String>,
    pub game_version: Option<String>,
    // Commodity
    pub commodity_kind: Option<String>,
    pub weight_scu: Option<f64>,
    pub avg_buy: Option<f64>,
    pub avg_sell: Option<f64>,
    pub is_illegal: Option<bool>,
    pub is_buyable: Option<bool>,
    pub is_sellable: Option<bool>,
    pub is_mineral: Option<bool>,
    pub is_raw: Option<bool>,
    pub is_refined: Option<bool>,
    pub is_harvestable: Option<bool>,
    // Item
    pub section: Option<String>,
    pub category: Option<String>,
    pub size: Option<String>,
    pub color: Option<String>,
    // Vehicle
    pub name_full: Option<String>,
    pub scu: Option<f64>,
    pub crew: Option<String>,
    pub length: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub mass: Option<f64>,
    pub pad_type: Option<String>,
    pub url_photo: Option<String>,
    pub url_store: Option<String>,
    pub roles: Vec<String>,
}

impl EntityInfo {
    fn from_commodity_json(item: &serde_json::Value) -> Self {
        Self {
            id: json_str_or_u64(item, "id"),
            name: item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            kind: "commodity".to_string(),
            slug: item.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            code: item.get("code").and_then(|v| v.as_str()).map(|s| s.to_string()),
            wiki: item.get("wiki").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            commodity_kind: item.get("kind").and_then(|v| v.as_str()).map(|s| s.to_string()),
            weight_scu: item.get("weight_scu").and_then(|v| v.as_f64()),
            avg_buy: item.get("price_buy").and_then(|v| v.as_f64()).filter(|&v| v > 0.0),
            avg_sell: item.get("price_sell").and_then(|v| v.as_f64()).filter(|&v| v > 0.0),
            is_illegal: json_bool(item, "is_illegal"),
            is_buyable: json_bool(item, "is_buyable"),
            is_sellable: json_bool(item, "is_sellable"),
            is_mineral: json_bool(item, "is_mineral"),
            is_raw: json_bool(item, "is_raw"),
            is_refined: json_bool(item, "is_refined"),
            is_harvestable: json_bool(item, "is_harvestable"),
            ..Default::default()
        }
    }

    fn from_item_json(item: &serde_json::Value) -> Self {
        Self {
            id: json_str_or_u64(item, "id"),
            name: item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            kind: "item".to_string(),
            slug: item.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            section: item.get("section").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            category: item.get("category").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            company_name: item.get("company_name").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            size: item.get("size").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            color: item.get("color").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            game_version: item.get("game_version").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            ..Default::default()
        }
    }

    fn from_vehicle_json(item: &serde_json::Value) -> Self {
        let mut roles = Vec::new();
        let role_flags = [
            ("is_boarding", "Boarding"), ("is_bomber", "Bomber"), ("is_cargo", "Cargo"),
            ("is_carrier", "Carrier"), ("is_civilian", "Civilian"), ("is_construction", "Construction"),
            ("is_datarunner", "Data Runner"), ("is_exploration", "Exploration"),
            ("is_industrial", "Industrial"), ("is_interdiction", "Interdiction"),
            ("is_medical", "Medical"), ("is_military", "Military"), ("is_mining", "Mining"),
            ("is_passenger", "Passenger"), ("is_racing", "Racing"), ("is_refinery", "Refinery"),
            ("is_refuel", "Refuel"), ("is_repair", "Repair"), ("is_research", "Research"),
            ("is_salvage", "Salvage"), ("is_scanning", "Scanning"), ("is_science", "Science"),
            ("is_stealth", "Stealth"),
        ];
        for (field, label) in role_flags {
            if item.get(field).and_then(|v| v.as_u64()).unwrap_or(0) == 1 {
                roles.push(label.to_string());
            }
        }

        let kind = if item.get("is_ground_vehicle").and_then(|v| v.as_u64()).unwrap_or(0) == 1 {
            "ground vehicle"
        } else {
            "vehicle"
        };

        Self {
            id: json_str_or_u64(item, "id"),
            name: item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            kind: kind.to_string(),
            slug: item.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name_full: item.get("name_full").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            company_name: item.get("company_name").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            scu: item.get("scu").and_then(|v| v.as_f64()).filter(|&v| v > 0.0),
            crew: item.get("crew").and_then(|v| v.as_str()).filter(|s| !s.is_empty() && *s != "0").map(|s| s.to_string()),
            length: item.get("length").and_then(|v| v.as_f64()).filter(|&v| v > 0.0),
            width: item.get("width").and_then(|v| v.as_f64()).filter(|&v| v > 0.0),
            height: item.get("height").and_then(|v| v.as_f64()).filter(|&v| v > 0.0),
            mass: item.get("mass").and_then(|v| v.as_f64()).filter(|&v| v > 0.0),
            pad_type: item.get("pad_type").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            url_photo: item.get("url_photo").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            url_store: item.get("url_store").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            game_version: item.get("game_version").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            roles,
            ..Default::default()
        }
    }
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

        Self { id, name, kind, slug, uuid: String::new() }
    }
}

/// A price entry from UEX API.
/// Unified across all price types — entity metadata identifies the source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceEntry {
    pub entity_id: String,
    pub entity_name: String,
    pub price_type: String,
    pub location: String,
    pub terminal: String,
    pub terminal_id: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub rent_price: f64,
    pub scu_available: Option<f64>,
    pub date_updated: String,
}

impl PriceEntry {
    /// Parse a commodity/raw-commodity price row.
    fn from_commodity_json(item: &serde_json::Value, price_type: &str) -> Self {
        let entity_id = json_str_or_u64(item, "id_commodity");
        let entity_name = item.get("commodity_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        Self {
            entity_id,
            entity_name,
            price_type: price_type.to_string(),
            location: location_from_json(item),
            terminal: terminal_name_from_json(item),
            terminal_id: json_str_or_u64(item, "id_terminal"),
            buy_price: item.get("price_buy").and_then(|v| v.as_f64()).unwrap_or(0.0),
            sell_price: item.get("price_sell").and_then(|v| v.as_f64()).unwrap_or(0.0),
            rent_price: 0.0,
            scu_available: item.get("scu_buy").or_else(|| item.get("scu_sell")).and_then(|v| v.as_f64()),
            date_updated: json_timestamp(item),
        }
    }

    /// Parse an item price row.
    fn from_item_json(item: &serde_json::Value) -> Self {
        let entity_id = json_str_or_u64(item, "id_item");
        let entity_name = item.get("item_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        Self {
            entity_id,
            entity_name,
            price_type: "item".to_string(),
            location: location_from_json(item),
            terminal: terminal_name_from_json(item),
            terminal_id: json_str_or_u64(item, "id_terminal"),
            buy_price: item.get("price_buy").and_then(|v| v.as_f64()).unwrap_or(0.0),
            sell_price: item.get("price_sell").and_then(|v| v.as_f64()).unwrap_or(0.0),
            rent_price: 0.0,
            scu_available: None,
            date_updated: json_timestamp(item),
        }
    }

    /// Parse a vehicle purchase price row.
    fn from_vehicle_purchase_json(item: &serde_json::Value) -> Self {
        let entity_id = json_str_or_u64(item, "id_vehicle");
        let entity_name = item.get("vehicle_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        Self {
            entity_id,
            entity_name,
            price_type: "vehicle_purchase".to_string(),
            location: location_from_json(item),
            terminal: terminal_name_from_json(item),
            terminal_id: json_str_or_u64(item, "id_terminal"),
            buy_price: item.get("price_buy").and_then(|v| v.as_f64()).unwrap_or(0.0),
            sell_price: 0.0,
            rent_price: 0.0,
            scu_available: None,
            date_updated: json_timestamp(item),
        }
    }

    /// Parse a vehicle rental price row.
    fn from_vehicle_rental_json(item: &serde_json::Value) -> Self {
        let entity_id = json_str_or_u64(item, "id_vehicle");
        let entity_name = item.get("vehicle_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        Self {
            entity_id,
            entity_name,
            price_type: "vehicle_rental".to_string(),
            location: location_from_json(item),
            terminal: terminal_name_from_json(item),
            terminal_id: json_str_or_u64(item, "id_terminal"),
            buy_price: 0.0,
            sell_price: 0.0,
            rent_price: item.get("price_rent").and_then(|v| v.as_f64()).unwrap_or(0.0),
            scu_available: None,
            date_updated: json_timestamp(item),
        }
    }

    /// Parse a fuel price row.
    fn from_fuel_json(item: &serde_json::Value) -> Self {
        let entity_id = json_str_or_u64(item, "id_commodity");
        let entity_name = item.get("commodity_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        Self {
            entity_id,
            entity_name,
            price_type: "fuel".to_string(),
            location: location_from_json(item),
            terminal: terminal_name_from_json(item),
            terminal_id: json_str_or_u64(item, "id_terminal"),
            buy_price: item.get("price_buy").and_then(|v| v.as_f64()).unwrap_or(0.0),
            sell_price: 0.0,
            rent_price: 0.0,
            scu_available: None,
            date_updated: json_timestamp(item),
        }
    }
}

/// Extract a string or u64 field as a String.
fn json_str_or_u64(item: &serde_json::Value, field: &str) -> String {
    item.get(field)
        .and_then(|v| v.as_u64().map(|n| n.to_string()).or_else(|| v.as_str().map(|s| s.to_string())))
        .unwrap_or_default()
}

/// Extract an integer field as Option<bool> (0 = false, 1 = true, absent = None).
fn json_bool(item: &serde_json::Value, field: &str) -> Option<bool> {
    item.get(field).and_then(|v| v.as_u64()).map(|v| v == 1)
}

/// Build a human-readable location string from a price row.
fn location_from_json(item: &serde_json::Value) -> String {
    item.get("star_system_name")
        .or_else(|| item.get("planet_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string()
}

/// Extract terminal_name from a price row.
fn terminal_name_from_json(item: &serde_json::Value) -> String {
    item.get("terminal_name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string()
}

/// Extract a date string from a price row (timestamp or string).
fn json_timestamp(item: &serde_json::Value) -> String {
    item.get("date_modified")
        .or_else(|| item.get("date_added"))
        .map(|v| {
            if let Some(n) = v.as_i64() { n.to_string() }
            else if let Some(s) = v.as_str() { s.to_string() }
            else { String::new() }
        })
        .unwrap_or_default()
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

            UexResult { id, name, kind: kind.to_string(), slug, uuid: String::new() }
        })
        .collect();
    Ok(results)
}

/// Fetch ALL items from UEX by iterating over all item categories in parallel.
///
/// The `/items` endpoint requires `id_category`, `id_company`, or `uuid` — there is no
/// "fetch all" variant. We first fetch `/categories` (type=item), then fan-out one request
/// per category and merge the results, deduplicating by item ID.
pub async fn fetch_all_items(api_key: &str) -> Result<Vec<UexResult>, String> {
    // Step 1: fetch all categories and collect item category IDs
    let categories_url = format!("{}/categories", UEX_BASE_URL);
    let cat_body = uex_get(&categories_url, &[], api_key).await?;
    let category_ids: Vec<u64> = cat_body
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|c| c.get("type").and_then(|v| v.as_str()) == Some("item"))
                .filter_map(|c| c.get("id").and_then(|v| v.as_u64()))
                .collect()
        })
        .unwrap_or_default();

    log::info!("Fetching items across {} categories in parallel", category_ids.len());

    // Step 2: spawn one task per category
    let api_key_arc = std::sync::Arc::new(api_key.to_string());
    let handles: Vec<_> = category_ids
        .into_iter()
        .map(|cat_id| {
            let key = api_key_arc.clone();
            tokio::spawn(async move {
                let url = format!("{}/items", UEX_BASE_URL);
                let id_str = cat_id.to_string();
                uex_get(&url, &[("id_category", &id_str)], &key).await
            })
        })
        .collect();

    // Step 3: collect results, deduplicating by numeric item ID
    let mut seen_ids = std::collections::HashSet::<u64>::new();
    let mut all_items: Vec<UexResult> = Vec::new();

    for handle in handles {
        match handle.await {
            Ok(Ok(body)) => {
                for item in extract_data_array(&body) {
                    let id = item.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                    if seen_ids.insert(id) {
                        let mut r = UexResult::from_json(item);
                        r.kind = "item".to_string();
                        r.uuid = item.get("uuid").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        all_items.push(r);
                    }
                }
            }
            Ok(Err(e)) => log::warn!("Failed to fetch items for a category: {}", e),
            Err(e) => log::warn!("Item fetch task panicked: {}", e),
        }
    }

    log::info!("Total items fetched: {}", all_items.len());
    Ok(all_items)
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

            UexResult { id, name, kind: "location".to_string(), slug, uuid: String::new() }
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

                    UexResult { id, name, kind: kind.to_string(), slug, uuid: String::new() }
                })
                .filter(|r| r.name.to_lowercase().contains(&query_lower))
                .collect()
        })
        .unwrap_or_default();

    Ok(results)
}

/// Search UEX for items by query string.
///
/// The `/items` endpoint requires `id_category`, `id_company`, or `uuid` and does not
/// support a bare name search. This fallback performs a full fetch across all categories
/// and filters client-side. It should only be reached when the items cache is completely
/// absent; under normal operation the cache is populated on startup.
pub async fn search_items(query: &str, api_key: &str) -> Result<Vec<UexResult>, String> {
    let all = fetch_all_items(api_key).await?;
    Ok(search_in_collection(&all, query))
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

                    UexResult { id, name, kind: "location".to_string(), slug, uuid: String::new() }
                })
                .filter(|r| r.name.to_lowercase().contains(&query_lower))
                .collect()
        })
        .unwrap_or_default();

    Ok(results)
}

/// Get prices for a specific commodity from UEX (direct API call, no cache).
pub async fn get_commodity_prices(commodity_id: &str, api_key: &str) -> Result<Vec<PriceEntry>, String> {
    let url = format!("{}/commodities_prices", UEX_BASE_URL);
    let body = uex_get(&url, &[("id_commodity", commodity_id)], api_key).await?;
    let entries = extract_data_array(&body)
        .into_iter()
        .map(|item| PriceEntry::from_commodity_json(item, "commodity"))
        .collect();
    Ok(entries)
}

/// Get raw commodity prices for a specific commodity (direct API call, no cache).
pub async fn get_raw_commodity_prices(commodity_id: &str, api_key: &str) -> Result<Vec<PriceEntry>, String> {
    let url = format!("{}/commodities_raw_prices", UEX_BASE_URL);
    let body = uex_get(&url, &[("id_commodity", commodity_id)], api_key).await?;
    let entries = extract_data_array(&body)
        .into_iter()
        .map(|item| PriceEntry::from_commodity_json(item, "raw_commodity"))
        .collect();
    Ok(entries)
}

/// Get item prices for a specific item (direct API call, no cache).
pub async fn get_item_prices(item_id: &str, api_key: &str) -> Result<Vec<PriceEntry>, String> {
    let url = format!("{}/items_prices", UEX_BASE_URL);
    let body = uex_get(&url, &[("id_item", item_id)], api_key).await?;
    let entries = extract_data_array(&body)
        .into_iter()
        .map(|item| PriceEntry::from_item_json(item))
        .collect();
    Ok(entries)
}

/// Get vehicle purchase prices for a specific vehicle (direct API call, no cache).
pub async fn get_vehicle_purchase_prices(vehicle_id: &str, api_key: &str) -> Result<Vec<PriceEntry>, String> {
    let url = format!("{}/vehicles_purchases_prices", UEX_BASE_URL);
    let body = uex_get(&url, &[("id_vehicle", vehicle_id)], api_key).await?;
    let entries = extract_data_array(&body)
        .into_iter()
        .map(|item| PriceEntry::from_vehicle_purchase_json(item))
        .collect();
    Ok(entries)
}

/// Get vehicle rental prices for a specific vehicle (direct API call, no cache).
pub async fn get_vehicle_rental_prices(vehicle_id: &str, api_key: &str) -> Result<Vec<PriceEntry>, String> {
    let url = format!("{}/vehicles_rentals_prices", UEX_BASE_URL);
    let body = uex_get(&url, &[("id_vehicle", vehicle_id)], api_key).await?;
    let entries = extract_data_array(&body)
        .into_iter()
        .map(|item| PriceEntry::from_vehicle_rental_json(item))
        .collect();
    Ok(entries)
}

/// Get fuel prices for a specific terminal (direct API call, no cache).
pub async fn get_fuel_prices(terminal_id: &str, api_key: &str) -> Result<Vec<PriceEntry>, String> {
    let url = format!("{}/fuel_prices", UEX_BASE_URL);
    let body = uex_get(&url, &[("id_terminal", terminal_id)], api_key).await?;
    let entries = extract_data_array(&body)
        .into_iter()
        .map(|item| PriceEntry::from_fuel_json(item))
        .collect();
    Ok(entries)
}

// ── Bulk-fetch price functions (all data, for cache prefetch) ──────────────

/// Fetch ALL commodity prices from UEX.
pub async fn fetch_all_commodity_prices(api_key: &str) -> Result<Vec<PriceEntry>, String> {
    let url = format!("{}/commodities_prices_all", UEX_BASE_URL);
    let body = uex_get(&url, &[], api_key).await?;
    let entries = extract_data_array(&body)
        .into_iter()
        .map(|item| PriceEntry::from_commodity_json(item, "commodity"))
        .collect();
    Ok(entries)
}

/// Fetch ALL raw commodity prices from UEX.
pub async fn fetch_all_raw_commodity_prices(api_key: &str) -> Result<Vec<PriceEntry>, String> {
    let url = format!("{}/commodities_raw_prices_all", UEX_BASE_URL);
    let body = uex_get(&url, &[], api_key).await?;
    let entries = extract_data_array(&body)
        .into_iter()
        .map(|item| PriceEntry::from_commodity_json(item, "raw_commodity"))
        .collect();
    Ok(entries)
}

/// Fetch ALL item prices from UEX.
pub async fn fetch_all_item_prices(api_key: &str) -> Result<Vec<PriceEntry>, String> {
    let url = format!("{}/items_prices_all", UEX_BASE_URL);
    let body = uex_get(&url, &[], api_key).await?;
    let entries = extract_data_array(&body)
        .into_iter()
        .map(|item| PriceEntry::from_item_json(item))
        .collect();
    Ok(entries)
}

/// Fetch ALL vehicle purchase prices from UEX.
pub async fn fetch_all_vehicle_purchase_prices(api_key: &str) -> Result<Vec<PriceEntry>, String> {
    let url = format!("{}/vehicles_purchases_prices_all", UEX_BASE_URL);
    let body = uex_get(&url, &[], api_key).await?;
    let entries = extract_data_array(&body)
        .into_iter()
        .map(|item| PriceEntry::from_vehicle_purchase_json(item))
        .collect();
    Ok(entries)
}

/// Fetch ALL vehicle rental prices from UEX.
pub async fn fetch_all_vehicle_rental_prices(api_key: &str) -> Result<Vec<PriceEntry>, String> {
    let url = format!("{}/vehicles_rentals_prices_all", UEX_BASE_URL);
    let body = uex_get(&url, &[], api_key).await?;
    let entries = extract_data_array(&body)
        .into_iter()
        .map(|item| PriceEntry::from_vehicle_rental_json(item))
        .collect();
    Ok(entries)
}

/// Fetch ALL fuel prices from UEX.
pub async fn fetch_all_fuel_prices(api_key: &str) -> Result<Vec<PriceEntry>, String> {
    let url = format!("{}/fuel_prices_all", UEX_BASE_URL);
    let body = uex_get(&url, &[], api_key).await?;
    let entries = extract_data_array(&body)
        .into_iter()
        .map(|item| PriceEntry::from_fuel_json(item))
        .collect();
    Ok(entries)
}

// ── Entity info functions ──────────────────────────────────────────────────

/// Fetch commodity details by id.
/// The `/commodities` endpoint returns all commodities; we filter by id client-side.
pub async fn get_commodity_info(commodity_id: &str, api_key: &str) -> Result<EntityInfo, String> {
    let url = format!("{}/commodities", UEX_BASE_URL);
    let body = uex_get(&url, &[], api_key).await?;
    extract_data_array(&body)
        .into_iter()
        .find(|item| json_str_or_u64(item, "id") == commodity_id)
        .map(|item| EntityInfo::from_commodity_json(item))
        .ok_or_else(|| format!("Commodity {} not found", commodity_id))
}

/// Fetch vehicle details by id.
/// The `/vehicles` endpoint returns all vehicles; we filter by id client-side.
pub async fn get_vehicle_info(vehicle_id: &str, api_key: &str) -> Result<EntityInfo, String> {
    let url = format!("{}/vehicles", UEX_BASE_URL);
    let body = uex_get(&url, &[], api_key).await?;
    extract_data_array(&body)
        .into_iter()
        .find(|item| json_str_or_u64(item, "id") == vehicle_id)
        .map(|item| EntityInfo::from_vehicle_json(item))
        .ok_or_else(|| format!("Vehicle {} not found", vehicle_id))
}

/// Fetch item details by uuid.
pub async fn get_item_info(uuid: &str, api_key: &str) -> Result<EntityInfo, String> {
    let url = format!("{}/items", UEX_BASE_URL);
    let body = uex_get(&url, &[("uuid", uuid)], api_key).await?;
    extract_data_array(&body)
        .first()
        .map(|item| EntityInfo::from_item_json(item))
        .ok_or_else(|| format!("Item with uuid {} not found", uuid))
}
