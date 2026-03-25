use log::info;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

#[cfg(target_os = "windows")]
use chrono::Utc;
#[cfg(target_os = "windows")]
use log::{debug, error};
#[cfg(target_os = "windows")]
use tauri_plugin_updater::UpdaterExt;

use crate::state::AppState;

const GITHUB_REPO: &str = "Soul-Returns/SoulOverlay";

/// Information about an available update, sent to the frontend via Tauri events.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdateInfo {
    pub version: String,
    pub date: Option<String>,
    pub body: Option<String>,
}

/// Fetch the release body from the GitHub Releases API for the given version tag.
/// Falls back to `None` silently if the request fails or the release isn't found.
#[cfg(target_os = "windows")]
async fn fetch_github_release_body(version: &str) -> Option<String> {
    use crate::commands::backend::http_client;

    let client = http_client().ok()?;
    let url = format!(
        "https://api.github.com/repos/{}/releases/tags/v{}",
        GITHUB_REPO, version
    );
    debug!("Fetching GitHub release body from {}", url);

    let resp = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "SoulOverlay-updater")
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        debug!("GitHub release API returned {}", resp.status());
        return None;
    }

    let json: serde_json::Value = resp.json().await.ok()?;
    json["body"].as_str().map(|s| s.to_string())
}

/// Check GitHub releases for a newer version.
/// Called from the startup auto-check in app_setup.rs.
/// Also exposed as a command so `UpdateInfo` is included in specta bindings.
#[tauri::command]
#[specta::specta]
pub async fn check_for_update(_app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    #[cfg(target_os = "windows")]
    {
        info!("Checking for updates...");

        let updater = _app.updater().map_err(|e| format!("Updater not available: {}", e))?;

        let update = updater
            .check()
            .await
            .map_err(|e| format!("Update check failed: {}", e))?;

        match update {
            Some(u) => {
                info!("Update available: v{}", u.version);
                let body = fetch_github_release_body(&u.version).await
                    .or_else(|| u.body.clone());
                Ok(Some(UpdateInfo {
                    version: u.version.clone(),
                    date: u.date.map(|d| d.to_string()),
                    body,
                }))
            }
            None => {
                info!("Already up to date");
                Ok(None)
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        info!("Auto-update not supported on this platform");
        Ok(None)
    }
}

/// Back up the database and settings files before an update.
/// Keeps the last 3 backups, removes older ones.
/// Called from the frontend before triggering download + install.
#[tauri::command]
#[specta::specta]
pub async fn backup_before_update(_state: State<'_, AppState>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let backup_dir = _state.paths.data_dir.join("backups");
        std::fs::create_dir_all(&backup_dir)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();

        // Back up database
        let db_src = &_state.paths.db_file;
        if db_src.exists() {
            let db_dst = backup_dir.join(format!("soul_overlay.db.backup-{}", timestamp));
            std::fs::copy(db_src, &db_dst)
                .map_err(|e| format!("Failed to back up database: {}", e))?;
            info!("Database backed up to {}", db_dst.display());
        }

        // Back up settings
        let settings_src = &_state.paths.settings_file;
        if settings_src.exists() {
            let settings_dst = backup_dir.join(format!("settings.json.backup-{}", timestamp));
            std::fs::copy(settings_src, &settings_dst)
                .map_err(|e| format!("Failed to back up settings: {}", e))?;
            info!("Settings backed up to {}", settings_dst.display());
        }

        // Prune old backups: keep last 3 sets (sorted by name = sorted by timestamp)
        prune_backups(&backup_dir, "soul_overlay.db.backup-", 3);
        prune_backups(&backup_dir, "settings.json.backup-", 3);

        info!("Pre-update backup complete");
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Backup not supported on this platform".to_string())
    }
}

#[cfg(target_os = "windows")]
/// Keep only the newest `keep` files matching the given prefix, delete the rest.
fn prune_backups(dir: &std::path::Path, prefix: &str, keep: usize) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|n| n.starts_with(prefix))
                .unwrap_or(false)
        })
        .collect();

    // Sort by name descending (newest first since names contain timestamps)
    entries.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    for old in entries.into_iter().skip(keep) {
        if let Err(e) = std::fs::remove_file(old.path()) {
            error!("Failed to remove old backup {}: {}", old.path().display(), e);
        }
    }
}
