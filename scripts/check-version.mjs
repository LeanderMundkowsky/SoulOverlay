/**
 * Verify that the version in tauri.conf.json, Cargo.toml, and package.json all match.
 * Run: node scripts/check-version.mjs
 */
import { readFileSync } from "fs";

const tauriConf = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf-8"));
const cargoToml = readFileSync("src-tauri/Cargo.toml", "utf-8");
const packageJson = JSON.parse(readFileSync("package.json", "utf-8"));

const tauriVersion = tauriConf.version;
const cargoMatch = cargoToml.match(/^version\s*=\s*"([^"]+)"/m);
const cargoVersion = cargoMatch ? cargoMatch[1] : null;
const npmVersion = packageJson.version;

console.log(`tauri.conf.json: ${tauriVersion}`);
console.log(`Cargo.toml:      ${cargoVersion}`);
console.log(`package.json:    ${npmVersion}`);

if (tauriVersion !== cargoVersion || tauriVersion !== npmVersion) {
  console.error("\n❌ Version mismatch! All three files must have the same version.");
  process.exit(1);
}

console.log("\n✅ All versions match.");
