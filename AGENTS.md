# SoulOverlay — Agent Context

SoulOverlay is a Tauri 2 + Vue 3 overlay for Star Citizen that shows UEX commodity
prices. It runs on Windows x86_64 as a transparent, always-on-top window.

---

## Environment

- **OS**: Windows (native) — all commands run in PowerShell or cmd
- **Stack**: Tauri 2 (Rust backend) + Vue 3 + TypeScript + Tailwind CSS frontend
- **Rust target**: `x86_64-pc-windows-msvc` — do NOT use MinGW (`windows-gnu`)

---

## Build Commands

| Task                          | Command                              |
|-------------------------------|--------------------------------------|
| Full release build            | `npm run tauri build`                |
| Frontend type-check + bundle  | `npm run build`                      |
| TypeScript type-check only    | `npx vue-tsc --noEmit`               |
| Dev server (hot-reload)       | `npm run tauri dev`                  |
| Cargo check (fast, no link)   | `cargo check --all-targets --workspace` (run from `src-tauri/`) |

**Important**: `cargo check` alone does NOT rebuild the Vue frontend. Use
`npm run tauri build` to produce a runnable binary — the Vue bundle is embedded
in the Rust binary at compile time.

**Output**: `src-tauri/target/release/bundle/nsis/SoulOverlay_*_setup.exe`

---

## Lint / Type-check

No ESLint or Prettier. The only automated quality gates are:

```powershell
npx vue-tsc --noEmit                              # frontend — must pass before committing
cargo check --all-targets --workspace             # Rust — run from src-tauri/
```

`tsconfig.json` enforces `strict: true`, `noUnusedLocals: true`, `noUnusedParameters: true`.
Fix all type errors — never use `any` or `@ts-ignore`.

---

## Testing

No automated tests (no Vitest, Jest, or `#[cfg(test)]` suites).
Verify changes manually by running the app, and always confirm both quality gates pass.

---

## Logging

Logs are written to `%APPDATA%\SoulOverlay\soul-overlay.log` (overwritten on each launch)
and to stderr. Format: `[timestamp][LEVEL][module] message`.
Set `RUST_LOG=debug` before launching to increase verbosity.

---

## Project Structure

```
SoulOverlay/
├── src/                        # Vue 3 frontend (TypeScript)
│   ├── assets/main.css         # Tailwind imports + global font
│   ├── components/
│   │   ├── overlay/            # SearchBar, CommodityPanel, StatusBar
│   │   └── settings/           # SettingsPanel (keybind capture, opacity, etc.)
│   ├── composables/            # useLogWatcher, useUex
│   ├── stores/                 # game.ts, settings.ts (Pinia)
│   ├── App.vue                 # Root layout, hotkey fallback, ESC handler
│   └── main.ts
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs              # Tauri builder, AppState, all #[tauri::command] fns
│   │   ├── main.rs             # Entry point — calls lib::run()
│   │   ├── settings.rs         # Settings struct (pure data, serde)
│   │   ├── window.rs           # Win32 overlay management, WNDPROC subclass, show/hide
│   │   ├── game_tracker.rs     # SC window polling thread, SharedGameState
│   │   ├── log_watcher.rs      # game.log tail + parse
│   │   ├── uex_client.rs       # UEX HTTP client + in-memory cache
│   │   ├── hotkey.rs           # WH_KEYBOARD_LL hook, OVERLAY_VISIBLE atomic
│   │   └── tray.rs             # System tray icon + menu
│   ├── capabilities/default.json
│   └── tauri.conf.json
└── AGENTS.md                   # This file
```

---

## TypeScript / Vue Style

### Import order (`.vue` and `.ts` files)
1. Vue core — `import { ref, watch, onMounted } from "vue"`
2. Tauri APIs — `@tauri-apps/api/core`, `@tauri-apps/api/event`
3. Local components — relative paths (`./components/...`)
4. Stores — `@/stores/...`
5. Composables — `@/composables/...`
6. Type-only imports — `import type { Foo } from "..."` last

Use `@/` alias (maps to `src/`) everywhere except `App.vue`, which uses relative paths.

### Components
- Always `<script setup lang="ts">` — no Options API, no `defineComponent`
- `defineProps<{ propName: Type }>()` — inline generic; use `withDefaults` only when needed
- `defineEmits<{ (e: "event-name"): void }>()` — typed
- Reactive state: `ref()` only — no `reactive()`
- Derive values inline in template or via `watch`; avoid `computed()`

### Naming
- Components: `PascalCase.vue`
- Composables: `use` prefix, camelCase (`useLogWatcher`)
- Stores: `use` prefix + `Store` suffix (`useGameStore`); store ID = filename (`"game"`)
- Tauri events: `kebab-case` (`"sc-window-found"`)
- Tauri commands: `snake_case` matching Rust fn name (`"uex_search"`)
- Props: `camelCase` in `defineProps`; templates use `kebab-case` automatically
- CSS custom properties: `--kebab-case`

### Types
- Prefer `interface` over `type` for object shapes
- Always use the `type` keyword for type-only imports
- Always type `invoke<ReturnType>()` explicitly
- Nullable fields: `string | null` (mirrors Rust `Option<String>`); avoid `undefined`
- Timer handles: `ReturnType<typeof setTimeout>`, not `number`

### Error handling
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
- Coerce errors to string with `String(e)` for display
- Re-throw in store actions when the call site also needs to handle it
- Clean up `listen()` handles: `unlisten?.()` in `onUnmounted`

### Tailwind
- Utility-first — no `@apply`, no custom component classes
- Opacity modifier: `bg-black/60`, `text-white/40`, `border-white/10`
- Palette: `white` (with opacity) for text/borders, `gray-900` for panels,
  `blue-500/600` for actions, `green-400` success, `red-400` errors, `yellow-400` warnings
- **Never** apply CSS `opacity` to the overlay root div — only to the background layer div

### Keybind capture
Use `document.addEventListener("keydown", handler, true)` (capture phase) during
keybind recording — `Alt`-modified keypresses are consumed by the webview if you rely
on element-level `@keydown`. Use `e.code` (not `e.key`) to identify physical keys;
`e.key` produces locale-dependent characters when Alt is held.

---

## Pinia Store Style

Use **Setup Store** pattern exclusively:

```ts
export const useXxxStore = defineStore("id", () => {
  const foo = ref<Type>(initialValue);
  async function doAction() { ... }
  return { foo, doAction };
});
```

- Export data interfaces from store files so components import type and store together
- No `$patch`, no Options-style mutations — mutate `.value` directly
- No `pinia-plugin-persistedstate` — persistence is Tauri store plugin (`tauri-plugin-store`)

---

## Rust Style

### Naming
- Structs/Enums: `PascalCase` | Functions/fields: `snake_case` | Constants: `SCREAMING_SNAKE_CASE`
- Tauri commands: `snake_case` matching the JS invoke string exactly
- Intentionally unused params: `_name` prefix

### Error handling
- Tauri commands return `Result<T, String>` — convert errors with `.map_err(|e| format!(...))`
- Log all errors/state changes: `log::error!()`, `log::warn!()`, `log::info!()`
- Discard acceptable failures: `let _ = expr;`
- `.unwrap()` only on `Mutex::lock()` (panics if poisoned — acceptable)
- Use `try_lock()` inside LL hook callbacks — never block inside a hook

### Platform guards
Every Windows-specific function needs a `#[cfg(not(windows))]` no-op stub so
`cargo check` passes:

```rust
#[cfg(windows)]
pub fn my_win32_fn(app: &AppHandle) { /* real impl */ }

#[cfg(not(windows))]
pub fn my_win32_fn(_app: &AppHandle) {
    log::info!("my_win32_fn: no-op on non-Windows");
}
```

### Thread safety
- Store `HWND` as `isize` (never the raw pointer) — required for `Send + Sync`
- Document `unsafe impl Send/Sync` with a `// SAFETY:` comment
- Use `Arc<Mutex<T>>` for shared state; `Arc<AtomicBool>` for simple flags
- Drop `MutexGuard` explicitly (`drop(guard)`) before calling anything that locks the same mutex
- Use `PostMessageW` to dispatch work to the main thread from hook callbacks —
  `run_on_main_thread` can deadlock when the overlay window is focused

### Hotkey architecture
- Global toggle uses `WH_KEYBOARD_LL` (low-level keyboard hook) — fires regardless of foreground window
- Hook callback must return fast: use atomics for all state checks, `try_lock` for mutex access
- `OVERLAY_VISIBLE` (`AtomicBool`) tracks visibility without touching Tauri APIs
- `MOD_CTRL/ALT/SHIFT` atomics are reset on every `register_hotkey` call to avoid stale state
- Toggle dispatch: `PostMessageW(overlay_hwnd, WM_APP+42, ...)` → handled by WNDPROC subclass on main thread
- Frontend fallback in `App.vue` catches the hotkey when the overlay is focused (LL hook unreliable then)

### Tauri IPC
```rust
let _ = app.emit("event-name", serde_json::json!({ "key": value }));
// Register commands:
.invoke_handler(tauri::generate_handler![cmd_one, cmd_two])
```

---

## Key Constraints — Do Not Violate

- Do NOT use `WS_EX_TRANSPARENT` — makes window click-through, breaks interaction
- Do NOT use `WS_EX_NOACTIVATE` while overlay is visible — prevents keyboard input
- Do NOT use `x86_64-pc-windows-gnu` (MinGW) — incompatible with the `windows` crate
- Do NOT import `tauri::WebviewWindowExt` — `.hwnd()` is an inherent method in Tauri 2
- Do NOT apply CSS `opacity` to the root overlay div — only to the background layer
- Do NOT call blocking operations inside `WH_KEYBOARD_LL` callbacks
