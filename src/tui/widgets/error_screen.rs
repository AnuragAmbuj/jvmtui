use crate::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub struct ErrorScreen;

impl ErrorScreen {
    pub fn render(frame: &mut Frame, area: Rect, error_message: &str, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Min(10),
                Constraint::Percentage(30),
            ])
            .split(area);

        let centered = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(chunks[1])[1];

        let block = Block::default()
            .title(" Error ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.error()))
            .style(Style::default().bg(theme.background()));

        let inner_area = block.inner(centered);
        frame.render_widget(block, centered);

        let error_text = format!(
            "⚠️  Connection Error\n\n{}\n\n\
            Press 'r' to retry connection\n\
            Press 'q' to quit",
            error_message
        );

        let error_widget = Paragraph::new(error_text)
            .style(Style::default().fg(theme.error()))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(error_widget, inner_area);
    }
}
