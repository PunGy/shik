#!/bin/bash
# Shik Release Build Script
# Builds release binaries and packages for multiple platforms

set -e

VERSION="${1:-0.1.0}"
RELEASE_DIR="releases"
PROJECT_NAME="shik"

echo "=========================================="
echo "Building Shik v${VERSION} release"
echo "=========================================="

# Create release directory
mkdir -p "$RELEASE_DIR"

# Detect current platform
CURRENT_OS=$(uname -s)
CURRENT_ARCH=$(uname -m)

echo "Current platform: ${CURRENT_OS} ${CURRENT_ARCH}"
echo ""

# Build release binary
echo "Building optimized release binary..."
cargo build --release

# Get binary size
BINARY_SIZE=$(ls -lh target/release/shik | awk '{print $5}')
echo "Binary size: ${BINARY_SIZE}"
echo ""

# Determine archive name based on platform
case "${CURRENT_OS}-${CURRENT_ARCH}" in
    "Darwin-arm64")
        PLATFORM="macos-aarch64"
        ;;
    "Darwin-x86_64")
        PLATFORM="macos-x86_64"
        ;;
    "Linux-x86_64")
        PLATFORM="linux-x86_64"
        ;;
    "Linux-aarch64")
        PLATFORM="linux-aarch64"
        ;;
    *)
        PLATFORM="${CURRENT_OS}-${CURRENT_ARCH}"
        ;;
esac

ARCHIVE_NAME="${PROJECT_NAME}-v${VERSION}-${PLATFORM}"

# Create tar.xz archive (best compression, standard for Linux)
echo "Creating tar.xz archive..."
tar -cJf "${RELEASE_DIR}/${ARCHIVE_NAME}.tar.xz" \
    -C target/release shik \
    --transform 's,^,shik-v'"${VERSION}"'/,' \
    2>/dev/null || \
tar -cJf "${RELEASE_DIR}/${ARCHIVE_NAME}.tar.xz" -C target/release shik

echo "Created: ${RELEASE_DIR}/${ARCHIVE_NAME}.tar.xz"

# Also create tar.gz for broader compatibility
echo "Creating tar.gz archive..."
tar -czf "${RELEASE_DIR}/${ARCHIVE_NAME}.tar.gz" -C target/release shik
echo "Created: ${RELEASE_DIR}/${ARCHIVE_NAME}.tar.gz"

# Linux-specific: Build .deb and .rpm packages
if [[ "$CURRENT_OS" == "Linux" ]]; then
    echo ""
    echo "Building Linux packages..."
    
    # Build .deb package (Debian/Ubuntu)
    if command -v cargo-deb &> /dev/null; then
        echo "Building .deb package..."
        cargo deb --no-build
        DEB_FILE=$(find target/debian -name "*.deb" -type f | head -1)
        if [[ -n "$DEB_FILE" ]]; then
            cp "$DEB_FILE" "${RELEASE_DIR}/"
            echo "Created: ${RELEASE_DIR}/$(basename "$DEB_FILE")"
        fi
    else
        echo "Note: Install cargo-deb for .deb packages: cargo install cargo-deb"
    fi
    
    # Build .rpm package (Fedora/RHEL/CentOS)
    if command -v cargo-generate-rpm &> /dev/null; then
        echo "Building .rpm package..."
        cargo generate-rpm
        RPM_FILE=$(find target/generate-rpm -name "*.rpm" -type f | head -1)
        if [[ -n "$RPM_FILE" ]]; then
            cp "$RPM_FILE" "${RELEASE_DIR}/"
            echo "Created: ${RELEASE_DIR}/$(basename "$RPM_FILE")"
        fi
    else
        echo "Note: Install cargo-generate-rpm for .rpm packages: cargo install cargo-generate-rpm"
    fi
fi

# macOS-specific: Create zip for easier distribution
if [[ "$CURRENT_OS" == "Darwin" ]]; then
    echo "Creating zip archive for macOS..."
    zip -j "${RELEASE_DIR}/${ARCHIVE_NAME}.zip" target/release/shik
    echo "Created: ${RELEASE_DIR}/${ARCHIVE_NAME}.zip"
fi

# Generate checksums
echo ""
echo "Generating checksums..."
cd "${RELEASE_DIR}"
if command -v sha256sum &> /dev/null; then
    sha256sum * > SHA256SUMS.txt 2>/dev/null || true
elif command -v shasum &> /dev/null; then
    shasum -a 256 * > SHA256SUMS.txt 2>/dev/null || true
fi
cd ..

echo ""
echo "=========================================="
echo "Release build complete!"
echo "=========================================="
echo ""
echo "Release artifacts in ${RELEASE_DIR}/:"
ls -la "${RELEASE_DIR}/"
echo ""
echo "Next steps:"
echo "  1. Test the binary: ./target/release/shik --help"
echo "  2. Upload archives to GitHub Releases"
echo "  3. Publish to crates.io: cargo publish"