# TUI Design

This document describes the Terminal User Interface design for JVM-TUI.

## Design Principles

1. **Information density**: Show maximum useful info without clutter
2. **Glanceable**: Key metrics visible at a glance
3. **Keyboard-first**: Full navigation without mouse
4. **Responsive**: Adapt to terminal size
5. **Consistent**: Same patterns across all views

## Color Scheme

```rust
// src/tui/theme.rs

use ratatui::style::Color;

pub struct Theme {
    // Base colors
    pub bg: Color,
    pub fg: Color,
    pub fg_dim: Color,
    
    // Semantic colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    
    // Chart colors
    pub heap: Color,
    pub metaspace: Color,
    pub gc: Color,
    pub threads: Color,
    
    // Thread state colors
    pub thread_runnable: Color,
    pub thread_blocked: Color,
    pub thread_waiting: Color,
    pub thread_timed_waiting: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: Color::Reset,
            fg: Color::White,
            fg_dim: Color::DarkGray,
            
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Cyan,
            
            heap: Color::Cyan,
            metaspace: Color::Magenta,
            gc: Color::Yellow,
            threads: Color::Blue,
            
            thread_runnable: Color::Green,
            thread_blocked: Color::Red,
            thread_waiting: Color::Yellow,
            thread_timed_waiting: Color::Cyan,
        }
    }
}
```

## Screen Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        Application Flow                          │
└─────────────────────────────────────────────────────────────────┘

┌───────────────┐     ┌───────────────┐     ┌───────────────────┐
│    Startup    │────▶│  JVM Picker   │────▶│    Monitoring     │
│    Screen     │     │    Screen     │     │     Screen        │
└───────────────┘     └───────┬───────┘     └─────────┬─────────┘
                              │                       │
                              │◀──── disconnect ──────┤
                              │                       │
                              │                       ▼
                              │              ┌────────────────┐
                              │              │  Tab Views:    │
                              │              │  - Overview    │
                              │              │  - Memory      │
                              │              │  - Threads     │
                              │              │  - GC          │
                              │              │  - Classes     │
                              │              └────────────────┘
                              │
                     ┌────────┴────────┐
                     │  Error Screen   │
                     │ (JDK not found) │
                     └─────────────────┘
```

## Layout System

### Main Layout Structure

```
┌─ JVM-TUI ────────────────────────────────────────────────────────┐
│                         Header Bar                                │
├──────────────────────────────────────────────────────────────────┤
│                          Tab Bar                                  │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│                                                                   │
│                        Content Area                               │
│                                                                   │
│                                                                   │
├──────────────────────────────────────────────────────────────────┤
│                         Footer Bar                                │
└──────────────────────────────────────────────────────────────────┘
```

### Implementation

```rust
// src/tui/screens/monitoring.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub fn render_monitoring_screen(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header
            Constraint::Length(1),  // Tab bar
            Constraint::Min(0),     // Content
            Constraint::Length(1),  // Footer
        ])
        .split(frame.area());
    
    render_header(frame, chunks[0], app);
    render_tabs(frame, chunks[1], app);
    render_current_view(frame, chunks[2], app);
    render_footer(frame, chunks[3], app);
}
```

## Header Bar

```
PID: 76660 │ OpenJDK 21.0.9 │ G1GC │ Heap: 512/2048 MB (25%) │ Uptime: 2h 34m 12s
```

```rust
fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let store = &app.metrics_store;
    
    let parts = vec![
        format!("PID: {}", app.pid),
        store.vm_version.as_ref()
            .map(|v| format!("{} {}", v.vm_name, v.jdk_version))
            .unwrap_or_else(|| "Loading...".into()),
        store.vm_flags.as_ref()
            .map(|f| f.gc_type.display_name().to_string())
            .unwrap_or_default(),
        store.heap_info.as_ref()
            .map(|s| format!("Heap: {:.0}/{:.0} MB ({:.0}%)",
                s.value.heap_used_mb(),
                s.value.heap_total_mb(),
                s.value.heap_used_pct()))
            .unwrap_or_default(),
        format!("Uptime: {}", store.uptime_display()),
    ];
    
    let header_text = parts.into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" │ ");
    
    let paragraph = Paragraph::new(header_text)
        .style(Style::default().add_modifier(Modifier::BOLD));
    
    frame.render_widget(paragraph, area);
}
```

## Tab Bar

```
[1:Overview] 2:Memory  3:Threads  4:GC  5:Classes
```

```rust
fn render_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let titles: Vec<Line> = Tab::iter()
        .enumerate()
        .map(|(i, tab)| {
            let num = i + 1;
            if tab == app.current_tab {
                Line::from(vec![
                    Span::styled(
                        format!("[{}:{}]", num, tab),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(format!("{}:{}", num, tab))
            }
        })
        .collect();
    
    let tabs = Tabs::new(titles)
        .select(app.current_tab as usize)
        .divider(" ");
    
    frame.render_widget(tabs, area);
}
```

## Overview Dashboard

```
┌─ Overview ───────────────────────────────────────────────────────┐
│ ┌─ Heap Memory ──────────────────┐ ┌─ CPU ─────────────────────┐ │
│ │ ▂▃▅▆█▇▅▃▂▁▂▃▅▆█▇▅▃▂▁            │ │ Process: 12%             │ │
│ │ 512 MB / 2048 MB (25%)         │ │ System:  45%             │ │
│ └────────────────────────────────┘ └────────────────────────────┘ │
│ ┌─ GC Activity ──────────────────┐ ┌─ Threads ──────────────────┐ │
│ │ Young GC: 695 (7.8s total)     │ │ Total: 76  Peak: 82       │ │
│ │ Full GC:  1 (0.2s total)       │ │ Daemon: 45                │ │
│ │ Throughput: 99.2%              │ │ ● 12 ◐ 34 ◑ 28 ✖ 2       │ │
│ └────────────────────────────────┘ └────────────────────────────┘ │
│ ┌─ Memory Pools ─────────────────────────────────────────────────┐│
│ │ Eden     ▓░░░░░░░░░  1.5%  │ Old    ▓▓▓▓▓▓▓░░░ 70%          ││
│ │ Survivor ░░░░░░░░░░  0.0%  │ Meta   ▓▓▓▓▓▓▓▓▓░ 98%          ││
│ └────────────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────────┘
```

```rust
// src/tui/views/overview.rs

pub fn render_overview(frame: &mut Frame, area: Rect, store: &MetricsStore) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),   // Top row (Heap + CPU)
            Constraint::Length(6),   // Middle row (GC + Threads)
            Constraint::Min(4),      // Memory pools
        ])
        .split(area);
    
    // Top row: Heap and CPU side by side
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[0]);
    
    render_heap_panel(frame, top_chunks[0], store);
    render_cpu_panel(frame, top_chunks[1], store);
    
    // Middle row: GC and Threads
    let mid_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);
    
    render_gc_summary(frame, mid_chunks[0], store);
    render_thread_summary(frame, mid_chunks[1], store);
    
    // Bottom: Memory pools
    render_memory_pools(frame, chunks[2], store);
}

fn render_heap_panel(frame: &mut Frame, area: Rect, store: &MetricsStore) {
    let block = Block::bordered().title(" Heap Memory ");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(1)])
        .split(inner);
    
    // Sparkline
    let data = store.heap_usage_pct_history();
    let sparkline = Sparkline::default()
        .data(&data)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(sparkline, chunks[0]);
    
    // Stats text
    if let Some(sample) = &store.heap_info {
        let info = &sample.value;
        let text = format!(
            "{:.0} MB / {:.0} MB ({:.1}%)",
            info.heap_used_mb(),
            info.heap_total_mb(),
            info.heap_used_pct()
        );
        frame.render_widget(Paragraph::new(text), chunks[1]);
    }
}
```

## Memory View

```
┌─ Memory ─────────────────────────────────────────────────────────┐
│ ┌─ Heap Usage ───────────────────────────────────────────────────┐│
│ │ ▂▃▅▆█▇▅▃▂▁▂▃▅▆█▇▅▃▂▁▂▃▅▆█▇▅▃▂▁▂▃▅▆█▇▅▃▂▁▂▃▅▆█▇▅▃▂▁▂▃▅▆█▇▅▃▂▁  ││
│ │ Used: 512 MB  Committed: 1024 MB  Max: 2048 MB                 ││
│ └────────────────────────────────────────────────────────────────┘│
│                                                                   │
│ ┌─ Heap Breakdown ───────────────────────────────────────────────┐│
│ │ Eden       [▓▓░░░░░░░░░░░░░░░░░░]   1.5%     15 MB            ││
│ │ Survivor 0 [░░░░░░░░░░░░░░░░░░░░]   0.0%      0 MB            ││
│ │ Survivor 1 [░░░░░░░░░░░░░░░░░░░░]   0.0%      0 MB            ││
│ │ Old Gen    [▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░]  69.8%    700 MB            ││
│ └────────────────────────────────────────────────────────────────┘│
│                                                                   │
│ ┌─ Non-Heap ─────────────────────────────────────────────────────┐│
│ │ Metaspace   Used: 412 MB  Committed: 428 MB  Reserved: 1408 MB ││
│ │ Class Space Used: 55 MB   Committed: 59 MB                     ││
│ └────────────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────────┘
```

## Thread View

```
┌─ Threads ────────────────────────────────────────────────────────┐
│ Total: 76  │  Daemon: 45  │  Peak: 82                            │
│                                                                   │
│ ● RUNNABLE 12  │  ◐ WAITING 34  │  ◑ TIMED_WAITING 28  │  ✖ BLOCKED 2 │
├──────────────────────────────────────────────────────────────────┤
│ ● main  [TIMED_WAITING]                                          │
│     at kotlinx.coroutines.BlockingCoroutine.joinBlocking         │
│                                                                   │
│ ● Common-Cleaner  [TIMED_WAITING] (daemon)                       │
│     at java.lang.ref.ReferenceQueue.await                        │
│                                                                   │
│▶● Reference Handler  [RUNNABLE] (daemon)                         │
│     at java.lang.ref.Reference.waitForReferencePendingList       │
│     at java.lang.ref.Reference.processPendingReferences          │
│     at java.lang.ref.Reference$ReferenceHandler.run              │
│                                                                   │
│ ● Finalizer  [WAITING] (daemon)                                  │
│     at java.lang.Object.wait                                     │
├──────────────────────────────────────────────────────────────────┤
│ [j/k] Navigate  [Enter] Expand/Collapse  [t] Full Dump           │
└──────────────────────────────────────────────────────────────────┘
```

## GC View

```
┌─ Garbage Collection ─────────────────────────────────────────────┐
│ GC Type: G1GC                                                    │
├──────────────────────────────────────────────────────────────────┤
│ ┌─ GC Events ────────────────────────────────────────────────────┐│
│ │ Young GC                                                       ││
│ │   Count: 695     Total Time: 7.803s    Avg: 11.2ms            ││
│ │                                                                ││
│ │ Full GC                                                        ││
│ │   Count: 1       Total Time: 0.236s    Avg: 236.0ms           ││
│ │                                                                ││
│ │ Concurrent GC (G1)                                             ││
│ │   Count: 436     Total Time: 4.121s    Avg: 9.5ms             ││
│ └────────────────────────────────────────────────────────────────┘│
│                                                                   │
│ ┌─ GC Throughput ────────────────────────────────────────────────┐│
│ │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 99.2%      ││
│ │ Time in GC: 12.16s / 3029s uptime                              ││
│ └────────────────────────────────────────────────────────────────┘│
├──────────────────────────────────────────────────────────────────┤
│ [Ctrl+g] Trigger GC                                              │
└──────────────────────────────────────────────────────────────────┘
```

## Footer Bar

Context-aware help showing relevant keybindings:

```rust
fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let hints = match app.current_tab {
        Tab::Overview => "[q] Quit  [r] Refresh  [1-5] Switch Tab  [?] Help",
        Tab::Memory => "[q] Quit  [r] Refresh  [h] Histogram  [?] Help",
        Tab::Threads => "[q] Quit  [j/k] Navigate  [Enter] Expand  [t] Dump  [?] Help",
        Tab::GC => "[q] Quit  [Ctrl+g] Trigger GC  [?] Help",
        Tab::Classes => "[q] Quit  [h] Histogram  [?] Help",
    };
    
    let footer = Paragraph::new(hints)
        .style(Style::default().fg(Color::DarkGray));
    
    frame.render_widget(footer, area);
}
```

## Responsive Layout

Handle different terminal sizes:

```rust
fn calculate_layout(area: Rect) -> LayoutType {
    let width = area.width;
    let height = area.height;
    
    if width < 60 || height < 15 {
        LayoutType::Minimal
    } else if width < 100 || height < 25 {
        LayoutType::Compact
    } else {
        LayoutType::Full
    }
}

enum LayoutType {
    Minimal,   // Single column, minimal info
    Compact,   // Reduced panels
    Full,      // All panels visible
}
```

## Widgets

### SparklinePanel

```rust
// src/tui/widgets/sparkline_panel.rs

pub struct SparklinePanel<'a> {
    title: &'a str,
    data: &'a [u64],
    value: String,
    color: Color,
}

impl<'a> SparklinePanel<'a> {
    pub fn new(title: &'a str, data: &'a [u64]) -> Self {
        Self {
            title,
            data,
            value: String::new(),
            color: Color::Cyan,
        }
    }
    
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }
    
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Widget for SparklinePanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title(self.title);
        let inner = block.inner(area);
        block.render(area, buf);
        
        let chunks = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(1),
        ]).split(inner);
        
        Sparkline::default()
            .data(self.data)
            .style(Style::default().fg(self.color))
            .render(chunks[0], buf);
        
        Paragraph::new(self.value)
            .render(chunks[1], buf);
    }
}
```

### MemoryGauge

```rust
// src/tui/widgets/memory_gauge.rs

pub struct MemoryGauge<'a> {
    label: &'a str,
    used: u64,
    total: u64,
}

impl Widget for MemoryGauge<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let pct = if self.total == 0 { 
            0.0 
        } else { 
            self.used as f64 / self.total as f64 
        };
        
        let gauge = Gauge::default()
            .label(format!("{}: {:.1}%", self.label, pct * 100.0))
            .ratio(pct)
            .gauge_style(Style::default().fg(Color::Cyan));
        
        gauge.render(area, buf);
    }
}
```
