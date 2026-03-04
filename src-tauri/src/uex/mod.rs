mod client;
mod commodities;
mod fleet;
mod fuel;
mod items;
mod locations;
mod types;
mod user;
mod vehicles;

pub use client::UexClient;
pub use commodities::{
    fetch_all_commodities, fetch_all_commodity_infos, fetch_all_commodity_prices_per_entity,
    fetch_all_raw_commodity_prices, get_commodity_prices,
    search_commodities,
};
pub use fleet::fetch_fleet;
pub use fuel::fetch_all_fuel_prices;
pub use items::{fetch_all_item_infos, fetch_all_item_prices, fetch_all_items, search_items};
pub use locations::{fetch_all_locations, search_locations};
pub use types::{EntityInfo, HangarVehicle, PriceEntry, UexResult, UexUserProfile};
pub use user::fetch_user_profile;
pub use vehicles::{
    fetch_all_vehicle_infos, fetch_all_vehicle_purchase_prices_per_entity,
    fetch_all_vehicle_rental_prices_per_entity,
    fetch_all_vehicles, fetch_vehicle_photo_map, search_vehicles,
};

/// Search a pre-fetched collection by name substring match.
pub fn search_in_collection(collection: &[UexResult], query: &str) -> Vec<UexResult> {
    let query_lower = query.to_lowercase();
    collection
        .iter()
        .filter(|r| r.name.to_lowercase().contains(&query_lower))
        .cloned()
        .collect()
}
