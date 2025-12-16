# Chapter 28: CI/CD Pipeline

Whis uses GitHub Actions for continuous integration and automated releases. Every push to `main` runs tests and lints. Every tag triggers a multi-platform build and GitHub Release.

In this chapter, we'll explore:
- GitHub Actions workflow structure
- Multi-platform build matrix
- Release automation
- Version management strategy

## Workflow Overview

Whis has two main workflows:

**`.github/workflows/test.yml`**: Runs on every push/PR
- Lint with `cargo clippy`
- Format check with `cargo fmt`
- Run tests with `cargo test`
- Build all crates

**`.github/workflows/release.yml`**: Runs on version tags
- Build CLI for Linux (x86_64, ARM64), macOS (Intel, Apple Silicon), Windows
- Build desktop apps (AppImage, DMG, MSI)
- Publish to crates.io
- Create GitHub Release with artifacts

## Release Workflow Structure

The release workflow (`release.yml`) has multiple jobs that run in parallel:

```yaml
jobs:
  prepare:
    # Extract version tag (v0.5.8)
  
  publish-crates:
    # Publish whis-core and whis to crates.io
  
  build-linux-x86:
    # Build CLI for Linux x86_64
  
  build-linux-arm64:
    # Build CLI for Linux ARM64 with cross
  
  build-macos:
    # Build CLI for macOS (Intel + Apple Silicon)
  
  build-windows:
    # Build CLI for Windows
  
  build-desktop-linux:
    # Build Tauri AppImage and DEB
  
  build-desktop-macos:
    # Build Tauri DMG
  
  build-desktop-windows:
    # Build Tauri MSI
  
  create-release:
    # Collect all artifacts and create GitHub Release
```

Each build job runs independently, then the final `create-release` job collects artifacts.

## Triggering a Release

### Automatic: Push a Tag

```bash
# Bump version in Cargo.toml files
vim crates/whis-core/Cargo.toml  # version = "0.5.9"
vim crates/whis-cli/Cargo.toml   # version = "0.5.9"
vim crates/whis-desktop/Cargo.toml

# Commit changes
git add .
git commit -m "Bump version to 0.5.9"

# Create and push tag
git tag v0.5.9
git push origin main
git push origin v0.5.9
```

GitHub Actions detects the tag and starts the release workflow.

### Manual: Workflow Dispatch

Trigger manually from GitHub UI:
1. Go to Actions → Release
2. Click "Run workflow"
3. Enter tag (e.g., `v0.5.9`)
4. Enter release title suffix (optional)

This is useful for rebuilding a release without creating a new tag.

## Multi-Platform Build Matrix

macOS job uses a strategy matrix to build both architectures:

```yaml
build-macos:
  runs-on: macos-latest
  strategy:
    matrix:
      target:
        - x86_64-apple-darwin
        - aarch64-apple-darwin
  steps:
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        targets: ${{ matrix.target }}
    
    - name: Build
      run: cargo build --release --target ${{ matrix.target }} -p whis
    
    - name: Package
      run: |
        mkdir -p dist
        cp target/${{ matrix.target }}/release/whis dist/
        tar -czvf whis-${{ matrix.target }}.tar.gz -C dist whis
```

This spawns two parallel jobs:
- One builds `x86_64-apple-darwin` (Intel)
- One builds `aarch64-apple-darwin` (Apple Silicon)

Both run on the same `macos-latest` runner (which is ARM64, but can cross-compile to x86_64).

## Cross-Compilation for Linux ARM64

Linux ARM64 uses `cross`:

```yaml
build-linux-arm64:
  runs-on: ubuntu-latest
  steps:
    - name: Install cross
      run: cargo install cross --git https://github.com/cross-rs/cross
    
    - name: Build with cross
      run: cross build --release --target aarch64-unknown-linux-gnu -p whis
```

`cross` runs the build in a Docker container with ARM64 toolchain and emulation. No native ARM64 runner needed.

## Publishing to crates.io

The `publish-crates` job publishes `whis-core` first, then `whis`:

```yaml
publish-crates:
  steps:
    - name: Publish whis-core
      run: |
        cargo publish -p whis-core --token ${{ secrets.CARGO_REGISTRY_TOKEN }} || true
        sleep 30  # Wait for index propagation
    
    - name: Publish whis
      run: |
        cargo publish -p whis --token ${{ secrets.CARGO_REGISTRY_TOKEN }} || true
```

Key points:
- **Order matters**: `whis` depends on `whis-core`, so publish core first
- **Wait for index**: crates.io takes ~30s to update index
- **Idempotent**: `|| true` makes re-runs succeed (already published = ok)

Token stored in GitHub Secrets: Settings → Secrets → `CARGO_REGISTRY_TOKEN`.

## Creating GitHub Release

The final job collects all artifacts and creates a release:

```yaml
create-release:
  needs:
    - build-linux-x86
    - build-linux-arm64
    - build-macos
    - build-windows
    - build-desktop-linux
    - build-desktop-macos
    - build-desktop-windows
  
  runs-on: ubuntu-latest
  steps:
    - name: Download all artifacts
      uses: actions/download-artifact@v4
      with:
        path: artifacts
    
    - name: Create Release
      uses: softprops/action-gh-release@v1
      with:
        tag_name: ${{ needs.prepare.outputs.version_tag }}
        name: "Whis ${{ needs.prepare.outputs.version_tag }}"
        files: |
          artifacts/**/*
        generate_release_notes: true
```

This:
1. Waits for all build jobs to finish
2. Downloads artifacts (CLI binaries, AppImages, DMG, MSI)
3. Creates GitHub Release with all files attached
4. Auto-generates release notes from commits

## Version Management

Whis uses **semantic versioning** (SemVer): `MAJOR.MINOR.PATCH`

- **MAJOR**: Breaking changes (API incompatible)
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes

Current version: `0.5.8` (pre-1.0, APIs may change)

Version is stored in multiple places:
- `crates/whis-core/Cargo.toml`: `version = "0.5.8"`
- `crates/whis-cli/Cargo.toml`: `version = "0.5.8"`
- `crates/whis-desktop/Cargo.toml`: `version = "0.5.8"`
- `crates/whis-desktop/ui/package.json`: `"version": "0.5.8"`
- `crates/whis-desktop/tauri.conf.json`: `"version": "0.5.8"`

**Important**: All versions must match. Use a script to bump:

```bash
#!/bin/bash
NEW_VERSION=$1

# Update Cargo.toml files
find crates -name Cargo.toml -exec sed -i "s/version = \".*\"/version = \"$NEW_VERSION\"/" {} \;

# Update package.json
sed -i "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" crates/whis-desktop/ui/package.json

# Update tauri.conf.json
sed -i "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" crates/whis-desktop/tauri.conf.json

echo "Bumped to $NEW_VERSION"
```

Run:

```bash
./scripts/bump-version.sh 0.5.9
git add .
git commit -m "Bump version to 0.5.9"
git tag v0.5.9
git push origin main --tags
```

## Caching Dependencies

Speed up builds by caching Cargo dependencies:

```yaml
- name: Cache cargo registry
  uses: actions/cache@v4
  with:
    path: ~/.cargo/registry
    key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

- name: Cache cargo index
  uses: actions/cache@v4
  with:
    path: ~/.cargo/git
    key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}

- name: Cache target directory
  uses: actions/cache@v4
  with:
    path: target
    key: ${{ runner.os }}-target-${{ hashFiles('**/Cargo.lock') }}
```

This caches:
- Downloaded crates (registry)
- Git dependencies (index)
- Compiled artifacts (target)

Cache key includes `Cargo.lock` hash, so cache invalidates when dependencies change.

Result: First build takes ~10 minutes, cached builds take ~2 minutes.

## Testing Before Release

The release workflow includes a `test` job that must pass:

```yaml
test:
  runs-on: ubuntu-latest
  steps:
    - name: Lint
      run: cargo clippy -- -D warnings
    
    - name: Format
      run: cargo fmt --all -- --check
    
    - name: Test
      run: cargo test --all-features
```

If tests fail, the release job doesn't run. This prevents broken builds from being published.

## Release Notes Automation

GitHub's `generate_release_notes` creates notes from commit messages:

```yaml
- uses: softprops/action-gh-release@v1
  with:
    generate_release_notes: true
```

Example output:

```markdown
## What's Changed
* Add support for ElevenLabs provider by @frankdierolf in #45
* Fix portal shortcut binding on GNOME by @frankdierolf in #47
* Improve error messages in CLI by @frankdierolf in #48

**Full Changelog**: https://github.com/frankdierolf/whis/compare/v0.5.7...v0.5.8
```

For more control, use a release notes template (`.github/release-template.md`).

## Deployment Checklist

Before releasing:

1. **Test locally**: Build and run all crates
2. **Update version**: Bump in all `Cargo.toml` and `package.json`
3. **Update changelog**: Document user-facing changes
4. **Commit changes**: `git commit -m "Release v0.5.9"`
5. **Create tag**: `git tag v0.5.9`
6. **Push**: `git push origin main --tags`
7. **Monitor CI**: Watch GitHub Actions for failures
8. **Verify artifacts**: Download and test builds
9. **Update docs**: Website, README if needed
10. **Announce**: Twitter, Reddit, HN (optional)

## Rollback Strategy

If a release is broken:

1. **Delete bad tag**: `git push --delete origin v0.5.9`
2. **Delete bad release**: GitHub → Releases → Delete
3. **Fix issue**: Make commits to `main`
4. **Create new tag**: `git tag v0.5.10` (bump patch)
5. **Re-release**: Push tag

Never reuse a version number. Users may have cached the bad version.

## Future Improvements

**Auto-versioning**: Use `cargo-release` to automate version bumping:

```bash
cargo install cargo-release
cargo release patch  # Bump patch version
cargo release minor  # Bump minor version
```

**Checksums**: Generate SHA256 checksums for downloads:

```yaml
- name: Generate checksums
  run: |
    cd artifacts
    sha256sum * > SHA256SUMS
```

**Update checker**: Notify users when new version available (in-app or CLI).

## Summary

**Key Takeaways**:

1. **GitHub Actions**: Multi-platform builds run in parallel
2. **Cross-compilation**: Use `cross` for ARM64 Linux
3. **Version management**: Keep versions in sync across all crates
4. **Release automation**: Tag triggers build → test → publish → release
5. **Caching**: Speed up builds by caching dependencies

**Workflow Jobs**:

- `prepare`: Extract version tag
- `publish-crates`: Publish to crates.io
- `build-*`: Build binaries for each platform
- `create-release`: Collect artifacts, create GitHub Release

**Release Process**:

1. Bump version in all files
2. Commit and create tag
3. Push tag to GitHub
4. CI builds and publishes automatically
5. Verify release artifacts

**Best Practices**:

- Test locally before pushing tags
- Use SemVer for version numbers
- Never reuse version numbers
- Cache dependencies for faster builds
- Generate release notes from commits

Whis's CI/CD pipeline ensures every release is built consistently across all platforms. Automation reduces manual work and prevents mistakes.

---

## Congratulations! 🎉

You've completed the Whis technical book. You now have a comprehensive understanding of:

- Rust fundamentals and patterns used in Whis
- Audio recording with cpal
- Transcription with multiple cloud providers
- Parallel processing with tokio
- Desktop apps with Tauri + Vue
- Global shortcuts across platforms (X11, Wayland, macOS, Windows)
- Building and distributing cross-platform applications

The full Whis codebase is now an open book to you. You can:
- Add new transcription providers
- Extend the UI with new features
- Contribute to the project
- Build your own audio/transcription tools

**Next Steps**:

- Explore the codebase hands-on
- Try adding a feature (Chapter 26 has ideas)
- Join the discussion on GitHub
- Share what you've learned

Thank you for reading! 🙏
