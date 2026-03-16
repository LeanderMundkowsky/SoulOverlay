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
- A [UEX Corp](https://uexcorp.space) API key for price lookups (free to obtain)

---

## Installation

Download the latest release from the [Releases](../../releases) page:

- **`SoulOverlay_x.x.x_x64_portable.exe`** — single standalone executable, no install needed
- **`SoulOverlay_x.x.x_x64-setup.exe`** — NSIS installer

> **Note:** Windows Smart App Control or SmartScreen may block unsigned executables.
> Right-click → **Run anyway**, or disable Smart App Control in
> Windows Security → App & browser control → Smart App Control settings.

---

## Developer Setup

### Option A — Native Windows (recommended, simplest)

The easiest path: clone the repo on Windows and build natively. No cross-compilation needed.

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

**3. Dev server**

```powershell
npm run tauri dev
```

This compiles the Rust backend and starts a Vite dev server with hot-reload for the frontend.

**4. Production build**

```powershell
npm run tauri build
```

Output:
- `src-tauri\target\release\bundle\portable\SoulOverlay_*_x64_portable.exe`
- `src-tauri\target\release\bundle\nsis\SoulOverlay_*_x64-setup.exe`

---

### Option B — WSL2 (Arch Linux) with cross-compilation

Develop in WSL2, cross-compile to Windows MSVC using `cargo-xwin`.

**1. System dependencies (run once)**

```bash
# Tauri Linux build tooling + GTK/WebKit (needed for `cargo check` and `tauri dev`)
sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl \
  appmenu-gtk-module libayatana-appindicator librsvg xdotool

# Rust Windows MSVC target
rustup target add x86_64-pc-windows-msvc

# cargo-xwin (downloads MSVC SDK automatically on first use, ~2 GB cached in ~/.xwin)
cargo install cargo-xwin

# Node dependencies
npm install
```

**2. Dev server** (requires WSLg or an X11 server for the window)

```bash
npm run tauri dev
```

**3. Cross-compile to Windows .exe**

```bash
# Recommended: Tauri CLI handles frontend build + Rust compile in one step
npm run tauri build -- --target x86_64-pc-windows-msvc

# Or: build frontend first, then cross-compile Rust separately
npm run build
cargo xwin build --release --target x86_64-pc-windows-msvc
```

> **Important:** Running `cargo xwin build` alone does **not** rebuild the Vue frontend.
> Always run `npm run build` first (or use `npm run tauri build`) to ensure the latest
> frontend is embedded in the binary.

**4. Run on Windows from WSL2**

```bash
# Copy the portable exe to your Windows desktop, then launch it
cp src-tauri/target/x86_64-pc-windows-msvc/release/bundle/portable/*.exe /mnt/c/Users/$USER/Desktop/
pwsh.exe -Command "Start-Process 'C:\Users\$env:USERNAME\Desktop\SoulOverlay_0.1.0_x64_portable.exe'"
```

---

## Common Commands

| Task | Command |
|---|---|
| Dev server | `npm run tauri dev` |
| Production build | `npm run tauri build` |
| Frontend type-check | `npx vue-tsc --noEmit` |
| Rust type-check (fast) | `cargo check --all-targets --workspace` (from `src-tauri/`) |
| Cross-compile (WSL2) | `npm run tauri build -- --target x86_64-pc-windows-msvc` |
| Flush cache (dev utility) | `npm run flush-cache` |

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

> **Required secrets:** `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
> must be configured in the repository's GitHub Actions secrets.

---

## Configuration

On first launch, open settings from the gear icon or the system tray → **Settings**:

| Setting | Description |
|---|---|
| **Toggle Hotkey** | Global hotkey to show/hide the overlay (default: `F6`) |
| **UEX API Key** | Your UEX Corp API key — required for price lookups |
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
    │   ├── state.rs                # AppState (all Mutex-wrapped fields)
    │   ├── app_setup.rs            # .setup() hook, prefetch, background refresh timer
    │   ├── settings.rs             # Settings struct (single source of truth for defaults)
    │   ├── config.rs               # AppPaths: centralized %APPDATA% path resolution
    │   ├── database.rs             # SQLite WAL mode, schema migrations
    │   ├── cache_store.rs          # Dual-layer cache: in-memory HashMap + SQLite (MessagePack)
    │   ├── window.rs               # Win32 overlay: show/hide, focus management, multi-monitor
    │   ├── game_tracker.rs         # SC window polling, geometry tracking
    │   ├── log_watcher.rs          # game.log tail + regex parse
    │   ├── image_proxy.rs          # Async image proxy for UEX photo URLs
    │   ├── tray.rs                 # System tray icon + menu
    │   ├── hotkey/                  # WH_KEYBOARD_LL global keyboard hook
    │   ├── uex/                    # UEX Corp API client + shared IPC types
    │   ├── providers/              # Trait-based data providers (fetch + cache per entity type)
    │   └── commands/               # All #[tauri::command] functions (IPC surface)
    ├── capabilities/default.json
    └── tauri.conf.json
```

---

## License

MIT
