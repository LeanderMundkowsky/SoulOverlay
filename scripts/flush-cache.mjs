#!/usr/bin/env node
// Flush all cache entries from the SoulOverlay SQLite database.
//
// Usage:
//   npm run flush-cache          → %APPDATA%\SoulOverlay       (production)
//   npm run flush-cache:dev      → %APPDATA%\SoulOverlayDev    (tauri dev)
//   npm run flush-cache:test     → %APPDATA%\SoulOverlayTest   (tester build)
//
// NOTE: Run this while the app is NOT running, or the in-memory cache
// will still hold stale data until the app restarts.

import Database from "better-sqlite3";
import { join } from "path";

const args = process.argv.slice(2);
const dirName = args.includes("--test")
  ? "SoulOverlayTest"
  : args.includes("--dev")
    ? "SoulOverlayDev"
    : "SoulOverlay";

const appData = process.env.APPDATA;
if (!appData) {
  console.error("APPDATA environment variable not set");
  process.exit(1);
}

const dbPath = join(appData, dirName, "soul_overlay.db");

try {
  const db = new Database(dbPath);
  const count = db.prepare("SELECT COUNT(*) as n FROM cache_entries").get();
  db.exec("DELETE FROM cache_entries");
  db.close();
  console.log(`Flushed ${count.n} cache entries from ${dbPath}`);
} catch (e) {
  console.error(`Failed to flush cache: ${e.message}`);
  process.exit(1);
}
