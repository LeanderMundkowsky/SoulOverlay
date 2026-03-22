use crate::settings::Settings;
use chrono::{DateTime, Duration, Utc};
use log::{info, warn};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

// ── Collection definitions ─────────────────────────────────────────────────

/// Known collection names used as cache keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum Collection {
    Commodities,
    CommodityPrices,
    RawCommodityPrices,
    ItemPrices,
    VehiclePurchasePrices,
    VehicleRentalPrices,
    FuelPrices,
    Vehicles,
    Items,
    Manufacturers,
    Locations,
    Fleet,
    UserProfile,
    EntityInfo,
    WikiSpecs,
    WikiloTrades,
}

impl Collection {
    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Commodities => "Commodities",
            Self::CommodityPrices => "Commodity Prices",
            Self::RawCommodityPrices => "Raw Commodity Prices",
            Self::ItemPrices => "Item Prices",
            Self::VehiclePurchasePrices => "Vehicle Purchase Prices",
            Self::VehicleRentalPrices => "Vehicle Rental Prices",
            Self::FuelPrices => "Fuel Prices",
            Self::Vehicles => "Vehicles",
            Self::Items => "Items",
            Self::Manufacturers => "Manufacturers",
            Self::Locations => "Locations",
            Self::Fleet => "Fleet",
            Self::UserProfile => "User Profile",
            Self::EntityInfo => "Entity Info",
            Self::WikiSpecs => "Wiki Specifications",
            Self::WikiloTrades => "Wikelo Trades",
        }
    }

    /// Fallback TTL for this collection in seconds (used when no entry exists yet).
    pub fn ttl_secs(&self) -> i64 {
        match self {
            Self::Items
            | Self::Commodities => 43200,
            Self::Manufacturers => 86400,
            Self::CommodityPrices
            | Self::RawCommodityPrices
            | Self::ItemPrices
            | Self::FuelPrices => 1200,
            Self::VehiclePurchasePrices
            | Self::VehicleRentalPrices
            | Self::Fleet
            | Self::Vehicles => 43200,
            Self::UserProfile => 3600,
            Self::EntityInfo => 86400,
            Self::Locations => 86400,
            Self::WikiSpecs => 604800,
            Self::WikiloTrades => 86400,
        }
    }

    /// Storage key used in the SQLite table and in-memory map.
    pub fn storage_key(&self) -> String {
        match self {
            Self::Commodities => "commodities".to_string(),
            Self::CommodityPrices => "commodity_prices".to_string(),
            Self::RawCommodityPrices => "raw_commodity_prices".to_string(),
            Self::ItemPrices => "item_prices".to_string(),
            Self::VehiclePurchasePrices => "vehicle_purchase_prices".to_string(),
            Self::VehicleRentalPrices => "vehicle_rental_prices".to_string(),
            Self::FuelPrices => "fuel_prices".to_string(),
            Self::Vehicles => "vehicles".to_string(),
            Self::Items => "items".to_string(),
            Self::Manufacturers => "manufacturers".to_string(),
            Self::Locations => "locations".to_string(),
            Self::Fleet => "fleet".to_string(),
            Self::UserProfile => "user_profile".to_string(),
            Self::EntityInfo => "entity_info".to_string(),
            Self::WikiSpecs => "wiki_specs".to_string(),
            Self::WikiloTrades => "wikelo_trades".to_string(),
        }
    }

    /// Storage key for per-ID sub-collections (e.g. `commodity_prices:42`).
    pub fn storage_key_with_id(&self, id: &str) -> String {
        format!("{}:{}", self.storage_key(), id)
    }

    /// All known collection variants.
    pub fn all() -> &'static [Collection] {
        &[
            Collection::Commodities,
            Collection::CommodityPrices,
            Collection::RawCommodityPrices,
            Collection::ItemPrices,
            Collection::VehiclePurchasePrices,
            Collection::VehicleRentalPrices,
            Collection::FuelPrices,
            Collection::Vehicles,
            Collection::Manufacturers,
            Collection::Locations,
            Collection::UserProfile,
            Collection::EntityInfo,
            Collection::WikiSpecs,
            Collection::WikiloTrades,
        ]
    }

    /// Look up a Collection by its storage key string.
    pub fn from_storage_key(key: &str) -> Option<Collection> {
        Collection::all().iter().find(|c| c.storage_key() == key).copied()
    }

    /// Whether this collection uses per-entity sub-keys.
    pub fn is_per_entity(&self) -> bool {
        matches!(
            self,
            Self::CommodityPrices
                | Self::RawCommodityPrices
                | Self::ItemPrices
                | Self::VehiclePurchasePrices
                | Self::VehicleRentalPrices
                | Self::FuelPrices
                | Self::EntityInfo
                | Self::WikiSpecs
        )
    }

    /// Resolve the TTL for this collection from user settings, falling back to
    /// the hardcoded default. Enforces a minimum of 60 seconds.
    pub fn ttl_for(&self, settings: &Settings) -> i64 {
        let key = self.storage_key();
        let raw = settings
            .cache_ttls
            .get(&key)
            .map(|&v| v as i64)
            .unwrap_or_else(|| self.ttl_secs());
        raw.max(60)
    }
}

// ── Cache entry ────────────────────────────────────────────────────────────

/// An in-memory cache entry holding the deserialized data + metadata.
struct MemoryEntry {
    /// Raw MessagePack bytes (same as stored in SQLite).
    data: Vec<u8>,
    /// When this entry was cached.
    cached_at: DateTime<Utc>,
    /// TTL that was in effect when the entry was stored.
    ttl_secs: i64,
}

impl MemoryEntry {
    fn is_expired(&self) -> bool {
        Utc::now() > self.cached_at + Duration::seconds(self.ttl_secs)
    }
}

// ── Status info returned to frontend ───────────────────────────────────────

/// Per-collection status info for the cache management UI.
#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct CollectionStatus {
    pub collection: Collection,
    pub display_name: String,
    pub cached_at: Option<String>,
    pub ttl_secs: i32,
    pub is_expired: bool,
    pub entry_count: u32,
}

/// Result of a cache `get` operation, distinguishing fresh vs stale data.
pub enum CacheResult<T> {
    /// Data is present and within TTL.
    Fresh(T),
    /// Data is present but expired. Caller should refresh in background.
    Stale(T),
    /// No data available at all.
    Missing,
}

// ── CacheStore ─────────────────────────────────────────────────────────────

/// Central cache store with in-memory HashMap mirror and SQLite persistence.
/// All public methods take `&self` and lock internally.
pub struct CacheStore {
    db: Mutex<Connection>,
    memory: Mutex<HashMap<String, MemoryEntry>>,
}

// SAFETY: rusqlite::Connection is Send but not Sync. We wrap it in a Mutex,
// so only one thread accesses it at a time, making the whole struct Sync.
unsafe impl Sync for CacheStore {}

impl CacheStore {
    /// Expose the underlying database connection for non-cache queries (e.g. favorites).
    pub fn db(&self) -> &Mutex<Connection> {
        &self.db
    }

    /// Create a new CacheStore backed by the given SQLite connection.
    /// Loads all existing cache entries from the database into memory.
    pub fn new(conn: Connection) -> Self {
        let store = Self {
            db: Mutex::new(conn),
            memory: Mutex::new(HashMap::new()),
        };
        store.load_all_from_db();
        store
    }

    /// Load all rows from `cache_entries` into the in-memory map.
    fn load_all_from_db(&self) {
        let db = self.db.lock().unwrap();
        let mut stmt =
            match db.prepare("SELECT collection, data, cached_at, ttl_secs FROM cache_entries") {
                Ok(s) => s,
                Err(e) => {
                    warn!("Failed to prepare load query: {}", e);
                    return;
                }
            };

        let rows = match stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Vec<u8>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        }) {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to load cache entries: {}", e);
                return;
            }
        };

        let mut memory = self.memory.lock().unwrap();
        let mut count = 0u32;
        for (key, data, cached_at_str, ttl_secs) in rows.flatten() {
            if let Ok(cached_at) = cached_at_str.parse::<DateTime<Utc>>() {
                memory.insert(
                    key,
                    MemoryEntry {
                        data,
                        cached_at,
                        ttl_secs,
                    },
                );
                count += 1;
            }
        }

        info!("Loaded {} cache entries from database", count);
    }

    /// Store data for a collection key. Serializes to MessagePack, writes to
    /// both in-memory map and SQLite.
    pub fn put<T: Serialize>(
        &self,
        key: &str,
        ttl_secs: i64,
        data: &T,
    ) -> Result<(), String> {
        let bytes = rmp_serde::to_vec(data)
            .map_err(|e| format!("Failed to serialize cache data: {}", e))?;

        let now = Utc::now();
        let ttl = ttl_secs;

        // Write to SQLite
        {
            let db = self.db.lock().unwrap();
            db.execute(
                "INSERT OR REPLACE INTO cache_entries (collection, data, cached_at, ttl_secs)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![key, bytes, now.to_rfc3339(), ttl],
            )
            .map_err(|e| format!("Failed to write cache entry: {}", e))?;
        }

        // Write to memory
        {
            let mut memory = self.memory.lock().unwrap();
            memory.insert(
                key.to_string(),
                MemoryEntry {
                    data: bytes,
                    cached_at: now,
                    ttl_secs: ttl,
                },
            );
        }

        Ok(())
    }

    /// Get data for a key, returning Fresh/Stale/Missing.
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> CacheResult<T> {
        let memory = self.memory.lock().unwrap();
        match memory.get(key) {
            Some(entry) => match rmp_serde::from_slice::<T>(&entry.data) {
                Ok(val) => {
                    if entry.is_expired() {
                        CacheResult::Stale(val)
                    } else {
                        CacheResult::Fresh(val)
                    }
                }
                Err(_) => CacheResult::Missing,
            },
            None => CacheResult::Missing,
        }
    }

    /// Invalidate a single key (remove from memory and database).
    pub fn invalidate(&self, key: &str) {
        {
            let mut memory = self.memory.lock().unwrap();
            memory.remove(key);
        }
        {
            let db = self.db.lock().unwrap();
            let _ = db.execute("DELETE FROM cache_entries WHERE collection = ?1", [key]);
        }
    }

    /// Invalidate all entries for a collection (including per-ID sub-entries).
    pub fn invalidate_collection(&self, collection: Collection) {
        let prefix = collection.storage_key();

        // Remove from memory: exact match + prefix matches (e.g. "commodity_prices:*")
        {
            let mut memory = self.memory.lock().unwrap();
            memory.retain(|k, _| k != &prefix && !k.starts_with(&format!("{}:", prefix)));
        }

        // Remove from database
        {
            let db = self.db.lock().unwrap();
            let _ = db.execute(
                "DELETE FROM cache_entries WHERE collection = ?1 OR collection LIKE ?2",
                rusqlite::params![prefix, format!("{}:%", prefix)],
            );
        }
    }

    /// Invalidate everything.
    pub fn invalidate_all(&self) {
        {
            let mut memory = self.memory.lock().unwrap();
            memory.clear();
        }
        {
            let db = self.db.lock().unwrap();
            let _ = db.execute("DELETE FROM cache_entries", []);
        }
    }

    /// Return status info for all known collections (for the cache management UI).
    pub fn status(&self) -> Vec<CollectionStatus> {
        let memory = self.memory.lock().unwrap();

        Collection::all()
            .iter()
            .map(|c| {
                let key = c.storage_key();

                // For per-entity collections, count all sub-entries
                let (cached_at, is_expired, entry_count, ttl_secs) = if c.is_per_entity() {
                    let prefix = format!("{}:", key);
                    let sub_entries: Vec<&MemoryEntry> = memory
                        .iter()
                        .filter(|(k, _)| k.starts_with(&prefix))
                        .map(|(_, v)| v)
                        .collect();

                    if sub_entries.is_empty() {
                        (None, true, 0, c.ttl_secs())
                    } else {
                        let latest = sub_entries.iter().map(|e| e.cached_at).max();
                        let any_expired = sub_entries.iter().any(|e| e.is_expired());
                        let ttl = sub_entries.first().map(|e| e.ttl_secs).unwrap_or_else(|| c.ttl_secs());
                        (latest, any_expired, sub_entries.len() as u32, ttl)
                    }
                } else {
                    match memory.get(&key) {
                        Some(entry) => {
                            let count =
                                rmp_serde::from_slice::<Vec<serde_json::Value>>(&entry.data)
                                    .map(|v| v.len() as u32)
                                    .unwrap_or(0);
                            (Some(entry.cached_at), entry.is_expired(), count, entry.ttl_secs)
                        }
                        None => (None, true, 0, c.ttl_secs()),
                    }
                };

                CollectionStatus {
                    collection: *c,
                    display_name: c.display_name().to_string(),
                    cached_at: cached_at.map(|t| t.to_rfc3339()),
                    ttl_secs: ttl_secs as i32,
                    is_expired,
                    entry_count,
                }
            })
            .collect()
    }

    /// Check if a key is expired or missing.
    /// For per-entity price collections, pass the base key (e.g. "commodity_prices")
    /// and this will check if ANY sub-entry exists and is fresh.
    pub fn is_expired(&self, key: &str) -> bool {
        let memory = self.memory.lock().unwrap();
        // Check exact key first
        if let Some(entry) = memory.get(key) {
            return entry.is_expired();
        }
        // Check for per-entity sub-keys (e.g. "commodity_prices:*")
        let prefix = format!("{}:", key);
        let sub_entries: Vec<&MemoryEntry> = memory
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(_, v)| v)
            .collect();
        if sub_entries.is_empty() {
            return true;
        }
        // Expired if ANY sub-entry is expired
        sub_entries.iter().any(|e| e.is_expired())
    }

    /// Total number of keys in the cache (for debug info).
    pub fn len(&self) -> usize {
        self.memory.lock().unwrap().len()
    }

    /// Update the TTL for all entries belonging to `collection` (both in-memory and SQLite).
    /// This does NOT reset `cached_at` — it only changes the TTL so expiry is recalculated.
    pub fn update_collection_ttl(&self, collection: Collection, new_ttl: i64) {
        let key = collection.storage_key();
        let prefix = format!("{}:", key);

        // Update in-memory entries
        {
            let mut memory = self.memory.lock().unwrap();
            for (k, entry) in memory.iter_mut() {
                if k == &key || k.starts_with(&prefix) {
                    entry.ttl_secs = new_ttl;
                }
            }
        }

        // Update in SQLite
        {
            let db = self.db.lock().unwrap();
            let _ = db.execute(
                "UPDATE cache_entries SET ttl_secs = ?1 WHERE collection = ?2 OR collection LIKE ?3",
                rusqlite::params![new_ttl, key, format!("{}:%", key)],
            );
        }
    }
}
