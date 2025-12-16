# Chapter 19: Tauri Architecture

The Whis desktop app is built with Tauri, a framework that combines a Rust backend with a web frontend. Unlike Electron (which bundles Chromium), Tauri uses the system's native webview, resulting in ~10 MB binaries instead of ~100 MB. This chapter explores how Whis structures its Tauri application, from entry point to tray icon, and how Rust communicates with Vue 3.6.

## What is Tauri?

[Tauri](https://tauri.app/) is a toolkit for building desktop applications with web technologies:

**Architecture**:
```
┌─────────────────────────────────────┐
│         Frontend (Vue 3.6)          │
│    HTML, CSS, TypeScript, Vite      │
└──────────────┬──────────────────────┘
               │ IPC (JSON-RPC)
┌──────────────┴──────────────────────┐
│         Backend (Rust)              │
│    Tauri Core, Commands, State      │
└──────────────┬──────────────────────┘
               │
┌──────────────┴──────────────────────┐
│    System WebView (WKWebView,       │
│    WebView2, WebKitGTK)             │
└─────────────────────────────────────┘
```

**Key benefits**:
1. **Small binaries**: System webview, no bundled browser
2. **Native performance**: Rust backend
3. **Security**: Process isolation, permissions model
4. **Cross-platform**: Windows, macOS, Linux

## Why Vue 3.6 (Alpha)?

From `package.json`:
```json
{
  "dependencies": {
    "vue": "^3.6.0-alpha.5"
  }
}
```

**From `whis-desktop/ui/package.json:13`**

**Vue 3.6 features**:
- **Vapor Mode** (experimental): Compiler-only reactivity, no virtual DOM
- Better TypeScript support
- Performance improvements

> **Note**: Vue 3.6 is alpha, but for a personal/hobby project, this gives early access to Vapor Mode—a compiler strategy that eliminates runtime overhead by generating optimized DOM operations at compile time.

**Vapor Mode goal**: Compile Vue templates to direct DOM manipulation:
```javascript
// Traditional: Virtual DOM diffing
createVNode('div', {}, text)

// Vapor: Direct DOM ops
div.textContent = text
```

This fits Whis's needs: simple UI, fast updates, small bundle.

## Project Structure

```
whis-desktop/
├── src/                   # Rust backend
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Tauri setup
│   ├── commands.rs       # Tauri commands (Rust → JS)
│   ├── state.rs          # Application state
│   ├── shortcuts.rs      # Global shortcuts
│   ├── tray.rs           # System tray
│   └── window.rs         # Window management
├── ui/                    # Vue frontend
│   ├── src/
│   │   ├── App.vue       # Root component
│   │   ├── main.ts       # Vue entry point
│   │   └── views/        # UI views
│   ├── package.json      # NPM dependencies
│   └── vite.config.ts    # Vite config
├── tauri.conf.json        # Tauri configuration
├── Cargo.toml            # Rust dependencies
└── icons/                # App icons
```

## Entry Point: `main.rs`

The Rust entry point handles special CLI flags before launching the GUI:

```rust
fn main() {
    // Set app_id for Wayland - must be BEFORE GTK init
    #[cfg(target_os = "linux")]
    {
        gtk::glib::set_prgname(Some("ink.whis.Whis"));
        gtk::glib::set_application_name("Whis");
    }

    let args: Vec<String> = std::env::args().collect();

    // Handle --toggle: send command to running instance
    if args.contains(&"--toggle".to_string()) {
        if let Err(e) = whis_desktop::shortcuts::send_toggle_command() {
            eprintln!("Failed to toggle: {e}");
            std::process::exit(1);
        }
        return;
    }

    // Handle --install: create .desktop file for Wayland
    if args.contains(&"--install".to_string()) {
        install_desktop_file();
        return;
    }

    // Handle --uninstall: remove .desktop file
    if args.contains(&"--uninstall".to_string()) {
        uninstall_desktop_file();
        return;
    }

    // Handle --help
    if args.contains(&"--help".to_string()) {
        println!("whis-desktop - Voice to text desktop application");
        // ... print help
        return;
    }

    // Start the GUI application
    whis_desktop::run();
}
```

**From `whis-desktop/src/main.rs:5-69`**

### Why `--toggle` Flag?

Wayland compositors that don't support global shortcuts can be configured to run:
```bash
whis-desktop --toggle
```

This communicates with the running instance via IPC (similar to CLI's `whis stop`).

### Wayland App ID

```rust
#[cfg(target_os = "linux")]
{
    gtk::glib::set_prgname(Some("ink.whis.Whis"));
    gtk::glib::set_application_name("Whis");
}
```

**From `whis-desktop/src/main.rs:9-13`**

**Why this matters**:
- Wayland uses app_id to identify applications
- Must match the `.desktop` file name: `ink.whis.Whis.desktop`
- Required for GNOME's GlobalShortcuts portal

**Desktop file** (`~/.local/share/applications/ink.whis.Whis.desktop`):
```ini
[Desktop Entry]
Name=Whis
Exec=/path/to/whis-desktop
Icon=ink.whis.Whis
StartupWMClass=ink.whis.Whis
```

**From `whis-desktop/src/main.rs:83-94`**

The `StartupWMClass` must match the app_id for proper window grouping.

## Tauri Setup: `lib.rs`

The `run()` function builds and configures the Tauri application:

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // 1. Load settings from disk
            let loaded_settings = settings::Settings::load();

            // 2. Initialize system tray (optional)
            let tray_available = match tray::setup_tray(app) {
                Ok(_) => true,
                Err(e) => {
                    eprintln!("Tray unavailable: {e}. Running in window mode.");
                    false
                }
            };

            // 3. Initialize application state
            app.manage(state::AppState::new(loaded_settings, tray_available));

            // 4. Setup global shortcuts
            shortcuts::setup_shortcuts(app);

            // 5. Start IPC listener for --toggle commands
            shortcuts::start_ipc_listener(app.handle().clone());

            // 6. Show window if no tray
            if !tray_available {
                window::show_main_window(app)?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::get_settings,
            commands::save_settings,
            commands::toggle_recording,
            // ... (20+ commands)
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**From `whis-desktop/src/lib.rs:10-64`**

### Builder Pattern

```rust
tauri::Builder::default()
    .plugin(...)
    .setup(...)
    .invoke_handler(...)
    .run(...)
```

**Tauri builder methods**:
- **`.plugin()`**: Add plugins (e.g., process control)
- **`.setup()`**: One-time initialization
- **`.invoke_handler()`**: Register commands callable from JS
- **`.run()`**: Start event loop

### Setup Phase

The `setup()` closure runs once on startup:

**1. Load Settings**:
```rust
let loaded_settings = settings::Settings::load();
```

From `~/.config/whis/settings.json` (Chapter 9).

**2. Initialize Tray**:
```rust
let tray_available = match tray::setup_tray(app) {
    Ok(_) => true,
    Err(e) => {
        eprintln!("Tray unavailable: {e}");
        false
    }
};
```

System tray is **optional**—some Linux systems don't have tray support.

**3. Manage State**:
```rust
app.manage(state::AppState::new(loaded_settings, tray_available));
```

**`app.manage()`**: Registers global state accessible from all commands.

**4. Setup Shortcuts**:
```rust
shortcuts::setup_shortcuts(app);
```

Platform-specific global shortcut registration (Chapter 22).

**5. IPC Listener**:
```rust
shortcuts::start_ipc_listener(app.handle().clone());
```

For `--toggle` CLI command communication.

**6. Show Window (Fallback)**:
```rust
if !tray_available {
    window::show_main_window(app)?;
}
```

If no tray, show window immediately. With tray, it stays hidden until user clicks tray icon.

## Tauri Configuration: `tauri.conf.json`

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Whis",
  "version": "0.5.9",
  "identifier": "ink.whis.Whis",
  "build": {
    "beforeDevCommand": { "cwd": "ui", "script": "npm run dev" },
    "devUrl": "http://localhost:5173",
    "beforeBuildCommand": { "cwd": "ui", "script": "npm run build" },
    "frontendDist": "ui/dist"
  },
  "app": {
    "withGlobalTauri": true,
    "trayIcon": {
      "iconPath": "icons/icon-idle.png",
      "iconAsTemplate": false
    },
    "windows": []
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "linux": {
      "appimage": {
        "bundleMediaFramework": false
      }
    }
  }
}
```

**From `whis-desktop/tauri.conf.json`**

### Key Fields

**`identifier`**: Unique app ID
- Format: reverse domain (`ink.whis.Whis`)
- Used for permissions, data storage, app registry

**`build.beforeDevCommand`**: Dev server
```json
{ "cwd": "ui", "script": "npm run dev" }
```
Runs `npm run dev` in `ui/` directory → Starts Vite dev server on port 5173.

**`build.devUrl`**: Where to load frontend during development
```json
"devUrl": "http://localhost:5173"
```

**`build.beforeBuildCommand`**: Production build
```json
{ "cwd": "ui", "script": "npm run build" }
```
Runs `npm run build` → Outputs to `ui/dist/`.

**`build.frontendDist`**: Where to find built frontend
```json
"frontendDist": "ui/dist"
```

**`app.withGlobalTauri`**: Enable `window.__TAURI__` global
- Allows accessing Tauri APIs from browser console (debug)

**`app.trayIcon`**: System tray icon
```json
{
  "iconPath": "icons/icon-idle.png",
  "iconAsTemplate": false
}
```

**`app.windows`**: Window configuration
```json
"windows": []
```
Empty array = no windows created at startup (tray-only mode).

**`bundle.targets`**: Build formats
```json
"targets": "all"
```
Builds all platform-appropriate formats:
- Linux: `.deb`, `.AppImage`, `.rpm`
- macOS: `.dmg`, `.app`
- Windows: `.msi`, `.exe`

## Frontend: Vue 3.6 + Vite

### Package.json

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-process": "^2.3.1",
    "vue": "^3.6.0-alpha.5"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^6.0.2",
    "vite": "npm:rolldown-vite@7.2.8",
    "vue-tsc": "^3.1.5"
  }
}
```

**From `whis-desktop/ui/package.json:10-22`**

**Dependencies**:
- **`@tauri-apps/api`**: JS bindings for Tauri commands
- **`vue`**: Vue 3.6 alpha (Vapor Mode support)

**Dev Dependencies**:
- **`@vitejs/plugin-vue`**: Vite plugin for Vue SFC
- **`vite`**: Using `rolldown-vite` (experimental Rust-based bundler)
- **`vue-tsc`**: TypeScript compiler for Vue

> **Note**: `rolldown-vite` is Vite reimplemented in Rust for faster builds. Experimental but fits the bleeding-edge theme (Vue 3.6 alpha, Vapor Mode).

### Vite Configuration

Vite builds the frontend:

**Development**: Hot Module Replacement (HMR) at `http://localhost:5173`

**Production**: Bundles to `ui/dist/`, embedded in Tauri binary

**Why Vite?**
- Fast dev server (ESM-native)
- Optimized production builds
- Vue 3 first-class support
- TypeScript out of the box

## Process Model

Tauri runs two processes:

```
┌─────────────────────────────────────┐
│     Main Process (Rust)             │
│  - Tauri core                       │
│  - Commands                         │
│  - System APIs (shortcuts, tray)    │
│  - whis-core (transcription)        │
└──────────────┬──────────────────────┘
               │ IPC (stdio)
┌──────────────┴──────────────────────┐
│     Renderer Process (WebView)      │
│  - Vue 3.6 app                      │
│  - UI rendering                     │
│  - Event handling                   │
└─────────────────────────────────────┘
```

**Communication**: JSON-RPC over stdio (process standard input/output).

**Security**: Renderer can't directly access system APIs—must call Tauri commands.

## Rust ↔ JavaScript Bridge

### Calling from JS

```typescript
// Frontend (TypeScript)
import { invoke } from '@tauri-apps/api/core';

const status = await invoke<string>('get_status');
console.log(status); // "Idle" | "Recording" | "Transcribing"
```

**Under the hood**:
1. `invoke()` sends JSON-RPC message to Rust process
2. Tauri deserializes message
3. Calls Rust `get_status` command
4. Serializes response to JSON
5. Sends back to renderer
6. `invoke()` promise resolves

### Registering Commands

```rust
// Backend (Rust)
#[tauri::command]
fn get_status(state: tauri::State<AppState>) -> String {
    let status = state.status.lock().unwrap();
    match *status {
        Status::Idle => "Idle".to_string(),
        Status::Recording => "Recording".to_string(),
        Status::Transcribing => "Transcribing".to_string(),
    }
}

// In lib.rs
.invoke_handler(tauri::generate_handler![get_status])
```

**`#[tauri::command]`** macro:
- Makes function callable from JS
- Handles serialization/deserialization
- Injects dependencies (e.g., `State`)

## Window Management

Whis can run in two modes:

### 1. Tray Mode (Default)

- No window on startup
- Tray icon visible
- Click tray → Show window
- Close window → Hide to tray

### 2. Window Mode (No Tray)

- Window shown on startup
- No tray icon
- Close window → Quit app

**Detecting tray availability**:
```rust
let tray_available = match tray::setup_tray(app) {
    Ok(_) => true,
    Err(e) => false,
};
```

Some Linux environments (Wayland, minimal DEs) don't support tray.

## Icon States

Whis changes the tray icon based on status:

| State | Icon |
|-------|------|
| Idle | `icon-idle.png` (white microphone) |
| Recording | `icon-recording.png` (red dot) |
| Transcribing | `icon-processing.png` (loading spinner) |

**Updating icon**:
```rust
app.tray_by_id("main")
    .unwrap()
    .set_icon(Some(Icon::Raw(RECORDING_ICON.to_vec())))
    .ok();
```

Icons are embedded at compile time:
```rust
const RECORDING_ICON: &[u8] = include_bytes!("../icons/icon-recording.png");
```

## Build Process

### Development

```bash
cd crates/whis-desktop
cargo tauri dev
```

**What happens**:
1. Runs `npm run dev` in `ui/` (Vite dev server)
2. Compiles Rust backend
3. Launches app, loading frontend from `http://localhost:5173`
4. Hot reloading: Changes to Vue files update instantly

### Production

```bash
cargo tauri build
```

**What happens**:
1. Runs `npm run build` in `ui/` (Vite production build)
2. Compiles Rust backend in release mode
3. Embeds `ui/dist/` into binary
4. Creates platform bundles (`.AppImage`, `.deb`, `.dmg`, etc.)

**Output** (Linux):
```
target/release/bundle/
├── appimage/
│   └── whis-desktop_0.5.9_amd64.AppImage
├── deb/
│   └── whis-desktop_0.5.9_amd64.deb
└── rpm/
    └── whis-desktop-0.5.9-1.x86_64.rpm
```

## Platform-Specific Details

### Linux

**WebView**: WebKitGTK (uses GTK 3 or 4)

**AppImage**:
- Self-contained binary
- No installation required
- `--install` flag creates `.desktop` file

**Wayland challenges**:
- App ID must match `.desktop` file
- Global shortcuts require portal or compositor config
- Tray support varies by DE

### macOS

**WebView**: WKWebView (native Safari engine)

**Bundle**: `.app` directory structure
- `Whis.app/Contents/MacOS/whis-desktop` (binary)
- `Whis.app/Contents/Resources/` (icons, assets)

**Permissions**: Requires accessibility permissions for global shortcuts

### Windows

**WebView**: WebView2 (Edge Chromium engine)
- Requires WebView2 runtime (usually pre-installed)
- Tauri installer bundles it if missing

**Bundle**: `.msi` installer or standalone `.exe`

## Summary

**Key Takeaways:**

1. **Tauri architecture**: Rust backend + web frontend, system webview
2. **Vue 3.6 alpha**: Vapor Mode for compiler-optimized reactivity
3. **Entry point**: `main.rs` handles CLI flags, `lib.rs` sets up Tauri
4. **Builder pattern**: Plugin, setup, commands, run
5. **Configuration**: `tauri.conf.json` defines build, bundle, tray
6. **Two processes**: Main (Rust) and Renderer (WebView), IPC bridge
7. **Commands**: `#[tauri::command]` for Rust → JS exposure
8. **Modes**: Tray mode (default) or window mode (fallback)

**Where This Matters in Whis:**

- App initialization (`whis-desktop/src/lib.rs`)
- Tauri configuration (`whis-desktop/tauri.conf.json`)
- Frontend setup (`whis-desktop/ui/src/main.ts`)
- Command registration (Chapter 21)
- State management (Chapter 20)

**Patterns Used:**

- **Builder pattern**: Fluent API for Tauri setup
- **Process isolation**: Security through separation
- **RAII**: Tray setup returns `Result`, cleanup on drop
- **Feature detection**: Graceful degradation (tray unavailable)

**Design Decisions:**

1. **Why Tauri over Electron?** Smaller binaries, native performance
2. **Why Vue 3.6 alpha?** Vapor Mode experimentation (hobby project)
3. **Why tray-first?** Less intrusive for background recording
4. **Why rolldown-vite?** Faster Rust-based bundler (experimental)

---

Next: [Chapter 20: Application State Management](./ch20-state.md)

This chapter explores how Whis manages shared state (settings, recording status) across Tauri commands using `Arc<Mutex<T>>`.
