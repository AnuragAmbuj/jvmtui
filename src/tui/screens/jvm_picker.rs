use crate::jvm::discovery::DiscoveredJvm;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub struct JvmPickerScreen {
    pub jvms: Vec<DiscoveredJvm>,
    pub list_state: ListState,
}

impl JvmPickerScreen {
    pub fn new(jvms: Vec<DiscoveredJvm>) -> Self {
        let mut list_state = ListState::default();
        if !jvms.is_empty() {
            list_state.select(Some(0));
        }
        Self { jvms, list_state }
    }

    pub fn next(&mut self) {
        if self.jvms.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.jvms.len() - 1 {
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
        if self.jvms.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.jvms.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn selected_jvm(&self) -> Option<&DiscoveredJvm> {
        self.list_state.selected().and_then(|i| self.jvms.get(i))
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        let title = Paragraph::new("JVM-TUI - Select JVM Process")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(title, chunks[0]);

        if self.jvms.is_empty() {
            let empty_msg = Paragraph::new(
                "No JVM processes found.\n\nMake sure you have running Java applications.",
            )
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Empty"));
            frame.render_widget(empty_msg, chunks[1]);
        } else {
            let items: Vec<ListItem> = self
                .jvms
                .iter()
                .map(|jvm| {
                    let content = format!("PID: {} - {}", jvm.pid, truncate(&jvm.main_class, 80));
                    ListItem::new(content).style(Style::default().fg(Color::White))
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Discovered JVMs"),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            frame.render_stateful_widget(list, chunks[1], &mut self.list_state);
        }

        let help = Paragraph::new("↑/k: Up | ↓/j: Down | Enter: Connect | r: Refresh | q: Quit")
            .style(Style::default().fg(Color::Gray))
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
