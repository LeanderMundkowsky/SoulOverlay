/**
 * Bump the version in package.json, tauri.conf.json, and Cargo.toml simultaneously.
 *
 * Usage:
 *   node scripts/bump-version.mjs <major|minor|patch>
 *   node scripts/bump-version.mjs 1.2.3
 */
import { readFileSync, writeFileSync } from "fs";

const arg = process.argv[2];
if (!arg) {
  console.error("Usage: node scripts/bump-version.mjs <major|minor|patch|x.y.z>");
  process.exit(1);
}

// Read current version from package.json (source of truth for current)
const pkgPath = "package.json";
const pkg = JSON.parse(readFileSync(pkgPath, "utf-8"));
const current = pkg.version;

function bumpSemver(version, part) {
  const [major, minor, patch] = version.split(".").map(Number);
  switch (part) {
    case "major":
      return `${major + 1}.0.0`;
    case "minor":
      return `${major}.${minor + 1}.0`;
    case "patch":
      return `${major}.${minor}.${patch + 1}`;
    default:
      return null;
  }
}

// Determine new version
let next;
if (["major", "minor", "patch"].includes(arg)) {
  next = bumpSemver(current, arg);
} else if (/^\d+\.\d+\.\d+$/.test(arg)) {
  next = arg;
} else {
  console.error(`Invalid argument: "${arg}". Use major, minor, patch, or an explicit x.y.z version.`);
  process.exit(1);
}

console.log(`Bumping version: ${current} → ${next}\n`);

// 1. package.json
pkg.version = next;
writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");
console.log(`  ✅ package.json`);

// 2. tauri.conf.json
const tauriPath = "src-tauri/tauri.conf.json";
const tauri = JSON.parse(readFileSync(tauriPath, "utf-8"));
tauri.version = next;
writeFileSync(tauriPath, JSON.stringify(tauri, null, 2) + "\n");
console.log(`  ✅ tauri.conf.json`);

// 3. Cargo.toml — replace only the first version = "..." line (the [package] version)
const cargoPath = "src-tauri/Cargo.toml";
let cargo = readFileSync(cargoPath, "utf-8");
let replaced = false;
cargo = cargo.replace(/^(version\s*=\s*")([^"]+)(")/m, (_match, prefix, _oldVer, suffix) => {
  replaced = true;
  return `${prefix}${next}${suffix}`;
});
if (!replaced) {
  console.error("  ❌ Could not find version field in Cargo.toml");
  process.exit(1);
}
writeFileSync(cargoPath, cargo);
console.log(`  ✅ Cargo.toml`);

console.log(`\nDone! All files updated to ${next}.`);
console.log(`Next: commit, tag with v${next}, and push.`);
