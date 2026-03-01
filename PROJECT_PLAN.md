# SoulOverlay — Project Plan

A lightweight Star Citizen overlay for Windows, built with Tauri 2 + Vue 3.
Inspired by ArkanisOverlay but leaner and purpose-built.

---

## Why This Stack vs ArkanisOverlay

| Concern            | ArkanisOverlay          | SoulOverlay                   |
|--------------------|-------------------------|-------------------------------|
| UI framework       | WPF (.NET)              | Vue 3 + Tailwind (familiar)   |
| Backend language   | C#                      | Rust (Tauri 2)                |
| Distribution       | .NET runtime required   | Single .exe (Tauri bundles)   |
| Cross-compile      | Windows-only dev        | WSL2 → Windows via cargo-xwin |
| Bundle size        | ~60MB                   | ~8MB target                   |
| UEX API auth       | Hardcoded / none        | Per-user API key in settings  |

---

## Architecture

```
┌──────────────────────────────────────────────────────┐
│  Windows Desktop                                      │
│                                                       │
│  ┌─────────────────────┐   ┌──────────────────────┐  │
│  │  Star Citizen        │   │  SoulOverlay (Tauri)  │  │
│  │  (Borderless Window) │   │                      │  │
│  │                      │   │  ┌────────────────┐  │  │
│  │  game.log ──────────────────▶ LogWatcher      │  │  │
│  │                      │   │  └────────────────┘  │  │
│  │  HWND ──────────────────────▶ GameTracker     │  │  │
│  │  (FindWindow)        │   │  └────────────────┘  │  │
│  └─────────────────────┘   │                      │  │
│                             │  ┌────────────────┐  │  │
│                             │  │  Vue 3 UI       │  │  │
│                             │  │  (transparent)  │  │  │
│                             │  └────────────────┘  │  │
│                             └──────────┬───────────┘  │
│                                        │              │
│                               UEX API (HTTPS)         │
└──────────────────────────────────────────────────────┘

Global hotkey (Alt+Shift+S) → toggle overlay visibility
System tray → Show/Hide, Settings, Quit
```

---

## Prerequisites Checklist

Run these once in WSL2 before first build:

```bash
# 1. Rust (via rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add x86_64-pc-windows-msvc

# 2. cargo-xwin (MSVC cross-compiler)
cargo install cargo-xwin

# 3. Node.js (LTS) + npm
# Use nvm or pacman: sudo pacman -S nodejs npm

# 4. Tauri CLI
cargo install tauri-cli --version '^2'
# or: npm install -D @tauri-apps/cli@^2

# 5. Arch Linux system deps (for Tauri dev tooling on Linux)
sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl \
  appmenu-gtk-module libappindicator-gtk3 librsvg xdotool

# 6. Verify
rustup show          # should include x86_64-pc-windows-msvc
cargo xwin --version
node --version
npm --version
```

---

## Phase Breakdown

### Phase 0 — Project Scaffold (~1h)

**Goal:** Compile-clean skeleton. No features yet.

Steps:
1. `npm create tauri-app@latest . -- --template vue-ts` (run from `SoulOverlay/`)
2. Replace `Cargo.toml` with full dependency list from `CLAUDE.md`
3. Replace `package.json` devDeps/deps with list from `CLAUDE.md`
4. Configure `tauri.conf.json`: transparent, decorations off, alwaysOnTop, skipTaskbar, visible=false
5. Set up Tailwind: `npx tailwindcss init -p`, configure `tailwind.config.js`, import CSS
6. Set up Pinia in `src/main.ts`
7. Create stub Rust files: `window.rs`, `game_tracker.rs`, `log_watcher.rs`, `uex_client.rs`, `hotkey.rs`, `tray.rs`
8. Add `mod` declarations in `lib.rs`
9. Set up `src-tauri/capabilities/default.json`
10. `cargo xwin build --target x86_64-pc-windows-msvc` → must compile clean

**Success criteria:** `.exe` produced in `target/x86_64-pc-windows-msvc/debug/` or `release/`.

---

### Phase 1 — Window Shell (~2h)

**Goal:** Overlay window appears on screen, is transparent, has no taskbar entry, dismisses with ESC.

Steps:
1. `window.rs`: get Tauri HWND, set `WS_EX_TOOLWINDOW` via `SetWindowLongPtrW`
2. `window.rs`: `show_overlay(app, sc_hwnd)` — `AttachThreadInput` + `BringWindowToTop` + `ShowWindow`
3. `window.rs`: `hide_overlay(app, sc_hwnd)` — `ShowWindow(SW_HIDE)` + `SetForegroundWindow(sc_hwnd)`
4. `window.rs`: `set_window_geometry(app, x, y, w, h)` — `SetWindowPos` with `HWND_TOPMOST`
5. `App.vue`: full-screen semi-transparent background, listen for ESC → `invoke('hide_overlay')`
6. Expose `hide_overlay` as a Tauri command in `lib.rs`

**Success criteria:** Run `.exe`, overlay shows, ESC hides it, no entry in taskbar.

---

### Phase 2 — Game Window Tracker (~3h)

**Goal:** Overlay automatically tracks Star Citizen's window geometry and focus state.

Steps:
1. `game_tracker.rs`: `GameTracker` struct with SC `HWND`, size/position, focus state
2. `GameTracker::start(app)`: background thread with Win32 message loop
3. `FindWindow("Star Citizen", "Star Citizen")` on startup
4. `SetWinEventHook(EVENT_OBJECT_CREATE)` to wait for SC if not running
5. On SC found: read geometry, call `set_window_geometry`, emit `sc-window-found`
6. `SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE)` + 120ms debounce for resize/move
7. `SetWinEventHook(EVENT_OBJECT_FOCUS)` for focus tracking
8. `SetWinEventHook(EVENT_SYSTEM_MINIMIZESTART/END)` for minimize handling
9. On SC exit: emit `sc-window-lost`, re-register `EVENT_OBJECT_CREATE`
10. Wire `GameTracker::start()` in `lib.rs`

**Success criteria:** Rename a test window to "Star Citizen", overlay snaps to its geometry and tracks moves.

---

### Phase 3 — Global Hotkey + Tray (~1h)

**Goal:** `Alt+Shift+S` toggles overlay; system tray icon present with working menu.

Steps:
1. Register `tauri_plugin_global_shortcut` in builder
2. Default shortcut `Alt+Shift+S` → toggle show/hide (only when SC is focused)
3. `tray.rs`: `TrayIconBuilder` with menu (Show/Hide, Settings, Quit)
4. Wire tray events in `lib.rs`
5. Register `tauri_plugin_single_instance`
6. Register `tauri_plugin_store`

**Success criteria:** Hotkey toggles overlay; tray icon visible; Quit works.

---

### Phase 4 — Log Watcher (~2h)

**Goal:** Location, ship, kills, deaths parsed from `game.log` and displayed live.

Steps:
1. `log_watcher.rs`: resolve log path, use `notify::RecommendedWatcher`
2. Track byte offset, read only new bytes on each write event
3. Parse lines with regex, emit Tauri events
4. Handle log rotation (file shrinks → reset offset)
5. Wire `LogWatcher::start()` in `lib.rs`
6. `src/composables/useLogWatcher.ts`: `listen()` calls, update `game.ts` store
7. `src/stores/game.ts`: `location`, `ship`, `lastKill`, `lastDeath`
8. `StatusBar.vue`: display current location + ship

**Success criteria:** Fly in SC → location updates in overlay without restart.

---

### Phase 5 — UEX Integration (~2h)

**Goal:** Search commodities/ships via UEX API, display prices in overlay.

Steps:
1. `uex_client.rs`: `UexResult`, `PriceEntry` structs; `search()` and `get_prices()` async fns via reqwest
2. 60s in-memory cache
3. Expose as `uex_search` and `uex_prices` Tauri commands
4. `src/composables/useUex.ts`: wraps invoke with loading/error state
5. `SearchBar.vue`: 300ms debounced input → search → results list
6. `CommodityPanel.vue`: sortable price table

**Success criteria:** Type "titanium" in overlay → real UEX prices appear.

---

### Phase 6 — Settings UI (~1.5h)

**Goal:** All user preferences (hotkey, API key, log path, opacity) persist across restarts.

Steps:
1. `Settings` struct in Rust (serde Serialize/Deserialize)
2. `get_settings` / `save_settings` Tauri commands using `tauri-plugin-store`
3. `src/stores/settings.ts`: load on mount, expose to components
4. `SettingsPanel.vue`: form for all settings
5. Wire opacity to root CSS
6. Tray "Settings" → open settings panel; gear icon in overlay UI

**Success criteria:** Change hotkey → works immediately; restart app → settings persist.

---

## Key Technical Decisions

### Full-Screen Transparency
- Tauri `transparent: true` + `decorations: false` + Win32 `WS_EX_TOOLWINDOW`
- Window size matches SC window geometry (not hardcoded 1920x1080)
- SC must run in **Borderless Windowed** mode (display first-run notice if not detected)

### Cross-Compilation (WSL2 → Windows)
- `cargo-xwin` downloads MSVC SDK automatically, no Windows dev machine needed
- Target: `x86_64-pc-windows-msvc` — required for `windows` Win32 crate
- Do NOT use `x86_64-pc-windows-gnu` (MinGW incompatible with `windows` crate)

### UEX API Authentication
- Per-user API key stored via `tauri-plugin-store` (encrypted at rest)
- Never hardcoded; user sets key in Settings panel on first launch
- API key passed from Vue → Rust via Tauri command params

### Win32 Thread Safety
- All Win32 calls touching the Tauri window HWND must run on the main/UI thread
- Use `app.run_on_main_thread(|| { ... })` when calling from background threads
- `GameTracker` runs its own Win32 message loop in a dedicated thread

---

## Windows Runtime File Locations

| File                  | Path                                                                 |
|-----------------------|----------------------------------------------------------------------|
| SC game log           | `%APPDATA%\..\Roberts Space Industries\StarCitizen\LIVE\game.log`   |
| SoulOverlay settings  | `%APPDATA%\dev.souloverlay\settings.json` (tauri-plugin-store)      |
| SoulOverlay app data  | `%APPDATA%\dev.souloverlay\`                                         |
| SC install (default)  | `C:\Program Files\Roberts Space Industries\StarCitizen\`             |

---

## Estimated Timeline

| Phase | Feature                  | Est. Hours |
|-------|--------------------------|------------|
| 0     | Scaffold                 | 1h         |
| 1     | Window shell             | 2h         |
| 2     | Game window tracker      | 3h         |
| 3     | Hotkey + tray            | 1h         |
| 4     | Log watcher              | 2h         |
| 5     | UEX integration          | 2h         |
| 6     | Settings UI              | 1.5h       |
| —     | Testing + polish         | 2h         |
| **Total** |                     | **~14.5h** |
