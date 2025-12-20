# Whis - Voice-to-Text Application
#
# Run `just` to see all available commands.
# Commands follow the pattern: {action}-{component}

set shell := ["bash", "-cu"]

# macOS build environment for whisper.cpp C++17 support
export MACOSX_DEPLOYMENT_TARGET := "10.15"
export CMAKE_OSX_DEPLOYMENT_TARGET := "10.15"

# Show available commands
default:
    @just --list --unsorted

# ============================================================================
# PRIVATE DEPENDENCY CHECKS
# ============================================================================

[private]
_check-cargo:
    @command -v cargo >/dev/null 2>&1 || { echo "❌ cargo not found. Run: just setup-cli"; exit 1; }

[private]
_check-npm:
    @command -v npm >/dev/null 2>&1 || { echo "❌ npm not found. Install Node.js: https://nodejs.org"; exit 1; }

[private]
_check-tauri:
    @command -v cargo-tauri >/dev/null 2>&1 || { echo "❌ tauri-cli not found. Run: just setup-desktop"; exit 1; }

# ============================================================================
# CLI
# ============================================================================

# Check and install CLI prerequisites
[group('cli')]
[linux]
setup-cli:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== CLI Prerequisites ==="
    echo ""

    # Check cargo
    if command -v cargo >/dev/null 2>&1; then
        echo "✓ cargo $(cargo --version | cut -d' ' -f2)"
    else
        echo "❌ cargo not found"
        echo "   Install Rust: https://rustup.rs"
        echo ""
        exit 1
    fi

    # Check ffmpeg
    if command -v ffmpeg >/dev/null 2>&1; then
        echo "✓ ffmpeg installed"
    else
        echo "❌ ffmpeg not found"
        echo "   Run: sudo apt install ffmpeg"
    fi

    # Check audio libs
    if dpkg -s libasound2-dev >/dev/null 2>&1; then
        echo "✓ libasound2-dev installed"
    else
        echo "❌ libasound2-dev not found"
        echo "   Run: sudo apt install libasound2-dev"
    fi

    echo ""
    echo "If all checks pass, run: just install-cli"

[group('cli')]
[macos]
setup-cli:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== CLI Prerequisites ==="
    echo ""

    # Check cargo
    if command -v cargo >/dev/null 2>&1; then
        echo "✓ cargo $(cargo --version | cut -d' ' -f2)"
    else
        echo "❌ cargo not found"
        echo "   Install Rust: https://rustup.rs"
        echo ""
        exit 1
    fi

    # Check ffmpeg
    if command -v ffmpeg >/dev/null 2>&1; then
        echo "✓ ffmpeg installed"
    else
        echo "❌ ffmpeg not found"
        echo "   Run: brew install ffmpeg"
    fi

    echo ""
    echo "If all checks pass, run: just install-cli"

[group('cli')]
[windows]
setup-cli:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== CLI Prerequisites ==="
    echo ""

    # Check cargo
    if command -v cargo >/dev/null 2>&1; then
        echo "✓ cargo $(cargo --version | cut -d' ' -f2)"
    else
        echo "❌ cargo not found"
        echo "   Install Rust: https://rustup.rs"
        echo ""
        exit 1
    fi

    # Check ffmpeg
    if command -v ffmpeg >/dev/null 2>&1; then
        echo "✓ ffmpeg installed"
    else
        echo "❌ ffmpeg not found"
        echo "   Download from: https://ffmpeg.org/download.html"
        echo "   Add to PATH after installation"
    fi

    echo ""
    echo "If all checks pass, run: just install-cli"

# Install CLI dependencies
[group('cli')]
install-cli: _check-cargo
    cargo fetch

# Build CLI
[group('cli')]
build-cli: _check-cargo
    cargo build -p whis

# Lint CLI code
[group('cli')]
lint-cli: _check-cargo
    cargo clippy -p whis --all-targets

# Format CLI code
[group('cli')]
fmt-cli: _check-cargo
    cargo fmt -p whis

# ============================================================================
# DESKTOP
# ============================================================================

# Check and install desktop prerequisites
[group('desktop')]
[linux]
setup-desktop:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Desktop Prerequisites ==="
    echo ""

    # Check cargo first
    if ! command -v cargo >/dev/null 2>&1; then
        echo "❌ cargo not found"
        echo "   Install Rust first: https://rustup.rs"
        exit 1
    fi
    echo "✓ cargo $(cargo --version | cut -d' ' -f2)"

    # Check npm
    if command -v npm >/dev/null 2>&1; then
        echo "✓ npm $(npm --version)"
    else
        echo "❌ npm not found"
        echo "   Install Node.js: https://nodejs.org"
        exit 1
    fi

    # Auto-install tauri-cli
    if command -v cargo-tauri >/dev/null 2>&1; then
        echo "✓ tauri-cli installed"
    else
        echo "→ Installing tauri-cli..."
        cargo install tauri-cli
        echo "✓ tauri-cli installed"
    fi

    # Check system libs
    MISSING=""
    for pkg in libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf; do
        if dpkg -s "$pkg" >/dev/null 2>&1; then
            echo "✓ $pkg"
        else
            echo "❌ $pkg not found"
            MISSING="$MISSING $pkg"
        fi
    done

    if [ -n "$MISSING" ]; then
        echo ""
        echo "Install missing packages:"
        echo "   sudo apt install$MISSING"
    else
        echo ""
        echo "All prerequisites installed! Run: just install-desktop"
    fi

[group('desktop')]
[macos]
setup-desktop:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Desktop Prerequisites ==="
    echo ""

    # Check cargo
    if ! command -v cargo >/dev/null 2>&1; then
        echo "❌ cargo not found"
        echo "   Install Rust: https://rustup.rs"
        exit 1
    fi
    echo "✓ cargo $(cargo --version | cut -d' ' -f2)"

    # Check npm
    if command -v npm >/dev/null 2>&1; then
        echo "✓ npm $(npm --version)"
    else
        echo "❌ npm not found"
        echo "   Install Node.js: https://nodejs.org"
        exit 1
    fi

    # Check Xcode CLI tools
    if xcode-select -p >/dev/null 2>&1; then
        echo "✓ Xcode Command Line Tools"
    else
        echo "❌ Xcode Command Line Tools not found"
        echo "   Run: xcode-select --install"
        exit 1
    fi

    # Auto-install tauri-cli
    if command -v cargo-tauri >/dev/null 2>&1; then
        echo "✓ tauri-cli installed"
    else
        echo "→ Installing tauri-cli..."
        cargo install tauri-cli
        echo "✓ tauri-cli installed"
    fi

    echo ""
    echo "All prerequisites installed! Run: just install-desktop"

[group('desktop')]
[windows]
setup-desktop:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Desktop Prerequisites ==="
    echo ""

    # Check cargo
    if ! command -v cargo >/dev/null 2>&1; then
        echo "❌ cargo not found"
        echo "   Install Rust: https://rustup.rs"
        exit 1
    fi
    echo "✓ cargo $(cargo --version | cut -d' ' -f2)"

    # Check npm
    if command -v npm >/dev/null 2>&1; then
        echo "✓ npm $(npm --version)"
    else
        echo "❌ npm not found"
        echo "   Install Node.js: https://nodejs.org"
        exit 1
    fi

    # Auto-install tauri-cli
    if command -v cargo-tauri >/dev/null 2>&1; then
        echo "✓ tauri-cli installed"
    else
        echo "→ Installing tauri-cli..."
        cargo install tauri-cli
        echo "✓ tauri-cli installed"
    fi

    echo ""
    echo "WebView2 is included with Windows 10/11."
    echo "All prerequisites installed! Run: just install-desktop"

# Install desktop dependencies
[group('desktop')]
install-desktop: _check-npm _check-tauri
    cd crates/whis-desktop/ui && npm ci --legacy-peer-deps

# Run desktop in dev mode
[group('desktop')]
dev-desktop: install-desktop
    cd crates/whis-desktop && cargo tauri dev

# Build desktop for release
[group('desktop')]
build-desktop: install-desktop
    cd crates/whis-desktop/ui && npm run build
    cd crates/whis-desktop && cargo tauri build

# Lint desktop code
[group('desktop')]
lint-desktop: _check-npm
    cd crates/whis-desktop/ui && npm run lint

# Format desktop code
[group('desktop')]
fmt-desktop: _check-npm
    cd crates/whis-desktop/ui && npm run lint:fix

# ============================================================================
# MOBILE
# ============================================================================

# Check and install mobile prerequisites
[group('mobile')]
setup-mobile:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Mobile (Android) Prerequisites ==="
    echo ""

    # Check cargo
    if ! command -v cargo >/dev/null 2>&1; then
        echo "❌ cargo not found"
        echo "   Install Rust: https://rustup.rs"
        exit 1
    fi
    echo "✓ cargo $(cargo --version | cut -d' ' -f2)"

    # Check npm
    if command -v npm >/dev/null 2>&1; then
        echo "✓ npm $(npm --version)"
    else
        echo "❌ npm not found"
        echo "   Install Node.js: https://nodejs.org"
        exit 1
    fi

    # Auto-install tauri-cli
    if command -v cargo-tauri >/dev/null 2>&1; then
        echo "✓ tauri-cli installed"
    else
        echo "→ Installing tauri-cli..."
        cargo install tauri-cli
        echo "✓ tauri-cli installed"
    fi

    # Check Android SDK
    if [ -n "${ANDROID_HOME:-}" ] && [ -d "$ANDROID_HOME" ]; then
        echo "✓ ANDROID_HOME: $ANDROID_HOME"
    else
        echo "❌ ANDROID_HOME not set"
        echo "   Install Android Studio: https://developer.android.com/studio"
        echo "   Set ANDROID_HOME to SDK location"
    fi

    # Check Java
    if command -v java >/dev/null 2>&1; then
        echo "✓ java installed"
    else
        echo "❌ java not found"
        echo "   Install JDK 17+ (included with Android Studio)"
    fi

    # Check Android targets
    echo ""
    echo "Required Rust targets for Android:"
    for target in aarch64-linux-android armv7-linux-androideabi; do
        if rustup target list --installed | grep -q "$target"; then
            echo "✓ $target"
        else
            echo "→ Adding $target..."
            rustup target add "$target"
            echo "✓ $target"
        fi
    done

    echo ""
    echo "If all checks pass, run: just install-mobile"

# Install mobile dependencies
[group('mobile')]
install-mobile: _check-npm _check-tauri
    cd crates/whis-mobile/ui && npm ci

# Run mobile on emulator
[group('mobile')]
dev-mobile: install-mobile
    cd crates/whis-mobile && cargo tauri android dev

# Build mobile APK
[group('mobile')]
build-mobile: install-mobile
    cd crates/whis-mobile/ui && npm run build
    cd crates/whis-mobile && cargo tauri android build

# Lint mobile code
[group('mobile')]
lint-mobile: _check-npm
    cd crates/whis-mobile/ui && npm run lint

# Format mobile code
[group('mobile')]
fmt-mobile: _check-npm
    cd crates/whis-mobile/ui && npm run lint:fix

# ============================================================================
# WEBSITE
# ============================================================================

# Check and install website prerequisites
[group('website')]
setup-website:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Website Prerequisites ==="
    echo ""

    if command -v npm >/dev/null 2>&1; then
        echo "✓ npm $(npm --version)"
        echo ""
        echo "All prerequisites installed! Run: just install-website"
    else
        echo "❌ npm not found"
        echo "   Install Node.js: https://nodejs.org"
    fi

# Install website dependencies
[group('website')]
install-website: _check-npm
    cd website && npm ci

# Run website dev server
[group('website')]
dev-website: install-website
    cd website && npm run dev

# Build website for production
[group('website')]
build-website: install-website
    cd website && npm run build

# Lint website code
[group('website')]
lint-website: _check-npm
    cd website && npm run lint

# Format website code
[group('website')]
fmt-website: _check-npm
    cd website && npm run lint:fix

# ============================================================================
# GLOBAL
# ============================================================================

# Verify all code (Rust + all UIs)
[group('misc')]
check: _check-cargo _check-npm
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features
    cd crates/whis-desktop/ui && npm run lint
    cd crates/whis-mobile/ui && npm run lint
    cd website && npm run lint

# Clean all build artifacts
[group('misc')]
clean:
    cargo clean
    rm -rf crates/whis-desktop/ui/dist
    rm -rf crates/whis-desktop/ui/node_modules
    rm -rf crates/whis-mobile/ui/dist
    rm -rf crates/whis-mobile/ui/node_modules
    rm -rf crates/whis-mobile/gen/android/app/build
    rm -rf website/dist
    rm -rf website/node_modules
