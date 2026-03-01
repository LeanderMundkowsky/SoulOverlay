# SoulOverlay — Agent Context

Read `CLAUDE.md` first for full project goals, architecture, and implementation phases.
This file covers build commands, code style, and conventions for AI agents working in this repo.

---

## Environment

- **Agent OS**: WSL2 (Arch Linux) — file editing happens here
- **Project target**: Windows x86_64 (the app runs on Windows)
- **Build/run commands that must execute on Windows**: use `pwsh.exe -Command "..."` from WSL2
- **Node/Cargo commands that work in WSL2**: run natively (no `pwsh.exe` needed for type-checks, linting, or cross-compilation via `cargo-xwin`)

```bash
# Run a Windows command from WSL2
pwsh.exe -Command "npm run tauri build"

# Cross-compile from WSL2 (no pwsh needed)
cargo xwin build --release --target x86_64-pc-windows-msvc
```

---

## Build Commands

| Task                                | Command                                                  | Where to run        |
|-------------------------------------|----------------------------------------------------------|---------------------|
| Full Windows build (recommended)    | `pwsh.exe -Command "npm run tauri build"`                | WSL2                |
| Cross-compile from WSL2             | `npm run tauri build -- --target x86_64-pc-windows-msvc` | WSL2                |
| Frontend only (type-check + bundle) | `npm run build`                                          | WSL2                |
| TypeScript type-check only          | `npx vue-tsc --noEmit`                                   | WSL2                |
| Dev server (needs display/WSLg)     | `npm run tauri dev`                                      | WSL2                |
| Cargo check (Linux target, fast)    | `cargo check --all-targets --workspace`                  | WSL2 (`src-tauri/`) |
| Cargo check (Windows target)        | `cargo xwin build --target x86_64-pc-windows-msvc`       | WSL2 (`src-tauri/`) |

**Important**: `cargo xwin build` alone does NOT rebuild the Vue frontend. Always use
`npm run tauri build` (or `npm run build` first) to ensure the frontend is bundled into
the binary. The Vue frontend is embedded in the Rust binary at build time.

**Output locations**:
- NSIS installer: `src-tauri/target/release/bundle/nsis/SoulOverlay_*_setup.exe`

---

## Lint / Type-check

There is no ESLint or Prettier configured. The only automated quality gate is TypeScript:

```bash
npx vue-tsc --noEmit        # Must pass before committing frontend changes
cargo check --all-targets   # Must pass (run from src-tauri/) before committing Rust changes
```

`tsconfig.json` enforces: `strict: true`, `noUnusedLocals: true`, `noUnusedParameters: true`.
Fix all type errors — do not use `any` or `@ts-ignore` to silence them.

---

## Testing

There are currently **no automated tests**. There is no test runner (Vitest, Jest, `#[cfg(test)]`).
Manual testing is done by running the app on Windows.

When adding new features, verify with `vue-tsc --noEmit` (frontend) and `cargo check` (Rust).

---

## Project Structure

```
SoulOverlay/
├── src/                    # Vue 3 frontend (TypeScript)
│   ├── assets/main.css     # Tailwind imports + global font
│   ├── components/
│   │   ├── overlay/        # SearchBar, CommodityPanel, StatusBar
│   │   └── settings/       # SettingsPanel
│   ├── composables/        # useLogWatcher, useUex
│   ├── stores/             # game.ts, settings.ts (Pinia)
│   ├── App.vue
│   └── main.ts
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs          # Tauri builder, AppState, all #[tauri::command] fns
│   │   ├── main.rs         # Entry point only — calls lib::run()
│   │   ├── settings.rs     # Settings struct (pure data)
│   │   ├── window.rs       # Win32 overlay window management
│   │   ├── game_tracker.rs # SC window polling + WinEvent hooks
│   │   ├── log_watcher.rs  # game.log tail + parse
│   │   ├── uex_client.rs   # UEX HTTP client + cache
│   │   ├── hotkey.rs       # Global shortcut registration
│   │   └── tray.rs         # System tray icon + menu
│   ├── capabilities/default.json
│   └── tauri.conf.json
├── CLAUDE.md               # Authoritative project spec — read first
└── AGENTS.md               # This file
```

---

## TypeScript / Vue Style

### Imports order (`.vue` and `.ts` files)
1. Vue core (`ref`, `watch`, `onMounted`, etc.) — named imports from `"vue"`
2. Tauri APIs (`@tauri-apps/api/core`, `@tauri-apps/api/event`)
3. Local components (relative paths: `./components/...`)
4. Stores (`@/stores/...`)
5. Composables (`@/composables/...`)
6. Type-only imports using the `type` keyword last

Use `@/` alias (maps to `src/`) in composables, stores, and non-`App.vue` components.
Use relative paths in `App.vue` for direct component imports.

### Components
- Always `<script setup lang="ts">` — no Options API, no `defineComponent`
- `defineProps<{ propName: Type }>()` — inline generic, no `withDefaults` unless defaults needed
- `defineEmits<{ (e: "event-name"): void }>()` — typed
- State: `ref()` for all reactive values; no `reactive()`
- No `computed()` — derive inline in template or via `watch`

### Naming
- Components: `PascalCase.vue`
- Composables: `use` prefix, camelCase (`useLogWatcher`, `useUex`)
- Stores: `use` prefix + `Store` suffix (`useGameStore`, `useSettingsStore`)
- Store IDs: lowercase string matching filename (`"game"`, `"settings"`)
- Tauri event names: `kebab-case` (`"sc-window-found"`, `"open-settings"`)
- Tauri command names: `snake_case` matching Rust fn name (`"uex_search"`, `"get_settings"`)
- Props: `camelCase` in `defineProps`; templates auto-convert to `kebab-case`
- CSS custom properties: `--kebab-case` (e.g., `--overlay-bg-opacity`)

### Types
- Prefer `interface` over `type` for data shapes
- Always use `type` keyword for type-only imports
- Always type `invoke<ReturnType>()` explicitly
- Use `string | null` (mirrors Rust `Option<String>`); avoid `undefined` for nullable fields
- Timer handles: `ReturnType<typeof setTimeout>` not `number`

### Error handling
```ts
const loading = ref(false);
const error = ref<string | null>(null);

async function doSomething() {
  loading.value = true;
  error.value = null;
  try {
    const result = await invoke<MyType>("command");
    // ...
  } catch (e) {
    error.value = String(e);
    console.error("context:", e);
  } finally {
    loading.value = false;
  }
}
```
- Always coerce errors to string via `String(e)` for display
- Re-throw in store actions when the call site also needs to handle it
- Clean up `listen()` handles via optional chaining: `unlisten?.()` in `onUnmounted`

### Tailwind conventions
- Utility-first — no `@apply`, no custom component classes
- Opacity modifier syntax: `bg-black/60`, `text-white/40`, `border-white/10`
- Color palette: `white` (with opacity) for text/borders, `gray-900` for panels,
  `blue-500/600` for actions, `green-400` for positive, `red-400` for errors, `yellow-400` for warnings
- Dynamic opacity: use CSS custom property + `:style` binding; never apply `opacity` to the
  entire overlay root (it dims the UI chrome — apply it only to the background layer)

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

- Export data interfaces from store files so components can import type + store together
- No `$patch`, no Options-style mutations — mutate `.value` directly inside actions
- No `pinia-plugin-persistedstate` — persistence is handled via Tauri store plugin on the Rust side

---

## Rust Style

### Naming
- Structs/Enums: `PascalCase`
- Functions/methods/fields: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Tauri commands: `snake_case` matching the JS-side string exactly
- Intentionally unused variables: `_name` prefix

### Error handling
- Tauri commands return `Result<T, String>` — convert all errors to `String`
- Use `.map_err(|e| format!("context: {}", e))` on fallible ops
- Use `log::error!()`, `log::warn!()`, `log::info!()` at all error/state-change sites
- Discard acceptable failures with `let _ = expr;`
- `.unwrap()` only on `Mutex::lock()` (standard practice — panics if poisoned)

### Platform guards
Every Windows-specific function must have a `#[cfg(not(windows))]` no-op stub:

```rust
#[cfg(windows)]
pub fn my_win32_fn(app: &AppHandle) {
    // real implementation
}

#[cfg(not(windows))]
pub fn my_win32_fn(_app: &AppHandle) {
    log::info!("my_win32_fn: no-op on non-Windows");
}
```

This is required so `cargo check` passes on the Linux dev machine.

### Thread safety
- Store `HWND` as `isize`, never as the raw pointer type — required for `Send + Sync`
- Document `unsafe impl Send/Sync` with a `// SAFETY:` comment
- Use `Arc<Mutex<T>>` for shared state; `Arc<AtomicBool>` for simple flags
- All Win32 calls touching the Tauri window must use `app.run_on_main_thread(move || { ... })`
- Drop `MutexGuard` explicitly (`drop(guard)`) before calling functions that also lock the same mutex

### Tauri IPC (Rust side)
```rust
// Emit event to frontend
let _ = app.emit("event-name", serde_json::json!({ "key": value }));
let _ = app.emit("zero-payload-event", ());

// Command registration in lib.rs builder
.invoke_handler(tauri::generate_handler![cmd_one, cmd_two])
```

---

## Key Constraints (do not violate)

- Do NOT use `WS_EX_TRANSPARENT` — makes window click-through, breaks interaction
- Do NOT use `WS_EX_NOACTIVATE` while overlay is visible — prevents keyboard input
- Do NOT use `x86_64-pc-windows-gnu` (MinGW) — incompatible with the `windows` crate
- Do NOT import `tauri::WebviewWindowExt` — `.hwnd()` is an inherent method in Tauri 2
- Do NOT call `tauri_plugin_global_shortcut::init()` — use `Builder::new().build()` instead
- Do NOT apply CSS `opacity` to the root overlay div — apply it only to the background layer
