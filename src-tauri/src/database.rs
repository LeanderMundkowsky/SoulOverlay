use log::{error, info};
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::path::PathBuf;

/// Return the path to the SQLite database file.
/// Location: `%APPDATA%\SoulOverlay\soul_overlay.db`
pub fn db_path() -> Result<PathBuf, String> {
    let app_data =
        std::env::var("APPDATA").map_err(|_| "APPDATA environment variable not set".to_string())?;
    let dir = PathBuf::from(app_data).join("SoulOverlay");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create database directory: {}", e))?;
    Ok(dir.join("soul_overlay.db"))
}

/// Open (or create) the database and run all pending migrations.
pub fn init() -> Result<Connection, String> {
    let path = db_path()?;
    info!("Opening database at {}", path.display());

    let mut conn =
        Connection::open(&path).map_err(|e| format!("Failed to open database: {}", e))?;

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
