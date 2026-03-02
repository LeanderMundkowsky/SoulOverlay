# SoulOverlay — Agent Context

## Shell Rules

- Never use `2>nul` in bash commands — this creates a literal file named `nul` instead of
  redirecting stderr. Always use `2>/dev/null`.

SoulOverlay is a Tauri 2 + Vue 3 overlay for Star Citizen that shows UEX commodity
prices. It runs on Windows x86_64 as a transparent, always-on-top window. Data from
the UEX Corp API is cached in a local SQLite database with per-collection TTLs and
served from an in-memory mirror for instant search.

## Build Commands

| Task                         | Command                                          |
|------------------------------|--------------------------------------------------|
| Full release build           | `npm run tauri build`                            |
| Frontend type-check + bundle | `npm run build`                                  |
| TypeScript type-check only   | `npx vue-tsc --noEmit`                           |
| Dev server (hot-reload)      | `npm run tauri dev`                              |
| Cargo check (fast, no link)  | `cargo check --all-targets --workspace` (from `src-tauri/`) |

`cargo check` does NOT rebuild the Vue frontend. Use `npm run tauri build` for a
runnable binary. Output: `src-tauri/target/release/bundle/nsis/SoulOverlay_*_setup.exe`

## Lint / Type-check / Testing

No ESLint, Prettier, Vitest, Jest, or `#[cfg(test)]` suites. Quality gates:

```powershell
npx vue-tsc --noEmit                        # frontend — must pass before committing
cargo check --all-targets --workspace       # Rust — run from src-tauri/
```

`tsconfig.json` enforces `strict: true`, `noUnusedLocals: true`, `noUnusedParameters: true`.
Fix all type errors. Never use `any` or `@ts-ignore`. Verify manually by running the app.

## Logging

Logs: `%APPDATA%\SoulOverlay\soul-overlay.log` (overwritten each launch) + stderr.
Database: `%APPDATA%\SoulOverlay\soul_overlay.db` (SQLite, WAL mode, persists across launches).
Format: `[timestamp][LEVEL][module] message`. Set `RUST_LOG=debug` for verbose output.

## Project Structure

```
src/                                 # Vue 3 + TypeScript frontend
├── assets/main.css                  # Tailwind directives + global font
├── components/
│   ├── icons/                       # 14 SVG icon wrappers (template-only, use currentColor)
│   ├── layout/                      # StatusBar, TabBar
│   ├── overlay/                     # SearchBar, SearchResultRow, CommodityPanel
│   ├── panels/                      # SettingsPanel, DebugPanel
│   ├── settings/                    # HotkeyCapture, OpacitySlider, SettingsField, CacheSettingsPanel
│   ├── tabs/                        # SearchTab, InventoryTab, PlaceholderTab
│   └── ui/                          # AlertBanner, LoadingSpinner, PanelHeader, ToggleSwitch
├── composables/                     # useUex, useCache, useLogWatcher, useOverlayEvents, useHotkeyMatch
├── stores/                          # game.ts, settings.ts (Pinia setup stores)
├── App.vue                          # Root layout, hotkey fallback, ESC handler
└── main.ts
src-tauri/src/                       # Rust backend
├── lib.rs                           # Module declarations + Tauri builder + run()
├── main.rs                          # Entry point — calls lib::run()
├── config.rs                        # AppPaths: centralized path resolution + settings I/O
├── state.rs                         # AppState struct (all Mutex-wrapped fields + AppPaths)
├── app_setup.rs                     # .setup() initialization sequence + background prefetch
├── settings.rs                      # Settings struct (pure serde data)
├── database.rs                      # SQLite connection init, WAL mode, schema migrations
├── cache_store.rs                   # CacheStore: per-collection TTL, in-memory HashMap + SQLite persistence
├── logging.rs                       # fern dual-logger setup (stderr + file, path from config)
├── platform.rs                      # HWND <-> isize helpers (breaks circular deps)
├── window.rs                        # Win32 overlay: WNDPROC subclass, show/hide, geometry
├── game_tracker.rs                  # SC window polling thread, SharedGameState
├── log_watcher.rs                   # game.log tail + regex parse + Tauri event emit
├── uex_client.rs                    # UEX HTTP client, fetch_all_* + search_* functions
├── tray.rs                          # System tray icon + menu
├── commands/                        # All #[tauri::command] functions
│   ├── mod.rs                       # pub mod for each submodule
│   ├── api.rs                       # api_search, api_commodity_prices + ApiResponse<T> envelope
│   ├── uex.rs                       # uex_search, uex_search_all, uex_prices (legacy)
│   ├── cache.rs                     # cache_status, cache_refresh, cache_refresh_all + prefetch_all
│   ├── settings.rs                  # get_settings, save_settings
│   ├── overlay.rs                   # show_overlay_cmd, hide_overlay_cmd
│   └── debug.rs                     # get_debug_info, get_game_state + DebugInfo/GameState types
└── hotkey/                          # Global keyboard hook
    ├── mod.rs                       # HookHandle, register_hotkey, LL hook callback, atomics
    └── keymap.rs                    # token_to_vk() VK code lookup table
```

## TypeScript / Vue Style

**Import order** (all `.vue` and `.ts` files):
1. Vue core (`ref`, `watch`, `onMounted`, ...)
2. Tauri APIs (`@tauri-apps/api/core`, `@tauri-apps/api/event`)
3. Local components
4. Stores (`@/stores/...`)
5. Composables (`@/composables/...`)
6. Type-only imports (`import type { Foo } from "..."`) last

Use `@/` alias everywhere except `App.vue` (relative paths).

**Components**: Always `<script setup lang="ts">`. No Options API, no `defineComponent`.
Props via `defineProps<{ prop: Type }>()`. Emits via `defineEmits<{ (e: "name"): void }>()`.
Reactive state: `ref()` only — no `reactive()`. Derive values via `watch` or inline; no `computed()`.

**Naming**: Components `PascalCase.vue` | Composables `useCamelCase` | Stores `useXxxStore`
(ID = filename) | Tauri events `kebab-case` | Tauri commands `snake_case` | Props `camelCase`

**Types**: `interface` for object shapes. Always `import type` for type-only imports.
Always `invoke<ReturnType>()`. Nullable: `T | null` (not `undefined`).
Timer handles: `ReturnType<typeof setTimeout>`.

**Error handling**:
```ts
const loading = ref(false);
const error = ref<string | null>(null);
async function doSomething() {
  loading.value = true;
  error.value = null;
  try {
    const result = await invoke<MyType>("command");
  } catch (e) {
    error.value = String(e);
    console.error("context:", e);
  } finally {
    loading.value = false;
  }
}
```
Re-throw in store actions when the caller also needs to handle. Clean up `listen()` handles
in `onUnmounted`. Use `e.code` (not `e.key`) for keybind capture (locale-independent).

**Tailwind**: Utility-first only (no `@apply`). Opacity via modifiers (`bg-black/60`).
Palette: white+opacity for text/borders, gray-900 for panels, blue-500/600 actions,
green-400 success, red-400 errors, yellow-400 warnings.
Never apply CSS `opacity` to the overlay root div — only to the background layer.

**Pinia**: Setup Store pattern exclusively. Export interfaces from store files.
No `$patch`, no Options mutations. Settings persisted via Rust backend (`invoke`).

## Path Configuration

All application files live under a single directory: `%APPDATA%\SoulOverlay\`.
Path resolution is centralized in `config.rs` via the `AppPaths` struct — no other
module should resolve `APPDATA` or hard-code file paths.

| File                | Path                                          | Purpose                    |
|---------------------|-----------------------------------------------|----------------------------|
| `soul-overlay.log`  | `%APPDATA%\SoulOverlay\soul-overlay.log`      | App log (overwritten each launch) |
| `soul_overlay.db`   | `%APPDATA%\SoulOverlay\soul_overlay.db`       | SQLite cache database      |
| `settings.json`     | `%APPDATA%\SoulOverlay\settings.json`         | User settings (plain JSON) |

**Initialization order** in `lib::run()`:
1. `AppPaths::init()` — resolves all paths, creates data directory
2. `logging::setup(&paths.log_file)` — sets up fern loggers
3. `database::init(&paths.db_file)` — opens SQLite, runs migrations
4. Build `AppState` (includes `paths`, `cache`, `current_settings`, etc.)
5. Tauri builder `.manage(app_state)` + `.setup(app_setup::initialize)`

**Settings I/O**: `AppPaths` provides `load_settings()` and `save_settings()` methods
that read/write `settings.json` as plain JSON. No `tauri-plugin-store` dependency —
settings are stored as a direct `serde_json` serialization of the `Settings` struct.

**SC game log**: The Star Citizen game log path (`game.log`) is resolved separately
in `log_watcher::default_log_path()` since it lives outside the app data directory.

## Cache / Database Architecture

**Storage stack**: SQLite (`rusqlite` with `bundled` feature) for persistence, in-memory
`HashMap<String, MemoryEntry>` for instant reads. Data is serialized to MessagePack
(`rmp-serde`) blobs in the `cache_entries` table. Schema migrations via `rusqlite_migration`.

**CacheStore** (`cache_store.rs`) is the central cache. It lives on `AppState.cache`
(not behind a `Mutex` — it manages its own internal locks). Key methods:
- `put<T>(key, collection, data)` — serialize + write to both memory and SQLite
- `get<T>(key) -> CacheResult<T>` — returns `Fresh(T)`, `Stale(T)`, or `Missing`
- `invalidate(key)` / `invalidate_collection(c)` / `invalidate_all()`
- `status() -> Vec<CollectionStatus>` — for the settings UI

**Collection enum** defines known data types with per-collection TTLs:

| Collection         | TTL      | Storage key pattern       | Prefetched on startup |
|--------------------|----------|---------------------------|-----------------------|
| `Commodities`      | 10 min   | `commodities`             | Yes                   |
| `CommodityPrices`  | 10 min   | `commodity_prices:{id}`   | No (per-commodity)    |
| `Vehicles`         | 24 hours | `vehicles`                | Yes                   |
| `Items`            | 24 hours | `items`                   | Yes                   |
| `Locations`        | 24 hours | `locations`               | Yes                   |

**Data flow**:
1. On startup, `CacheStore::new()` loads all SQLite rows into memory
2. `app_setup` spawns a background task (`prefetch_all`) that fetches any expired collections
3. Search commands (`api_search`, etc.) read from the in-memory mirror — `Fresh` data is
   returned directly, `Stale` data is returned with `stale: true` in the `ApiResponse`
   envelope, `Missing` data triggers a direct API fallback
4. Price lookups use per-commodity keys (`commodity_prices:42`), fetched on demand and cached

**ApiResponse envelope** (returned by all `commands/api.rs` endpoints):
```json
{ "ok": true,  "data": [...], "error": null,  "stale": false }
{ "ok": true,  "data": [...], "error": null,  "stale": true  }
{ "ok": false, "data": null,  "error": "...", "stale": false }
```

The `stale` flag lets the frontend show a "refreshing..." banner while serving cached data.

## Rust Style

**Naming**: Structs `PascalCase` | functions/fields `snake_case` | constants `SCREAMING_SNAKE`
Tauri commands: `snake_case` matching the JS `invoke` string. Unused params: `_name` prefix.

**Error handling**: Commands return `Result<T, String>`. Convert with `.map_err(|e| format!(...))`.
Discard acceptable failures: `let _ = expr;`. `.unwrap()` only on `Mutex::lock()`.
`try_lock()` inside LL hook callbacks — never block in a hook.

**Platform**: Windows-only (x86_64-pc-windows-msvc). No `#[cfg(windows)]` guards or
non-Windows stubs — Win32 APIs are called directly without conditional compilation.

**Thread safety**: Store `HWND` as `isize` for `Send + Sync`. Document `unsafe impl`
with `// SAFETY:`. Use `Arc<Mutex<T>>` for shared state, `Arc<AtomicBool>` for flags.
Drop `MutexGuard` explicitly before calling code that may lock. Use `PostMessageW` from
hook callbacks — `run_on_main_thread` can deadlock when the overlay is focused.

**Commands module**: `tauri::generate_handler!` requires full paths (e.g.,
`commands::uex::uex_search`). `pub use` re-exports do NOT carry the hidden `__cmd__` items.

**Hotkey architecture**: `WH_KEYBOARD_LL` hook with atomics for fast state checks.
`OVERLAY_VISIBLE` AtomicBool tracks visibility. Modifier atomics reset on `register_hotkey`.
Toggle via `PostMessageW(hwnd, WM_APP+42, ...)` handled by WNDPROC subclass.
Frontend fallback in `App.vue` catches the hotkey when overlay is focused.

**Tauri IPC**: Events via `app.emit("kebab-name", json!({...}))`. Commands registered via
`generate_handler!` with full module paths.

## Key Constraints — Do Not Violate

- Do NOT use `WS_EX_TRANSPARENT` — makes window click-through, breaks interaction
- Do NOT use `WS_EX_NOACTIVATE` while overlay is visible — prevents keyboard input
- Do NOT use `x86_64-pc-windows-gnu` (MinGW) — incompatible with the `windows` crate
- Do NOT import `tauri::WebviewWindowExt` — `.hwnd()` is inherent in Tauri 2
- Do NOT apply CSS `opacity` to the root overlay div — only to the background layer
- Do NOT call blocking operations inside `WH_KEYBOARD_LL` callbacks
- Do NOT use `any`, `@ts-ignore`, `reactive()`, or `computed()` in the frontend
