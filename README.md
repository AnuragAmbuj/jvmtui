# JVM-TUI Documentation

Welcome to the JVM-TUI documentation. This is a comprehensive guide to building a beautiful, agentless Terminal User Interface for JVM monitoring.

## Overview

**JVM-TUI** is a modern, keyboard-driven Terminal User Interface for Java developers, offering feature-parity with VisualVM while being lightweight, SSH-friendly, and fully operable from the command line.

> *"JVM-TUI - Because production JVMs don't have GUIs."*

## Documentation Index

| # | Document | Description |
|---|----------|-------------|
| 1 | [Product Requirements](./01-product-requirements.md) | Complete PRD with feature matrix |
| 2 | [Architecture](./02-architecture.md) | System design and technical decisions |
| 3 | [Agentless Approach](./03-agentless-approach.md) | How we monitor JVMs without agents |
| 4 | [Project Structure](./04-project-structure.md) | Directory layout and module organization |
| 5 | [Dependencies](./05-dependencies.md) | Cargo dependencies and rationale |
| 6 | [JDK Tools Integration](./06-jdk-tools-integration.md) | Working with jcmd, jstat, jps |
| 7 | [Parsers](./07-parsers.md) | Parsing JDK tool output |
| 8 | [Connector Trait](./08-connector-trait.md) | Abstraction for JVM connections |
| 9 | [Metrics Collection](./09-metrics-collection.md) | Async polling and data storage |
| 10 | [TUI Design](./10-tui-design.md) | Ratatui layouts and views |
| 11 | [Keyboard Navigation](./11-keyboard-navigation.md) | Vim-style keybindings |
| 12 | [Thread View Design](./12-thread-view-design.md) | Thread summary with expandable stacks |
| 13 | [Configuration](./13-configuration.md) | CLI arguments and config files |
| 14 | [Implementation Phases](./14-implementation-phases.md) | Development roadmap |
| 15 | [Testing Strategy](./15-testing-strategy.md) | Test plan and fixtures |

## Quick Start

```bash
# Clone and build
git clone https://github.com/you/jvm-tui
cd jvm-tui
cargo build --release

# Run (auto-discovers local JVMs)
./target/release/jvm-tui

# Attach to specific PID
./target/release/jvm-tui --pid 12345

# Custom polling interval
./target/release/jvm-tui --interval 500ms
```

## Project Goals

| Goal | Description |
|------|-------------|
| **VisualVM Parity** | 100% feature coverage with VisualVM |
| **Agentless-First** | No JVM agent required for local monitoring |
| **Sub-second Latency** | <200ms refresh rate |
| **Keyboard-Driven** | Full vim-style navigation |
| **SSH-Friendly** | Works perfectly over SSH sessions |
| **Extensible** | Plugin system for custom dashboards |

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           JVM-TUI (Rust)                                │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌─────────────────┐  ┌───────────────────────────┐   │
│  │   Ratatui    │  │  Async Runtime  │  │    JVM Connector Layer    │   │
│  │   TUI Layer  │  │     (Tokio)     │  │                           │   │
│  └──────────────┘  └─────────────────┘  │  ┌─────────────────────┐  │   │
│                                          │  │  JdkToolsConnector │  │   │
│                                          │  │  (jcmd/jstat/jps)  │  │   │
│                                          │  │  [DEFAULT]         │  │   │
│                                          │  └─────────────────────┘  │   │
│                                          │  ┌─────────────────────┐  │   │
│                                          │  │  JolokiaConnector   │  │   │
│                                          │  │  (HTTP/JSON)        │  │   │
│                                          │  │  [OPTIONAL]         │  │   │
│                                          │  └─────────────────────┘  │   │
│                                          └───────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                              │
           ┌──────────────────┼──────────────────┐
           │                  │                  │
           ▼                  ▼                  ▼
    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
    │ Local JVM 1 │    │ Local JVM 2 │    │ Remote JVM  │
    │ (pid:12345) │    │ (pid:67890) │    │ (Jolokia)   │
    └─────────────┘    └─────────────┘    └─────────────┘
```

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| JVM Communication | Agentless (jcmd/jstat) | No deployment friction in production |
| TUI Framework | Ratatui | Best-in-class Rust TUI library |
| Async Runtime | Tokio | Industry standard, subprocess support |
| Polling Interval | Configurable (250ms-10s) | Flexibility for different use cases |
| Thread Dumps | Summary first, expand on demand | Performance + usability balance |

## Current Status

- [x] Planning & Architecture
- [x] Documentation
- [ ] Phase 1: MVP (Foundation)
- [ ] Phase 2: Full Monitoring
- [ ] Phase 3: Advanced Features (Jolokia, SSH, Plugins)

## Target Users

| User Type | Use Case |
|-----------|----------|
| **Senior Java Engineers** | Day-to-day JVM debugging and monitoring |
| **Platform Engineers** | Production JVM health checks |
| **SREs** | Incident response and troubleshooting |
| **Performance Engineers** | GC tuning and profiling |

## License

MIT OR Apache-2.0
