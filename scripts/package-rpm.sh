#!/bin/bash
set -e

VERSION=$1
BINARY_PATH=$2

if [ -z "$VERSION" ] || [ -z "$BINARY_PATH" ]; then
    echo "Usage: $0 <VERSION> <BINARY_PATH>"
    exit 1
fi

echo "Creating RPM package for version $VERSION..."

# Create package structure
PKGDIR="pkg-rpm"
mkdir -p "$PKGDIR/usr/bin"
mkdir -p "$PKGDIR/usr/share/doc/jvm-tui"
mkdir -p "$PKGDIR/usr/share/licenses/jvm-tui"

# Copy binary
cp "$BINARY_PATH" "$PKGDIR/usr/bin/jvm-tui"
chmod +x "$PKGDIR/usr/bin/jvm-tui"

# Copy documentation
cp README.md "$PKGDIR/usr/share/doc/jvm-tui/"
cp LICENSE-MIT "$PKGDIR/usr/share/licenses/jvm-tui/"
cp LICENSE-APACHE "$PKGDIR/usr/share/licenses/jvm-tui/"

# Create RPM using fpm
fpm -s dir -t rpm \
    -n jvm-tui \
    -v "$VERSION" \
    --iteration 1 \
    --architecture x86_64 \
    --license "MIT OR Apache-2.0" \
    --maintainer "Anurag Ambuj <anurag.ambuj@example.com>" \
    --url "https://github.com/AnuragAmbuj/jvmtui" \
    --description "A beautiful TUI for JVM monitoring - like VisualVM for your terminal. Monitor heap usage, garbage collection, memory pools, and threads in real-time." \
    -C "$PKGDIR" \
    usr

# Cleanup
rm -rf "$PKGDIR"

echo "RPM package created successfully"
