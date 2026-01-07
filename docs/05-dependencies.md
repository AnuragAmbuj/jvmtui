# Dependencies

This document explains all Cargo dependencies used in JVM-TUI and the rationale for each choice.

## Cargo.toml

```toml
[package]
name = "jvm-tui"
version = "0.1.0"
edition = "2024"
rust-version = "1.91.0"
description = "A beautiful TUI for JVM monitoring - like VisualVM for your terminal"
authors = ["Your Name"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/you/jvm-tui"
keywords = ["jvm", "java", "monitoring", "tui", "terminal"]
categories = ["command-line-utilities", "development-tools::profiling"]

[dependencies]
# TUI Framework
ratatui = "0.29"
crossterm = "0.28"

# Async Runtime
tokio = { version = "1.43", features = ["full", "process", "sync"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# CLI
clap = { version = "4.5", features = ["derive", "env"] }

# Error Handling
color-eyre = "0.6"
thiserror = "2.0"

# Utilities
strum = { version = "0.26", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
regex = "1.11"
once_cell = "1.20"
directories = "5.0"
humantime = "2.1"

# HTTP Client (optional)
reqwest = { version = "0.12", features = ["json"], optional = true }

[dev-dependencies]
pretty_assertions = "1.4"
tokio-test = "0.4"
insta = "1.41"

[features]
default = []
jolokia = ["reqwest"]

[[bin]]
name = "jvm-tui"
path = "src/main.rs"

[profile.release]
lto = true
strip = true
codegen-units = 1
```

## Dependency Breakdown

### TUI Framework

#### ratatui `0.29`

**Purpose:** Terminal UI rendering library

**Why this crate:**
- Most actively maintained TUI library in Rust
- Excellent widget library (Tabs, Sparkline, Gauge, Chart)
- Declarative rendering model
- Great documentation and examples

**Key features used:**
- `Tabs` - Tab navigation
- `Sparkline` - Real-time mini charts
- `Gauge` - Progress bars
- `List` - Thread/JVM lists
- `Paragraph` - Text display
- `Block` - Bordered containers

```rust
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, List, Sparkline, Tabs},
};
```

#### crossterm `0.28`

**Purpose:** Cross-platform terminal manipulation

**Why this crate:**
- Works on Windows, macOS, Linux
- Paired with ratatui
- Raw mode, alternate screen, event handling

**Key features used:**
- Raw mode for TUI
- Keyboard event capture
- Terminal size detection

```rust
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
```

### Async Runtime

#### tokio `1.43`

**Purpose:** Async runtime for concurrent operations

**Why this crate:**
- Industry standard async runtime
- Excellent subprocess support (`tokio::process`)
- Channels for inter-task communication
- Timers for polling intervals

**Features enabled:**
- `full` - All tokio features
- `process` - Async subprocess spawning
- `sync` - Channels, mutexes, etc.

```rust
use tokio::{
    process::Command,
    sync::mpsc,
    time::{interval, Duration},
};
```

### Serialization

#### serde `1.0`

**Purpose:** Serialization/deserialization framework

**Why this crate:**
- De facto standard for Rust serialization
- Derive macros for easy implementation
- Used for config files and future JSON export

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub polling_interval: Duration,
}
```

#### serde_json `1.0`

**Purpose:** JSON serialization

**Why this crate:**
- Standard JSON library
- Used for Jolokia responses (optional)
- Future: metrics export

#### toml `0.8`

**Purpose:** TOML config file parsing

**Why this crate:**
- Human-friendly config format
- Native serde support
- Standard for Rust projects

```rust
let config: Config = toml::from_str(&config_str)?;
```

### CLI

#### clap `4.5`

**Purpose:** Command-line argument parsing

**Why this crate:**
- Most popular CLI parser
- Derive macros for clean code
- Auto-generated help
- Environment variable support

**Features enabled:**
- `derive` - Derive macros
- `env` - Environment variable fallback

```rust
#[derive(Parser)]
#[command(name = "jvm-tui")]
pub struct Cli {
    #[arg(short, long)]
    pub pid: Option<u32>,
    
    #[arg(short = 'i', long, default_value = "1s")]
    pub interval: Duration,
}
```

### Error Handling

#### color-eyre `0.6`

**Purpose:** Colorful error reports with backtraces

**Why this crate:**
- Beautiful error output
- Automatic backtrace capture
- Integrates with `eyre::Result`

```rust
use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    // ...
}
```

#### thiserror `2.0`

**Purpose:** Derive macro for custom error types

**Why this crate:**
- Clean error type definitions
- Automatic `Display` implementation
- Error source chaining

```rust
#[derive(Error, Debug)]
pub enum JdkToolsError {
    #[error("jcmd not found in PATH")]
    JcmdNotFound,
    
    #[error("Failed to execute {command}: {source}")]
    ExecutionFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },
}
```

### Utilities

#### strum `0.26`

**Purpose:** Enum utilities (iteration, string conversion)

**Why this crate:**
- Iterate over enum variants
- Convert enums to/from strings
- Used for Tab enum

```rust
#[derive(Display, EnumIter, FromRepr)]
pub enum Tab {
    Overview,
    Memory,
    Threads,
    GC,
    Classes,
}

// Iterate over all tabs
for tab in Tab::iter() {
    // ...
}
```

#### chrono `0.4`

**Purpose:** Date and time handling

**Why this crate:**
- Timestamp formatting
- Duration calculations
- Uptime display

```rust
use chrono::{DateTime, Local, Duration};

let uptime = Duration::seconds(uptime_secs as i64);
let formatted = format!("{}h {}m {}s", 
    uptime.num_hours(),
    uptime.num_minutes() % 60,
    uptime.num_seconds() % 60
);
```

#### regex `1.11`

**Purpose:** Regular expression parsing

**Why this crate:**
- Standard regex library
- Used for parsing jcmd/jstat output
- Compiled regex caching with once_cell

```rust
use regex::Regex;
use once_cell::sync::Lazy;

static HEAP_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"total\s+(\d+)K,\s+used\s+(\d+)K").unwrap()
});
```

#### once_cell `1.20`

**Purpose:** Lazy static initialization

**Why this crate:**
- Compile regex patterns once
- Thread-safe lazy initialization
- Zero-cost after initialization

#### directories `5.0`

**Purpose:** Platform-specific directory paths

**Why this crate:**
- Find config directory (`~/.config/jvm-tui/`)
- Cross-platform support
- Follows OS conventions

```rust
use directories::ProjectDirs;

if let Some(dirs) = ProjectDirs::from("", "", "jvm-tui") {
    let config_path = dirs.config_dir().join("config.toml");
}
```

#### humantime `2.1`

**Purpose:** Parse human-readable durations

**Why this crate:**
- Parse "500ms", "1s", "5m" from CLI
- User-friendly duration input

```rust
use humantime::parse_duration;

let interval = parse_duration("500ms")?; // Duration
```

### Optional Dependencies

#### reqwest `0.12` (feature: `jolokia`)

**Purpose:** HTTP client for Jolokia connector

**Why this crate:**
- Most popular Rust HTTP client
- Async support with tokio
- JSON deserialization built-in

**Only included when:** `--features jolokia`

```rust
#[cfg(feature = "jolokia")]
use reqwest::Client;
```

### Dev Dependencies

#### pretty_assertions `1.4`

**Purpose:** Colored diff output in test failures

```rust
use pretty_assertions::assert_eq;

assert_eq!(parsed, expected); // Shows colored diff on failure
```

#### tokio-test `0.4`

**Purpose:** Testing utilities for async code

```rust
#[tokio::test]
async fn test_discovery() {
    let jvms = discover_local_jvms().await.unwrap();
    assert!(!jvms.is_empty());
}
```

#### insta `1.41`

**Purpose:** Snapshot testing for parsers

```rust
use insta::assert_debug_snapshot;

#[test]
fn test_parse_heap_info() {
    let output = include_str!("../../assets/sample_outputs/jcmd_heap_info.txt");
    let parsed = parse_gc_heap_info(output).unwrap();
    assert_debug_snapshot!(parsed);
}
```

## Dependency Graph

```
jvm-tui
├── ratatui ─────────────────► TUI rendering
│   └── crossterm ───────────► Terminal control
├── tokio ───────────────────► Async runtime
│   └── process ─────────────► Subprocess (jcmd/jstat)
├── clap ────────────────────► CLI parsing
├── serde + toml ────────────► Config files
├── color-eyre + thiserror ──► Error handling
├── regex + once_cell ───────► Output parsing
├── strum ───────────────────► Enum utilities
├── chrono ──────────────────► Time formatting
├── directories ─────────────► Config paths
├── humantime ───────────────► Duration parsing
└── [reqwest] ───────────────► Jolokia (optional)
```

## Version Pinning Strategy

| Category | Strategy | Example |
|----------|----------|---------|
| Major deps | Pin major.minor | `ratatui = "0.29"` |
| Stable deps | Pin major | `serde = "1.0"` |
| Tokio | Pin major.minor | `tokio = "1.43"` |

## Binary Size Optimization

```toml
[profile.release]
lto = true          # Link-time optimization
strip = true        # Strip symbols
codegen-units = 1   # Single codegen unit for better optimization
```

Expected release binary size: ~3-5 MB (depending on platform)
