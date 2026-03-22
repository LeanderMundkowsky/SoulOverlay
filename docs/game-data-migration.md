# Data Source Migration: UEX → Wiki API

## Status: ✅ Implemented

Completed migration from UEX Corp API as sole data source to a hybrid approach:

- **api.star-citizen.wiki** — Items, vehicles, manufacturers (catalog + specs)
- **UEX Corp API** — Prices, locations/terminals, commodities, user data

## Data Source Map

| Data | Source | Caching |
|------|--------|---------|
| **Vehicles** | Wiki `/api/vehicles` (289 entries) | Blob, 12h TTL |
| **Manufacturers** | Wiki `/api/manufacturers` (124 entries) | Blob, 24h TTL |
| **Items** | Wiki `/api/items?filter[name]=X` | On-demand search, not cached |
| **Item entity info** | Wiki `/api/items/{uuid}` | Per-entity, 24h TTL |
| **Commodities** | UEX `/commodities` | Blob, 12h TTL |
| **Locations** | UEX `/locations` + `/terminals` | Blob, 24h TTL |
| **All prices** | UEX (commodity, item, vehicle, fuel) | Per-entity, 20min–12h TTL |
| **User data** | UEX (profile, fleet) | Blob, 1h TTL |

## Key Architecture Decisions

### UUID as Primary Key
Items and vehicles use Wiki API UUIDs as their primary identifier.
Commodities and locations keep UEX IDs (no clean UUID available).

### EntityMapper (vehicles only)
Vehicle prices are fetched from UEX using UEX IDs. The `EntityMapper` builds a
bidirectional Wiki UUID ↔ UEX ID mapping via normalized name matching during
vehicle catalog refresh.

Items don't need mapping — UEX `/items_prices_all` includes `item_uuid` which
matches Wiki UUIDs directly.

### Items: On-Demand Search (not cached catalog)
With 19,334 items, caching the full catalog is impractical. Items are searched
on-demand via the Wiki API filter endpoint. The `:i` search prefix still works.

### Commodities Stay on UEX
Wiki "Cargo" items are physical SCU containers (1SCU/2SCU variants), not
abstract trade commodities. UEX commodity IDs map directly to price endpoints.

## Files Changed

### New
- `src-tauri/src/wiki/mapper.rs` — EntityMapper for Wiki UUID ↔ UEX ID
- `src-tauri/src/providers/manufacturers/` — Manufacturer catalog provider

### Modified
- `src-tauri/src/wiki/client.rs` — Added `list_vehicles`, `list_all_vehicles`, `list_all_manufacturers`
- `src-tauri/src/wiki/dto.rs` — Added `WikiManufacturerListItem`
- `src-tauri/src/providers/vehicles/provider.rs` — Catalog fetches from Wiki API
- `src-tauri/src/providers/items/provider.rs` — Removed catalog, added Wiki search
- `src-tauri/src/providers/items/dto.rs` — Added `item_uuid` to price DTO, removed dead UEX DTOs
- `src-tauri/src/providers/entity_info/provider.rs` — Removed bulk item info fetch
- `src-tauri/src/commands/api.rs` — Wiki-based item search, on-demand entity info
- `src-tauri/src/commands/uex.rs` — Updated search to use Wiki for items
- `src-tauri/src/cache_store.rs` — Added `Manufacturers` collection
- `src-tauri/src/state.rs` — Added `EntityMapper` to AppState
- `src-tauri/src/providers/mod.rs` — Added manufacturers provider, EntityMapper in context
