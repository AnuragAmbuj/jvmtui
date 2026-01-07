# Architecture

This document describes the technical architecture of JVM-TUI.

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           JVM-TUI (Rust)                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                        Application Layer                         │   │
│  │  ┌─────────┐  ┌──────────┐  ┌─────────┐  ┌──────────────────┐   │   │
│  │  │   CLI   │  │   App    │  │ Config  │  │  Metrics Store   │   │   │
│  │  │ Parser  │  │  State   │  │ Manager │  │  (Ring Buffers)  │   │   │
│  │  └─────────┘  └──────────┘  └─────────┘  └──────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                    │                                    │
│  ┌─────────────────────────────────┼───────────────────────────────┐   │
│  │                        TUI Layer│                                │   │
│  │  ┌──────────┐  ┌──────────┐  ┌─┴────────┐  ┌────────────────┐   │   │
│  │  │ Terminal │  │  Event   │  │  Screen  │  │    Widgets     │   │   │
│  │  │  Setup   │  │  Loop    │  │  Router  │  │ (Sparkline,etc)│   │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                    │                                    │
│  ┌─────────────────────────────────┼───────────────────────────────┐   │
│  │                     JVM Connector Layer                          │   │
│  │  ┌──────────────────────────────┴───────────────────────────┐   │   │
│  │  │                    JvmConnector Trait                     │   │   │
│  │  └──────────────────────────────┬───────────────────────────┘   │   │
│  │           ┌─────────────────────┼─────────────────────┐         │   │
│  │           │                     │                     │         │   │
│  │  ┌────────┴───────┐   ┌────────┴────────┐   ┌────────┴──────┐  │   │
│  │  │ JdkTools       │   │   Jolokia       │   │    Future     │  │   │
│  │  │ Connector      │   │   Connector     │   │   Connectors  │  │   │
│  │  │ [DEFAULT]      │   │   [OPTIONAL]    │   │   (SSH, etc)  │  │   │
│  │  └────────────────┘   └─────────────────┘   └───────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                    │                                    │
└────────────────────────────────────┼────────────────────────────────────┘
                                     │
              ┌──────────────────────┼──────────────────────┐
              │                      │                      │
              ▼                      ▼                      ▼
       ┌─────────────┐        ┌─────────────┐        ┌─────────────┐
       │ Local JVM 1 │        │ Local JVM 2 │        │ Remote JVM  │
       │ (jcmd/jstat)│        │ (jcmd/jstat)│        │ (Jolokia)   │
       └─────────────┘        └─────────────┘        └─────────────┘
```

## Layer Responsibilities

### Application Layer

| Component | Responsibility |
|-----------|----------------|
| **CLI Parser** | Parse command-line arguments (clap) |
| **App State** | Main state machine, screen transitions |
| **Config Manager** | Load/save config file, merge with CLI args |
| **Metrics Store** | Ring buffers for time-series data |

### TUI Layer

| Component | Responsibility |
|-----------|----------------|
| **Terminal Setup** | Raw mode, alternate screen, cleanup |
| **Event Loop** | Key events, tick events, resize events |
| **Screen Router** | JVM Picker → Monitoring → Error screens |
| **Widgets** | Reusable UI components (sparklines, gauges) |

### JVM Connector Layer

| Component | Responsibility |
|-----------|----------------|
| **JvmConnector Trait** | Abstract interface for JVM communication |
| **JdkToolsConnector** | Default: subprocess jcmd/jstat |
| **JolokiaConnector** | Optional: HTTP/JSON to Jolokia agent |

## Data Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Polling    │────▶│   Metrics    │────▶│     TUI      │
│   Interval   │     │    Store     │     │   Render     │
└──────────────┘     └──────────────┘     └──────────────┘
       │                    │                    │
       │                    │                    │
       ▼                    ▼                    ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Connector   │     │ Ring Buffers │     │   Terminal   │
│  (jcmd/etc)  │     │  (history)   │     │   Output     │
└──────────────┘     └──────────────┘     └──────────────┘
```

### Polling Cycle

1. **Tick event** fires (configurable interval: 250ms-10s)
2. **Metrics Collector** invokes connector methods in parallel
3. **Connector** spawns jcmd/jstat subprocesses
4. **Parsers** extract structured data from text output
5. **Metrics Store** receives new samples, rotates ring buffers
6. **TUI** re-renders with updated data

## Concurrency Model

```rust
// Main thread: TUI event loop
loop {
    terminal.draw(|f| ui.render(f))?;
    
    match event_rx.recv()? {
        Event::Key(key) => handle_key(key),
        Event::Tick => {}, // Trigger re-render
        Event::Metrics(data) => store.update(data),
    }
}

// Background task: Metrics collection
async fn collect_metrics(connector: Arc<dyn JvmConnector>) {
    let mut interval = tokio::time::interval(poll_interval);
    loop {
        interval.tick().await;
        let metrics = connector.collect_all().await;
        metrics_tx.send(metrics).await;
    }
}
```

## Module Dependency Graph

```
main.rs
   │
   ├── cli.rs (argument parsing)
   │
   ├── config.rs (configuration)
   │
   ├── app.rs (application state)
   │      │
   │      ├── jvm/ (JVM communication)
   │      │    ├── connector.rs (trait)
   │      │    ├── discovery.rs
   │      │    ├── jdk_tools/ (default impl)
   │      │    │    ├── connector.rs
   │      │    │    ├── detector.rs
   │      │    │    ├── executor.rs
   │      │    │    └── parsers/
   │      │    └── jolokia/ (optional impl)
   │      │
   │      └── metrics/ (data storage)
   │           ├── collector.rs
   │           ├── store.rs
   │           └── ring_buffer.rs
   │
   └── tui/ (user interface)
        ├── terminal.rs
        ├── event.rs
        ├── screens/
        │    ├── jvm_picker.rs
        │    ├── monitoring.rs
        │    └── error.rs
        ├── views/
        │    ├── overview.rs
        │    ├── memory.rs
        │    ├── threads.rs
        │    └── ...
        └── widgets/
             ├── sparkline_panel.rs
             ├── memory_gauge.rs
             └── ...
```

## Error Handling Strategy

```rust
// Application-level errors
#[derive(Error, Debug)]
pub enum AppError {
    #[error("JDK tools not found: {0}")]
    JdkToolsNotFound(#[from] JdkToolsError),
    
    #[error("No JVMs found on this machine")]
    NoJvmsFound,
    
    #[error("Failed to connect to JVM {pid}: {reason}")]
    ConnectionFailed { pid: u32, reason: String },
    
    #[error("JVM disconnected unexpectedly")]
    Disconnected,
    
    #[error("Terminal error: {0}")]
    Terminal(#[from] std::io::Error),
}

// Graceful degradation
impl App {
    fn handle_error(&mut self, error: AppError) {
        match error {
            AppError::JdkToolsNotFound(_) => {
                self.screen = Screen::SetupHelp;
            }
            AppError::NoJvmsFound => {
                self.show_message("No JVMs found. Press 'r' to refresh.");
            }
            AppError::Disconnected => {
                self.screen = Screen::JvmPicker;
                self.show_message("JVM disconnected. Select another.");
            }
            _ => {
                self.show_error(error.to_string());
            }
        }
    }
}
```

## Security Considerations

| Concern | Mitigation |
|---------|------------|
| Subprocess injection | Validate PID is numeric only |
| Sensitive data in output | Don't log raw jcmd output |
| Accidental GC trigger | Require confirmation dialog |
| Config file permissions | Warn if world-readable |

## Performance Characteristics

| Operation | Expected Latency | Notes |
|-----------|------------------|-------|
| jcmd spawn + parse | 10-50ms | Per command |
| jstat spawn + parse | 5-20ms | Lightweight |
| Full metrics cycle | 50-150ms | Parallel execution |
| TUI render | 1-5ms | Diff-based |
| Ring buffer insert | O(1) | Fixed allocation |

## Future Architecture Extensions

### Phase 3: Remote JVMs

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   JVM-TUI   │────▶│ SSH Tunnel  │────▶│ Remote JVM  │
│   (local)   │     │             │     │ (Jolokia)   │
└─────────────┘     └─────────────┘     └─────────────┘
```

### Phase 3: Plugin System

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn on_metrics(&mut self, metrics: &Metrics);
    fn render(&self, frame: &mut Frame, area: Rect);
}
```
