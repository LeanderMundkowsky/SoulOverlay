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
    ]);

    migrations.to_latest(conn).map_err(|e| {
        error!("Database migration failed: {}", e);
        format!("Database migration failed: {}", e)
    })?;

    info!("Database migrations applied");
    Ok(())
}
