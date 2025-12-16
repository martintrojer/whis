# Part VI: Tauri Desktop Application

Now we explore `whis-desktop`—the GUI application built with Tauri that bridges Rust and Vue.

## What You'll Learn

This part covers the hybrid desktop app:

- **Chapter 19: Tauri Architecture** - Rust backend, WebView frontend, IPC via `invoke`
- **Chapter 20: Application State** - State management with `Mutex`, state machines
- **Chapter 21: Tauri Commands** - Exposing Rust functions to JavaScript
- **Chapter 22: Global Shortcuts** - X11, Wayland Portal, and fallback strategies

## The Desktop App's Complexity

`whis-desktop` is the most sophisticated application because it must:

1. **Handle multiple environments** - X11, Wayland (GNOME/KDE/Hyprland), macOS, Windows
2. **Manage global shortcuts** - Different APIs per platform
3. **Bridge Rust ↔ Vue** - Type-safe communication via Tauri commands
4. **Operate windowless** - System tray without a visible window

```admonish warning
Chapter 22 (Global Shortcuts) is particularly complex due to the Wayland portal system. We'll break it down step-by-step.
```

## Code Organization

Files in `crates/whis-desktop/src/`:

- `lib.rs` - Tauri app setup and plugin registration
- `state.rs` - Application state struct
- `commands.rs` - Tauri commands exposed to frontend
- `shortcuts.rs` - Platform detection and shortcut binding (largest file)
- `tray.rs` - System tray icon and menu

## Time Estimate

- **Quick read**: ~1.5 hours
- **Thorough read with code exploration**: ~3-4 hours
- **With exercises**: +1 hour

---

Let's start with [Chapter 19: Tauri Architecture](./ch19-architecture.md).
