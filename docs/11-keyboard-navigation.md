# Keyboard Navigation

This document describes the vim-style keyboard navigation system in JVM-TUI.

## Design Principles

1. **Vim-familiar**: Use standard vim motions where applicable
2. **Discoverable**: Footer shows context-relevant shortcuts
3. **Consistent**: Same keys work the same everywhere
4. **No conflicts**: Avoid common terminal shortcuts (Ctrl+C, etc.)

## Global Keybindings

These work in all screens:

| Key | Action | Description |
|-----|--------|-------------|
| `q` | Quit | Exit application |
| `?` | Help | Show help overlay |
| `r` | Refresh | Force immediate refresh |
| `Esc` | Back/Cancel | Go back or cancel operation |

## JVM Picker Screen

| Key | Action | Description |
|-----|--------|-------------|
| `j` / `↓` | Next | Select next JVM |
| `k` / `↑` | Previous | Select previous JVM |
| `Enter` | Connect | Connect to selected JVM |
| `r` | Refresh | Re-scan for JVMs |
| `g` | Go to top | Select first JVM |
| `G` | Go to bottom | Select last JVM |

## Monitoring Screen

### Tab Navigation

| Key | Action | Description |
|-----|--------|-------------|
| `1` | Overview | Switch to Overview tab |
| `2` | Memory | Switch to Memory tab |
| `3` | Threads | Switch to Threads tab |
| `4` | GC | Switch to GC tab |
| `5` | Classes | Switch to Classes tab |
| `h` / `←` | Previous tab | Switch to previous tab |
| `l` / `→` | Next tab | Switch to next tab |
| `Tab` | Next tab | Alternative for next tab |
| `Shift+Tab` | Previous tab | Alternative for previous tab |

### Scrolling (in scrollable views)

| Key | Action | Description |
|-----|--------|-------------|
| `j` / `↓` | Scroll down | Move selection/scroll down |
| `k` / `↑` | Scroll up | Move selection/scroll up |
| `Ctrl+d` | Page down | Scroll half page down |
| `Ctrl+u` | Page up | Scroll half page up |
| `g` `g` | Top | Go to top (press g twice) |
| `G` | Bottom | Go to bottom |

### View-Specific Actions

#### Threads View

| Key | Action | Description |
|-----|--------|-------------|
| `Enter` | Expand/Collapse | Toggle stack trace visibility |
| `Space` | Expand/Collapse | Alternative toggle |
| `t` | Thread dump | Take full thread dump |
| `/` | Search | Search thread names |
| `n` | Next match | Jump to next search match |
| `N` | Previous match | Jump to previous match |

#### GC View

| Key | Action | Description |
|-----|--------|-------------|
| `Ctrl+g` | Trigger GC | Request garbage collection (with confirm) |

#### Classes View

| Key | Action | Description |
|-----|--------|-------------|
| `h` | Histogram | Load class histogram |

## Implementation

### Key Event Handler

```rust
// src/tui/event.rs

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub enum Action {
    // Global
    Quit,
    Help,
    Refresh,
    Back,
    
    // Navigation
    NextTab,
    PrevTab,
    GoToTab(usize),
    
    // Scrolling
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    ScrollToTop,
    ScrollToBottom,
    
    // Selection
    SelectNext,
    SelectPrev,
    Expand,
    Collapse,
    Toggle,
    
    // Actions
    TriggerGC,
    ThreadDump,
    ClassHistogram,
    
    // Search
    StartSearch,
    NextMatch,
    PrevMatch,
    
    // None
    Noop,
}

pub struct KeyHandler {
    last_key: Option<KeyCode>,
    last_key_time: std::time::Instant,
}

impl KeyHandler {
    pub fn new() -> Self {
        Self {
            last_key: None,
            last_key_time: std::time::Instant::now(),
        }
    }
    
    pub fn handle(&mut self, key: KeyEvent, context: &Context) -> Action {
        let now = std::time::Instant::now();
        
        // Check for key sequences (like 'gg')
        let is_sequence = self.last_key.is_some() 
            && now.duration_since(self.last_key_time).as_millis() < 500;
        
        let action = match (key.code, key.modifiers, is_sequence, self.last_key) {
            // Global
            (KeyCode::Char('q'), KeyModifiers::NONE, _, _) => Action::Quit,
            (KeyCode::Char('?'), _, _, _) => Action::Help,
            (KeyCode::Char('r'), KeyModifiers::NONE, _, _) => Action::Refresh,
            (KeyCode::Esc, _, _, _) => Action::Back,
            
            // Tab navigation (number keys)
            (KeyCode::Char('1'), KeyModifiers::NONE, _, _) => Action::GoToTab(0),
            (KeyCode::Char('2'), KeyModifiers::NONE, _, _) => Action::GoToTab(1),
            (KeyCode::Char('3'), KeyModifiers::NONE, _, _) => Action::GoToTab(2),
            (KeyCode::Char('4'), KeyModifiers::NONE, _, _) => Action::GoToTab(3),
            (KeyCode::Char('5'), KeyModifiers::NONE, _, _) => Action::GoToTab(4),
            
            // Tab navigation (vim style)
            (KeyCode::Char('h'), KeyModifiers::NONE, _, _) 
                if context.in_tab_bar => Action::PrevTab,
            (KeyCode::Char('l'), KeyModifiers::NONE, _, _) 
                if context.in_tab_bar => Action::NextTab,
            (KeyCode::Left, _, _, _) => Action::PrevTab,
            (KeyCode::Right, _, _, _) => Action::NextTab,
            (KeyCode::Tab, KeyModifiers::NONE, _, _) => Action::NextTab,
            (KeyCode::BackTab, _, _, _) => Action::PrevTab,
            
            // Scrolling
            (KeyCode::Char('j'), KeyModifiers::NONE, _, _) => Action::SelectNext,
            (KeyCode::Char('k'), KeyModifiers::NONE, _, _) => Action::SelectPrev,
            (KeyCode::Down, _, _, _) => Action::SelectNext,
            (KeyCode::Up, _, _, _) => Action::SelectPrev,
            
            (KeyCode::Char('d'), KeyModifiers::CONTROL, _, _) => Action::PageDown,
            (KeyCode::Char('u'), KeyModifiers::CONTROL, _, _) => Action::PageUp,
            (KeyCode::PageDown, _, _, _) => Action::PageDown,
            (KeyCode::PageUp, _, _, _) => Action::PageUp,
            
            // 'gg' sequence for top
            (KeyCode::Char('g'), KeyModifiers::NONE, true, Some(KeyCode::Char('g'))) 
                => Action::ScrollToTop,
            (KeyCode::Char('G'), _, _, _) => Action::ScrollToBottom,
            (KeyCode::Home, _, _, _) => Action::ScrollToTop,
            (KeyCode::End, _, _, _) => Action::ScrollToBottom,
            
            // Selection
            (KeyCode::Enter, _, _, _) => Action::Toggle,
            (KeyCode::Char(' '), _, _, _) => Action::Toggle,
            
            // View-specific actions
            (KeyCode::Char('t'), KeyModifiers::NONE, _, _) 
                if context.current_tab == Tab::Threads => Action::ThreadDump,
            (KeyCode::Char('g'), KeyModifiers::CONTROL, _, _) 
                if context.current_tab == Tab::GC => Action::TriggerGC,
            (KeyCode::Char('h'), KeyModifiers::NONE, _, _) 
                if context.current_tab == Tab::Classes => Action::ClassHistogram,
            
            // Search
            (KeyCode::Char('/'), KeyModifiers::NONE, _, _) => Action::StartSearch,
            (KeyCode::Char('n'), KeyModifiers::NONE, _, _) 
                if context.has_search => Action::NextMatch,
            (KeyCode::Char('N'), _, _, _) 
                if context.has_search => Action::PrevMatch,
            
            _ => Action::Noop,
        };
        
        // Update last key for sequences
        self.last_key = Some(key.code);
        self.last_key_time = now;
        
        action
    }
}

pub struct Context {
    pub current_tab: Tab,
    pub in_tab_bar: bool,
    pub has_search: bool,
    pub can_scroll: bool,
}
```

### Help Overlay

```rust
// src/tui/widgets/help_overlay.rs

pub fn render_help_overlay(frame: &mut Frame) {
    let area = centered_rect(60, 80, frame.area());
    
    // Clear background
    frame.render_widget(Clear, area);
    
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Global", Style::default().bold().underlined()),
        ]),
        Line::from("  q          Quit"),
        Line::from("  ?          Toggle this help"),
        Line::from("  r          Force refresh"),
        Line::from("  Esc        Go back / Cancel"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation", Style::default().bold().underlined()),
        ]),
        Line::from("  1-5        Switch to tab"),
        Line::from("  h/l, ←/→   Previous/Next tab"),
        Line::from("  j/k, ↓/↑   Scroll down/up"),
        Line::from("  Ctrl+d/u   Page down/up"),
        Line::from("  gg         Go to top"),
        Line::from("  G          Go to bottom"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Actions", Style::default().bold().underlined()),
        ]),
        Line::from("  Enter      Expand/Collapse"),
        Line::from("  t          Thread dump (Threads view)"),
        Line::from("  Ctrl+g     Trigger GC (GC view)"),
        Line::from("  h          Class histogram (Classes view)"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Search", Style::default().bold().underlined()),
        ]),
        Line::from("  /          Start search"),
        Line::from("  n/N        Next/Previous match"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press any key to close", Style::default().dim()),
        ]),
    ];
    
    let help = Paragraph::new(help_text)
        .block(Block::bordered().title(" Help "))
        .wrap(Wrap { trim: false });
    
    frame.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
```

## Confirmation Dialogs

For destructive actions like triggering GC:

```rust
pub struct ConfirmDialog {
    message: String,
    confirmed: bool,
}

impl ConfirmDialog {
    pub fn trigger_gc() -> Self {
        Self {
            message: "Trigger garbage collection? [y/N]".into(),
            confirmed: false,
        }
    }
    
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<bool> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Some(true),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(false),
            _ => None,
        }
    }
}
```

## Keybinding Customization (Future)

Config file structure for custom keybindings:

```toml
# ~/.config/jvm-tui/config.toml

[keybindings]
quit = "q"
help = "?"
refresh = "r"

[keybindings.navigation]
next_tab = "l"
prev_tab = "h"
scroll_down = "j"
scroll_up = "k"

[keybindings.actions]
thread_dump = "t"
trigger_gc = "C-g"  # Ctrl+g
```

## Visual Feedback

Show key feedback in status area:

```rust
pub struct KeyFeedback {
    keys: Vec<String>,
    timeout: std::time::Instant,
}

impl KeyFeedback {
    pub fn push(&mut self, key: &str) {
        self.keys.push(key.to_string());
        self.timeout = std::time::Instant::now();
        
        // Keep only last 3 keys
        if self.keys.len() > 3 {
            self.keys.remove(0);
        }
    }
    
    pub fn display(&self) -> Option<String> {
        if self.timeout.elapsed().as_millis() > 1000 {
            return None;
        }
        Some(self.keys.join(" "))
    }
}
```
