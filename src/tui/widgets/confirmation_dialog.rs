use crate::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub struct ConfirmationDialog;

impl ConfirmationDialog {
    pub fn render(frame: &mut Frame, area: Rect, title: &str, message: &str, theme: &Theme) {
        let popup_area = Self::centered_rect(50, 25, area);

        frame.render_widget(Clear, popup_area);

        let outer_block = Block::default()
            .title(format!(" {} ", title))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.warning()))
            .style(Style::default().bg(theme.background()));

        frame.render_widget(outer_block.clone(), popup_area);

        let inner_area = popup_area.inner(ratatui::layout::Margin {
            horizontal: 2,
            vertical: 1,
        });

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(inner_area);

        let message_widget = Paragraph::new(message)
            .style(Style::default().fg(theme.text()))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(message_widget, chunks[0]);

        let prompt = Paragraph::new("Press [Y] to confirm, [N] to cancel")
            .style(Style::default().fg(theme.text_dim()))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(theme.border())),
            );

        frame.render_widget(prompt, chunks[1]);
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}
