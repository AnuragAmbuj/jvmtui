# JVM-TUI

> A lightweight terminal user interface for monitoring Java Virtual Machines - no agents required.

[![CI](https://github.com/AnuragAmbuj/jvmtui/actions/workflows/ci.yml/badge.svg)](https://github.com/AnuragAmbuj/jvmtui/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

**JVM-TUI** brings powerful JVM monitoring to your terminal with a keyboard-driven interface. Monitor heap usage, garbage collection, memory pools, and threads in real-time - perfect for SSH sessions and production environments where GUI tools aren't available.

## Features

- **Auto-Discovery** - Automatically finds all running JVMs on your system
- **Real-Time Monitoring** - Live heap usage, GC statistics, and memory pool metrics
- **Keyboard-Driven** - Full Vim-style navigation (no mouse needed)
- **No Agents Required** - Uses standard JDK tools (jcmd, jstat, jps)
- **SSH-Friendly** - Works perfectly over remote connections
- **TUI** - Responsive interface inside a terminal
- **Lightweight** - Only 1.3MB binary with minimal resource usage

---

## Terminal Prototypes

### JVM Selection
Auto-discovers and lists all running JVMs:

```
┌─────────────────────────────────────────────────────────────────────┐
│ JVM-TUI - Select JVM Process                                        │
├─────────────────────────────────────────────────────────────────────┤
│ Discovered JVMs                                                     │
│                                                                     │
│ >> PID: 46168 - com.intellij.idea.Main                             │
│    PID: 48127 - sonarlint-ls.jar                                   │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ Controls: ↑/k: Up | ↓/j: Down | Enter: Connect | r: Refresh | q: Quit │
└─────────────────────────────────────────────────────────────────────┘
```

### Overview Dashboard
Real-time heap usage, GC stats, and memory pools:

```
┌─────────────────────────────────────────────────────────────────────┐
│ JVM Info                                                            │
│ PID: 46168 │ JDK 21.0.8 │ Uptime: 108h 27m                         │
├─────────────────────────────────────────────────────────────────────┤
│ [1:Overview] 2:Memory 3:Threads 4:GC 5:Classes                     │
├─────────────────────────────────────────────────────────────────────┤
│ ┌─ Heap Usage: 714 / 817 MB (87.4%) ─────────────────────────────┐ │
│ │ ▁▂▃▄▅▆▇█▇▆▅▄▃▂▁▂▃▄▅▆▇█▇▆▅▄▃▂▁▂▃▄▅▆▇█▇▆▅▄▃▂▁▂▃▄▅▆▇█           │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│ ┌─ GC Statistics ─────────────────────────────────────────────────┐ │
│ │ Young GC: 125,500 collections (498.25s total)                   │ │
│ │ Full GC: 37 collections (9.22s total)                           │ │
│ │ Avg Young GC: 3.97ms | Avg Full GC: 249.19ms                   │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

### Memory View
Detailed memory pool breakdown with visual gauges:

```
┌─────────────────────────────────────────────────────────────────────┐
│ ┌─ Heap Usage Timeline (max: 817 MB) ────────────────────────────┐ │
│ │ ▁▁▂▂▃▃▄▄▅▅▆▆▇▇██▇▇▆▆▅▅▄▄▃▃▂▂▁▁▂▂▃▃▄▄▅▅▆▆▇▇██▇▇▆▆▅▅▄▄▃▃▂▂     │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│ ┌─ Metaspace ─────────────────────────────────────────────────────┐ │
│ │ Metaspace: 505 / 1507 MB (33.5%)                                │ │
│ │ ████████████░░░░░░░░░░░░░░░░░░░░                                │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```


## Quick Start

### Prerequisites

- **Rust** 1.75 or later ([Install Rust](https://rustup.rs/))
- **JDK** 11+ with command-line tools (jcmd, jstat, jps)

### Installation

```bash
# Clone the repository
git clone https://github.com/AnuragAmbuj/jvmtui.git
cd jvmtui

# Build the release binary
cargo build --release

# The binary will be at target/release/jvm-tui
```

### Usage

```bash
# Auto-discover and select a JVM
./target/release/jvm-tui

# Connect to a specific JVM by PID
./target/release/jvm-tui --pid 12345

# Custom polling interval (default: 1s)
./target/release/jvm-tui --interval 500ms

# Show help
./target/release/jvm-tui --help
```

## Keyboard Controls

### JVM Picker Screen
| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Connect to selected JVM |
| `r` | Refresh JVM list |
| `q` | Quit application |

### Monitoring Screen
| Key | Action |
|-----|--------|
| `1-5` | Switch to tab (Overview, Memory, Threads, GC, Classes) |
| `h` / `←` | Previous tab |
| `l` / `→` | Next tab |
| `q` | Disconnect and quit |

## Requirements

### JDK Tools

JVM-TUI requires the following JDK command-line tools:

- **jcmd** - JVM diagnostic commands
- **jstat** - JVM statistics
- **jps** - JVM process listing (fallback)

These tools are included with standard JDK installations (not JRE).

#### Verification

Check if tools are available:

```bash
jcmd -h
jstat -h
jps -h
```

#### Installation Guide

**macOS (Homebrew):**
```bash
brew install openjdk@21
```

**Ubuntu/Debian:**
```bash
sudo apt install openjdk-21-jdk
```

**RHEL/CentOS/Fedora:**
```bash
sudo dnf install java-21-openjdk-devel
```

**Windows:**
Download from [Adoptium](https://adoptium.net/) and add `bin` directory to PATH.

## What You Can Monitor

### Overview Dashboard
- Real-time heap usage sparkline
- GC collection counts and times
- Average GC pause times
- Memory pool summary
- JVM uptime and version

### Memory View
- Heap usage timeline
- Memory pool breakdowns (Metaspace, Class Space, etc.)
- Color-coded capacity warnings
- Used/Max/Committed metrics

### Threads View
- Thread count by state
- Thread list (coming in Phase 2)
- Stack traces (coming in Phase 2)

### GC View *(Planned)*
- GC event timeline
- Pause time distribution
- Throughput calculation

### Classes View *(Planned)*
- Class histogram
- Top memory consumers
- Class loading statistics

## How It Works

JVM-TUI uses **agentless monitoring** - it communicates with JVMs using standard JDK tools:

1. **Discovery**: Uses `jcmd -l` (or `jps -l` as fallback) to find running JVMs
2. **Connection**: Executes JDK commands (jcmd, jstat) to query JVM state
3. **Parsing**: Parses command output into structured data
4. **Collection**: Polls metrics at configurable intervals (default: 1s)
5. **Display**: Renders live data in a terminal UI

**Benefits:**
- No JVM agent installation required
- No JVM restarts needed
- Works with any JVM process
- Minimal performance impact
- Safe for production use

## Development

### Build from Source

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release
```

### Run Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check for issues
cargo check
```

---

## Contributing

Contributions are welcome! This project is in active development.

**Phase 1 (MVP)** Complete
- JVM discovery and connection
- Real-time metrics collection
- Overview and Memory views
- Basic TUI scaffold

**Phase 2** (Planned)
- Enhanced GC view with timeline
- Full thread dumps with stack traces
- Class histogram view
- Trigger GC action

See [docs/14-implementation-phases.md](docs/14-implementation-phases.md) for detailed roadmap.

## License

This project is dual-licensed under:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

You may choose either license for your use.

## 🙏 Acknowledgments

Built with:
- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Tokio](https://tokio.rs/) - Async runtime
- [Clap](https://github.com/clap-rs/clap) - CLI parsing

Inspired by [VisualVM](https://visualvm.github.io/) and modern CLI tools like [htop](https://htop.dev/).

## Contact

**Anurag Ambuj**
- GitHub: [@AnuragAmbuj](https://github.com/AnuragAmbuj)
- Repository: [github.com/AnuragAmbuj/jvmtui](https://github.com/AnuragAmbuj/jvmtui)

---

<p align="center">
  <i>JVM-TUI - Because production JVMs don't have GUIs.</i>
</p>
