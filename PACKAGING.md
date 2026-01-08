# Packaging & Release Guide

This guide explains how jvm-tui is packaged and released for multiple platforms.

## Automated Release Process

Releases are fully automated via GitHub Actions. To create a new release:

1. Update version in `Cargo.toml`
2. Tag the commit: `git tag v0.1.0 && git push origin v0.1.0`
3. GitHub Actions will:
   - Build binaries for Linux (x86_64, aarch64)
   - Build binaries for macOS (x86_64, arm64)
   - Create DEB packages (Debian/Ubuntu)
   - Create RPM packages (Fedora/RHEL)
   - Create Arch Linux packages
   - Generate Homebrew formula
   - Upload all artifacts to GitHub Release

## Supported Platforms

### Linux

| Distro | Package Format | Architecture | Status |
|---------|--------------|----------------|---------|
| Ubuntu/Debian | DEB | amd64, arm64 | ✅ Automated |
| Fedora/RHEL | RPM | x86_64 | ✅ Automated |
| Arch Linux | PKGBUILD | x86_64 | ✅ Automated |

### macOS

| Architecture | Package Format | Status |
|--------------|----------------|---------|
| x86_64 (Intel) | Binary + Homebrew | ✅ Automated |
| arm64 (Apple Silicon) | Binary + Homebrew | ✅ Automated |

## Manual Package Building

If you need to build packages manually:

### DEB (Debian/Ubuntu)

```bash
# Install dependencies
sudo apt-get install -y ruby-dev
gem install fpm

# Build binary
cargo build --release

# Create package
./scripts/package-deb.sh <VERSION> target/release/jvm-tui
```

### RPM (Fedora/RHEL)

```bash
# Install dependencies
sudo apt-get install -y ruby-dev rpm
gem install fpm

# Build binary
cargo build --release

# Create package
./scripts/package-rpm.sh <VERSION> target/release/jvm-tui
```

### Arch Linux

```bash
# Install base-devel
sudo pacman -S base-devel

# Build package
./scripts/package-arch.sh <VERSION>
```

### macOS (Homebrew)

```bash
# Generate formula template
./scripts/generate-homebrew.sh <VERSION>

# Build bottles (after release exists)
brew install --build-from-source Formula/jvm-tui.rb
```

## Package Scripts

All packaging scripts are in the `scripts/` directory:

- `package-deb.sh` - Creates DEB packages for Debian/Ubuntu
- `package-rpm.sh` - Creates RPM packages for Fedora/RHEL
- `package-arch.sh` - Creates Arch Linux packages
- `generate-homebrew.sh` - Generates Homebrew formula template

## Repository Managers

### Homebrew Tap

We maintain a custom tap for Homebrew:
- Repository: https://github.com/AnuragAmbuj/homebrew-jvm-tui
- Installation: `brew install AnuragAmbuj/jvm-tui/jvm-tui`

The GitHub Actions workflow automatically creates a PR to update the tap when a new version is released.

### AUR (Arch User Repository)

Coming soon - will submit PKGBUILD to AUR for automatic building.

## Release Checklist

Before tagging a release:

- [ ] Update version in `Cargo.toml`
- [ ] Update CHANGELOG.md with new features
- [ ] Update README.md with any new features
- [ ] Test all packaging scripts locally
- [ ] Ensure all CI checks pass
- [ ] Run `cargo test --all`
- [ ] Run `cargo clippy --all-targets`
- [ ] Run `cargo fmt --check`

After tagging a release:

- [ ] Verify GitHub Actions workflow completes successfully
- [ ] Check all artifacts are uploaded
- [ ] Test binary from release on Linux
- [ ] Test binary from release on macOS
- [ ] Verify DEB package installs correctly
- [ ] Verify RPM package installs correctly
- [ ] Verify Homebrew formula updates
- [ ] Announce release in project discussions/issues

## Troubleshooting

### Build Failures

If package builds fail:

1. Check CI logs for specific errors
2. Verify all dependencies are installed
3. Ensure version string format is correct (e.g., `0.1.0`)
4. Test scripts locally first

### Package Verification

After building packages:

```bash
# DEB verification
dpkg -I jvm-tui_*.deb
dpkg -c jvm-tui_*.deb

# RPM verification
rpm -qip jvm-tui-*.rpm
rpm -qlp jvm-tui-*.rpm

# Arch verification
tar -tzf jvm-tui-*.pkg.tar.zst
```

## Contributing

If you want to add support for a new platform:

1. Create a packaging script in `scripts/`
2. Add a matrix job to `.github/workflows/release.yml`
3. Update this documentation
4. Test the package locally
5. Submit a pull request

## License

All packages are licensed under the same license as the project: MIT OR Apache-2.0
