# Project Structure

This document describes the directory layout and module organization of JVM-TUI.

## Directory Tree

```
jvm-tui/
├── Cargo.toml                     # Package manifest
├── Cargo.lock                     # Dependency lock file
├── README.md                      # Project README
├── LICENSE-MIT                    # MIT License
├── LICENSE-APACHE                 # Apache 2.0 License
│
├── docs/                          # Documentation
│   ├── README.md                  # Docs index
│   ├── 01-product-requirements.md
│   ├── 02-architecture.md
│   ├── ...
│   └── diagrams/                  # Architecture diagrams
│
├── src/                           # Source code
│   ├── main.rs                    # Entry point
│   ├── lib.rs                     # Library root (for testing)
│   ├── app.rs                     # Application state machine
│   ├── cli.rs                     # CLI argument parsing
│   ├── config.rs                  # Configuration management
│   ├── error.rs                   # Error types
│   │
│   ├── jvm/                       # JVM communication layer
│   │   ├── mod.rs
│   │   ├── connector.rs           # JvmConnector trait
│   │   ├── discovery.rs           # JVM discovery
│   │   ├── types.rs               # Shared types
│   │   │
│   │   ├── jdk_tools/             # Agentless connector
│   │   │   ├── mod.rs
│   │   │   ├── connector.rs       # JdkToolsConnector
│   │   │   ├── detector.rs        # Tool availability check
│   │   │   ├── executor.rs        # Subprocess execution
│   │   │   └── parsers/           # Output parsers
│   │   │       ├── mod.rs
│   │   │       ├── jcmd.rs
│   │   │       ├── jstat.rs
│   │   │       └── jps.rs
│   │   │
│   │   └── jolokia/               # Optional HTTP connector
│   │       ├── mod.rs
│   │       └── connector.rs
│   │
│   ├── tui/                       # Terminal UI layer
│   │   ├── mod.rs
│   │   ├── terminal.rs            # Terminal setup/cleanup
│   │   ├── event.rs               # Event handling
│   │   │
│   │   ├── screens/               # Full-screen views
│   │   │   ├── mod.rs
│   │   │   ├── jvm_picker.rs      # JVM selection
│   │   │   ├── monitoring.rs      # Main monitoring
│   │   │   └── error.rs           # Error/setup screen
│   │   │
│   │   ├── views/                 # Tab content views
│   │   │   ├── mod.rs
│   │   │   ├── overview.rs        # Dashboard
│   │   │   ├── memory.rs          # Memory details
│   │   │   ├── threads.rs         # Thread view
│   │   │   ├── gc.rs              # GC statistics
│   │   │   └── classes.rs         # Class loading
│   │   │
│   │   └── widgets/               # Reusable UI components
│   │       ├── mod.rs
│   │       ├── sparkline_panel.rs
│   │       ├── memory_gauge.rs
│   │       ├── thread_table.rs
│   │       ├── stat_card.rs
│   │       └── help_footer.rs
│   │
│   └── metrics/                   # Metrics collection & storage
│       ├── mod.rs
│       ├── collector.rs           # Async collector
│       ├── store.rs               # MetricsStore
│       └── ring_buffer.rs         # Fixed-size history
│
├── tests/                         # Integration tests
│   ├── parsers/
│   │   ├── jcmd_tests.rs
│   │   └── jstat_tests.rs
│   └── integration/
│       └── discovery_tests.rs
│
└── assets/                        # Test fixtures & resources
    └── sample_outputs/
        ├── jcmd_heap_info.txt
        ├── jcmd_thread_print.txt
        ├── jstat_gcutil.txt
        └── jps_output.txt
```

## Module Descriptions

### Root Modules

| Module | Purpose |
|--------|---------|
| `main.rs` | Entry point, CLI parsing, app bootstrap |
| `lib.rs` | Library root for integration testing |
| `app.rs` | Main `App` struct, state machine, screen transitions |
| `cli.rs` | Clap-based CLI argument definitions |
| `config.rs` | Config file loading, merging with CLI args |
| `error.rs` | `AppError` enum, error handling utilities |

### JVM Module (`src/jvm/`)

```rust
// mod.rs - Public exports
pub mod connector;
pub mod discovery;
pub mod types;
pub mod jdk_tools;

#[cfg(feature = "jolokia")]
pub mod jolokia;
```

| File | Purpose |
|------|---------|
| `connector.rs` | `JvmConnector` trait definition |
| `discovery.rs` | `discover_local_jvms()` function |
| `types.rs` | `HeapInfo`, `GcStats`, `ThreadInfo`, etc. |

#### JDK Tools Submodule (`src/jvm/jdk_tools/`)

| File | Purpose |
|------|---------|
| `connector.rs` | `JdkToolsConnector` implementation |
| `detector.rs` | `JdkToolsStatus::detect()` |
| `executor.rs` | Subprocess spawning with timeout |
| `parsers/jcmd.rs` | Parse jcmd output (heap_info, thread_print, etc.) |
| `parsers/jstat.rs` | Parse jstat output (gcutil, gc, class) |
| `parsers/jps.rs` | Parse jps output (fallback discovery) |

### TUI Module (`src/tui/`)

```rust
// mod.rs - Public exports
pub mod terminal;
pub mod event;
pub mod screens;
pub mod views;
pub mod widgets;
```

| File | Purpose |
|------|---------|
| `terminal.rs` | `setup_terminal()`, `restore_terminal()` |
| `event.rs` | `Event` enum, `EventHandler`, key mapping |

#### Screens (`src/tui/screens/`)

| File | Purpose |
|------|---------|
| `jvm_picker.rs` | JVM selection list with discovery |
| `monitoring.rs` | Main monitoring screen with tabs |
| `error.rs` | Error display, setup instructions |

#### Views (`src/tui/views/`)

| File | Purpose |
|------|---------|
| `overview.rs` | Dashboard with sparklines, key stats |
| `memory.rs` | Heap breakdown, metaspace, memory pools |
| `threads.rs` | Thread list with expandable stacks |
| `gc.rs` | GC statistics, event timeline |
| `classes.rs` | Class loading stats, histogram |

#### Widgets (`src/tui/widgets/`)

| File | Purpose |
|------|---------|
| `sparkline_panel.rs` | Sparkline with title, value, unit |
| `memory_gauge.rs` | Horizontal bar with segments |
| `thread_table.rs` | Thread list with state colors |
| `stat_card.rs` | Single stat display (value + label) |
| `help_footer.rs` | Context-aware keybinding hints |

### Metrics Module (`src/metrics/`)

| File | Purpose |
|------|---------|
| `collector.rs` | `MetricsCollector` - async polling loop |
| `store.rs` | `MetricsStore` - holds all metric buffers |
| `ring_buffer.rs` | `RingBuffer<T>` - fixed-size circular buffer |

## File Size Guidelines

| File Type | Target Lines | Max Lines |
|-----------|--------------|-----------|
| Module (mod.rs) | 20-50 | 100 |
| Data types | 50-150 | 300 |
| Parser | 100-200 | 400 |
| View | 150-300 | 500 |
| Widget | 50-150 | 250 |

## Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| Files | snake_case | `jdk_tools.rs` |
| Modules | snake_case | `mod jdk_tools` |
| Structs | PascalCase | `HeapInfo` |
| Traits | PascalCase | `JvmConnector` |
| Functions | snake_case | `parse_heap_info()` |
| Constants | SCREAMING_SNAKE | `DEFAULT_INTERVAL` |
| Type aliases | PascalCase | `Result<T> = std::result::Result<T, AppError>` |

## Import Organization

```rust
// 1. Standard library
use std::collections::HashMap;
use std::time::Duration;

// 2. External crates
use color_eyre::Result;
use ratatui::prelude::*;
use tokio::sync::mpsc;

// 3. Internal modules (absolute)
use crate::jvm::connector::JvmConnector;
use crate::metrics::store::MetricsStore;

// 4. Internal modules (relative)
use super::widgets::SparklinePanel;
```

## Feature Flags

```toml
[features]
default = []
jolokia = ["reqwest"]  # Enable Jolokia HTTP connector
```

```rust
// Conditional compilation
#[cfg(feature = "jolokia")]
pub mod jolokia;

#[cfg(feature = "jolokia")]
use jolokia::JolokiaConnector;
```

## Test Organization

```
tests/
├── parsers/                    # Unit tests for parsers
│   ├── jcmd_tests.rs          # Test jcmd parsing
│   └── jstat_tests.rs         # Test jstat parsing
│
└── integration/                # Integration tests
    └── discovery_tests.rs     # Test JVM discovery (requires JVM)
```

### Test File Naming

| Test Type | Location | Naming |
|-----------|----------|--------|
| Unit tests | Same file as code | `#[cfg(test)] mod tests` |
| Integration | `tests/` directory | `*_tests.rs` |
| Fixtures | `assets/sample_outputs/` | Descriptive name |

## Assets Directory

```
assets/
└── sample_outputs/
    ├── jcmd_heap_info.txt       # Sample GC.heap_info output
    ├── jcmd_heap_info_g1.txt    # G1GC specific
    ├── jcmd_heap_info_zgc.txt   # ZGC specific
    ├── jcmd_thread_print.txt    # Sample Thread.print output
    ├── jcmd_vm_version.txt      # Sample VM.version output
    ├── jstat_gcutil.txt         # Sample -gcutil output
    ├── jstat_gc.txt             # Sample -gc output
    └── jps_output.txt           # Sample jps -l output
```

These fixtures are used for:
- Parser unit tests
- Snapshot testing (insta)
- Documentation examples
