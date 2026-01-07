# Product Requirements Document: JVM-TUI

> A Beautiful Terminal UI for JVM Observability

## 1. Product Vision

JVM-TUI is a modern, beautiful, keyboard-driven Terminal User Interface (TUI) for Java developers, offering feature-parity with VisualVM, while being:

- **Lightweight & fast**
- **SSH-friendly** (remote prod access)
- **Scriptable & automatable**
- **Opinionated for JVM engineers**

Think **VisualVM + htop + IntelliJ ergonomics**, all inside your terminal.

---

## 2. Target Users

### Primary
- Senior Java / JVM engineers
- Platform & Infra engineers
- SREs managing JVM workloads

### Secondary
- Engineering managers (high-level JVM health)
- Performance engineers

---

## 3. Core Goals (Non-Negotiable)

1. **100% VisualVM feature coverage**
2. **Sub-second refresh latency**
3. **Zero-config attach to local JVM**
4. **Secure remote JVM support**
5. **Fully keyboard-driven UX**
6. **Extensible via plugins**

---

## 4. Non-Goals

- Replacing full profilers (YourKit, async-profiler)
- GUI replacement for beginners
- JVM language IDE

---

## 5. Feature Parity Matrix (VisualVM → JVM-TUI)

### 5.1 JVM Discovery & Attach

| Feature | Description |
|---------|-------------|
| Local JVM discovery | List all JVMs via jcmd/jps |
| PID attach | Attach via PID |
| JMX attach | Attach via JMX URL (Phase 3) |
| Remote JVM | SSH tunnel + JMX (Phase 3) |
| Saved connections | Named JVM profiles |

### 5.2 Overview Dashboard

**Live JVM snapshot:**
- JVM name, version, vendor
- Start time, uptime
- JVM arguments
- System properties
- OS metrics

**TUI UX:**
- Top panel: JVM identity
- Middle: real-time charts
- Bottom: alerts & GC events

### 5.3 CPU Monitoring

| Feature | Details |
|---------|---------|
| Process CPU | % CPU usage |
| Per-thread CPU | Thread-level CPU |
| Sampling view | Top CPU threads |
| CPU history | Time-series graph |

**UX:** Flame-bar inspired TUI graphs, sort threads by CPU

### 5.4 Memory Monitoring

| Feature | Details |
|---------|---------|
| Heap usage | Used / Committed / Max |
| Non-heap | Metaspace, CodeCache |
| Pool breakdown | Eden, Survivor, Old |
| Allocation rate | Bytes/sec |
| Live objects | Approx counts |

**Heap Visualizations:**
- ASCII sparklines
- GC overlay markers

### 5.5 Garbage Collection (GC)

| Feature | Details |
|---------|---------|
| GC algorithm | G1, ZGC, Shenandoah |
| GC events | Minor / Major |
| Pause times | Histogram |
| Throughput | % time in GC |
| Allocation failure | Detection |

**Advanced:**
- Region-level stats (G1)
- Concurrent phase timelines

### 5.6 Thread Dump Analysis

| Feature | Details |
|---------|---------|
| Live thread dump | On-demand |
| Auto refresh | Interval based |
| Thread states | RUNNABLE, BLOCKED |
| Deadlock detection | Automatic |
| Thread grouping | By state / pool |

**UX:**
- Summary view first, full stack on demand
- Expandable stack traces
- Syntax-highlighted stacks
- Search & filter

### 5.7 Class & Classloader View

| Feature | Details |
|---------|---------|
| Loaded classes | Count |
| Unloaded classes | Count |
| Per-classloader stats | Size & count |
| Class histogram | Top consumers |

### 5.8 Profiling (Sampling)

| Feature | Details |
|---------|---------|
| CPU sampling | Low overhead |
| Memory sampling | Allocation hotspots |
| Stack aggregation | Tree view |
| Method ranking | Hot paths |

*Note: Uses JFR / JVMTI sampling, not instrumentation.*

### 5.9 Heap Dump

| Feature | Details |
|---------|---------|
| Heap dump trigger | On demand |
| Remote dump | Stream to local |
| Histogram view | Object sizes |
| Dominator tree | Textual |

### 5.10 JVM MBeans Browser

| Feature | Details |
|---------|---------|
| Full MBean tree | JMX |
| Attribute read | Live |
| Operation invoke | With params |
| Notifications | Subscribe |

### 5.11 JFR Integration (Bonus over VisualVM)

| Feature | Details |
|---------|---------|
| Start/Stop JFR | Runtime |
| Event streaming | Live |
| TUI event viewer | GC, CPU, IO |

---

## 6. TUI UX Design Principles

### Layout

```
┌─ JVM-TUI ─────────────────────────────────────────┐
│ PID 1234 │ Java 21 │ G1GC │ Uptime: 2h 34m       │
├───────────────────────────────────────────────────┤
│ CPU ▂▃▅▆█   Heap ▃▆█   GC ▂▃▁                     │
├───────────────────────────────────────────────────┤
│ [Overview] Memory  Threads  GC  Classes           │
├───────────────────────────────────────────────────┤
│ Detail Pane (context aware)                       │
└───────────────────────────────────────────────────┘
```

### Interaction
- Vim-like keybindings
- Mouse optional
- Modal navigation

---

## 7. Keyboard Shortcuts

| Action | Key |
|--------|-----|
| Switch tab | 1-9 |
| Next/Prev tab | h/l |
| Scroll up/down | j/k |
| Refresh | r |
| Thread dump | t |
| Trigger GC | Ctrl+g |
| Search | / |
| Help | ? |
| Quit | q |

---

## 8. Architecture (High-Level)

### 8.1 Components
- **Attach Engine** (jcmd/jstat subprocess)
- **Metrics Collector** (async polling)
- **TUI Renderer** (Ratatui)
- **Plugin System** (Phase 3)

### 8.2 Technology Choices

| Layer | Choice |
|-------|--------|
| Language | Rust |
| TUI | Ratatui |
| Async | Tokio |
| JVM APIs | jcmd, jstat, jps (agentless) |
| Optional | Jolokia (HTTP/JSON) |

---

## 9. Security

- Read-only mode by default
- Explicit destructive action confirmation
- JMX TLS support (Phase 3)
- SSH key based auth (Phase 3)

---

## 10. Performance Targets

| Metric | Target |
|--------|--------|
| Refresh latency | < 200ms |
| Attach time | < 1s |
| Overhead | < 1% CPU |

---

## 11. Roadmap

### Phase 1 (MVP)
- JVM attach via jcmd/jstat
- Overview, CPU, Memory, Threads
- Auto-discovery of local JVMs

### Phase 2
- GC deep dive
- Class histogram
- Trigger GC action

### Phase 3
- Jolokia connector (remote)
- SSH tunnel support
- JFR streaming
- Plugin system

---

## 12. Success Metrics

- < 5 min time-to-first-insight
- Used in prod SSH sessions
- Adopted by senior JVM engineers

---

## 13. One-Line Pitch

**JVM-TUI — Because production JVMs don't have GUIs.**
