use crate::app::ExportFormat;
use crate::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

pub struct FormatSelectorDialog;

impl FormatSelectorDialog {
    pub fn render(frame: &mut Frame, area: Rect, selected_format: ExportFormat, theme: &Theme) {
        let popup_area = Self::centered_rect(50, 30, area);

        frame.render_widget(Clear, popup_area);

        let outer_block = Block::default()
            .title(" Select Export Format ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.info()))
            .style(Style::default().bg(theme.background()));

        frame.render_widget(outer_block.clone(), popup_area);

        let inner_area = popup_area.inner(ratatui::layout::Margin {
            horizontal: 2,
            vertical: 1,
        });

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(inner_area);

        let formats = [
            ExportFormat::Json,
            ExportFormat::Prometheus,
            ExportFormat::Csv,
        ];

        let items: Vec<ListItem> = formats
            .iter()
            .map(|format| {
                let symbol = if *format == selected_format {
                    "  >> "
                } else {
                    "     "
                };
                let content = format!(
                    "{}{} (.{})",
                    symbol,
                    format.display_name(),
                    format.extension()
                );
                let style = if *format == selected_format {
                    Style::default()
                        .fg(theme.success())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text())
                };
                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default())
            .style(Style::default().fg(theme.text()));

        frame.render_widget(list, chunks[0]);

        let prompt = Paragraph::new("↑/k: Up | ↓/j: Down | Enter: Confirm | Esc/q: Cancel")
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
