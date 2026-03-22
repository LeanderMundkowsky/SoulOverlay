use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Serializable snapshot of vehicle ID mappings, stored in cache so the
/// mapper survives app restarts without re-fetching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapperSnapshot {
    pub uuid_to_uex: HashMap<String, String>,
    pub uex_to_uuid: HashMap<String, String>,
    pub name_to_uuid: HashMap<String, String>,
}

/// Bidirectional mapper between Wiki API UUIDs and UEX internal IDs.
///
/// - **Vehicles:** Name-based matching (UEX vehicle prices don't include UUID).
/// - **Items:** Direct UUID match (UEX `/items_prices_all` includes `item_uuid`).
/// - **Commodities:** Not mapped (stay on UEX IDs throughout).
#[derive(Debug, Default)]
pub struct EntityMapper {
    /// Wiki vehicle UUID → UEX vehicle ID
    vehicle_uuid_to_uex: HashMap<String, String>,
    /// UEX vehicle ID → Wiki vehicle UUID
    vehicle_uex_to_uuid: HashMap<String, String>,
    /// Normalized vehicle name → Wiki vehicle UUID (for fallback matching)
    vehicle_name_to_uuid: HashMap<String, String>,
}

impl EntityMapper {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the vehicle mapping tables.
    ///
    /// Called during vehicle catalog refresh after fetching from both
    /// Wiki API (uuid, name) and UEX API (id, name).
    pub fn set_vehicle_maps(
        &mut self,
        wiki_vehicles: &[(String, String)],  // (uuid, name)
        uex_vehicles: &[(String, String)],   // (uex_id, name)
    ) {
        self.vehicle_uuid_to_uex.clear();
        self.vehicle_uex_to_uuid.clear();
        self.vehicle_name_to_uuid.clear();

        // Build name → wiki_uuid index
        let mut wiki_by_name: HashMap<String, &str> = HashMap::new();
        for (uuid, name) in wiki_vehicles {
            let norm = normalize_name(name);
            wiki_by_name.insert(norm.clone(), uuid.as_str());
            self.vehicle_name_to_uuid.insert(norm, uuid.clone());
        }

        // Match UEX vehicles to Wiki vehicles by normalized name
        let mut matched = 0u32;
        for (uex_id, uex_name) in uex_vehicles {
            let norm = normalize_name(uex_name);
            if let Some(&wiki_uuid) = wiki_by_name.get(&norm) {
                self.vehicle_uuid_to_uex.insert(wiki_uuid.to_string(), uex_id.clone());
                self.vehicle_uex_to_uuid.insert(uex_id.clone(), wiki_uuid.to_string());
                matched += 1;
            }
        }

        log::info!(
            "EntityMapper: matched {}/{} vehicles (wiki={}, uex={})",
            matched,
            uex_vehicles.len(),
            wiki_vehicles.len(),
            uex_vehicles.len(),
        );
    }

    /// Restore mapper state from a cached snapshot.
    pub fn restore_from_snapshot(&mut self, snap: MapperSnapshot) {
        self.vehicle_uuid_to_uex = snap.uuid_to_uex;
        self.vehicle_uex_to_uuid = snap.uex_to_uuid;
        self.vehicle_name_to_uuid = snap.name_to_uuid;
        log::info!(
            "EntityMapper: restored {} vehicle mappings from cache",
            self.vehicle_uuid_to_uex.len(),
        );
    }

    /// Export current state as a serializable snapshot.
    pub fn snapshot(&self) -> MapperSnapshot {
        MapperSnapshot {
            uuid_to_uex: self.vehicle_uuid_to_uex.clone(),
            uex_to_uuid: self.vehicle_uex_to_uuid.clone(),
            name_to_uuid: self.vehicle_name_to_uuid.clone(),
        }
    }

    /// Returns true if the mapper has no vehicle mappings.
    pub fn is_empty(&self) -> bool {
        self.vehicle_uuid_to_uex.is_empty()
    }

    /// Look up the UEX vehicle ID for a Wiki vehicle UUID.
    pub fn vehicle_uuid_to_uex_id(&self, wiki_uuid: &str) -> Option<&str> {
        self.vehicle_uuid_to_uex.get(wiki_uuid).map(|s| s.as_str())
    }

    /// Look up the Wiki vehicle UUID for a UEX vehicle ID.
    pub fn vehicle_uex_id_to_uuid(&self, uex_id: &str) -> Option<&str> {
        self.vehicle_uex_to_uuid.get(uex_id).map(|s| s.as_str())
    }

    /// Look up the Wiki vehicle UUID by normalized name.
    pub fn vehicle_name_to_uuid(&self, name: &str) -> Option<&str> {
        let norm = normalize_name(name);
        self.vehicle_name_to_uuid.get(&norm).map(|s| s.as_str())
    }

    /// Get all UEX vehicle IDs that have Wiki UUID mappings.
    pub fn all_mapped_vehicle_uex_ids(&self) -> Vec<String> {
        self.vehicle_uex_to_uuid.keys().cloned().collect()
    }
}

/// Normalize a name for matching: lowercase, collapse whitespace.
fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
