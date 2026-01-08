# Implementation Phases

This document describes the phased implementation plan for JVM-TUI.

## Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Implementation Timeline                       │
└─────────────────────────────────────────────────────────────────┘

Phase 1: MVP Foundation          Phase 2: Full Monitoring         Phase 3: Advanced
    (2-3 weeks)                       (2-3 weeks)                   (2-3 weeks)
        │                                 │                             │
        ▼                                 ▼                             ▼
┌───────────────────┐           ┌───────────────────┐         ┌───────────────────┐
│ • Project setup   │           │ • GC deep-dive    │         │ • Jolokia support │
│ • JDK detection   │           │ • Class histogram │         │ • SSH tunnels     │
│ • JVM discovery   │           │ • Full thread dump│         │ • Config file     │
│ • Basic parsers   │           │ • Trigger GC      │         │ • JFR integration │
│ • TUI scaffold    │           │ • Help overlay    │         │ • Export features │
│ • Overview view   │           │ • Error handling  │         │ • Plugin system   │
│ • Memory view     │           │ • Polish & UX     │         │                   │
│ • Thread summary  │           │                   │         │                   │
└───────────────────┘           └───────────────────┘         └───────────────────┘
```

## Phase 1: MVP Foundation (Weeks 1-3)

### Goals
- Functional application that can monitor a local JVM
- Core metrics visible: heap, GC, threads
- Keyboard navigation working

### Checklist

#### 1.1 Project Setup
- [ ] Create Cargo.toml with all dependencies
- [ ] Set up directory structure
- [ ] Configure rustfmt and clippy
- [ ] Create README.md with basic usage
- [ ] Set up CI/CD (GitHub Actions)

**Files created:**
```
Cargo.toml
src/main.rs
src/lib.rs
src/error.rs
.github/workflows/ci.yml
```

#### 1.2 JDK Tools Detection
- [ ] Implement tool detection (`jcmd`, `jstat`, `jps`)
- [ ] Check `JAVA_HOME` and `PATH`
- [ ] Generate installation guidance
- [ ] Graceful degradation for missing tools

**Files created:**
```
src/jvm/mod.rs
src/jvm/jdk_tools/mod.rs
src/jvm/jdk_tools/detector.rs
```

**Acceptance criteria:**
- Detects presence/absence of each tool
- Shows platform-specific install instructions
- Works on macOS, Linux, Windows

#### 1.3 JVM Discovery
- [ ] Parse `jcmd -l` output
- [ ] Fallback to `jps -l`
- [ ] Filter out JDK tools themselves
- [ ] Extract display-friendly names

**Files created:**
```
src/jvm/discovery.rs
src/jvm/jdk_tools/parsers/jps.rs
```

**Acceptance criteria:**
- Lists all running JVMs
- Shows PID and main class
- Filters jcmd/jps from results

#### 1.4 JVM Picker TUI
- [ ] Create JVM selection screen
- [ ] Implement list navigation (j/k)
- [ ] Connect on Enter
- [ ] Refresh on 'r'
- [ ] Handle empty state

**Files created:**
```
src/tui/mod.rs
src/tui/terminal.rs
src/tui/screens/mod.rs
src/tui/screens/jvm_picker.rs
```

**Acceptance criteria:**
- Shows discovered JVMs in a list
- Vim-style navigation works
- Selected JVM highlighted
- Can connect to selected JVM

#### 1.5 Core Parsers
- [ ] `jstat -gcutil` parser
- [ ] `jcmd GC.heap_info` parser
- [ ] `jcmd VM.version` parser
- [ ] `jcmd VM.uptime` parser
- [ ] `jcmd VM.flags` parser

**Files created:**
```
src/jvm/jdk_tools/parsers/mod.rs
src/jvm/jdk_tools/parsers/jstat.rs
src/jvm/jdk_tools/parsers/jcmd.rs
```

**Acceptance criteria:**
- All parsers have unit tests
- Handle malformed input gracefully
- Support G1, ZGC, Parallel GC output formats

#### 1.6 Connector Implementation
- [ ] Define `JvmConnector` trait
- [ ] Implement `JdkToolsConnector`
- [ ] Add subprocess executor with timeout
- [ ] Implement caching for static info

**Files created:**
```
src/jvm/connector.rs
src/jvm/types.rs
src/jvm/jdk_tools/connector.rs
src/jvm/jdk_tools/executor.rs
```

**Acceptance criteria:**
- All trait methods implemented
- Timeout protection on all commands
- Static info (version, flags) cached

#### 1.7 Metrics Collection
- [ ] Implement ring buffer
- [ ] Create MetricsStore
- [ ] Implement async collector
- [ ] Add configurable polling interval

**Files created:**
```
src/metrics/mod.rs
src/metrics/ring_buffer.rs
src/metrics/store.rs
src/metrics/collector.rs
```

**Acceptance criteria:**
- Configurable interval (250ms-10s)
- History retained (default 300 samples)
- Parallel metric collection
- Graceful handling of disconnection

#### 1.8 Main TUI Scaffold
- [ ] Create monitoring screen
- [ ] Implement tab bar
- [ ] Add header with JVM info
- [ ] Add footer with keybindings
- [ ] Wire up tab switching

**Files created:**
```
src/tui/screens/monitoring.rs
src/tui/event.rs
src/app.rs
```

**Acceptance criteria:**
- Tabs display and switch (1-5, h/l)
- Header shows JVM info
- Footer shows context-aware hints
- Smooth navigation

#### 1.9 Overview Dashboard
- [ ] Heap sparkline with value
- [ ] GC summary (counts, times)
- [ ] Thread summary (counts by state)
- [ ] Memory pool gauges

**Files created:**
```
src/tui/views/mod.rs
src/tui/views/overview.rs
src/tui/widgets/mod.rs
src/tui/widgets/sparkline_panel.rs
src/tui/widgets/stat_card.rs
```

**Acceptance criteria:**
- All widgets render correctly
- Data updates in real-time
- Responsive to terminal size

#### 1.10 Memory View
- [ ] Heap usage sparkline (larger)
- [ ] Heap breakdown bars (Eden, Old, etc.)
- [ ] Metaspace stats
- [ ] Class space stats

**Files created:**
```
src/tui/views/memory.rs
src/tui/widgets/memory_gauge.rs
```

#### 1.11 Thread Summary View
- [ ] Thread state summary
- [ ] Thread list with states
- [ ] Stack preview (1 frame)
- [ ] Expand/collapse (Enter)

**Files created:**
```
src/tui/views/threads.rs
src/tui/widgets/thread_table.rs
```

**Acceptance criteria:**
- Shows thread count by state
- Color-coded state symbols
- Single frame preview
- Expansion toggles

#### 1.12 CLI & Basic Config
- [ ] Implement CLI with clap
- [ ] Add --pid option
- [ ] Add --interval option
- [ ] Add --help

**Files created:**
```
src/cli.rs
```

### Phase 1 Deliverable
A working MVP that can:
- Auto-discover local JVMs
- Connect to a selected JVM
- Display real-time heap, GC, and thread metrics
- Navigate with keyboard

---

## Phase 2: Full Monitoring (Weeks 4-6) ✅ COMPLETE

### Goals
- Feature parity with basic VisualVM functionality
- Polished error handling and UX
- All views complete

### Checklist

#### 2.1 GC Deep-Dive View ✅
- [x] GC event breakdown (Young/Full/Concurrent)
- [x] Average pause times
- [x] Throughput calculation
- [x] GC timeline visualization

**Files created:**
```
src/tui/views/gc.rs (212 lines)
```

#### 2.2 Class Loading View ✅
- [x] Loaded/Unloaded counts
- [x] Class histogram (on demand)
- [x] Top memory consumers
- [x] Scrollable table

**Files created:**
```
src/tui/views/classes.rs (123 lines)
Class histogram parsing in src/jvm/jdk_tools/parsers/jcmd.rs
```

#### 2.3 Full Thread Dump ✅
- [x] Parse complete thread dump
- [x] Stack trace display with depth
- [x] Thread search (/) with n/N navigation
- [x] Export to file

**Implementation:**
- Thread dump parsing in jcmd.rs
- Search functionality in threads.rs
- Export in export.rs

#### 2.4 Actions ✅
- [x] Trigger GC with confirmation dialog
- [x] Request thread dump (automatic)
- [x] Request class histogram (automatic)
- [x] Export data with confirmation

**Files created:**
```
src/tui/widgets/confirmation_dialog.rs
src/export.rs
```

#### 2.5 Help Overlay ✅
- [x] Full keybinding reference
- [x] Context-sensitive sections
- [x] Toggle with '?'

**Files created:**
```
src/tui/widgets/help_overlay.rs (200 lines)
```

#### 2.6 Error Handling ✅
- [x] JVM disconnection recovery with 'r' key
- [x] Tool execution errors with user-friendly messages
- [x] Timeout handling in connector
- [x] Error screen widget

**Files created:**
```
src/tui/widgets/error_screen.rs
src/tui/widgets/loading_screen.rs
```

#### 2.7 Polish ✅
- [x] Responsive layouts
- [x] Loading indicators
- [x] Smooth scrolling (j/k navigation)
- [x] Visual consistency with terminal-adaptive theme
- [x] Search bar widget
- [x] Export success notifications

**Files created:**
```
src/tui/widgets/search_bar.rs
src/theme.rs (terminal-adaptive colors)
```

### Phase 2 Deliverable ✅ DELIVERED
A polished monitoring tool with:
- Complete GC analysis with timeline
- Class histogram with scrolling
- Full thread dumps with search
- Action support (trigger GC, export data)
- Comprehensive help system
- Error recovery and loading states
- Terminal-adaptive colors for any terminal theme

**Commits:**
- `c428aca` - feat: Complete Phase 2 - Enhanced UX, error handling, and export features
- `d2d1e00` - feat: Add thread search functionality - Complete all Phase 2 tasks
- `46313f4` - feat: Add terminal-adaptive color system for universal compatibility

---

## Phase 3: Remote Monitoring & Configuration (Weeks 7-9)

### Goals
- Remote JVM support via Jolokia
- Configuration persistence
- Enhanced export capabilities
- SSH tunnel support for secure access

**Note**: JFR integration moved to Phase 4 for focused implementation

### Checklist

#### 3.1 Configuration File System
- [ ] TOML config loading from XDG/home directories
- [ ] Saved connection profiles (local, Jolokia, SSH)
- [ ] Persistent user preferences
- [ ] Config validation and defaults
- [ ] CLI arg overrides

**Files to create:**
```
src/config.rs (~150 lines)
config.example.toml (example config)
docs/configuration.md (config guide)
```

#### 3.2 Enhanced Export Features
- [ ] Export metrics to JSON ✅ (already done)
- [ ] Export thread dump to file ✅ (already done)
- [ ] Prometheus format export
- [ ] CSV export for metrics
- [ ] Configurable export paths
- [ ] Export format selection UI

**Files to modify:**
```
src/export.rs (add Prometheus and CSV exporters)
src/tui/widgets/confirmation_dialog.rs (format selection)
```

#### 3.3 Jolokia Connector (Remote JVMs)
- [ ] Add HTTP client dependencies (reqwest)
- [ ] Define Jolokia request/response types
- [ ] Implement JolokiaConnector trait
- [ ] Map JMX MBeans to metrics
- [ ] Handle basic authentication
- [ ] Parse Jolokia JSON responses
- [ ] Add to connection picker UI

**Files to create:**
```
src/jvm/jolokia/mod.rs
src/jvm/jolokia/types.rs (~200 lines)
src/jvm/jolokia/connector.rs (~400 lines)
src/jvm/jolokia/parsers.rs (~300 lines)
```

#### 3.4 SSH Tunnel Support
- [ ] Add SSH library dependencies (async-ssh2-tokio or thrussh)
- [ ] Implement SSH tunnel manager
- [ ] Support password authentication
- [ ] Support key-based authentication
- [ ] Port forwarding setup
- [ ] Integrate with JolokiaConnector
- [ ] Tunnel lifecycle management
- [ ] Add to connection picker UI

**Files to create:**
```
src/jvm/ssh/mod.rs
src/jvm/ssh/tunnel.rs (~300 lines)
src/jvm/jolokia/tunneled_connector.rs (~200 lines)
```

### Phase 3 Deliverable
A production-ready tool with:
- Local JVM monitoring (Phase 1 & 2)
- Remote JVM monitoring via Jolokia
- SSH tunnel support for secure remote access
- Persistent configuration with saved connections
- Enhanced export formats (JSON, Prometheus, CSV)

**Detailed plan**: See `docs/phase3-implementation-plan.md`

---

## Phase 4: JFR Integration & Advanced Features (Future)

### Goals
- Java Flight Recorder integration
- Advanced profiling capabilities
- Historical data analysis

### Checklist

#### 4.1 JFR Integration
- [ ] Start/stop JFR recording via JMX
- [ ] Download JFR files from remote JVMs
- [ ] Parse JFR file format
- [ ] Basic event viewer
- [ ] Recording management UI

**Files to create:**
```
src/jvm/jfr/mod.rs
src/jvm/jfr/recorder.rs
src/jvm/jfr/parser.rs
src/tui/views/jfr.rs
```

#### 4.2 Advanced Features (Future)
- [ ] Multi-JVM comparison view
- [ ] Historical data persistence
- [ ] Alerting system integration
- [ ] Custom MBean queries
- [ ] Plugin system for custom dashboards

### Phase 4 Deliverable
A comprehensive JVM monitoring and profiling tool with:
- All Phase 1-3 features
- JFR recording and analysis
- Advanced profiling capabilities
- Historical trend analysis

---

## Definition of Done

Each task is considered done when:

1. **Code complete**: All functionality implemented
2. **Tests passing**: Unit tests for parsers, integration tests where applicable
3. **Documentation**: Code comments, README updated
4. **No clippy warnings**: `cargo clippy` passes
5. **Formatted**: `cargo fmt` applied
6. **Manual testing**: Works on macOS/Linux with real JVMs

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| JDK tool output format changes | Extensive test fixtures, defensive parsing |
| Performance on many threads | Lazy loading, virtualized lists |
| Terminal compatibility | Test on iTerm2, Terminal.app, kitty, alacritty |
| Windows support | Use crossterm, avoid Unix-specific code |

## Success Metrics

| Metric | Target |
|--------|--------|
| Time to first working build | < 1 hour |
| Startup time | < 500ms |
| Memory usage | < 50MB |
| Attach time | < 1s |
| Refresh latency | < 200ms |

## Post-MVP Roadmap

After Phase 3:
- Plugin system for custom dashboards
- Multi-JVM comparison view
- Historical data persistence
- Integration with alerting systems
- Custom MBean queries
