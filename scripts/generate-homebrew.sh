#!/bin/bash
set -e

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <VERSION>"
    exit 1
fi

echo "Generating Homebrew formula for version $VERSION..."

HOMEBREW_TAP_DIR="homebrew-tap"
mkdir -p "$HOMEBREW_TAP_DIR/Formula"

cat > "$HOMEBREW_TAP_DIR/Formula/jvm-tui.rb" << EOF
# frozen_string_literal: true

class JvmTui < Formula
  desc "A beautiful TUI for JVM monitoring - like VisualVM for your terminal"
  homepage "https://github.com/AnuragAmbuj/jvmtui"
  url "https://github.com/AnuragAmbuj/jvmtui/archive/refs/tags/v${VERSION}.tar.gz"
  sha256 "PLACEHOLDER_SHA256"

  license any_of: ["MIT", "Apache-2.0"]

  bottle do
    root_url "https://github.com/AnuragAmbuj/homebrew-jvm-tui/releases/download/v${VERSION}"
    sha256 cellar: :any_skip_relocation, arm64_sonoma:   "PLACEHOLDER_ARM64_SHA256"
    sha256 cellar: :any_skip_relocation, arm64_ventura:  "PLACEHOLDER_ARM64_SHA256"
    sha256 cellar: :any_skip_relocation, arm64_monterey: "PLACEHOLDER_ARM64_SHA256"
    sha256 cellar: :any_skip_relocation, sonoma:         "PLACEHOLDER_X86_64_SHA256"
    sha256 cellar: :any_skip_relocation, ventura:        "PLACEHOLDER_X86_64_SHA256"
    sha256 cellar: :any_skip_relocation, monterey:       "PLACEHOLDER_X86_64_SHA256"
  end

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system bin/"jvm-tui", "--help"
  end
end
EOF

echo "Homebrew formula template created at $HOMEBREW_TAP_DIR/Formula/jvm-tui.rb"
echo "Note: Update the SHA256 checksums with actual values after building bottles"
