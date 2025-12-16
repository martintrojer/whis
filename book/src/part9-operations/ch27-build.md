# Chapter 27: Building for Platforms

Whis supports multiple platforms: Linux (x86_64, ARM64), macOS (Intel, Apple Silicon), and Windows. Each platform has different build requirements and distribution formats.

In this chapter, we'll cover:
- Building CLI binaries for different platforms
- Tauri bundling for desktop apps (AppImage, DMG, MSI)
- Cross-compilation strategies
- Flatpak packaging for Linux

## Building CLI Binaries

The `whis` CLI crate builds to a single binary with minimal dependencies.

### Linux (Native)

```bash
# Install dependencies
sudo apt-get install libasound2-dev libx11-dev libxtst-dev

# Build release binary
cargo build --release -p whis

# Binary at target/release/whis (~5MB)
```

Dependencies:
- `libasound2-dev`: ALSA audio support
- `libx11-dev`: X11 for global hotkeys
- `libxtst-dev`: X11 input simulation

### macOS (Native)

```bash
# Build for current architecture
cargo build --release -p whis

# Binary at target/release/whis (~4MB)
```

macOS has no extra dependencies—CoreAudio and Cocoa frameworks are in the system.

### Windows (Native)

```bash
# Build on Windows
cargo build --release -p whis

# Binary at target\release\whis.exe (~5MB)
```

Windows dependencies (WASAPI, Windows API) are linked automatically.

### Cross-Compilation with cargo-cross

To build for other platforms without a native machine, use [cross](https://github.com/cross-rs/cross):

```bash
# Install cross
cargo install cross

# Build for Linux ARM64 from x86_64 machine
cross build --release -p whis --target aarch64-unknown-linux-gnu

# Build for Windows from Linux
cross build --release -p whis --target x86_64-pc-windows-gnu
```

`cross` uses Docker containers with the target toolchain and system libraries. No manual setup needed.

## Tauri Desktop Bundling

Tauri CLI creates platform-specific packages: AppImage (Linux), DMG (macOS), MSI (Windows).

### Build Desktop App

```bash
cd crates/whis-desktop

# Install Node dependencies
cd ui && npm install && cd ..

# Build and bundle
cargo tauri build
```

This produces:

**Linux**:
- `target/release/bundle/appimage/Whis_0.5.8_amd64.AppImage` (~25MB)
- `target/release/bundle/deb/whis_0.5.8_amd64.deb` (~15MB)

**macOS**:
- `target/release/bundle/dmg/Whis_0.5.8_x64.dmg` (~20MB)
- `target/release/bundle/macos/Whis.app/` (app bundle)

**Windows**:
- `target/release/bundle/msi/Whis_0.5.8_x64_en-US.msi` (~18MB)
- `target/release/bundle/nsis/Whis_0.5.8_x64-setup.exe` (~18MB)

### AppImage for Linux

AppImage is a portable format—users download one file and run it. No installation needed.

Tauri uses [appimage-builder](https://appimage-builder.readthedocs.io/) internally. The `tauri.conf.json` specifies:

```json
{
  "bundle": {
    "linux": {
      "appimage": {
        "bundleMediaFramework": false
      }
    }
  }
}
```

To run:

```bash
chmod +x Whis_0.5.8_amd64.AppImage
./Whis_0.5.8_amd64.AppImage
```

The AppImage contains:
- Whis binary
- WebView runtime (if not using system WebKit)
- .desktop file
- Icons

### DMG for macOS

DMG (Disk Image) is macOS's standard distribution format. Double-click to mount, drag app to Applications folder.

Tauri creates a DMG with:
- Whis.app bundle
- Applications folder shortcut
- Background image (customizable)

Code signing (for notarization):

```bash
# Sign the app
codesign --force --deep --sign "Developer ID Application: Your Name" \
  target/release/bundle/macos/Whis.app

# Create DMG
cargo tauri build

# Notarize with Apple
xcrun notarytool submit target/release/bundle/dmg/Whis_0.5.8_x64.dmg \
  --apple-id your@email.com \
  --team-id TEAM_ID \
  --password APP_SPECIFIC_PASSWORD \
  --wait
```

Without notarization, users see "unidentified developer" warning.

### MSI for Windows

MSI (Microsoft Installer) is Windows's standard package format.

To build:

```bash
cargo tauri build
```

Creates `Whis_0.5.8_x64_en-US.msi`. Double-click to install.

Tauri uses [WiX Toolset](https://wixtoolset.org/) to generate the MSI. The installer:
- Copies app to Program Files
- Creates Start Menu shortcuts
- Adds to Windows Apps & Features (for uninstall)

## Flatpak Packaging

Flatpak is a sandboxed app distribution format for Linux. It provides:
- Automatic updates
- Dependency isolation
- Security via sandboxing
- Distribution via Flathub

### Flatpak Manifest

Create `ink.whis.Whis.yml`:

```yaml
app-id: ink.whis.Whis
runtime: org.gnome.Platform
runtime-version: '47'
sdk: org.gnome.Sdk
sdk-extensions:
  - org.freedesktop.Sdk.Extension.rust-stable
  - org.freedesktop.Sdk.Extension.node20

command: whis-desktop

finish-args:
  # Wayland/X11 access
  - --socket=wayland
  - --socket=fallback-x11
  # Audio recording
  - --device=all
  # Network for API calls
  - --share=network
  # Config directory
  - --filesystem=xdg-config/whis:create
  # Clipboard access (special workaround)
  - --talk-name=org.freedesktop.portal.Desktop
  - --env=GTK_USE_PORTAL=1

modules:
  - name: whis
    buildsystem: simple
    build-commands:
      # Build Rust binary
      - cargo --offline fetch --manifest-path Cargo.toml
      - cargo --offline build --release --manifest-path crates/whis-desktop/Cargo.toml
      
      # Install binary
      - install -Dm755 target/release/whis-desktop /app/bin/whis-desktop
      
      # Install .desktop file
      - install -Dm644 crates/whis-desktop/packaging/linux/ink.whis.Whis.desktop \
          /app/share/applications/ink.whis.Whis.desktop
      
      # Install icons
      - install -Dm644 crates/whis-desktop/icons/icon.svg \
          /app/share/icons/hicolor/scalable/apps/ink.whis.Whis.svg

    sources:
      - type: git
        url: https://github.com/frankdierolf/whis
        tag: v0.5.8
      
      - cargo-sources.json  # Generated by flatpak-cargo-generator
```

### Building Flatpak

```bash
# Install flatpak-builder
sudo apt install flatpak-builder

# Build
flatpak-builder --force-clean --repo=repo build-dir ink.whis.Whis.yml

# Install locally
flatpak build-bundle repo whis.flatpak ink.whis.Whis

# Test
flatpak install whis.flatpak
flatpak run ink.whis.Whis
```

### Publishing to Flathub

Flathub is the central Flatpak repository. To publish:

1. Fork [flathub/flathub](https://github.com/flathub/flathub)
2. Add `ink.whis.Whis.yml` to repository
3. Submit PR with manifest
4. Flathub CI builds and tests
5. After approval, app appears on Flathub

Users install with:

```bash
flatpak install flathub ink.whis.Whis
```

### Flatpak Challenges

**Clipboard Access**: Flatpak sandboxes clipboard. Whis uses a workaround (`wl-clipboard` + portal) as described in Chapter 10.

**Portal Shortcuts**: The app_id must match the .desktop file exactly for GlobalShortcuts portal to work. This is why we set `gtk::glib::set_prgname(Some("ink.whis.Whis"))` early in `main.rs`.

## Feature Flags for Platform Code

Use Cargo features to include/exclude platform-specific dependencies:

```toml
[target.'cfg(target_os = "linux")'.dependencies]
x11-clipboard = { version = "0.9", optional = true }
wayland-clipboard = { version = "0.1", optional = true }

[features]
default = ["x11", "wayland"]
x11 = ["dep:x11-clipboard"]
wayland = ["dep:wayland-clipboard"]
```

In code:

```rust
#[cfg(all(target_os = "linux", feature = "x11"))]
fn copy_to_clipboard_x11(text: &str) -> Result<()> {
    // X11 implementation
}

#[cfg(all(target_os = "linux", feature = "wayland"))]
fn copy_to_clipboard_wayland(text: &str) -> Result<()> {
    // Wayland implementation
}
```

Build for specific platform:

```bash
# Linux without Wayland support
cargo build --no-default-features --features x11

# macOS (no Linux features)
cargo build
```

## CI/CD Build Matrix

GitHub Actions workflow for multi-platform builds:

```yaml
jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: whis-linux-x86_64
          
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            artifact: whis-linux-arm64
          
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: whis-macos-intel
          
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: whis-macos-apple-silicon
          
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: whis-windows-x64

    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Build
        run: cargo build --release --target ${{ matrix.target }} -p whis
      
      - name: Package
        run: |
          mkdir dist
          cp target/${{ matrix.target }}/release/whis* dist/
          tar -czvf ${{ matrix.artifact }}.tar.gz -C dist .
      
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: ${{ matrix.artifact }}.tar.gz
```

This builds binaries for all platforms in parallel on GitHub's infrastructure.

## Summary

**Key Takeaways**:

1. **CLI binaries**: Build with `cargo build --release`, use `cross` for cross-compilation
2. **Tauri bundles**: `cargo tauri build` creates AppImage/DMG/MSI automatically
3. **Flatpak**: Sandboxed Linux distribution via Flathub
4. **Feature flags**: Include/exclude platform-specific code at compile time

**Distribution Formats**:

- **Linux**: AppImage (portable), DEB (Debian/Ubuntu), Flatpak (Flathub)
- **macOS**: DMG (drag-to-install), PKG (installer), Homebrew cask
- **Windows**: MSI (installer), NSIS (installer), Portable EXE

**Platform-Specific Considerations**:

- **Linux**: Wayland vs X11, portal permissions
- **macOS**: Code signing, notarization for Gatekeeper
- **Windows**: User vs system install, antivirus false positives

**CI/CD**: Use build matrices to compile for all platforms in parallel. Cache dependencies to speed up builds.

---

Next: [Chapter 28: CI/CD Pipeline](./ch28-cicd.md)
