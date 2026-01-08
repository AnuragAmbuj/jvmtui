use crate::config::ConnectionProfile;
use crate::jvm::discovery::DiscoveredJvm;
use crate::theme::Theme;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub enum PickerItem {
    SavedConnection(ConnectionProfile),
    DiscoveredJvm(DiscoveredJvm),
}

impl PickerItem {
    pub fn display_name(&self) -> String {
        match self {
            PickerItem::SavedConnection(conn) => {
                format!("[Saved] {} ({})", conn.name(), conn.connection_type())
            }
            PickerItem::DiscoveredJvm(jvm) => {
                format!("PID: {} - {}", jvm.pid, truncate(&jvm.main_class, 60))
            }
        }
    }

    pub fn is_saved(&self) -> bool {
        matches!(self, PickerItem::SavedConnection(_))
    }
}

pub struct JvmPickerScreen {
    pub items: Vec<PickerItem>,
    pub list_state: ListState,
}

impl JvmPickerScreen {
    pub fn new(jvms: Vec<DiscoveredJvm>, saved_connections: Vec<ConnectionProfile>) -> Self {
        let mut items: Vec<PickerItem> = Vec::new();

        for conn in saved_connections {
            items.push(PickerItem::SavedConnection(conn));
        }

        for jvm in jvms {
            items.push(PickerItem::DiscoveredJvm(jvm));
        }

        let mut list_state = ListState::default();
        if !items.is_empty() {
            list_state.select(Some(0));
        }

        Self { items, list_state }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn selected_item(&self) -> Option<&PickerItem> {
        self.list_state.selected().and_then(|i| self.items.get(i))
    }

    pub fn selected_jvm(&self) -> Option<&DiscoveredJvm> {
        match self.selected_item()? {
            PickerItem::DiscoveredJvm(jvm) => Some(jvm),
            _ => None,
        }
    }

    pub fn selected_connection(&self) -> Option<&ConnectionProfile> {
        match self.selected_item()? {
            PickerItem::SavedConnection(conn) => Some(conn),
            _ => None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, theme: &Theme) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        let title = Paragraph::new("JVM-TUI - Select Connection")
            .style(
                Style::default()
                    .fg(theme.primary())
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(title, chunks[0]);

        if self.items.is_empty() {
            let empty_msg = Paragraph::new(
                "No JVM processes or saved connections found.\n\n\
                 - Make sure you have running Java applications, or\n\
                 - Add saved connections to your config file",
            )
            .style(Style::default().fg(theme.warning()))
            .block(Block::default().borders(Borders::ALL).title("Empty"));
            frame.render_widget(empty_msg, chunks[1]);
        } else {
            let list_items: Vec<ListItem> = self
                .items
                .iter()
                .map(|item| {
                    let content = item.display_name();
                    let style = if item.is_saved() {
                        Style::default().fg(theme.info())
                    } else {
                        Style::default().fg(theme.text())
                    };
                    ListItem::new(content).style(style)
                })
                .collect();

            let title = if self.items.iter().any(|i| i.is_saved()) {
                "Saved Connections & Discovered JVMs"
            } else {
                "Discovered JVMs"
            };

            let list = List::new(list_items)
                .block(Block::default().borders(Borders::ALL).title(title))
                .highlight_style(
                    Style::default()
                        .bg(theme.primary())
                        .fg(theme.background())
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            frame.render_stateful_widget(list, chunks[1], &mut self.list_state);
        }

        let help = Paragraph::new("↑/k: Up | ↓/j: Down | Enter: Connect | r: Refresh | q: Quit")
            .style(Style::default().fg(theme.text_dim()))
            .block(Block::default().borders(Borders::ALL).title("Controls"));

        frame.render_widget(help, chunks[2]);
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
