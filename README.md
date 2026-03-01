# SoulOverlay

A lightweight, full-screen transparent overlay for Star Citizen on Windows. Displays over the game (which must run in **Borderless Windowed** mode), toggled by a global hotkey.

**Features:**
- UEX Corp commodity price lookup with live data
- Game log parsing — tracks your location, ship, kills, and deaths in real time
- Configurable global hotkey (default: `Alt+Shift+S`)
- System tray icon with quick access to settings
- Adjustable background opacity
- Settings persistence across restarts

**Stack:** [Tauri 2](https://tauri.app) (Rust backend) + [Vue 3](https://vuejs.org) (TypeScript frontend) + [Tailwind CSS](https://tailwindcss.com)

---

## Requirements

- Star Citizen must be running in **Borderless Windowed** mode
- Windows 10/11 x64
- A [UEX Corp](https://uexcorp.space) API key for commodity prices (free to obtain)

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
| Cargo check (Linux, fast) | `cargo check --all-targets --workspace` (from `src-tauri/`) |
| Cross-compile (WSL2) | `npm run tauri build -- --target x86_64-pc-windows-msvc` |

---

## Configuration

On first launch, open settings from the gear icon or the system tray → **Settings**:

| Setting | Description |
|---|---|
| **Toggle Hotkey** | Global hotkey to show/hide the overlay (default: `Alt+Shift+S`) |
| **UEX API Key** | Your UEX Corp API key — required for commodity price lookups |
| **Game Log Path** | Path to `game.log` — leave empty to use the default RSI location |
| **Overlay Opacity** | Background transparency (does not affect UI text/buttons) |

Settings are persisted automatically across restarts.

**Default log path:**
```
C:\Users\<USERNAME>\AppData\Roberts Space Industries\StarCitizen\LIVE\game.log
```

---

## Project Structure

```
SoulOverlay/
├── src/                        # Vue 3 frontend (TypeScript)
│   ├── assets/main.css         # Tailwind + global font
│   ├── components/
│   │   ├── overlay/            # SearchBar, CommodityPanel, StatusBar
│   │   └── settings/           # SettingsPanel
│   ├── composables/            # useLogWatcher, useUex
│   ├── stores/                 # game.ts, settings.ts (Pinia)
│   ├── App.vue
│   └── main.ts
└── src-tauri/
    ├── src/
    │   ├── lib.rs              # Tauri builder, AppState, all commands
    │   ├── window.rs           # Win32 overlay window management
    │   ├── game_tracker.rs     # SC window polling + WinEvent hooks
    │   ├── log_watcher.rs      # game.log tail + parse
    │   ├── uex_client.rs       # UEX HTTP client + cache
    │   ├── hotkey.rs           # Global shortcut registration
    │   └── tray.rs             # System tray icon + menu
    ├── capabilities/default.json
    └── tauri.conf.json
```

---

## License

MIT
