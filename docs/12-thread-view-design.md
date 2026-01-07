# Thread View Design

This document describes the thread view implementation with summary-first display and on-demand stack expansion.

## Design Goals

1. **Summary first**: Show thread overview without overwhelming detail
2. **On-demand expansion**: Full stack traces only when requested
3. **State visibility**: Color-coded thread states at a glance
4. **Efficient**: Don't load full stacks until needed

## Thread States

| State | Symbol | Color | Description |
|-------|--------|-------|-------------|
| RUNNABLE | ● | Green | Thread is executing |
| BLOCKED | ✖ | Red | Waiting for monitor lock |
| WAITING | ◐ | Yellow | Waiting indefinitely |
| TIMED_WAITING | ◑ | Cyan | Waiting with timeout |
| NEW | ○ | Blue | Not yet started |
| TERMINATED | ◌ | Gray | Completed execution |

```rust
// src/jvm/types.rs

impl ThreadState {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::New => "○",
            Self::Runnable => "●",
            Self::Blocked => "✖",
            Self::Waiting => "◐",
            Self::TimedWaiting => "◑",
            Self::Terminated => "◌",
        }
    }
    
    pub fn color(&self) -> Color {
        match self {
            Self::New => Color::Blue,
            Self::Runnable => Color::Green,
            Self::Blocked => Color::Red,
            Self::Waiting => Color::Yellow,
            Self::TimedWaiting => Color::Cyan,
            Self::Terminated => Color::DarkGray,
        }
    }
    
    pub fn priority(&self) -> u8 {
        // For sorting: show problematic states first
        match self {
            Self::Blocked => 0,      // Most critical
            Self::Runnable => 1,
            Self::TimedWaiting => 2,
            Self::Waiting => 3,
            Self::New => 4,
            Self::Terminated => 5,
        }
    }
}
```

## View Layout

```
┌─ Threads ────────────────────────────────────────────────────────┐
│ ┌─ Summary ──────────────────────────────────────────────────────┐│
│ │ Total: 76  │  Daemon: 45  │  Peak: 82                          ││
│ │                                                                 ││
│ │ ● RUNNABLE 12  │  ◐ WAITING 34  │  ◑ TIMED_WAITING 28  │  ✖ 2 ││
│ └─────────────────────────────────────────────────────────────────┘│
│ ┌─ Thread List ──────────────────────────────────────────────────┐│
│ │   ● main  [TIMED_WAITING]                                      ││
│ │       at kotlinx.coroutines.BlockingCoroutine.joinBlocking     ││
│ │                                                                 ││
│ │   ● Common-Cleaner  [TIMED_WAITING] (daemon)                   ││
│ │       at java.lang.ref.ReferenceQueue.await                    ││
│ │                                                                 ││
│ │ ▶ ● Reference Handler  [RUNNABLE] (daemon)  ← Selected         ││
│ │       at java.lang.ref.Reference.waitForReferencePendingList   ││
│ │       at java.lang.ref.Reference.processPendingReferences      ││
│ │       at java.lang.ref.Reference$ReferenceHandler.run          ││
│ │       ... (expanded)                                           ││
│ │                                                                 ││
│ │   ● Finalizer  [WAITING] (daemon)                              ││
│ │       at java.lang.Object.wait                                 ││
│ └─────────────────────────────────────────────────────────────────┘│
├──────────────────────────────────────────────────────────────────┤
│ [j/k] Navigate  [Enter] Expand/Collapse  [t] Full Dump  [/] Search│
└──────────────────────────────────────────────────────────────────┘
```

## Data Structures

```rust
// src/tui/views/threads.rs

use crate::jvm::types::{ThreadDump, ThreadInfo, ThreadState, ThreadSummary};
use std::collections::HashSet;

pub struct ThreadsView {
    // Data
    summary: Option<ThreadSummary>,
    threads: Vec<ThreadViewItem>,
    
    // UI State
    selected_index: usize,
    expanded_threads: HashSet<usize>,  // Indices of expanded threads
    scroll_offset: usize,
    
    // Loading state
    loading_full_dump: bool,
    
    // Search
    search_query: Option<String>,
    search_matches: Vec<usize>,
    current_match: usize,
}

#[derive(Debug, Clone)]
pub struct ThreadViewItem {
    pub info: ThreadInfo,
    pub stack_preview: String,      // First 1-2 frames
    pub full_stack_loaded: bool,
}

impl ThreadViewItem {
    pub fn from_info(info: ThreadInfo) -> Self {
        let stack_preview = info.stack_trace
            .first()
            .cloned()
            .unwrap_or_else(|| "(no stack trace)".into());
        
        Self {
            info,
            stack_preview,
            full_stack_loaded: false,
        }
    }
    
    pub fn display_lines(&self, expanded: bool, max_lines: usize) -> Vec<Line<'_>> {
        let mut lines = Vec::new();
        
        // Header line
        let header = self.header_line();
        lines.push(header);
        
        // Stack trace
        if expanded && self.full_stack_loaded {
            for frame in self.info.stack_trace.iter().take(max_lines) {
                lines.push(Line::from(Span::styled(
                    format!("      {}", frame),
                    Style::default().fg(Color::DarkGray),
                )));
            }
            if self.info.stack_trace.len() > max_lines {
                lines.push(Line::from(Span::styled(
                    format!("      ... ({} more frames)", 
                        self.info.stack_trace.len() - max_lines),
                    Style::default().fg(Color::DarkGray).italic(),
                )));
            }
        } else {
            // Show preview only
            lines.push(Line::from(Span::styled(
                format!("      {}", self.stack_preview),
                Style::default().fg(Color::DarkGray),
            )));
        }
        
        lines
    }
    
    fn header_line(&self) -> Line<'_> {
        let state = &self.info.state;
        
        let mut spans = vec![
            Span::styled(
                format!("{} ", state.symbol()),
                Style::default().fg(state.color()),
            ),
            Span::styled(
                &self.info.name,
                Style::default().bold(),
            ),
            Span::raw("  "),
            Span::styled(
                format!("[{}]", state),
                Style::default().fg(state.color()),
            ),
        ];
        
        if self.info.daemon {
            spans.push(Span::styled(
                " (daemon)",
                Style::default().fg(Color::DarkGray),
            ));
        }
        
        Line::from(spans)
    }
}
```

## View Implementation

```rust
impl ThreadsView {
    pub fn new() -> Self {
        Self {
            summary: None,
            threads: Vec::new(),
            selected_index: 0,
            expanded_threads: HashSet::new(),
            scroll_offset: 0,
            loading_full_dump: false,
            search_query: None,
            search_matches: Vec::new(),
            current_match: 0,
        }
    }
    
    /// Update with new thread summary (called on each poll)
    pub fn update_summary(&mut self, summary: ThreadSummary) {
        self.summary = Some(summary);
    }
    
    /// Update with thread dump (includes stack traces)
    pub fn update_threads(&mut self, dump: ThreadDump) {
        self.threads = dump.threads
            .into_iter()
            .map(ThreadViewItem::from_info)
            .collect();
        
        // Sort by state priority, then name
        self.threads.sort_by(|a, b| {
            a.info.state.priority()
                .cmp(&b.info.state.priority())
                .then_with(|| a.info.name.cmp(&b.info.name))
        });
        
        // Reset selection if out of bounds
        if self.selected_index >= self.threads.len() {
            self.selected_index = self.threads.len().saturating_sub(1);
        }
        
        self.loading_full_dump = false;
    }
    
    /// Toggle expansion of currently selected thread
    pub fn toggle_selected(&mut self) {
        if self.expanded_threads.contains(&self.selected_index) {
            self.expanded_threads.remove(&self.selected_index);
        } else {
            self.expanded_threads.insert(self.selected_index);
            // Mark as loaded (in real impl, might need async load)
            if let Some(thread) = self.threads.get_mut(self.selected_index) {
                thread.full_stack_loaded = true;
            }
        }
    }
    
    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected_index < self.threads.len().saturating_sub(1) {
            self.selected_index += 1;
            self.ensure_visible();
        }
    }
    
    /// Move selection up
    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.ensure_visible();
        }
    }
    
    /// Ensure selected item is visible
    fn ensure_visible(&mut self) {
        // Implementation depends on visible area height
    }
    
    /// Search threads by name
    pub fn search(&mut self, query: &str) {
        self.search_query = Some(query.to_lowercase());
        self.update_search_matches();
    }
    
    fn update_search_matches(&mut self) {
        self.search_matches.clear();
        
        if let Some(ref query) = self.search_query {
            for (i, thread) in self.threads.iter().enumerate() {
                if thread.info.name.to_lowercase().contains(query) {
                    self.search_matches.push(i);
                }
            }
        }
        
        self.current_match = 0;
    }
    
    /// Jump to next search match
    pub fn next_match(&mut self) {
        if !self.search_matches.is_empty() {
            self.current_match = (self.current_match + 1) % self.search_matches.len();
            self.selected_index = self.search_matches[self.current_match];
            self.ensure_visible();
        }
    }
    
    /// Jump to previous search match
    pub fn prev_match(&mut self) {
        if !self.search_matches.is_empty() {
            self.current_match = if self.current_match == 0 {
                self.search_matches.len() - 1
            } else {
                self.current_match - 1
            };
            self.selected_index = self.search_matches[self.current_match];
            self.ensure_visible();
        }
    }
}
```

## Rendering

```rust
impl ThreadsView {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical([
            Constraint::Length(5),   // Summary panel
            Constraint::Min(0),       // Thread list
        ]).split(area);
        
        self.render_summary(frame, chunks[0]);
        self.render_thread_list(frame, chunks[1]);
    }
    
    fn render_summary(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().title(" Thread Summary ");
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        if let Some(ref summary) = self.summary {
            let stats_line = Line::from(vec![
                Span::styled("Total: ", Style::default().bold()),
                Span::raw(format!("{}", summary.total_count)),
                Span::raw("  │  "),
                Span::styled("Daemon: ", Style::default().bold()),
                Span::raw(format!("{}", summary.daemon_count)),
                Span::raw("  │  "),
                Span::styled("Peak: ", Style::default().bold()),
                Span::raw(format!("{}", summary.peak_count)),
            ]);
            
            // State breakdown
            let state_spans: Vec<Span> = [
                ThreadState::Runnable,
                ThreadState::Waiting,
                ThreadState::TimedWaiting,
                ThreadState::Blocked,
            ].iter()
            .filter_map(|state| {
                summary.by_state.get(state).map(|count| {
                    vec![
                        Span::styled(
                            format!("{} ", state.symbol()),
                            Style::default().fg(state.color()),
                        ),
                        Span::raw(format!("{} ", state)),
                        Span::styled(
                            format!("{}", count),
                            Style::default().bold(),
                        ),
                        Span::raw("  │  "),
                    ]
                })
            })
            .flatten()
            .collect();
            
            let text = vec![
                stats_line,
                Line::from(""),
                Line::from(state_spans),
            ];
            
            let paragraph = Paragraph::new(text);
            frame.render_widget(paragraph, inner);
        }
    }
    
    fn render_thread_list(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title(" Threads ")
            .title_bottom(Line::from(
                " [j/k] Navigate  [Enter] Expand  [t] Dump  [/] Search "
            ).right_aligned());
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        if self.loading_full_dump {
            let loading = Paragraph::new("Loading thread dump...")
                .alignment(Alignment::Center);
            frame.render_widget(loading, inner);
            return;
        }
        
        // Build list items
        let items: Vec<ListItem> = self.threads
            .iter()
            .enumerate()
            .map(|(i, thread)| {
                let is_expanded = self.expanded_threads.contains(&i);
                let is_selected = i == self.selected_index;
                let is_match = self.search_matches.contains(&i);
                
                let lines = thread.display_lines(is_expanded, 20);
                
                let mut item = ListItem::new(lines);
                
                if is_match {
                    item = item.style(Style::default().bg(Color::DarkGray));
                }
                
                item
            })
            .collect();
        
        let list = List::new(items)
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol("▶ ");
        
        let mut state = ListState::default();
        state.select(Some(self.selected_index));
        
        frame.render_stateful_widget(list, inner, &mut state);
    }
}
```

## Async Stack Loading

For large thread dumps, load stacks lazily:

```rust
impl ThreadsView {
    /// Request full thread dump asynchronously
    pub fn request_full_dump(&mut self) {
        self.loading_full_dump = true;
        // Actual loading happens via connector in App
    }
    
    /// Load full stack for a specific thread
    pub async fn load_thread_stack(
        &mut self,
        index: usize,
        connector: &dyn JvmConnector,
    ) -> Result<()> {
        // Get fresh thread dump
        let dump = connector.get_thread_dump().await?;
        
        // Find matching thread
        if let Some(thread) = self.threads.get_mut(index) {
            if let Some(fresh) = dump.threads.iter()
                .find(|t| t.name == thread.info.name) 
            {
                thread.info.stack_trace = fresh.stack_trace.clone();
                thread.full_stack_loaded = true;
            }
        }
        
        Ok(())
    }
}
```

## Stack Trace Formatting

Pretty-print stack traces with syntax highlighting:

```rust
fn format_stack_frame(frame: &str) -> Line<'_> {
    // Pattern: "at package.Class.method(File.java:123)"
    
    let parts: Vec<&str> = frame.splitn(2, '(').collect();
    
    if parts.len() == 2 {
        let method_part = parts[0].trim_start_matches("at ");
        let location = parts[1].trim_end_matches(')');
        
        // Split method into package.Class.method
        let method_parts: Vec<&str> = method_part.rsplitn(2, '.').collect();
        
        if method_parts.len() == 2 {
            let method = method_parts[0];
            let class = method_parts[1];
            
            return Line::from(vec![
                Span::raw("at "),
                Span::styled(class, Style::default().fg(Color::Cyan)),
                Span::raw("."),
                Span::styled(method, Style::default().fg(Color::Yellow)),
                Span::raw("("),
                Span::styled(location, Style::default().fg(Color::DarkGray)),
                Span::raw(")"),
            ]);
        }
    }
    
    // Fallback: plain text
    Line::from(Span::styled(frame, Style::default().fg(Color::DarkGray)))
}
```

## Deadlock Detection

Highlight potential deadlocks:

```rust
fn detect_deadlocks(threads: &[ThreadInfo]) -> Vec<DeadlockInfo> {
    let blocked: Vec<_> = threads.iter()
        .filter(|t| t.state == ThreadState::Blocked)
        .collect();
    
    // Simple detection: look for circular wait patterns
    // Full implementation would parse lock info from stack traces
    
    // For now, just highlight all blocked threads
    blocked.iter()
        .map(|t| DeadlockInfo {
            thread_name: t.name.clone(),
            waiting_for: "unknown".into(),
            held_by: "unknown".into(),
        })
        .collect()
}

struct DeadlockInfo {
    thread_name: String,
    waiting_for: String,
    held_by: String,
}
```
