# SoulOverlay — AI Agent Context

This is a Star Citizen overlay application built with Tauri 2 (Rust backend) + Vue 3 (frontend).
Read this file fully before starting any task. All technical decisions are already made.

## Project Goal

A lightweight, full-screen transparent overlay for Star Citizen on Windows.
Displays over the game (which must run in Borderless Windowed mode), toggled by a global hotkey.
Features: game log parsing, UEX commodity price lookup, ship/item search, system tray, settings persistence.

## Build Environment

- Development OS: WSL2 Arch Linux
- Target OS: Windows (x86_64-pc-windows-msvc)
- Cross-compilation: cargo-xwin
- Rust target: x86_64-pc-windows-msvc
- Node.js + npm: available in WSL2
- Tauri version: 2.x (latest stable)

### Cross-compile setup (run once before first build)

```bash
# Install cargo-xwin
cargo install cargo-xwin

# Add Windows MSVC target
rustup target add x86_64-pc-windows-msvc

# Arch Linux system deps for Tauri (Linux dev tooling only — not needed for the Windows build output)
sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl \
  appmenu-gtk-module libappindicator-gtk3 librsvg xdotool

# Build command (use instead of plain `cargo tauri build`)
cargo xwin build --release --target x86_64-pc-windows-msvc
```

## Project Structure

```
SoulOverlay/
├── CLAUDE.md                         # This file
├── PROJECT_PLAN.md                   # Human-readable plan
├── src/                              # Vue 3 frontend
│   ├── assets/
│   ├── components/
│   │   ├── overlay/
│   │   │   ├── SearchBar.vue         # UEX search input + results
│   │   │   ├── CommodityPanel.vue    # Commodity price table
│   │   │   └── StatusBar.vue         # Game state (location, ship)
│   │   └── settings/
│   │       └── SettingsPanel.vue     # Hotkey, API key, log path
│   ├── composables/
│   │   ├── useLogWatcher.ts          # Listen to log-event Tauri events
│   │   └── useUex.ts                 # invoke('uex_search', ...) wrapper
│   ├── stores/
│   │   ├── game.ts                   # Pinia: location, ship, player
│   │   └── settings.ts               # Pinia: persisted preferences
│   ├── App.vue
│   └── main.ts
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs                    # Tauri builder, plugin registration
│   │   ├── main.rs                   # Entry point (calls lib::run)
│   │   ├── window.rs                 # Win32: WS_EX_TOOLWINDOW, HWND_TOPMOST
│   │   ├── game_tracker.rs           # FindWindow + SetWinEventHook for SC
│   │   ├── log_watcher.rs            # Tail game.log, parse events
│   │   ├── uex_client.rs             # reqwest calls to UEX API
│   │   ├── hotkey.rs                 # Hotkey re-registration on settings change
│   │   └── tray.rs                   # SystemTray menu builder
│   ├── capabilities/
│   │   └── default.json              # Tauri permissions
│   ├── icons/                        # App icons (generate with: npm run tauri icon)
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

## Rust Crates (src-tauri/Cargo.toml)

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-global-shortcut = "2"
tauri-plugin-store = "2"
tauri-plugin-single-instance = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
notify = "6"
log = "0.4"
env_logger = "0.11"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
  "Win32_Foundation",
  "Win32_UI_WindowsAndMessaging",
  "Win32_UI_Accessibility",
  "Win32_Graphics_Gdi",
  "Win32_System_Threading",
] }
```

## Frontend Packages (package.json devDependencies + dependencies)

```json
{
  "dependencies": {
    "vue": "^3.4",
    "pinia": "^2.1",
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-global-shortcut": "^2",
    "@tauri-apps/plugin-store": "^2"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5",
    "vite": "^5",
    "typescript": "^5",
    "tailwindcss": "^3",
    "autoprefixer": "^10",
    "postcss": "^8",
    "vue-tsc": "^2"
  }
}
```

## tauri.conf.json Key Settings

```json
{
  "app": {
    "windows": [
      {
        "label": "overlay",
        "title": "SoulOverlay",
        "transparent": true,
        "decorations": false,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "visible": false,
        "resizable": false,
        "fullscreen": false,
        "width": 1920,
        "height": 1080
      }
    ],
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    }
  },
  "bundle": {
    "active": true,
    "targets": ["nsis"],
    "identifier": "dev.souloverlay",
    "windows": {}
  }
}
```

## Win32 Overlay Approach (mirrors ArkanisOverlay)

Star Citizen runs as a Borderless Windowed application.
- Window class: `Star Citizen`
- Window title: `Star Citizen`
- Log path: `C:\Users\<USERNAME>\AppData\Roberts Space Industries\StarCitizen\LIVE\game.log`

### How the overlay works (implement in window.rs + game_tracker.rs)

1. On startup: call `FindWindow("Star Citizen", "Star Citizen")` to get SC's HWND
2. If not found: register `SetWinEventHook(EVENT_OBJECT_CREATE)` to wait for SC to launch
3. When SC is found: use `GetClientRect` + `ClientToScreen` to get its size/position
4. Resize Tauri window to match via `SetWindowPos` with `HWND_TOPMOST`
5. Set `WS_EX_TOOLWINDOW` extended style (no taskbar entry) via `SetWindowLongPtrW`
6. Track SC focus changes via `SetWinEventHook(EVENT_OBJECT_FOCUS)`
7. On hotkey press:
   - Call `AttachThreadInput` to steal input focus
   - Call `BringWindowToTop` + `ShowWindow`
   - Vue overlay becomes interactive
8. On overlay hide (Escape or hotkey again):
   - Hide Tauri window
   - Call `SetForegroundWindow(sc_hwnd)` to return focus to SC
9. Track SC window move/resize via `SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE)`
   - Debounce 120ms, then update Tauri window position/size

## Tauri IPC Events (Rust → Vue via app.emit)

| Event name          | Payload                | Description                      |
|---------------------|------------------------|----------------------------------|
| `sc-window-found`   | `null`                 | SC launched / detected           |
| `sc-window-lost`    | `null`                 | SC closed                        |
| `sc-location`       | `{ location: string }` | Player location changed          |
| `sc-death`          | `{ killer: string }`   | Player died                      |
| `sc-kill`           | `{ victim: string }`   | Player killed someone            |
| `sc-ship-changed`   | `{ ship: string }`     | Active ship changed              |

## Tauri Commands (Vue → Rust via invoke)

| Command          | Params                                       | Returns            |
|------------------|----------------------------------------------|--------------------|
| `uex_search`     | `{ query: string, api_key: string }`         | `Vec<UexResult>`   |
| `uex_prices`     | `{ commodity: string, api_key: string }`     | `Vec<PriceEntry>`  |
| `get_settings`   | none                                         | `Settings`         |
| `save_settings`  | `Settings`                                   | `()`               |
| `hide_overlay`   | none                                         | `()`               |

## game.log Parsing Rules

Log path: `C:\Users\<USERNAME>\AppData\Roberts Space Industries\StarCitizen\LIVE\game.log`

Relevant log patterns to parse (examples — refine against real logs):
```
# Location change
<Location: (.+)>

# Player death
<Actor Death> .+ killed by (.+)

# Kill
(.+) killed (.+)

# Ship change (prefix varies by SC version)
\[Ship\] (.+)
```

Use the `notify` crate to watch the file for writes (append-only).
Track byte offset — only read new bytes on each change event, never re-read the full file.
Handle log rotation: SC creates a new log file on each game launch. Re-open from offset 0 when file shrinks or inode changes.

## Capabilities (src-tauri/capabilities/default.json)

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "main-capability",
  "description": "Main window capabilities",
  "windows": ["overlay"],
  "permissions": [
    "core:default",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "global-shortcut:allow-is-registered",
    "store:allow-get",
    "store:allow-set",
    "store:allow-save",
    "http:default"
  ]
}
```

## Implementation Phases & Task Checklist

Work through these phases in order. Mark each task done as you complete it.

### Phase 0 — Project Scaffold

- [ ] Scaffold with: `npm create tauri-app@latest . -- --template vue-ts`
- [ ] Replace generated Cargo.toml dependencies with the crates listed above
- [ ] Install npm dependencies listed above
- [ ] Configure `tauri.conf.json` with settings listed above
- [ ] Set up Tailwind CSS: `npx tailwindcss init -p` → configure `tailwind.config.js` and import in `src/assets/main.css`
- [ ] Set up Pinia in `src/main.ts`
- [ ] Create stub files for all Rust modules: `window.rs`, `game_tracker.rs`, `log_watcher.rs`, `uex_client.rs`, `hotkey.rs`, `tray.rs`
- [ ] Add `mod` declarations for all stubs in `lib.rs`
- [ ] Verify `cargo xwin build --target x86_64-pc-windows-msvc` compiles clean with stubs

### Phase 1 — Window Shell

- [ ] Implement `window.rs`: on app ready, retrieve main window HWND and set `WS_EX_TOOLWINDOW` via `SetWindowLongPtrW`
- [ ] Implement `window.rs`: `pub fn show_overlay(app: &AppHandle, sc_hwnd: HWND)` — AttachThreadInput + BringWindowToTop + show Tauri window
- [ ] Implement `window.rs`: `pub fn hide_overlay(app: &AppHandle, sc_hwnd: HWND)` — hide Tauri window + SetForegroundWindow to SC
- [ ] Implement `window.rs`: `pub fn set_window_geometry(app: &AppHandle, x: i32, y: i32, w: u32, h: u32)` — SetWindowPos with HWND_TOPMOST
- [ ] Write `App.vue`: full-screen dark semi-transparent background (`bg-black/60`), listen for ESC key → invoke `hide_overlay`
- [ ] Build and test: overlay appears on screen, ESC dismisses it, no taskbar entry

### Phase 2 — Game Window Tracker

- [ ] Implement `game_tracker.rs`: define `GameTracker` struct holding current SC `HWND`, size, position, focus state
- [ ] Implement `GameTracker::start(app: AppHandle)` — spawns a background thread with its own Win32 message loop
- [ ] Implement `FindWindow("Star Citizen", "Star Citizen")` lookup on start
- [ ] Implement `SetWinEventHook(EVENT_OBJECT_CREATE)` to detect SC launch when not already running
- [ ] On window found: read initial size/position, call `set_window_geometry`, emit `sc-window-found` event
- [ ] Implement `SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE)` with 120ms debounce timer for resize/move
- [ ] On geometry change: call `set_window_geometry` with new values
- [ ] Implement `SetWinEventHook(EVENT_OBJECT_FOCUS)` — update `is_focused` state, store SC HWND
- [ ] Implement `SetWinEventHook(EVENT_SYSTEM_MINIMIZESTART)` — hide overlay when SC minimized
- [ ] Implement `SetWinEventHook(EVENT_SYSTEM_MINIMIZEEND)` — restore overlay visibility when SC restored
- [ ] On SC process exit: emit `sc-window-lost`, clear HWND, re-register `EVENT_OBJECT_CREATE` hook
- [ ] Wire `GameTracker::start()` call into `lib.rs` setup closure
- [ ] Test: launch SC (or a window titled "Star Citizen"), verify overlay tracks its geometry

### Phase 3 — Global Hotkey + Tray

- [ ] Register `tauri_plugin_global_shortcut` in `lib.rs` builder
- [ ] Default shortcut: `Alt+Shift+S`
- [ ] In hotkey handler: only call `show_overlay` if SC window is focused (check `GameTracker` state); toggle if already visible
- [ ] Implement `tray.rs`: build `TrayIconBuilder` with icon and menu (Show/Hide, Settings, Quit)
- [ ] Wire tray menu events: Settings → emit event to open settings panel; Quit → `app.exit(0)`
- [ ] Register `tauri_plugin_single_instance` — on second instance, bring overlay to front
- [ ] Register `tauri_plugin_store` for settings
- [ ] Test: hotkey toggles overlay; tray icon visible in system tray; Quit works

### Phase 4 — Log Watcher

- [ ] Implement `log_watcher.rs`: resolve log path from settings or default (`%APPDATA%\..\Roberts Space Industries\StarCitizen\LIVE\game.log`)
- [ ] Use `notify::RecommendedWatcher` to watch the log file for `EventKind::Modify`
- [ ] Track byte offset in a `u64` — on each event, seek to offset, read new bytes only
- [ ] Parse new lines with regex patterns listed above
- [ ] On match: emit corresponding Tauri event (`sc-location`, `sc-death`, etc.) via `app.emit`
- [ ] Handle log rotation: check if file size < last offset (file recreated); if so, reset offset to 0
- [ ] Wire `LogWatcher::start(app, path)` into `lib.rs` setup closure
- [ ] Create `src/composables/useLogWatcher.ts`: call `listen('sc-location', ...)` etc., update `game.ts` Pinia store
- [ ] Create `src/stores/game.ts` Pinia store: `location`, `ship`, `lastKill`, `lastDeath` reactive state
- [ ] Build `StatusBar.vue`: displays current location and ship from store
- [ ] Test: fly around in SC, confirm location updates in overlay

### Phase 5 — UEX Integration

- [ ] Implement `uex_client.rs`: define `UexResult` and `PriceEntry` structs (derive Serialize)
- [ ] Implement `async fn search(query: &str, api_key: &str) -> Result<Vec<UexResult>>` using reqwest
- [ ] Implement `async fn get_prices(commodity: &str, api_key: &str) -> Result<Vec<PriceEntry>>` using reqwest
- [ ] Add simple in-memory cache (`HashMap<String, (Instant, Value)>`) with 60s TTL
- [ ] Expose as Tauri commands: `#[tauri::command] async fn uex_search(...)` and `uex_prices(...)`
- [ ] Register commands in `lib.rs` builder: `.invoke_handler(tauri::generate_handler![uex_search, uex_prices, ...])`
- [ ] Add `http:default` permission in capabilities
- [ ] Create `src/composables/useUex.ts`: wraps `invoke('uex_search', ...)` with `loading` ref and `error` ref
- [ ] Build `SearchBar.vue`: debounced input (300ms) → `uex_search` → display results list
- [ ] Build `CommodityPanel.vue`: `uex_prices` results shown as sortable table
- [ ] Test: search "titanium", verify real prices appear

### Phase 6 — Settings UI

- [ ] Define `Settings` struct in Rust (derive Serialize/Deserialize): `hotkey: String`, `uex_api_key: String`, `log_path: Option<String>`, `overlay_opacity: f32`
- [ ] Implement `get_settings` Tauri command: load from `tauri-plugin-store`
- [ ] Implement `save_settings` Tauri command: persist to store, trigger side effects (re-register hotkey, restart log watcher if path changed)
- [ ] Create `src/stores/settings.ts` Pinia store: load settings via `invoke('get_settings')` on app mount
- [ ] Build `SettingsPanel.vue`: form fields for all settings, save button calls `invoke('save_settings', ...)`
- [ ] Wire overlay opacity: apply CSS `opacity` to root element based on settings value
- [ ] Open settings panel from tray "Settings" menu item and from a gear icon in the overlay UI
- [ ] Test: change hotkey, verify old shortcut deregistered and new one works; restart app, verify settings persist

## Dev Workflow

```bash
# Install dependencies
npm install

# Development server (for UI iteration — window opens via WSLg if available)
npm run tauri dev

# Build Windows .exe from WSL2
cargo xwin build --release --target x86_64-pc-windows-msvc

# Or via Tauri CLI:
npm run tauri build -- --target x86_64-pc-windows-msvc

# Type-check Vue/TypeScript
npx vue-tsc --noEmit

# Tauri CLI help
npm run tauri -- --help
```

## Important Notes

- The `windows` crate Win32 bindings require `x86_64-pc-windows-msvc` target. Do NOT use `x86_64-pc-windows-gnu` (MinGW) — it is incompatible with the `windows` crate.
- `cargo-xwin` downloads the MSVC SDK automatically on first use (~2GB cached in `~/.xwin`).
- During development, `npm run tauri dev` will attempt to open a window. Use WSLg or X11 forwarding for this, OR just build the `.exe` and run it on the Windows side for manual testing.
- SC **must** be in Borderless Windowed mode. The overlay should display a first-run notice explaining this requirement.
- Do NOT use `WS_EX_TRANSPARENT` — it makes the window click-through and the user cannot interact with the overlay.
- Do NOT use `WS_EX_NOACTIVATE` while the overlay is visible — it prevents keyboard input into the overlay.
- `WS_EX_TOOLWINDOW` is sufficient to hide the overlay from Alt+Tab and the taskbar.
- All Win32 calls that touch the overlay window must happen on the main/UI thread. Use `app.run_on_main_thread(|| { ... })` when calling from background threads.
