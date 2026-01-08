use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub struct LoadingScreen;

impl LoadingScreen {
    pub fn render(frame: &mut Frame, area: Rect, message: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Min(5),
                Constraint::Percentage(40),
            ])
            .split(area);

        let centered = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ])
            .split(chunks[1])[1];

        let block = Block::default()
            .title(" Loading ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));

        let inner_area = block.inner(centered);
        frame.render_widget(block, centered);

        let loading_text = format!("⏳ {}\n\nPlease wait...", message);

        let loading_widget = Paragraph::new(loading_text)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);

        frame.render_widget(loading_widget, inner_area);
    }
}
