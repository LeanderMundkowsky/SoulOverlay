// Shared utilities for build scripts.

import { readFileSync } from "fs";
import { join } from "path";

// --- .env loader -------------------------------------------------------------
// Loads KEY=VALUE pairs from .env in the project root into the returned object.
// Skips blank lines and comments. Does not mutate process.env.
export function loadDotEnv(root) {
  const envPath = join(root, ".env");
  try {
    const lines = readFileSync(envPath, "utf8").split(/\r?\n/);
    const vars = {};
    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith("#")) continue;
      const eq = trimmed.indexOf("=");
      if (eq === -1) continue;
      const key = trimmed.slice(0, eq).trim();
      let val = trimmed.slice(eq + 1).trim();
      // Strip optional surrounding quotes
      if ((val.startsWith('"') && val.endsWith('"')) ||
          (val.startsWith("'") && val.endsWith("'"))) {
        val = val.slice(1, -1);
      }
      vars[key] = val;
    }
    return vars;
  } catch {
    return {};
  }
}

// --- config.toml reader ------------------------------------------------------
// Reads a value from lines like:  KEY = { value = "..." }
export function readTomlEnv(toml, key) {
  const match = toml.match(
    new RegExp(`^\\s*${key}\\s*=\\s*\\{\\s*value\\s*=\\s*"([^"]*)"`, "m"),
  );
  return match ? match[1] : null;
}

export function loadCargoConfig(root) {
  const configPath = join(root, "src-tauri", ".cargo", "config.toml");
  try {
    return readFileSync(configPath, "utf8");
  } catch {
    console.error(`\n✗ Could not read ${configPath}`);
    console.error(
      "  Copy config.toml.example to config.toml and fill in your values.\n",
    );
    process.exit(1);
  }
}
