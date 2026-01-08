# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- JFR Integration (Flight Recorder support)
- AUR (Arch User Repository) package submission
- Windows binary releases

## [0.1.0] - 2025-01-08

### Added
- Initial release of jvm-tui
- Local JVM monitoring with auto-discovery
- 5 comprehensive monitoring views: Overview, Memory, Threads, GC, Classes
- Real-time metrics collection with configurable intervals
- Thread search functionality with `/` command
- Class histogram view
- Manual GC trigger with confirmation
- Export capabilities (JSON, Prometheus, CSV)
- Keyboard-driven Vim-style interface
- Terminal-adaptive color system

### Remote Monitoring Features
- SSH+JDK connector for agent-free remote monitoring
- Jolokia HTTP connector for agent-based monitoring
- TOML-based configuration system
- Saved connections with multiple connection types
- Export format selector UI

### Documentation
- Comprehensive README with usage examples
- Architecture documentation (docs/ARCHITECTURE.md)
- Configuration examples (config.example.toml)
- Packaging and release guide (docs/PACKAGING.md)

### Installation
- Pre-built binaries for Linux (x86_64, aarch64)
- Pre-built binaries for macOS (x86_64, arm64)
- DEB packages for Debian/Ubuntu
- RPM packages for Fedora/RHEL
- Arch Linux PKGBUILD
- Homebrew formula template

[Unreleased]: https://github.com/AnuragAmbuj/jvmtui/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/AnuragAmbuj/jvmtui/releases/tag/v0.1.0
