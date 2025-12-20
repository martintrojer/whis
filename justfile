# Whis - Voice-to-Text Application
#
# ┌─────────────────────────────────────────────────────────┐
# │ Quick Start for Contributors                            │
# │                                                         │
# │   just build          Build CLI                         │
# │   just install        Install CLI                       │
# │   just check          Type-check all crates             │
# │   just fmt && lint    Format and lint code              │
# │                                                         │
# │ Desktop app:                                            │
# │   just desktop-dev    Dev mode with hot reload          │
# │   just desktop-build  Build release                     │
# │   just install-desktop Install to system                │
# │                                                         │
# │ Mobile (Android):                                       │
# │   just android-init   First-time setup                  │
# │   just android-dev    Dev mode                          │
# └─────────────────────────────────────────────────────────┘

set shell := ["bash", "-cu"]

# ============================================================================
# ENVIRONMENT CONFIGURATION
# ============================================================================

# macOS build environment for whisper.cpp C++17 support
# These are ignored on non-macOS platforms
export MACOSX_DEPLOYMENT_TARGET := "10.15"
export CMAKE_OSX_DEPLOYMENT_TARGET := "10.15"

# Default recipe - show help
default:
    @just --list --unsorted

# ============================================================================
# PRIVATE HELPERS (tool availability checks)
# ============================================================================

[private]
_require-npm:
    @command -v npm >/dev/null 2>&1 || { echo "npm not found. Install Node.js: https://nodejs.org"; exit 1; }

[private]
_ensure-tauri:
    @command -v cargo-tauri >/dev/null 2>&1 || cargo install tauri-cli

[private]
_ensure-cross:
    @command -v cross >/dev/null 2>&1 || cargo install cross --git https://github.com/cross-rs/cross

[private]
_ensure-mdbook:
    @command -v mdbook >/dev/null 2>&1 || cargo install mdbook

[private]
_ensure-audit:
    @command -v cargo-audit >/dev/null 2>&1 || cargo install cargo-audit

[private]
_ensure-outdated:
    @command -v cargo-outdated >/dev/null 2>&1 || cargo install cargo-outdated

# ============================================================================
# BUILD COMMANDS
# ============================================================================

# Build CLI in debug mode
[group('cli')]
build:
    cargo build -p whis

# Build CLI in release mode
[group('cli')]
build-release:
    cargo build --release -p whis

# Build all crates
[group('cli')]
build-all:
    cargo build --workspace

# Build core library with specific features
[private]
build-core features="":
    cargo build -p whis-core {{ if features != "" { "--features " + features } else { "" } }}

# ============================================================================
# DESKTOP APP
# ============================================================================

# Install desktop frontend dependencies
[private]
desktop-deps: _require-npm
    cd crates/whis-desktop/ui && npm ci --legacy-peer-deps

# Build desktop frontend
[private]
desktop-ui: _require-npm
    cd crates/whis-desktop/ui && npm run build

# Run desktop app in development mode
[group('desktop')]
desktop-dev: desktop-deps _ensure-tauri
    cd crates/whis-desktop && cargo tauri dev

# Build desktop app for release (AppImage, deb, rpm)
[group('desktop')]
desktop-build: desktop-deps desktop-ui _ensure-tauri
    cd crates/whis-desktop && cargo tauri build

# ============================================================================
# MOBILE APP (ANDROID)
# ============================================================================

# Initialize Android project (first time setup)
[group('android')]
android-init: _ensure-tauri
    cd crates/whis-mobile && cargo tauri android init

# Install mobile frontend dependencies
[private]
mobile-deps: _require-npm
    cd crates/whis-mobile/ui && npm ci

# Build mobile frontend
[private]
mobile-ui: _require-npm
    cd crates/whis-mobile/ui && npm run build

# Run mobile app on Android emulator/device
[group('android')]
android-dev: mobile-deps _ensure-tauri
    cd crates/whis-mobile && cargo tauri android dev

# Build Android APK (debug)
[group('android')]
android-build: mobile-deps mobile-ui _ensure-tauri
    cd crates/whis-mobile && cargo tauri android build

# Build Android APK (release)
[private]
android-release: mobile-deps mobile-ui _ensure-tauri
    cd crates/whis-mobile && cargo tauri android build --release

# ============================================================================
# RUNNING
# ============================================================================

# Run CLI in debug mode
[group('cli')]
run *args:
    cargo run -p whis -- {{ args }}

# Run CLI in release mode
[group('cli')]
run-release *args:
    cargo run --release -p whis -- {{ args }}

# Show CLI configuration
[group('cli')]
config:
    cargo run -p whis -- config --show

# Start listening for voice input
[group('cli')]
listen:
    cargo run -p whis -- listen

# ============================================================================
# TESTING & QUALITY
# ============================================================================

# Run clippy linter
[group('quality')]
lint:
    cargo clippy --all-targets --all-features

# Run clippy with warnings as errors
[private]
lint-strict:
    cargo clippy --all-targets --all-features -- -D warnings

# Format all code
[group('quality')]
fmt:
    cargo fmt --all

# Check formatting without modifying
[group('quality')]
fmt-check:
    cargo fmt --all -- --check

# Check all crates for errors (fast, no build)
[group('quality')]
check:
    cargo check --workspace

# Local pre-commit check (format, lint)
[group('quality')]
ci: fmt-check lint

# ============================================================================
# CLEANING
# ============================================================================

# Clean frontend builds
[private]
clean-frontend:
    rm -rf crates/whis-desktop/ui/dist
    rm -rf crates/whis-desktop/ui/node_modules
    rm -rf crates/whis-mobile/ui/dist
    rm -rf crates/whis-mobile/ui/node_modules

# Clean Android build artifacts
[private]
clean-android:
    rm -rf crates/whis-mobile/gen/android/app/build

# Clean everything (Rust artifacts, frontend, Android)
[group('misc')]
clean-all: clean-frontend clean-android
    cargo clean

# ============================================================================
# DOCUMENTATION
# ============================================================================

# Build Rust documentation
[group('docs')]
docs:
    cargo doc --all --no-deps

# Build and open Rust documentation
[private]
docs-open:
    cargo doc --all --no-deps --open

# Build mdBook documentation
[group('docs')]
book: _ensure-mdbook
    cd book && mdbook build

# Serve mdBook documentation with live reload
[private]
book-serve: _ensure-mdbook
    cd book && mdbook serve --open

# ============================================================================
# INSTALLATION
# ============================================================================

# Install CLI locally from source
[group('cli')]
install:
    cargo install --path crates/whis-cli

# Install desktop app (Linux: AppImage to ~/.local/bin)
[group('desktop')]
[linux]
install-desktop: desktop-build
    #!/usr/bin/env bash
    set -euo pipefail

    # Find the built AppImage
    APPIMAGE=$(find target/release/bundle/appimage -name "*.AppImage" -type f | head -1)
    if [[ -z "$APPIMAGE" ]]; then
        echo "Error: No AppImage found. Run 'just desktop-build' first."
        exit 1
    fi

    # Install to ~/.local/bin
    mkdir -p ~/.local/bin
    cp "$APPIMAGE" ~/.local/bin/Whis.AppImage
    chmod +x ~/.local/bin/Whis.AppImage

    # Use built-in install for proper desktop integration
    ~/.local/bin/Whis.AppImage --install

# Install desktop app (macOS: copy .app to /Applications)
[group('desktop')]
[macos]
install-desktop: desktop-build
    #!/usr/bin/env bash
    set -euo pipefail

    # Find the built .app bundle
    APP_BUNDLE=$(find target/release/bundle/macos -name "*.app" -type d | head -1)
    if [[ -z "$APP_BUNDLE" ]]; then
        echo "Error: No .app bundle found. Run 'just desktop-build' first."
        exit 1
    fi

    APP_NAME=$(basename "$APP_BUNDLE")

    # Remove existing installation if present
    if [[ -d "/Applications/$APP_NAME" ]]; then
        echo "Removing existing /Applications/$APP_NAME..."
        rm -rf "/Applications/$APP_NAME"
    fi

    # Copy to /Applications
    cp -R "$APP_BUNDLE" /Applications/

    echo "✓ Installed $APP_NAME to /Applications/"
    echo ""
    echo "Find 'Whis' in your Applications folder or Spotlight"

# Install desktop app (Windows: run the MSI installer)
[group('desktop')]
[windows]
install-desktop: desktop-build
    #!/usr/bin/env bash
    set -euo pipefail

    # Find the MSI installer
    MSI=$(find target/release/bundle/msi -name "*.msi" -type f | head -1 2>/dev/null || true)

    if [[ -n "$MSI" ]]; then
        echo "Running installer: $MSI"
        msiexec /i "$MSI"
    else
        # Try NSIS exe installer
        EXE=$(find target/release/bundle/nsis -name "*.exe" -type f | head -1 2>/dev/null || true)
        if [[ -n "$EXE" ]]; then
            echo "Running installer: $EXE"
            "$EXE"
        else
            echo "Error: No installer found. Run 'just desktop-build' first."
            exit 1
        fi
    fi

    echo "✓ Whis desktop app installed"

# Uninstall desktop app (Linux)
[group('desktop')]
[linux]
uninstall-desktop:
    #!/usr/bin/env bash
    # Use built-in uninstall if available
    if [[ -x ~/.local/bin/Whis.AppImage ]]; then
        ~/.local/bin/Whis.AppImage --uninstall || true
    fi
    rm -f ~/.local/bin/Whis.AppImage
    echo "✓ Whis desktop app uninstalled"

# Uninstall desktop app (macOS)
[group('desktop')]
[macos]
uninstall-desktop:
    rm -rf "/Applications/Whis.app"
    echo "✓ Whis desktop app uninstalled"

# Install development tools (skips already-installed)
[private]
install-tools:
    @command -v cargo-tauri >/dev/null 2>&1 || cargo install tauri-cli
    @command -v cross >/dev/null 2>&1 || cargo install cross --git https://github.com/cross-rs/cross
    @command -v cargo-outdated >/dev/null 2>&1 || cargo install cargo-outdated
    @command -v cargo-audit >/dev/null 2>&1 || cargo install cargo-audit
    @command -v mdbook >/dev/null 2>&1 || cargo install mdbook

# ============================================================================
# PUBLISHING & RELEASE
# ============================================================================

# Publish whis-core to crates.io (dry run)
[private]
publish-core-dry:
    cargo publish -p whis-core --dry-run

# Publish whis CLI to crates.io (dry run)
[private]
publish-cli-dry:
    cargo publish -p whis --dry-run

# Publish whis-core to crates.io
[private]
publish-core:
    cargo publish -p whis-core

# Publish whis CLI to crates.io
[private]
publish-cli:
    cargo publish -p whis

# ============================================================================
# DEPENDENCY MANAGEMENT
# ============================================================================

# Update Cargo dependencies
[group('misc')]
update:
    cargo update

# Check for outdated dependencies
[private]
outdated: _ensure-outdated
    cargo outdated

# Audit dependencies for security issues
[private]
audit: _ensure-audit
    cargo audit

# Update npm dependencies
[private]
update-npm: _require-npm
    cd crates/whis-desktop/ui && npm update
    cd crates/whis-mobile/ui && npm update

# ============================================================================
# CROSS-COMPILATION
# ============================================================================

# Build CLI for Linux ARM64
[private]
build-arm64: _ensure-cross
    cross build --release --target aarch64-unknown-linux-gnu -p whis

# Build CLI for macOS Intel
[private]
build-macos-intel:
    cargo build --release --target x86_64-apple-darwin -p whis

# Build CLI for macOS Apple Silicon
[private]
build-macos-arm:
    cargo build --release --target aarch64-apple-darwin -p whis

# ============================================================================
# SETUP (First-time)
# ============================================================================

# Show Linux system dependencies
[group('misc')]
[linux]
setup-info:
    @echo "Install these packages:"
    @echo "  sudo apt-get install -y \\"
    @echo "    libasound2-dev libx11-dev libxtst-dev \\"
    @echo "    libwebkit2gtk-4.1-dev libappindicator3-dev \\"
    @echo "    librsvg2-dev patchelf ffmpeg"
    @echo ""
    @echo "For global hotkey support:"
    @echo "  sudo usermod -aG input \$USER"
    @echo "  echo 'KERNEL==\"uinput\", GROUP=\"input\", MODE=\"0660\"' | sudo tee /etc/udev/rules.d/99-uinput.rules"
    @echo "  sudo udevadm control --reload-rules && sudo udevadm trigger"
    @echo "  # Then logout and login"

# Show macOS setup info
[group('misc')]
[macos]
setup-info:
    @echo "Install FFmpeg:"
    @echo "  brew install ffmpeg"

# Add Rust compilation targets
[private]
setup-targets:
    rustup target add x86_64-unknown-linux-gnu
    rustup target add aarch64-unknown-linux-gnu
    rustup target add x86_64-apple-darwin
    rustup target add aarch64-apple-darwin
    rustup target add aarch64-linux-android
    rustup target add armv7-linux-androideabi
