use crate::metrics::store::MetricsStore;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

pub struct ClassesView;

impl ClassesView {
    pub fn render(frame: &mut Frame, area: Rect, store: &MetricsStore) {
        Self::render_with_scroll(frame, area, store, 0);
    }

    pub fn render_with_scroll(frame: &mut Frame, area: Rect, store: &MetricsStore, scroll: usize) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(7), Constraint::Min(0)])
            .split(area);

        Self::render_summary(frame, chunks[0], store);
        Self::render_class_list(frame, chunks[1], store, scroll);
    }

    fn render_summary(frame: &mut Frame, area: Rect, store: &MetricsStore) {
        let classes = &store.class_histogram;

        let total_instances: u64 = classes.iter().map(|c| c.instances).sum();
        let total_bytes: u64 = classes.iter().map(|c| c.bytes).sum();

        let summary_text = format!(
            "Total Classes: {}\n\
             Total Instances: {}\n\
             Total Memory: {:.2} MB\n\
             \n\
             Showing top memory consumers...",
            classes.len(),
            total_instances,
            total_bytes as f64 / 1024.0 / 1024.0
        );

        let summary = Paragraph::new(summary_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Class Histogram Summary"),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(summary, area);
    }

    fn render_class_list(frame: &mut Frame, area: Rect, store: &MetricsStore, scroll: usize) {
        let classes = &store.class_histogram;

        if classes.is_empty() {
            let placeholder = Paragraph::new(
                "No class histogram data available.\n\n\
                 Class histogram collection is expensive and runs less frequently.\n\
                 Wait a moment for data to appear...",
            )
            .block(Block::default().borders(Borders::ALL).title("Class List"))
            .style(Style::default().fg(Color::Gray));

            frame.render_widget(placeholder, area);
            return;
        }

        let header = Row::new(vec![
            Cell::from("Rank").style(Style::default().fg(Color::Yellow)),
            Cell::from("Instances").style(Style::default().fg(Color::Yellow)),
            Cell::from("Bytes").style(Style::default().fg(Color::Yellow)),
            Cell::from("MB").style(Style::default().fg(Color::Yellow)),
            Cell::from("Class Name").style(Style::default().fg(Color::Yellow)),
        ])
        .height(1);

        let rows: Vec<Row> = classes
            .iter()
            .skip(scroll)
            .take(100)
            .map(|class| {
                let mb = class.bytes as f64 / 1024.0 / 1024.0;
                let color = if mb > 50.0 {
                    Color::Red
                } else if mb > 10.0 {
                    Color::Yellow
                } else {
                    Color::White
                };

                Row::new(vec![
                    Cell::from(class.rank.to_string()),
                    Cell::from(class.instances.to_string()),
                    Cell::from(class.bytes.to_string()),
                    Cell::from(format!("{:.2}", mb)).style(Style::default().fg(color)),
                    Cell::from(class.name.clone()),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(6),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(8),
                Constraint::Percentage(60),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Top 100 Classes by Memory Usage"),
        )
        .style(Style::default().fg(Color::White));

        frame.render_widget(table, area);
    }
}
