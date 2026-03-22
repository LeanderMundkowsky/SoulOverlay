# SoulOverlay

A lightweight, full-screen transparent overlay for Star Citizen on Windows. Works over any
application running in **Borderless Windowed** mode, toggled by a global hotkey that fires
even in exclusive-foreground games.

**Features:**
- UEX Corp commodity, item, vehicle, and fuel price lookup with live data
- Favorites and watch list for tracking specific prices
- Fleet and user profile (with UEX secret key)
- Game log parsing — tracks your location, ship, kills, and deaths in real time
- Configurable global hotkey (default: `F6`)
- Multi-monitor aware — opens on the currently active monitor
- Configurable in-app keybinds, font size, and per-collection cache TTLs
- System tray icon with quick access to settings
- Adjustable background opacity
- Settings persistence across restarts

**Stack:** [Tauri 2](https://tauri.app) (Rust backend) + [Vue 3](https://vuejs.org) (TypeScript frontend) + [Tailwind CSS](https://tailwindcss.com)

---

## Requirements

- Windows 10/11 x64
- Star Citizen in **Borderless Windowed** mode (for SC overlay use)

---

## Installation

Download the latest release from the [Releases](../../releases) page:

- **`SoulOverlay_x.x.x_x64-setup.exe`** — NSIS installer

> **Note:** Windows Smart App Control or SmartScreen may block unsigned executables.
> Right-click → **Run anyway**, or disable Smart App Control in
> Windows Security → App & browser control → Smart App Control settings.

---

## Developer Setup

Clone the repo on Windows and build natively.

**1. Install prerequisites**

```powershell
# Rust toolchain
winget install Rustlang.Rustup

# Node.js (LTS)
winget install OpenJS.NodeJS

# Visual Studio Build Tools with C++ workload (required by Tauri)
winget install Microsoft.VisualStudio.2022.BuildTools
# In the installer select: "Desktop development with C++"
```

Restart your terminal after installing so `rustup`, `cargo`, and `node` are on `PATH`.

**2. Clone and install dependencies**

```powershell
git clone https://github.com/youruser/SoulOverlay.git
cd SoulOverlay
npm install
```

**3. Dev environment variables**

Two compile-time env vars must be set before running `npm run tauri dev` or building:

| Variable | Description | Dev default |
|---|---|---|
| `BACKEND_URL` | SoulOverlay backend base URL | `http://localhost:8000` |
| `SOUL_APP_TOKEN` | Static auth token (must match backend) | any dev token |

Copy the example config and fill it in:

```powershell
Copy-Item src-tauri\.cargo\config.toml.example src-tauri\.cargo\config.toml
# Edit the file — set BACKEND_URL and SOUL_APP_TOKEN to match your local backend
```

Or set them inline:

```powershell
$env:BACKEND_URL="http://localhost:8000"; $env:SOUL_APP_TOKEN="dev-token"
```

**4. Dev server**

```powershell
npm run tauri dev
```

This compiles the Rust backend and starts a Vite dev server with hot-reload for the frontend.

**5. Production build**

```powershell
npm run tauri build
```

Output: `src-tauri\target\release\bundle\nsis\SoulOverlay_*_x64-setup.exe`

---

## Building for Linux (Experimental)

While SoulOverlay is primarily designed for Windows, the codebase includes experimental Linux support with KDE integration. Linux builds are **not** included in official releases, but you can build locally for development or personal use.

**Prerequisites (Ubuntu/Debian):**

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libxdo-dev \
  libxi-dev \
  libxtst-dev
```

**Build steps:**

1. Clone the GitHub repository and install dependencies

2. Set environment variables (same as Windows — see Dev environment variables section above)

3. Run development server or build:

```bash
# Development with hot-reload
npm run tauri dev

# Release build (executable only, no installer)
npm run tauri build
```

The release binary will be at: `src-tauri/target/release/soul-overlay`

> **Note:** Auto-update functionality is not available on Linux. You'll need to manually rebuild to get the latest version.

---

## Common Commands

| Task | Command |
|---|---|
| Dev server | `npm run tauri dev` |
| Production build | `npm run tauri build` |
| Frontend type-check | `npx vue-tsc --noEmit` |
| Rust type-check (fast) | `cargo check --all-targets --workspace` (from `src-tauri/`) |
| Flush cache (dev utility) | `npm run flush-cache` |
| Bump version (all files) | `npm run bump patch` / `minor` / `major` / `1.2.3` |
| Verify versions match | `npm run check-version` |

---

## Database Migrations

Schema changes are managed by [`rusqlite_migration`](https://docs.rs/rusqlite_migration) in
`src-tauri/src/database.rs`. All migrations are defined as an ordered vec of `M::up(...)` entries
in the `run_migrations()` function — there are no separate migration files.

On every app launch, `to_latest()` runs automatically and applies any new migrations that haven't
been executed yet. This means schema updates are applied seamlessly after an app update — no manual
steps required from users.

**When adding a schema change:**

1. Append a new `M::up(...)` entry at the end of the vec in `database.rs`
2. Never modify or reorder existing migrations — they are immutable
3. Use `ALTER TABLE ... RENAME COLUMN` for simple renames
4. For complex changes, use the create-copy-drop-rename pattern to preserve data
5. Test by running the app against an existing database to verify the migration applies cleanly

---

## Creating a Release

Releases are built automatically by GitHub Actions when you push a version tag.

**1. Bump the version** in all three files at once:

```powershell
npm run bump patch   # 0.1.0 → 0.1.1
npm run bump minor   # 0.1.0 → 0.2.0
npm run bump major   # 0.1.0 → 1.0.0
npm run bump 1.2.3   # explicit version
```

**2. Verify versions match** (optional — bump already sets all three):

```powershell
node scripts/check-version.mjs
```

**3. Commit, tag, and push:**

```powershell
git add -A && git commit -m "Release v0.2.0"
git tag v0.2.0
git push origin main --tags
```

**4. Wait for the workflow** — GitHub Actions builds the NSIS installer, signs it with Ed25519,
and creates a **draft** release with the installer and `latest.json` update manifest.

**5. Publish** — go to GitHub → Releases, review the draft, edit release notes, then click
**Publish release**. Once published, the auto-updater endpoint goes live and existing installs
will see the update on next launch.

> **Required GitHub secrets and variables:**
> - `TAURI_SIGNING_PRIVATE_KEY` — Ed25519 private key for installer signing (secret)
> - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — passphrase for the signing key (secret)
> - `SOUL_APP_TOKEN` — static app token baked into the binary (secret)
> - `BACKEND_URL` — SoulOverlay backend base URL, e.g. `https://overlay.soulreturns.com` (variable)

---

## Backend Integration

SoulOverlay fetches a shared UEX Corp API key from the developer-controlled backend at startup.
This means users don't need to register their own UEX application — price lookups work out of
the box. Only the personal **UEX Secret Key** (for fleet and profile features) requires
per-user configuration.

The backend is a Symfony REST API at `https://overlay.soulreturns.com`. On startup, the app
calls `GET /api/config` with an `X-Soul-App-Token` header (baked into the binary at compile
time). The response provides the shared UEX API key, which is stored in memory and used for
all price lookups. If the backend is unreachable, public UEX data still works (UEX allows
unauthenticated requests at a lower rate limit).

**Debug panel** shows `Fetched` (green) or `Unavailable` (yellow) for the API key status.

---

## Configuration

On first launch, open settings from the gear icon or the system tray → **Settings**:

| Setting | Description |
|---|---|
| **Toggle Hotkey** | Global hotkey to show/hide the overlay (default: `F6`) |
| **UEX Secret Key** | Optional — enables fleet and user profile features |
| **Game Log Path** | Path to `game.log` — leave empty to use the default RSI location |
| **Overlay Opacity** | Background transparency (does not affect UI text/buttons) |
| **Font Size** | Base font size in pixels (default: 14) |
| **Cache TTLs** | Per-collection cache expiry overrides |

Settings are persisted automatically across restarts.

**Default log path:**
```
C:\Users\<USERNAME>\AppData\Roberts Space Industries\StarCitizen\LIVE\game.log
```

---

## Project Structure

```
SoulOverlay/
├── src/                            # Vue 3 frontend (TypeScript)
│   ├── assets/main.css             # Tailwind directives + global font
│   ├── bindings.ts                 # AUTO-GENERATED by tauri-specta — DO NOT EDIT
│   ├── components/
│   │   ├── icons/                  # SVG icon wrappers
│   │   ├── layout/                 # StatusBar, TabBar
│   │   ├── overlay/                # SearchBar, PricePanel, FavoritesPanel, WatchListPanel, etc.
│   │   ├── panels/                 # SettingsPanel, DebugPanel
│   │   ├── settings/               # HotkeyCapture, OpacitySlider, CacheSettingsPanel
│   │   ├── tabs/                   # SearchTab, HangarTab, InventoryTab, DetailsTab, ProfileTab
│   │   └── ui/                     # AlertBanner, LoadingSpinner, ToggleSwitch, ContextMenu, etc.
│   ├── composables/                # useUex, useCache, useLogWatcher, useOverlayEvents, useDragDrop
│   ├── stores/                     # Pinia: game, settings, favorites, hangar, details, user, watchlist
│   ├── utils/                      # imageProxy, priceFormatters, sorting
│   ├── App.vue                     # Root layout, keybind handling, ESC hierarchy
│   └── main.ts                     # Entry point — loads settings BEFORE app.mount()
└── src-tauri/
    ├── src/
    │   ├── lib.rs                  # Module declarations, tauri_specta builder, collect_commands!
    │   ├── main.rs                 # Entry point — calls lib::run()
    │   ├── constants.rs            # Compile-time BACKEND_URL, SOUL_APP_TOKEN (injected by build.rs)
    │   ├── state.rs                # AppState (all Mutex-wrapped fields, including fetched_api_key)
    │   ├── app_setup.rs            # .setup() hook, API key fetch, prefetch, background refresh timer
    │   ├── settings.rs             # Settings struct (single source of truth for defaults)
    │   ├── config.rs               # AppPaths: centralized %APPDATA% path resolution
    │   ├── database.rs             # SQLite WAL mode, schema migrations
    │   ├── cache_store.rs          # Dual-layer cache: in-memory HashMap + SQLite (MessagePack)
    │   ├── activity.rs             # ActivityLog: tracks last user action timestamps
    │   ├── logging.rs              # fern dual-logger (stderr + file)
    │   ├── platform.rs             # HWND ↔ isize helpers
    │   ├── window.rs               # Win32 overlay: show/hide, focus management, multi-monitor
    │   ├── process_tracker.rs      # SC process polling (ToolHelp32), emits sc-window-found/lost
    │   ├── log_watcher.rs          # game.log tail + regex parse
    │   ├── image_proxy.rs          # Async image proxy for UEX photo URLs
    │   ├── tray.rs                 # System tray icon + menu
    │   ├── hotkey/                 # WH_KEYBOARD_LL global keyboard hook
    │   ├── uex/                    # UEX Corp API client + shared IPC types
    │   ├── wiki/                   # Star Citizen Wiki API client + types
    │   ├── providers/              # Trait-based data providers (fetch + cache per entity type)
    │   └── commands/               # All #[tauri::command] functions (IPC surface)
    ├── build.rs                    # Injects BACKEND_URL + SOUL_APP_TOKEN at compile time
    ├── .cargo/config.toml.example  # Dev env var template (copy → config.toml, gitignored)
    ├── capabilities/default.json
    └── tauri.conf.json
```

---

## License

MIT
