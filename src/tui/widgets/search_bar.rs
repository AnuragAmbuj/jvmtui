use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub struct SearchBar;

impl SearchBar {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        query: &str,
        results_count: usize,
        current_index: usize,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(90), Constraint::Min(3)])
            .split(area);

        let search_area = chunks[1];

        let result_info = if results_count > 0 {
            format!(" [{}/{}] ", current_index + 1, results_count)
        } else if !query.is_empty() {
            " [No matches] ".to_string()
        } else {
            String::new()
        };

        let search_text = format!("Search: {}{}", query, result_info);

        let search_widget = Paragraph::new(search_text)
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" / to search | n: next | N: prev | Esc: cancel ")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Left);

        frame.render_widget(search_widget, search_area);
    }
}
