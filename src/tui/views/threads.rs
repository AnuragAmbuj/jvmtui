use crate::jvm::types::ThreadState;
use crate::metrics::store::MetricsStore;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use std::collections::HashMap;

pub struct ThreadsView;

impl ThreadsView {
    pub fn render(frame: &mut Frame, area: Rect, store: &MetricsStore) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(9), Constraint::Min(0)])
            .split(area);

        Self::render_summary_section(frame, chunks[0], store);
        Self::render_thread_list(frame, chunks[1], store);
    }

    fn render_summary_section(frame: &mut Frame, area: Rect, store: &MetricsStore) {
        let threads = &store.thread_snapshot;

        let mut state_counts: HashMap<ThreadState, usize> = HashMap::new();
        for thread in threads {
            *state_counts.entry(thread.state).or_insert(0) += 1;
        }

        let summary_text = format!(
            "Total Threads: {}\n\n\
             Runnable:      {}\n\
             Blocked:       {}\n\
             Waiting:       {}\n\
             Timed Waiting: {}\n\
             Terminated:    {}",
            threads.len(),
            state_counts.get(&ThreadState::Runnable).unwrap_or(&0),
            state_counts.get(&ThreadState::Blocked).unwrap_or(&0),
            state_counts.get(&ThreadState::Waiting).unwrap_or(&0),
            state_counts.get(&ThreadState::TimedWaiting).unwrap_or(&0),
            state_counts.get(&ThreadState::Terminated).unwrap_or(&0),
        );

        let summary = Paragraph::new(summary_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Thread Summary"),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(summary, area);
    }

    fn render_thread_list(frame: &mut Frame, area: Rect, store: &MetricsStore) {
        let threads = &store.thread_snapshot;

        let header = Row::new(vec![
            Cell::from("ID").style(Style::default().fg(Color::Yellow)),
            Cell::from("Name").style(Style::default().fg(Color::Yellow)),
            Cell::from("State").style(Style::default().fg(Color::Yellow)),
            Cell::from("Stack Depth").style(Style::default().fg(Color::Yellow)),
        ])
        .height(1);

        let rows: Vec<Row> = threads
            .iter()
            .take(50)
            .map(|thread| {
                let state_color = match thread.state {
                    ThreadState::Runnable => Color::Green,
                    ThreadState::Blocked => Color::Red,
                    ThreadState::Waiting => Color::Yellow,
                    ThreadState::TimedWaiting => Color::Cyan,
                    ThreadState::Terminated => Color::Gray,
                    ThreadState::New => Color::Blue,
                };

                let state_str = match thread.state {
                    ThreadState::Runnable => "RUNNABLE",
                    ThreadState::Blocked => "BLOCKED",
                    ThreadState::Waiting => "WAITING",
                    ThreadState::TimedWaiting => "TIMED_WAITING",
                    ThreadState::Terminated => "TERMINATED",
                    ThreadState::New => "NEW",
                };

                Row::new(vec![
                    Cell::from(thread.id.to_string()),
                    Cell::from(thread.name.clone()),
                    Cell::from(state_str).style(Style::default().fg(state_color)),
                    Cell::from(thread.stack_trace.len().to_string()),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(6),
                Constraint::Percentage(50),
                Constraint::Length(15),
                Constraint::Length(12),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Thread List (Top 50)"),
        )
        .style(Style::default().fg(Color::White));

        frame.render_widget(table, area);
    }
}
