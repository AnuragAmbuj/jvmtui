# JVM-TUI

> A lightweight terminal user interface for monitoring Java Virtual Machines - no agents required.

[![CI](https://github.com/AnuragAmbuj/jvmtui/actions/workflows/ci.yml/badge.svg)](https://github.com/AnuragAmbuj/jvmtui/actions)
[![Release](https://img.shields.io/github/v/release/AnuragAmbuj/jvmtui)](https://github.com/AnuragAmbuj/jvmtui/releases/latest)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

**JVM-TUI** brings powerful JVM monitoring to your terminal with a keyboard-driven interface. Monitor heap usage, garbage collection, memory pools, and threads in real-time - perfect for SSH sessions and production environments where GUI tools aren't available.

## Features

### Core Capabilities
- **Auto-Discovery** - Automatically finds all running JVMs on your system
- **Real-Time Monitoring** - Live heap usage, GC statistics, and memory pool metrics
- **Keyboard-Driven** - Full Vim-style navigation (no mouse needed)
- **No Agents Required** - Uses standard JDK tools (jcmd, jstat, jps)
- **SSH-Friendly** - Works perfectly over remote connections
- **Terminal-Adaptive** - Automatically adapts to your terminal's color scheme
- **Lightweight** - Pure Rust, ~3MB binary with minimal resource usage

### Remote Monitoring (Phase 3 Complete)
- **SSH+JDK** - Monitor remote JVMs over SSH (no agent needed!)
- **Jolokia HTTP** - Connect to JVMs with Jolokia agent via HTTP/HTTPS
- **Saved Connections** - Store favorite JVMs in config file
- **Multiple Export Formats** - JSON, Prometheus, CSV with format selector
- **Configuration System** - TOML-based config with auto-discovery

### Advanced Features (Phase 2 Complete)
- **5 Comprehensive Views** - Overview, Memory, Threads, GC, Classes
- **Thread Search** - Find threads by name or ID with `/` command
- **Class Histogram** - On-demand class memory analysis
- **GC Triggers** - Manually trigger garbage collection with confirmation
- **Export Capabilities** - Export thread dumps and metrics to files
- **Error Recovery** - Automatic reconnection and graceful error handling
- **Help System** - Built-in keybinding reference (press `?`)
- **Smooth Scrolling** - Navigate large thread/class lists with j/k

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

---

## Remote Monitoring Options

JVM-TUI supports **three connection types** for monitoring JVMs:

### 1. Local JVMs (Auto-Discovery)
Automatically discovers JVMs running on your local machine using `jcmd` and `jps`.

**No configuration needed** - just run `jvm-tui` and select from discovered JVMs.

### 2. SSH + JDK Tools (Agent-Free Remote)
Monitor remote JVMs over SSH **without installing any agent**. Uses standard JDK tools (jcmd, jstat) remotely.

**Configuration:**
```toml
[[connections]]
type = "ssh-jdk"
name = "Production Server"
ssh_host = "prod.example.com"
ssh_user = "appuser"
ssh_key = "~/.ssh/id_rsa"
pid = 12345
```

**Requirements:**
- SSH access to remote server
- JDK tools (jcmd, jstat) on remote server
- JVM process ID

**Advantages:**
- ✅ No agent installation needed
- ✅ Works with any JVM
- ✅ Standard SSH (port 22)
- ✅ Firewall-friendly

### 3. Jolokia HTTP (Agent-Based Remote)
Connect to remote JVMs via HTTP using the Jolokia agent.

**Configuration:**
```toml
[[connections]]
type = "jolokia"
name = "API Server"
url = "http://api.example.com:8080/jolokia"
username = "admin"    # optional
password = "secret"   # optional
```

**Setup on remote JVM:**
```bash
# Download Jolokia agent
wget https://repo1.maven.org/maven2/org/jolokia/jolokia-jvm/1.7.1/jolokia-jvm-1.7.1-agent.jar

# Start JVM with Jolokia
java -javaagent:jolokia-jvm-1.7.1-agent.jar=port=8080 -jar your-app.jar
```

**Advantages:**
- ✅ HTTP/HTTPS transport
- ✅ Built-in authentication
- ✅ Widely used in production

### Connection Comparison

| Feature | Local | SSH+JDK | Jolokia | Native JMX¹ |
|---------|-------|---------|---------|-------------|
| **Agent Required** | No | No | Yes | No |
| **Network Protocol** | - | SSH | HTTP | RMI |
| **JRE on Monitor Host** | No | No | No | **YES** |
| **Pure Rust** | ✅ | ✅ | ✅ | ❌ |
| **Firewall Friendly** | N/A | ✅ | ✅ | ❌ |
| **Authentication** | - | SSH keys | HTTP Basic | JMX auth |
| **Status** | ✅ Working | ✅ Working | ✅ Working | Not supported |

¹ **Why no native JMX support?** Native JMX requires the Java runtime (RMI + Java serialization). The `jmx` Rust crate exists but uses JNI (Java Native Interface), requiring JRE installation on the monitoring machine. Our SSH+JDK connector provides the same functionality in pure Rust without requiring Java on your local machine.

---

## Quick Start

### Prerequisites

- **Rust** 1.75 or later ([Install Rust](https://rustup.rs/))
- **JDK** 11+ with command-line tools (jcmd, jstat, jps)

### Installation

#### Option 1: Package Manager (Recommended)

**macOS (Homebrew):**
```bash
brew install jvm-tui
```

**Ubuntu/Debian:**
```bash
wget https://github.com/AnuragAmbuj/jvmtui/releases/latest/download/jvm-tui_latest_amd64.deb
sudo dpkg -i jvm-tui_latest_amd64.deb
```

**Fedora/RHEL:**
```bash
wget https://github.com/AnuragAmbuj/jvmtui/releases/latest/download/jvm-tui-latest.x86_64.rpm
sudo dnf install jvm-tui-latest.x86_64.rpm
```

**Arch Linux:**
```bash
# From AUR (recommended)
yay -S jvm-tui

# Or manually from PKGBUILD
git clone https://aur.archlinux.org/jvm-tui.git
cd jvm-tui
makepkg -si
```

**Other Linux distributions:**
```bash
# Download pre-built binary
wget https://github.com/AnuragAmbuj/jvmtui/releases/latest/download/jvm-tui-x86_64-unknown-linux-gnu.tar.gz
tar -xzf jvm-tui-x86_64-unknown-linux-gnu.tar.gz
sudo mv jvm-tui /usr/local/bin/
```

#### Option 2: Build from Source

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

# Use a custom config file
./target/release/jvm-tui --config /path/to/config.toml

# Connect to a specific JVM by PID
./target/release/jvm-tui --pid 12345

# Custom polling interval (default: 1s)
./target/release/jvm-tui --interval 500ms

# Show help
./target/release/jvm-tui --help
```

### Configuration

Create a `config.toml` file to save connections and preferences:

```toml
[preferences]
default_interval = "1s"
max_history_samples = 300
export_directory = "~/jvm-exports"

# Local JVM by PID
[[connections]]
type = "local"
name = "IntelliJ IDEA"
pid = 46168

# Remote JVM via SSH (no agent needed!)
[[connections]]
type = "ssh-jdk"
name = "Production API"
ssh_host = "prod-api.example.com"
ssh_user = "deploy"
ssh_key = "~/.ssh/production"
pid = 98765

# Remote JVM via Jolokia HTTP
[[connections]]
type = "jolokia"
name = "Staging Server"
url = "https://staging.example.com:8778/jolokia"
username = "monitor"
password = "${JOLOKIA_PASS}"
```

**Config file locations** (checked in order):
1. `--config <path>` CLI argument
2. `$JVM_TUI_CONFIG` environment variable
3. `./config.toml` (current directory)
4. `~/.config/jvm-tui/config.toml` (XDG config)
5. `~/.jvm-tui.toml` (home directory)

See [`config.example.toml`](config.example.toml) for full documentation.

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
| `j` / `↓` | Scroll down (Threads/Classes views) |
| `k` / `↑` | Scroll up (Threads/Classes views) |
| `/` | Search threads (Threads view) |
| `g` | Trigger garbage collection |
| `r` | Reset metrics |
| `e` | Export data |
| `?` | Show help |
| `q` | Disconnect and quit |

## Terminal Compatibility

JVM-TUI automatically adapts to your terminal's color scheme using:

- **Color::Reset** - Uses your terminal's default foreground/background colors
- **ANSI color palette** - Standard colors (Red, Green, Yellow, Cyan, etc.) that terminals automatically adapt
- **Indexed colors** - Terminal-native color codes for dimmed text

This means JVM-TUI works correctly on:
- ✅ Dark terminals (black/dark gray background)
- ✅ Light terminals (white/light gray background)  
- ✅ Custom terminal themes (Solarized, Nord, Dracula, etc.)
- ✅ macOS Terminal, iTerm2, Alacritty, Kitty, tmux, etc.

No configuration needed - it just works!

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
- Thread count by state (Runnable, Blocked, Waiting, etc.)
- Full thread list with scrolling (j/k navigation)
- Thread search functionality (press `/`)
- Stack trace display with depth info
- Color-coded thread states

### GC View
- GC event timeline (Young GC and Full GC)
- GC statistics with deltas
- Average pause time calculations
- Collection count tracking
- Throughput metrics

### Classes View
- Class histogram on demand
- Top 100 memory consumers
- Scrollable class list (j/k navigation)
- Total instances and bytes tracking
- Color-coded memory usage warnings

### Export Formats
Press `e` to export data in multiple formats:

- **JSON** - Full metrics snapshot with structured data
- **Prometheus** - Time-series metrics in Prometheus text format
  - Heap metrics: `jvm_memory_heap_used_bytes`, `jvm_memory_heap_max_bytes`
  - GC metrics: `jvm_gc_collections_total{gc="young|old"}`
  - Memory pools: `jvm_memory_pool_*_bytes{pool="..."}`
  - Thread counts: `jvm_threads_total{state="..."}`
- **CSV** - Tabular data with headers (`metric_name,value,unit,timestamp`)

Exports are saved to the configured directory (default: `~/.local/share/jvm-tui/`).

## How It Works

JVM-TUI supports **three connection methods**, all without requiring custom agents:

### Local Monitoring (JDK Tools)
1. **Discovery**: Uses `jcmd -l` (or `jps -l` as fallback) to find running JVMs
2. **Connection**: Executes JDK commands (jcmd, jstat) locally
3. **Parsing**: Parses command output into structured data
4. **Collection**: Polls metrics at configurable intervals (default: 1s)
5. **Display**: Renders live data in a terminal UI

### Remote Monitoring via SSH+JDK
1. **SSH Connection**: Connects to remote server via SSH (key or password auth)
2. **Remote Execution**: Runs `jcmd <pid>` and `jstat <pid>` on remote host
3. **Local Parsing**: Parses output locally using the same parsers
4. **Pure Rust**: No JRE required on the monitoring machine

### Remote Monitoring via Jolokia
1. **HTTP Request**: Sends JSON-RPC requests to Jolokia agent endpoint
2. **MBean Access**: Reads JMX attributes via HTTP (Memory, GC, Threading, etc.)
3. **Response Parsing**: Deserializes JSON responses
4. **Optional Auth**: Supports HTTP Basic authentication

**Architecture Decision: Why not native JMX?**

Native JMX requires Java runtime (RMI protocol + Java serialization). While the [`jmx` crate](https://crates.io/crates/jmx) exists, it uses JNI (Java Native Interface) which:
- Requires JRE installation on the monitoring machine
- Adds JNI overhead and complexity
- Is unmaintained (last update: 2020)

Our SSH+JDK connector provides the same functionality in **pure Rust** without requiring Java locally.

**Benefits:**
- ✅ No JVM agent installation (Local and SSH+JDK modes)
- ✅ No JVM restarts needed
- ✅ Works with any JVM process
- ✅ Pure Rust - no JRE dependency
- ✅ Minimal performance impact
- ✅ Safe for production use

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

### Development Status

**Phase 1 (MVP)** ✅ Complete
- JVM discovery and connection
- Real-time metrics collection  
- Overview and Memory views
- Basic TUI scaffold with tab navigation
- Async metrics polling with configurable intervals

**Phase 2 (Full Monitoring)** ✅ Complete
- Enhanced GC view with timeline and statistics
- Full thread dumps with stack traces
- Thread search functionality (press `/`)
- Class histogram view with scrolling
- Trigger GC action with confirmation
- Help overlay with keybinding reference
- Error handling and recovery
- Export features (thread dumps, metrics to JSON)
- Loading indicators and smooth scrolling
- Terminal-adaptive color system

**Phase 3 (Remote Monitoring & Configuration)** ✅ Complete
- ✅ Configuration system (TOML-based, auto-discovery)
- ✅ Saved connections with multiple connection types
- ✅ Jolokia HTTP connector for remote JVMs
- ✅ SSH+JDK connector for agent-free remote monitoring
- ✅ Enhanced export formats (JSON, Prometheus, CSV)
- ✅ Export format selector UI
- ✅ Configurable export directory

**Phase 4 (JFR Integration)** 📋 Planned
- JFR recording management
- Flight recording analysis
- Event streaming
- Custom JFR event configuration

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
- [Reqwest](https://github.com/seanmonstar/reqwest) - HTTP client (Jolokia)
- [async-ssh2-tokio](https://github.com/Miyoshi-Ryota/async-ssh2-tokio) - SSH client
- [Clap](https://github.com/clap-rs/clap) - CLI parsing
- [Serde](https://serde.rs/) - Serialization

Inspired by [VisualVM](https://visualvm.github.io/) and modern CLI tools like [htop](https://htop.dev/).

## Contact

**Anurag Ambuj**
- GitHub: [@AnuragAmbuj](https://github.com/AnuragAmbuj)
- Repository: [github.com/AnuragAmbuj/jvmtui](https://github.com/AnuragAmbuj/jvmtui)

---

<p align="center">
  <i>JVM-TUI - Because production JVMs don't have GUIs.</i>
</p>
