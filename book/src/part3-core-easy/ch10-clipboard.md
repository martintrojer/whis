# Chapter 10: Clipboard Operations

After transcribing your voice to text, Whis copies the result to your system clipboard so you can paste it anywhere. This sounds simple, but clipboard access is surprisingly platform-specific and has sandboxing challenges. This chapter explores how Whis uses the `arboard` crate and works around Flatpak/Wayland limitations.

## The Clipboard Interface

Whis exposes a single public function:

```rust
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    // In Flatpak, use bundled wl-copy directly
    if is_flatpak() {
        return copy_via_wl_copy(text);
    }

    // Standard approach for non-Flatpak environments
    let mut clipboard = Clipboard::new()
        .context("Failed to access clipboard")?;
    clipboard
        .set_text(text)
        .context("Failed to copy text to clipboard")?;

    Ok(())
}
```

**From `whis-core/src/clipboard.rs:36-50`**

**Two code paths**:
1. **Flatpak**: Use bundled `wl-copy` command (Wayland-specific)
2. **Everything else**: Use `arboard` crate (cross-platform)

Why two paths? Let's explore.

## The `arboard` Crate

[`arboard`](https://github.com/1Password/arboard) is a cross-platform clipboard library. Under the hood, it uses:

- **Linux X11**: `x11-clipboard` crate
- **Linux Wayland**: `wlr-data-control` protocol
- **macOS**: `NSPasteboard` APIs
- **Windows**: `Win32` clipboard APIs

**Basic usage**:

```rust
use arboard::Clipboard;

let mut clipboard = Clipboard::new()?;
clipboard.set_text("Hello from Rust!")?;

// Later, read it back
let content = clipboard.get_text()?;
```

**Why not just use `arboard` everywhere?**  
Because of the Flatpak + Wayland problem.

## The Flatpak Problem

**Flatpak** is a Linux sandboxing system that isolates apps from the system. When Whis runs as a Flatpak:
- It can't directly access X11 clipboard
- It can't use the `wlr-data-control` Wayland protocol (GNOME doesn't implement it)

**Solution**: Bundle the `wl-copy` command (from the `wl-clipboard` package) inside the Flatpak and call it directly.

### Detecting Flatpak

```rust
fn is_flatpak() -> bool {
    std::path::Path::new("/.flatpak-info").exists()
}
```

**From `whis-core/src/clipboard.rs:7-9`**

Every Flatpak container has a `/.flatpak-info` file in the root filesystem. Checking for this file reliably detects Flatpak environments.

> **Key Insight**: This is a runtime check, not a compile-time check. The same binary works inside and outside Flatpak by detecting the environment.

### Using `wl-copy`

`wl-copy` is a Wayland utility that copies stdin to the clipboard:

```bash
echo "Hello" | wl-copy
```

Whis calls this programmatically:

```rust
fn copy_via_wl_copy(text: &str) -> Result<()> {
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to spawn wl-copy")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .context("Failed to write to wl-copy")?;
    }

    let status = child.wait().context("Failed to wait for wl-copy")?;
    if !status.success() {
        anyhow::bail!("wl-copy exited with non-zero status");
    }

    Ok(())
}
```

**From `whis-core/src/clipboard.rs:16-34`**

**Step-by-step**:

1. **Spawn `wl-copy`** with stdin piped
2. **Write text to stdin** (the clipboard content)
3. **Close stdin** (implicitly via drop)
4. **Wait for process** to complete
5. **Check exit status** (0 = success)

**Why pipe stdin?**  
Because `wl-copy` reads from stdin. We can't pass the text as an argument (too long, shell escaping issues).

**Error handling**:
- If `wl-copy` isn't in `PATH`: "Failed to spawn wl-copy"
- If write fails: "Failed to write to wl-copy"
- If exit code ≠ 0: "wl-copy exited with non-zero status"

## Flatpak Bundling Details

How does `wl-copy` get into the Flatpak?

**In `whis-desktop/packaging/linux/flatpak.yml`** (hypothetical example):

```yaml
modules:
  - name: wl-clipboard
    buildsystem: meson
    sources:
      - type: archive
        url: https://github.com/bugaevc/wl-clipboard/archive/v2.1.0.tar.gz
```

The Flatpak build script compiles `wl-clipboard` and includes `wl-copy` in the app's `bin/` directory. When running inside Flatpak, `Command::new("wl-copy")` finds it in the isolated `$PATH`.

## Cross-Platform Considerations

### Linux

**X11**:
- `arboard` works perfectly
- No Flatpak workarounds needed

**Wayland (GNOME/Mutter)**:
- Desktop: Use `arboard` (works with most compositors)
- Flatpak: Use `wl-copy` fallback

**Wayland (Sway/wlroots)**:
- `arboard` works (supports `wlr-data-control` protocol)

### macOS

Uses `NSPasteboard` APIs via `arboard`. No special handling needed.

```rust
// This works on macOS (arboard handles the details)
let mut clipboard = Clipboard::new()?;
clipboard.set_text(text)?;
```

### Windows

Uses Win32 clipboard APIs via `arboard`. No special handling needed.

```rust
// This works on Windows (arboard handles the details)
let mut clipboard = Clipboard::new()?;
clipboard.set_text(text)?;
```

## Feature Flag

Remember from Chapter 8 that clipboard support is optional:

**In `whis-core/Cargo.toml`**:

```toml
[features]
default = ["ffmpeg", "clipboard"]
clipboard = ["arboard"]
```

**Why optional?**
- Headless servers don't have clipboards
- CI environments fail on clipboard access
- Some users might not need clipboard (e.g., only transcribing files)

When compiled without `clipboard` feature:

```rust
#[cfg(not(feature = "clipboard"))]
pub fn copy_to_clipboard(_text: &str) -> Result<()> {
    anyhow::bail!("Clipboard support not compiled in")
}
```

The function exists but immediately returns an error.

## Error Scenarios

What can go wrong?

### 1. No Display Server (Headless)

```
Error: Failed to access clipboard
Caused by: No display server found
```

**When this happens**: SSH session, Docker container, CI pipeline

**Solution**: Don't call `copy_to_clipboard()` in headless environments, or catch the error gracefully.

### 2. Permission Denied (Sandboxing)

```
Error: Failed to access clipboard
Caused by: Permission denied
```

**When this happens**: Strict sandboxing (Snap, Flatpak without permissions)

**Solution**: Grant clipboard permissions in sandbox config.

### 3. `wl-copy` Not Found

```
Error: Failed to spawn wl-copy
Caused by: No such file or directory
```

**When this happens**: Flatpak build didn't bundle `wl-clipboard`

**Solution**: Fix Flatpak packaging to include `wl-copy`.

## Testing Clipboard Functionality

Manual test:

```bash
# Compile with clipboard feature
cargo build --features clipboard

# Run a simple test
cargo run --example clipboard_test
```

**`examples/clipboard_test.rs`** (hypothetical):

```rust
use whis_core::clipboard::copy_to_clipboard;

fn main() -> anyhow::Result<()> {
    copy_to_clipboard("Test from Whis!")?;
    println!("✓ Text copied to clipboard");
    println!("Try pasting (Ctrl+V) to verify");
    Ok(())
}
```

If this succeeds, clipboard integration works in your environment.

## Implementation Alternatives

Why not use other approaches?

### Alternative 1: `clipboard` Crate

The [`clipboard`](https://crates.io/crates/clipboard) crate is older and less maintained. `arboard` is the modern replacement with better platform support.

### Alternative 2: Always Use System Commands

```rust
// Hypothetical: call xclip/wl-copy/pbcopy directly
#[cfg(target_os = "linux")]
fn copy_linux(text: &str) -> Result<()> {
    Command::new("xclip").arg("-selection").arg("clipboard")...
}

#[cfg(target_os = "macos")]
fn copy_macos(text: &str) -> Result<()> {
    Command::new("pbcopy")...
}
```

**Problems**:
- Requires external tools to be installed
- Different tools on different distros (xclip vs xsel vs wl-copy)
- Doesn't work on Windows without WSL

`arboard` handles all of this internally.

### Alternative 3: Wayland-rs

Use Wayland protocol directly via `wayland-rs`. 

**Problems**:
- Requires implementing protocol clients manually
- Different protocols for different compositors
- Hundreds of lines of code vs. one `arboard` call

Not worth it for a simple clipboard copy.

## Real-World Usage in Whis

Here's how the desktop app uses clipboard after transcription:

```rust
// In whis-desktop/src/commands.rs
#[tauri::command]
pub async fn transcribe_and_copy(
    audio_data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.lock().await;
    let provider = settings.provider.clone();
    let api_key = settings.get_api_key()
        .ok_or("No API key configured")?;
    drop(settings);

    // Transcribe
    let transcript = provider
        .transcribe(&audio_data, &api_key)
        .await
        .map_err(|e| e.to_string())?;

    // Copy to clipboard
    #[cfg(feature = "clipboard")]
    copy_to_clipboard(&transcript)
        .map_err(|e| format!("Clipboard error: {}", e))?;

    Ok(transcript)
}
```

The GUI shows a notification when clipboard copy succeeds.

## Summary

**Key Takeaways:**

1. **`arboard` crate**: Cross-platform clipboard abstraction
2. **Flatpak workaround**: Use bundled `wl-copy` on Wayland/GNOME
3. **Detection**: Check `/.flatpak-info` at runtime
4. **Feature flag**: Clipboard support is optional (`clipboard` feature)
5. **Error handling**: Graceful failures in headless/sandboxed environments

**Where This Matters in Whis:**

- CLI copies transcription to clipboard after each recording (`whis-cli/src/commands/listen.rs`)
- Desktop GUI copies and shows notification (`whis-desktop/src/commands.rs`)
- Mobile skips clipboard (not always available on Android/iOS)

**Platform Matrix:**

| Platform | Method | Sandboxing |
|----------|--------|------------|
| Linux X11 | `arboard` | Works everywhere |
| Linux Wayland (Sway) | `arboard` | Works (wlr-data-control) |
| Linux Wayland (GNOME) | `arboard` | Native only |
| Linux Flatpak | `wl-copy` | Bundled workaround |
| macOS | `arboard` | Works everywhere |
| Windows | `arboard` | Works everywhere |

**Design Patterns:**

- **Runtime detection**: Same binary adapts to environment
- **Command pattern**: Spawn external tool when library fails
- **Graceful degradation**: Feature flags allow compilation without clipboard

---

Next: [Chapter 11: Audio Recording - The Basics](./ch11-audio-basics.md)
