#!/usr/bin/env node
// Build the SoulOverlayTest portable exe.
//
// Reads TESTER_BACKEND_URL (and SOUL_APP_TOKEN) from src-tauri/.cargo/config.toml.
// Reads signing keys (TAURI_SIGNING_PRIVATE_KEY, etc.) from .env if present.
//
// Usage:
//   npm run build:tester
//
// To override the backend URL without editing config.toml:
//   $env:BACKEND_URL = "http://other-server:8000"; npm run build:tester

import { execSync } from "child_process";
import { copyFileSync, mkdirSync, statSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";
import { loadDotEnv, loadCargoConfig, readTomlEnv } from "./build-utils.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const toml = loadCargoConfig(root);
const dotEnv = loadDotEnv(root);

// BACKEND_URL: prefer the explicit env var override, then TESTER_BACKEND_URL
// from config.toml (so the caller can still override with $env:BACKEND_URL).
const backendUrl =
  process.env.BACKEND_URL ??
  readTomlEnv(toml, "TESTER_BACKEND_URL") ??
  readTomlEnv(toml, "BACKEND_URL");
const soulAppToken =
  process.env.SOUL_APP_TOKEN ?? readTomlEnv(toml, "SOUL_APP_TOKEN");

if (!backendUrl) {
  console.error("\n✗ Could not determine TESTER_BACKEND_URL from config.toml\n");
  process.exit(1);
}

console.log(`\n→ Building SoulOverlayTest`);
console.log(`  BACKEND_URL    = ${backendUrl}`);
console.log(`  SOUL_APP_TOKEN = ${soulAppToken ? "(set)" : "(not set)"}\n`);

// --- Run tauri build ---------------------------------------------------------
const env = {
  ...dotEnv,          // .env values (signing keys, etc.)
  ...process.env,     // shell env takes precedence over .env
  BACKEND_URL: backendUrl,
  ...(soulAppToken ? { SOUL_APP_TOKEN: soulAppToken } : {}),
};

try {
  execSync(
    "npm run tauri -- build --config src-tauri/tauri.tester.conf.json -- --features tester",
    { env, stdio: "inherit", cwd: root },
  );
} catch {
  console.error("\n✗ tauri build failed\n");
  process.exit(1);
}

// --- Copy output -------------------------------------------------------------
const src = join(root, "src-tauri", "target", "release", "soul-overlay.exe");
const outDir = join(root, "dist-tester");
const dest = join(outDir, "SoulOverlayTest.exe");

mkdirSync(outDir, { recursive: true });
copyFileSync(src, dest);

const size = (statSync(dest).size / 1024 / 1024).toFixed(1);
console.log(`\n✓ Done — SoulOverlayTest.exe (${size} MB)`);
console.log(`  ${dest}\n`);
