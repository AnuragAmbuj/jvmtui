#!/bin/bash
set -e

VERSION=$1
BINARY_PATH=$2

if [ -z "$VERSION" ] || [ -z "$BINARY_PATH" ]; then
    echo "Usage: $0 <VERSION> <BINARY_PATH>"
    exit 1
fi

echo "Creating DEB package for version $VERSION..."

# Create package structure
PKGDIR="pkg-deb"
mkdir -p "$PKGDIR/DEBIAN"
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

# Create control file
cat > "$PKGDIR/DEBIAN/control" << EOF
Package: jvm-tui
Version: $VERSION-1
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Anurag Ambuj <anurag.ambuj@example.com>
Description: A beautiful TUI for JVM monitoring - like VisualVM for your terminal
 JVM-TUI brings powerful JVM monitoring to your terminal with a keyboard-driven
 interface. Monitor heap usage, garbage collection, memory pools, and threads in
 real-time - perfect for SSH sessions and production environments where GUI tools
 aren't available.
 .
 Features:
  - Auto-Discovery of running JVMs
  - Real-Time Monitoring with live metrics
  - Keyboard-Driven interface (Vim-style navigation)
  - No Agents Required (uses standard JDK tools)
  - Remote Monitoring via SSH+JDK or Jolokia HTTP
  - Multiple Export Formats (JSON, Prometheus, CSV)
EOF

# Create conffiles (if needed)
touch "$PKGDIR/DEBIAN/conffiles"

# Create md5sums
cd "$PKGDIR"
find . -type f ! -path './DEBIAN/*' -exec md5sum {} > DEBIAN/md5sums \;
cd ..

# Set permissions
chmod -R 0755 "$PKGDIR/DEBIAN"
chmod 0644 "$PKGDIR/DEBIAN/control"
chmod 0644 "$PKGDIR/DEBIAN/md5sums"

# Build the package
fakeroot dpkg-deb --build "$PKGDIR" "jvm-tui_${VERSION}-1_amd64.deb"

# Cleanup
rm -rf "$PKGDIR"

echo "Created jvm-tui_${VERSION}-1_amd64.deb"
