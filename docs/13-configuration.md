# Configuration

This document describes the configuration system for JVM-TUI.

## Configuration Sources

Configuration is loaded from multiple sources with the following priority (highest to lowest):

1. **Command-line arguments** (highest priority)
2. **Environment variables**
3. **Config file** (`~/.config/jvm-tui/config.toml`)
4. **Default values** (lowest priority)

## Command-Line Arguments

```bash
$ jvm-tui --help

jvm-tui - A beautiful TUI for JVM monitoring

Usage: jvm-tui [OPTIONS]

Options:
  -p, --pid <PID>           PID of JVM to attach to (skip picker)
  -i, --interval <DURATION> Polling interval [default: 1s]
  -c, --config <PATH>       Path to config file
      --jolokia-url <URL>   Jolokia URL for remote JVM
      --debug               Enable debug output
  -h, --help                Print help
  -V, --version             Print version

Examples:
  jvm-tui                    # Auto-discover and pick JVM
  jvm-tui -p 12345           # Attach to specific PID
  jvm-tui -i 500ms           # Poll every 500ms
  jvm-tui --jolokia-url http://localhost:8778/jolokia
```

### Implementation

```rust
// src/cli.rs

use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "jvm-tui")]
#[command(about = "A beautiful TUI for JVM monitoring")]
#[command(version)]
#[command(after_help = "Examples:
  jvm-tui                    # Auto-discover and pick JVM
  jvm-tui -p 12345           # Attach to specific PID
  jvm-tui -i 500ms           # Poll every 500ms")]
pub struct Cli {
    /// PID of JVM to attach to (skip picker if provided)
    #[arg(short, long, value_name = "PID")]
    pub pid: Option<u32>,
    
    /// Polling interval for metrics collection
    #[arg(short, long, default_value = "1s", value_parser = parse_duration)]
    pub interval: Duration,
    
    /// Path to config file
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    
    /// Jolokia URL for remote JVM (requires --features jolokia)
    #[arg(long, value_name = "URL")]
    pub jolokia_url: Option<String>,
    
    /// Enable debug output
    #[arg(long, default_value = "false")]
    pub debug: bool,
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    humantime::parse_duration(s).map_err(|e| e.to_string())
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
```

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `JVM_TUI_INTERVAL` | Polling interval | `500ms` |
| `JVM_TUI_CONFIG` | Config file path | `~/.config/jvm-tui/config.toml` |
| `JVM_TUI_DEBUG` | Enable debug mode | `true` |
| `JAVA_HOME` | JDK installation path | `/usr/lib/jvm/java-21` |

```rust
// Environment variable support in clap
#[arg(short, long, env = "JVM_TUI_INTERVAL", default_value = "1s")]
pub interval: Duration,
```

## Config File

### Location

Default location follows XDG Base Directory specification:

- **Linux/macOS**: `~/.config/jvm-tui/config.toml`
- **Windows**: `%APPDATA%\jvm-tui\config.toml`

```rust
use directories::ProjectDirs;

fn config_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "jvm-tui")
        .map(|dirs| dirs.config_dir().join("config.toml"))
}
```

### Format

```toml
# JVM-TUI Configuration
# ~/.config/jvm-tui/config.toml

# ─────────────────────────────────────────────────────────────────
# Polling Settings
# ─────────────────────────────────────────────────────────────────

[polling]
# How often to collect metrics (250ms to 10s)
interval = "1s"

# How many data points to keep in history
# At 1s interval, 300 = 5 minutes of history
history_size = 300

# Timeout for individual JDK tool commands
command_timeout = "5s"

# ─────────────────────────────────────────────────────────────────
# Display Settings
# ─────────────────────────────────────────────────────────────────

[display]
# Show memory values in absolute terms (MB)
show_absolute_memory = true

# Show memory values as percentages
show_percentage = true

# Height of sparkline charts (in lines)
sparkline_height = 3

# Default number of stack frames to show (collapsed)
thread_stack_preview_depth = 1

# Maximum stack frames when expanded
thread_stack_max_depth = 50

# ─────────────────────────────────────────────────────────────────
# Saved JVM Connections
# ─────────────────────────────────────────────────────────────────

[[connections]]
name = "My App (Local)"
type = "local"
pid = 12345

[[connections]]
name = "Production Server"
type = "jolokia"
url = "http://prod-server:8778/jolokia"

# ─────────────────────────────────────────────────────────────────
# Keybindings (Future)
# ─────────────────────────────────────────────────────────────────

# [keybindings]
# quit = "q"
# refresh = "r"
# help = "?"
```

### Config Structs

```rust
// src/config.rs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub polling: PollingConfig,
    pub display: DisplayConfig,
    #[serde(default)]
    pub connections: Vec<SavedConnection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PollingConfig {
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
    pub history_size: usize,
    #[serde(with = "humantime_serde")]
    pub command_timeout: Duration,
}

impl Default for PollingConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(1),
            history_size: 300,
            command_timeout: Duration::from_secs(5),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplayConfig {
    pub show_absolute_memory: bool,
    pub show_percentage: bool,
    pub sparkline_height: u16,
    pub thread_stack_preview_depth: usize,
    pub thread_stack_max_depth: usize,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_absolute_memory: true,
            show_percentage: true,
            sparkline_height: 3,
            thread_stack_preview_depth: 1,
            thread_stack_max_depth: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConnection {
    pub name: String,
    #[serde(flatten)]
    pub connection_type: ConnectionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ConnectionType {
    Local { pid: u32 },
    Jolokia { url: String },
}

impl Default for Config {
    fn default() -> Self {
        Self {
            polling: PollingConfig::default(),
            display: DisplayConfig::default(),
            connections: Vec::new(),
        }
    }
}
```

### Loading Config

```rust
// src/config.rs

use std::fs;
use color_eyre::Result;

impl Config {
    /// Load config from default location
    pub fn load() -> Result<Self> {
        if let Some(path) = config_path() {
            if path.exists() {
                return Self::load_from(&path);
            }
        }
        Ok(Self::default())
    }
    
    /// Load config from specific path
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save config to default location
    pub fn save(&self) -> Result<()> {
        if let Some(path) = config_path() {
            // Create parent directory if needed
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            let content = toml::to_string_pretty(self)?;
            fs::write(path, content)?;
        }
        Ok(())
    }
    
    /// Merge CLI args into config (CLI takes priority)
    pub fn merge_cli(&mut self, cli: &Cli) {
        // CLI interval overrides config
        self.polling.interval = cli.interval;
        
        // Clamp interval to valid range
        self.polling.interval = self.polling.interval.clamp(
            Duration::from_millis(250),
            Duration::from_secs(10),
        );
    }
}
```

## Runtime Configuration

Some settings can be changed at runtime:

```rust
// src/app.rs

impl App {
    /// Change polling interval at runtime
    pub fn set_polling_interval(&mut self, interval: Duration) {
        let interval = interval.clamp(
            Duration::from_millis(250),
            Duration::from_secs(10),
        );
        self.config.polling.interval = interval;
        
        // Notify collector of new interval
        if let Some(ref handle) = self.collector_handle {
            handle.set_interval(interval);
        }
    }
    
    /// Toggle display setting
    pub fn toggle_absolute_memory(&mut self) {
        self.config.display.show_absolute_memory = 
            !self.config.display.show_absolute_memory;
    }
}
```

## Validation

```rust
impl Config {
    /// Validate configuration values
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Interval bounds
        if self.polling.interval < Duration::from_millis(250) {
            return Err(ConfigError::IntervalTooSmall);
        }
        if self.polling.interval > Duration::from_secs(10) {
            return Err(ConfigError::IntervalTooLarge);
        }
        
        // History size
        if self.polling.history_size == 0 {
            return Err(ConfigError::InvalidHistorySize);
        }
        if self.polling.history_size > 10000 {
            return Err(ConfigError::HistorySizeTooLarge);
        }
        
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Polling interval must be at least 250ms")]
    IntervalTooSmall,
    
    #[error("Polling interval must be at most 10s")]
    IntervalTooLarge,
    
    #[error("History size must be greater than 0")]
    InvalidHistorySize,
    
    #[error("History size must be at most 10000")]
    HistorySizeTooLarge,
}
```

## First-Run Experience

Create default config on first run:

```rust
pub fn ensure_config_exists() -> Result<()> {
    if let Some(path) = config_path() {
        if !path.exists() {
            // Create directory
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Write default config with comments
            let default_config = include_str!("../assets/default_config.toml");
            fs::write(&path, default_config)?;
            
            eprintln!("Created default config at: {}", path.display());
        }
    }
    Ok(())
}
```

## Example Usage

```rust
// src/main.rs

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    
    // Parse CLI
    let cli = Cli::parse_args();
    
    // Load config
    let mut config = if let Some(ref path) = cli.config {
        Config::load_from(path)?
    } else {
        Config::load()?
    };
    
    // Merge CLI args (CLI takes priority)
    config.merge_cli(&cli);
    
    // Validate
    config.validate()?;
    
    // Run app
    let app = App::new(config, cli)?;
    app.run().await
}
```
