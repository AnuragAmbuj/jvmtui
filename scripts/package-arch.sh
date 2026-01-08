#!/bin/bash
set -e

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <VERSION>"
    exit 1
fi

echo "Creating Arch Linux package for version $VERSION..."

# Create PKGBUILD
cat > PKGBUILD << EOF
# Maintainer: Anurag Ambuj <anurag.ambuj@example.com>
pkgname=jvm-tui
pkgver=$VERSION
pkgrel=1
pkgdesc="A beautiful TUI for JVM monitoring - like VisualVM for your terminal"
arch=('x86_64')
url="https://github.com/AnuragAmbuj/jvmtui"
license=('MIT' 'Apache-2.0')
depends=('gcc-libs')
makedepends=('cargo')
provides=('jvm-tui')
conflicts=('jvm-tui')
source=("\$pkgname-\$pkgver.tar.gz::https://github.com/AnuragAmbuj/jvmtui/archive/v\${pkgver}.tar.gz")
sha256sums=('SKIP')

prepare() {
    cd "\$pkgname-\$pkgver"
    cargo fetch --locked
}

build() {
    cd "\$pkgname-\$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --all-features
}

check() {
    cd "\$pkgname-\$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    cargo test --frozen --all-features
}

package() {
    cd "\$pkgname-\$pkgver"
    install -Dm0755 -t "\$pkgdir/usr/bin/" "target/release/jvm-tui"
    install -Dm0644 -t "\$pkgdir/usr/share/licenses/\$pkgname/" "LICENSE-MIT" "LICENSE-APACHE"
    install -Dm0644 -t "\$pkgdir/usr/share/doc/\$pkgname/" README.md
}
EOF

# Build the package
makepkg -s

echo "Arch Linux package created successfully: jvm-tui-\$VERSION-1-x86_64.pkg.tar.zst"
