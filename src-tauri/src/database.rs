use log::{error, info};
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::path::Path;

/// Open (or create) the database at the given path and run all pending migrations.
pub fn init(path: &Path) -> Result<Connection, String> {
    info!("Opening database at {}", path.display());

    let mut conn = Connection::open(path).map_err(|e| format!("Failed to open database: {}", e))?;

    // WAL mode for better concurrent read performance
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| format!("Failed to set WAL mode: {}", e))?;

    run_migrations(&mut conn)?;

    info!("Database initialized successfully");
    Ok(conn)
}

/// Define and run schema migrations.
fn run_migrations(conn: &mut Connection) -> Result<(), String> {
    let migrations = Migrations::new(vec![
        // v1: cache_entries table for collection-level blob storage
        M::up(
            "CREATE TABLE IF NOT EXISTS cache_entries (
                collection TEXT PRIMARY KEY,
                data       BLOB NOT NULL,
                cached_at  TEXT NOT NULL,
                ttl_secs   INTEGER NOT NULL
            );",
        ),
        // v2: favorites table for user-bookmarked entities
        M::up(
            "CREATE TABLE IF NOT EXISTS favorites (
                id       TEXT NOT NULL,
                name     TEXT NOT NULL,
                kind     TEXT NOT NULL,
                slug     TEXT NOT NULL DEFAULT '',
                uuid     TEXT NOT NULL DEFAULT '',
                added_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE UNIQUE INDEX idx_favorites_kind_id ON favorites(kind, id);",
        ),
        // v3: watch_list table for tracked price entries
        M::up(
            "CREATE TABLE IF NOT EXISTS watch_list (
                entity_id      TEXT NOT NULL,
                entity_name    TEXT NOT NULL,
                entity_kind    TEXT NOT NULL,
                entity_slug    TEXT NOT NULL DEFAULT '',
                terminal_id    TEXT NOT NULL,
                terminal_name  TEXT NOT NULL,
                price_type     TEXT NOT NULL,
                added_at       TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE UNIQUE INDEX idx_watch_list_key ON watch_list(entity_id, terminal_id, price_type);",
        ),
        // v4: inventory table for local item/commodity tracking
        M::up(
            "CREATE TABLE IF NOT EXISTS inventory (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_id       TEXT NOT NULL,
                entity_name     TEXT NOT NULL,
                entity_kind     TEXT NOT NULL,
                location_id     TEXT NOT NULL,
                location_name   TEXT NOT NULL,
                location_slug   TEXT NOT NULL,
                quantity        INTEGER NOT NULL DEFAULT 1,
                collection      TEXT NOT NULL DEFAULT '',
                added_at        TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE UNIQUE INDEX idx_inventory_entity_location_collection
                ON inventory(entity_id, location_id, collection);",
        ),
        // v5: cz_self_timers for contested zone countdown timers
        M::up(
            "CREATE TABLE IF NOT EXISTS cz_self_timers (
                id               TEXT PRIMARY KEY,
                zone             TEXT NOT NULL,
                label            TEXT NOT NULL,
                category         TEXT NOT NULL DEFAULT '',
                default_seconds  INTEGER NOT NULL,
                remaining_seconds INTEGER NOT NULL,
                end_epoch        INTEGER NOT NULL DEFAULT 0,
                status           TEXT NOT NULL DEFAULT 'idle'
            );",
        ),
        // v6: add source column to favorites for wiki-only items
        M::up(
            "ALTER TABLE favorites ADD COLUMN source TEXT NOT NULL DEFAULT 'uex';",
        ),
        // v7: wikelo_completions table for tracking completed Wikelo contracts
        M::up(
            "CREATE TABLE IF NOT EXISTS wikelo_completions (
                mission_id   TEXT PRIMARY KEY,
                completed_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        ),
        // v8: migrate inventory table to backend-as-source schema
        // Rename old table to inventory_legacy (preserving existing data for one-time migration).
        // Create new table with backend integer ID, collections_json, unique on (entity_id, location_id).
        M::up(
            "ALTER TABLE inventory RENAME TO inventory_legacy;
            CREATE TABLE IF NOT EXISTS inventory (
                id               INTEGER PRIMARY KEY,
                entity_id        TEXT NOT NULL,
                entity_name      TEXT NOT NULL,
                entity_kind      TEXT NOT NULL,
                location_id      TEXT NOT NULL,
                location_name    TEXT NOT NULL,
                location_slug    TEXT NOT NULL,
                quantity         INTEGER NOT NULL DEFAULT 1,
                collections_json TEXT NOT NULL DEFAULT '[]',
                added_at         TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE UNIQUE INDEX idx_inventory_entity_location ON inventory(entity_id, location_id);",
        ),
    ]);

    migrations.to_latest(conn).map_err(|e| {
        error!("Database migration failed: {}", e);
        format!("Database migration failed: {}", e)
    })?;

    info!("Database migrations applied");
    Ok(())
}
