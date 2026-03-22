#!/usr/bin/env node
// Build the SoulOverlay release installer (NSIS).
//
// Reads BACKEND_URL and SOUL_APP_TOKEN from src-tauri/.cargo/config.toml.
// Reads signing keys (TAURI_SIGNING_PRIVATE_KEY, etc.) from .env if present.
//
// Usage:
//   npm run build:release
//
// To override the backend URL without editing config.toml:
//   $env:BACKEND_URL = "https://overlay.soulreturns.com"; npm run build:release

import { execSync } from "child_process";
import { readdirSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";
import { loadDotEnv, loadCargoConfig, readTomlEnv } from "./build-utils.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const toml = loadCargoConfig(root);
const dotEnv = loadDotEnv(root);

const backendUrl =
  process.env.BACKEND_URL ??
  readTomlEnv(toml, "PROD_BACKEND_URL") ??
  readTomlEnv(toml, "BACKEND_URL");
const soulAppToken =
  process.env.SOUL_APP_TOKEN ?? readTomlEnv(toml, "SOUL_APP_TOKEN");

if (!backendUrl) {
  console.error("\n✗ Could not determine BACKEND_URL from config.toml\n");
  process.exit(1);
}

console.log(`\n→ Building SoulOverlay (release)`);
console.log(`  BACKEND_URL    = ${backendUrl}`);
console.log(`  SOUL_APP_TOKEN = ${soulAppToken ? "(set)" : "(not set)"}`);
console.log(
  `  Signing key    = ${dotEnv.TAURI_SIGNING_PRIVATE_KEY || process.env.TAURI_SIGNING_PRIVATE_KEY ? "(set)" : "(not set — .env missing?)"}\n`,
);

// --- Run tauri build ---------------------------------------------------------
const env = {
  ...dotEnv,          // .env values (signing keys, etc.)
  ...process.env,     // shell env takes precedence over .env
  BACKEND_URL: backendUrl,
  ...(soulAppToken ? { SOUL_APP_TOKEN: soulAppToken } : {}),
};

try {
  execSync("npm run tauri -- build", { env, stdio: "inherit", cwd: root });
} catch {
  console.error("\n✗ tauri build failed\n");
  process.exit(1);
}

// --- Report output -----------------------------------------------------------
const nsisDir = join(root, "src-tauri", "target", "release", "bundle", "nsis");
let installer = "(not found)";
try {
  const files = readdirSync(nsisDir).filter((f) => f.endsWith("_setup.exe"));
  if (files.length > 0) installer = join(nsisDir, files[0]);
} catch {
  // bundle dir may not exist if bundling was skipped
}

console.log(`\n✓ Done`);
console.log(`  ${installer}\n`);
