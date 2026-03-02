# SoulOverlay — Copilot Instructions

SoulOverlay is a Tauri 2 + Vue 3 overlay for Star Citizen on Windows x86_64. It renders a
transparent, always-on-top window over the game showing UEX Corp commodity prices. The
full agent context lives in `AGENTS.md` — read it first for deep-dive conventions.

## Build & Quality Gates

```powershell
# Frontend type-check (must pass before committing)
npx vue-tsc --noEmit

# Rust type-check (fast, no link step) — run from src-tauri/
cargo check --all-targets --workspace

# Dev server with hot-reload
npm run tauri dev

# Full release build → src-tauri/target/release/bundle/nsis/SoulOverlay_*_setup.exe
npm run tauri build
```

No ESLint, Prettier, Vitest, or `#[cfg(test)]` suites exist. Quality is enforced by
`vue-tsc` (`strict: true`, `noUnusedLocals`, `noUnusedParameters`) and `cargo check`.

## Architecture Overview

The app has two layers that communicate exclusively via Tauri IPC:

**Rust backend** (`src-tauri/src/`): Owns all state. Key modules:
- `lib.rs` — startup sequence, `AppState` construction, `generate_handler!` registration
- `app_setup.rs` — `.setup()` hook, spawns background `prefetch_all` task
- `cache_store.rs` — dual-layer cache: in-memory `HashMap` + SQLite (`rmp-serde` blobs)
- `window.rs` — Win32 WNDPROC subclass; manages show/hide, geometry, transparency
- `hotkey/mod.rs` — `WH_KEYBOARD_LL` global hook; toggles via `PostMessageW(WM_APP+42)`
- `commands/api.rs` — primary IPC surface; returns `ApiResponse<T>` envelope

**Vue frontend** (`src/`): Stateless UI. All data lives in Rust; frontend calls `invoke`.
- Pinia stores (`game.ts`, `settings.ts`) are thin wrappers around `invoke` calls
- Composables (`useUex`, `useCache`, `useOverlayEvents`) handle polling and event listening
- `App.vue` owns the hotkey fallback for when the overlay has focus (Tauri global shortcut
  doesn't fire on the focused window)

**Initialization order** (critical — do not reorder):
1. `AppPaths::init()` — resolves `%APPDATA%\SoulOverlay\`, creates data dir
2. `logging::setup()` — fern to stderr + `soul-overlay.log`
3. `database::init()` — SQLite WAL mode + schema migrations
4. `AppState` construction → `.manage()`
5. `.setup()` → `app_setup::initialize()` → spawns prefetch thread

## IPC Contract

All `commands/api.rs` commands return:
```json
{ "ok": true,  "data": [...], "error": null,  "stale": false }
{ "ok": false, "data": null,  "error": "...", "stale": false }
```
`stale: true` means cached data was returned and a background refresh is in progress.

Frontend always uses `invoke<ApiResponse<T>>("command_name")`. Commands are registered
with full module paths in `generate_handler!` — `pub use` re-exports do NOT work there.

## Key Constraints

- **Win32 window flags**: Never add `WS_EX_TRANSPARENT` (breaks clicks) or `WS_EX_NOACTIVATE`
  while visible (prevents keyboard input).
- **Hook callbacks**: Never block in `WH_KEYBOARD_LL`. Use `try_lock()`, not `lock()`.
  Use `PostMessageW` to communicate from hook — `run_on_main_thread` can deadlock.
- **Tauri IPC**: Import `tauri::WebviewWindowExt` is NOT needed — `.hwnd()` is inherent in Tauri 2.
- **Frontend state**: `reactive()` and `computed()` are banned. Use `ref()` only; derive via `watch`.
- **CSS opacity**: Never set `opacity` on the root overlay div — only on the background layer.
- **Path resolution**: All app file paths go through `config::AppPaths`. Never call `APPDATA` directly.
- **Target**: `x86_64-pc-windows-msvc` only. No MinGW (`-gnu`), no `#[cfg(windows)]` guards.

## Vue Conventions

- `<script setup lang="ts">` everywhere. No Options API.
- Import order: Vue core → Tauri APIs → local components → stores → composables → `import type`
- Use `@/` alias in all files except `App.vue` (which uses relative paths)
- Error handling pattern: `loading/error` refs, `try/catch/finally`, `String(e)` for messages
- Tailwind palette: gray-900 panels, blue-500/600 actions, green-400 success, red-400 errors, yellow-400 warnings
- Use `e.code` (not `e.key`) for keybind capture

## Rust Conventions

- Commands return `Result<T, String>`; convert errors with `.map_err(|e| format!(...))`
- `HWND` stored as `isize` for `Send + Sync`; document all `unsafe impl` with `// SAFETY:`
- Drop `MutexGuard` explicitly before any call that may re-acquire the same lock
- Tauri events emitted as `app.emit("kebab-case-name", json!({...}))`

## Runtime Files

| File | Path |
|------|------|
| Log | `%APPDATA%\SoulOverlay\soul-overlay.log` |
| Database | `%APPDATA%\SoulOverlay\soul_overlay.db` |
| Settings | `%APPDATA%\SoulOverlay\settings.json` |

Set `RUST_LOG=debug` for verbose backend logging.
