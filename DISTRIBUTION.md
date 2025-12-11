# Shik Distribution Guide

This document describes how to build and distribute Shik for multiple platforms (Linux, macOS, Windows).

## Quick Start

```bash
# Build and package for your current platform
./scripts/build-release.sh 0.1.0

# Output will be in releases/ directory
```

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (1.70+)
- For Linux packages: `cargo-deb` and `cargo-generate-rpm`
- For cross-compilation: appropriate target toolchains

## Building for Your Platform

### Development Build
```bash
cargo build
```

### Release Build (Optimized)
```bash
cargo build --release
```

The binary will be located at `target/release/shik` (or `shik.exe` on Windows).

## Distribution Formats

### 1. crates.io (Recommended - Primary Distribution)

This is the **best practice** for Rust projects. Users install with a single command:

```bash
cargo install shik
```

#### Publishing to crates.io

1. Create an account on [crates.io](https://crates.io)

2. Get your API token from https://crates.io/settings/tokens

3. Login via cargo:
   ```bash
   cargo login <your-api-token>
   ```

4. Verify the package builds and metadata is correct:
   ```bash
   cargo publish --dry-run
   ```

5. Publish:
   ```bash
   cargo publish
   ```

**Important**: Before publishing, update the `repository` and `homepage` URLs in `Cargo.toml` to your actual GitHub repository.

### 2. Pre-built Binaries (GitHub Releases)

Create archives for users who don't have Rust installed:

#### tar.xz (Best for Linux/macOS)
```bash
# Best compression, standard for Linux distributions
tar -cJf shik-v0.1.0-linux-x86_64.tar.xz -C target/release shik
```

#### tar.gz (Broader compatibility)
```bash
tar -czf shik-v0.1.0-linux-x86_64.tar.gz -C target/release shik
```

#### zip (Windows/macOS)
```bash
zip -j shik-v0.1.0-windows-x86_64.zip target/release/shik.exe
```

### 3. Linux Packages (.deb, .rpm)

#### Debian/Ubuntu (.deb)

Install the tool:
```bash
cargo install cargo-deb
```

Build the package:
```bash
cargo deb
```

Output: `target/debian/shik_0.1.0-1_amd64.deb`

Users install with:
```bash
sudo dpkg -i shik_0.1.0-1_amd64.deb
# or
sudo apt install ./shik_0.1.0-1_amd64.deb
```

#### Fedora/RHEL/CentOS (.rpm)

Install the tool:
```bash
cargo install cargo-generate-rpm
```

Build the package (after `cargo build --release`):
```bash
cargo generate-rpm
```

Output: `target/generate-rpm/shik-0.1.0-1.x86_64.rpm`

Users install with:
```bash
sudo rpm -i shik-0.1.0-1.x86_64.rpm
# or
sudo dnf install ./shik-0.1.0-1.x86_64.rpm
```

## Automated Build Script

The `scripts/build-release.sh` script automates the entire process:

```bash
./scripts/build-release.sh 0.1.0
```

This will:
1. Build an optimized release binary
2. Create tar.xz and tar.gz archives
3. On Linux: build .deb and .rpm packages (if tools installed)
4. On macOS: create a zip archive
5. Generate SHA256 checksums

## Cross-Platform Distribution

### Option 1: Native Builds on Each Platform

The simplest and most reliable approach is to build natively on each target platform:

| Platform | Build Command | Output Binary |
|----------|---------------|---------------|
| Linux (x86_64) | `cargo build --release` | `target/release/shik` |
| macOS (x86_64) | `cargo build --release` | `target/release/shik` |
| macOS (ARM64) | `cargo build --release` | `target/release/shik` |
| Windows (x86_64) | `cargo build --release` | `target/release/shik.exe` |

### Option 2: Cross-Compilation from macOS

```bash
# Add target platforms
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin

# Install cross-compilation toolchains (macOS)
brew install FiloSottile/musl-cross/musl-cross  # For Linux
brew install mingw-w64                           # For Windows

# Build for each target
CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

### Option 3: Using `cross` Tool (Docker-based)

```bash
cargo install cross

cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target x86_64-unknown-linux-musl
cross build --release --target x86_64-pc-windows-gnu
```

## Installation Instructions for End Users

### Method 1: cargo install (Recommended)
```bash
# From crates.io (after you publish)
cargo install shik

# From git repository
cargo install --git https://github.com/yourusername/shik
```

### Method 2: Pre-built Binary

#### Linux
```bash
# Download and extract
curl -LO https://github.com/yourusername/shik/releases/download/v0.1.0/shik-v0.1.0-linux-x86_64.tar.xz
tar -xJf shik-v0.1.0-linux-x86_64.tar.xz

# Install system-wide
sudo mv shik /usr/local/bin/

# Or install for current user
mkdir -p ~/.local/bin
mv shik ~/.local/bin/
```

#### macOS
```bash
curl -LO https://github.com/yourusername/shik/releases/download/v0.1.0/shik-v0.1.0-macos-aarch64.tar.xz
tar -xJf shik-v0.1.0-macos-aarch64.tar.xz
sudo mv shik /usr/local/bin/
```

#### Windows
1. Download `shik-v0.1.0-windows-x86_64.zip`
2. Extract `shik.exe`
3. Add to PATH or move to a directory in PATH

### Method 3: Linux Packages

#### Debian/Ubuntu
```bash
curl -LO https://github.com/yourusername/shik/releases/download/v0.1.0/shik_0.1.0-1_amd64.deb
sudo apt install ./shik_0.1.0-1_amd64.deb
```

#### Fedora/RHEL
```bash
curl -LO https://github.com/yourusername/shik/releases/download/v0.1.0/shik-0.1.0-1.x86_64.rpm
sudo dnf install ./shik-0.1.0-1.x86_64.rpm
```

## Release Checklist

Before releasing a new version:

1. [ ] Update version in `Cargo.toml`
2. [ ] Update `repository` and `homepage` URLs to your actual GitHub repo
3. [ ] Run tests: `cargo test`
4. [ ] Build release: `cargo build --release`
5. [ ] Test the binary: `./target/release/shik sample.shk`
6. [ ] Run `cargo publish --dry-run` to verify
7. [ ] Create git tag: `git tag v0.1.0`
8. [ ] Push tag: `git push origin v0.1.0`
9. [ ] Run build script: `./scripts/build-release.sh 0.1.0`
10. [ ] Upload artifacts to GitHub Releases
11. [ ] Publish to crates.io: `cargo publish`

## Build Optimization Notes

The `Cargo.toml` includes release profile optimizations:

- `opt-level = 3`: Maximum optimization
- `lto = true`: Link-time optimization for smaller binaries
- `codegen-units = 1`: Better optimization at cost of compile time
- `strip = true`: Strip symbols for smaller binary size

These settings produce optimized, production-ready binaries (~770KB).

## Troubleshooting

### Linux Binary Not Running
If the Linux binary fails with "not found" errors, it may be a dynamic linking issue. Build with musl for static linking:
```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

### Windows Build Fails
Ensure mingw-w64 is properly installed and in PATH:
```bash
brew install mingw-w64  # macOS
apt install mingw-w64   # Ubuntu/Debian
```

### macOS Code Signing
For distribution outside the App Store, you may need to sign the binary:
```bash
codesign --sign - target/release/shik
```

Or users can allow it via System Preferences > Security & Privacy.

### cargo publish Fails
Common issues:
- Missing `license` field in Cargo.toml ✓ (already added)
- Invalid `repository` URL - update to your actual GitHub URL
- Package name already taken - choose a different name