use crate::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
};

pub struct HelpOverlay;

impl HelpOverlay {
    pub fn render(frame: &mut Frame, area: Rect, theme: &Theme) {
        let popup_area = Self::centered_rect(80, 90, area);

        frame.render_widget(Clear, popup_area);

        let outer_block = Block::default()
            .title(" Help - Press ? or Esc to close ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_focused()))
            .style(Style::default().bg(theme.background()));

        frame.render_widget(outer_block, popup_area);

        let inner_area = popup_area.inner(ratatui::layout::Margin {
            horizontal: 2,
            vertical: 1,
        });

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(8),
                Constraint::Length(6),
                Constraint::Length(9),
                Constraint::Min(0),
            ])
            .split(inner_area);

        Self::render_section(
            frame,
            sections[0],
            "Global",
            vec![("q", "Quit application"), ("?", "Toggle this help screen")],
            theme,
        );

        Self::render_section(
            frame,
            sections[1],
            "Navigation",
            vec![
                ("1-5", "Switch to tab (Overview/Memory/Threads/GC/Classes)"),
                ("h / ←", "Previous tab"),
                ("l / →", "Next tab"),
                ("Tab", "Next tab"),
                ("Shift+Tab", "Previous tab"),
            ],
            theme,
        );

        Self::render_section(
            frame,
            sections[2],
            "Actions",
            vec![
                ("g", "Trigger garbage collection (with confirmation)"),
                ("r", "Reset metrics store"),
                ("e", "Export current view data"),
            ],
            theme,
        );

        Self::render_section(
            frame,
            sections[3],
            "View-Specific",
            vec![
                ("j / ↓", "Scroll down (Threads/Classes views)"),
                ("k / ↑", "Scroll up (Threads/Classes views)"),
                ("/", "Search threads (Threads view)"),
                ("n", "Next search result (during search)"),
                ("N", "Previous search result (during search)"),
                ("Esc", "Cancel search (during search)"),
            ],
            theme,
        );

        let about_text = "JVM-TUI v0.1.0\n\
                         A beautiful, lightweight terminal interface for JVM monitoring.\n\
                         \n\
                         GitHub: https://github.com/AnuragAmbuj/jvmtui\n\
                         License: MIT OR Apache-2.0";

        let about = Paragraph::new(about_text)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .title(" About ")
                    .border_style(Style::default().fg(theme.border())),
            )
            .style(Style::default().fg(theme.text_dim()))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(about, sections[4]);
    }

    fn render_section(
        frame: &mut Frame,
        area: Rect,
        title: &str,
        keybindings: Vec<(&str, &str)>,
        theme: &Theme,
    ) {
        let rows: Vec<Row> = keybindings
            .iter()
            .map(|(key, desc)| {
                Row::new(vec![
                    Cell::from(*key).style(Style::default().fg(theme.highlight()).bold()),
                    Cell::from(*desc).style(Style::default().fg(theme.text())),
                ])
            })
            .collect();

        let table = Table::new(rows, [Constraint::Length(15), Constraint::Percentage(85)])
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .title(format!(" {} ", title))
                    .border_style(Style::default().fg(theme.border())),
            )
            .column_spacing(2);

        frame.render_widget(table, area);
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
