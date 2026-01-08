use crate::metrics::store::MetricsStore;
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
};

pub struct MemoryView;

impl MemoryView {
    pub fn render(frame: &mut Frame, area: Rect, store: &MetricsStore, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(0)])
            .split(area);

        Self::render_heap_sparkline(frame, chunks[0], store, theme);
        Self::render_memory_pools(frame, chunks[1], store, theme);
    }

    fn render_heap_sparkline(frame: &mut Frame, area: Rect, store: &MetricsStore, theme: &Theme) {
        let heap_data: Vec<u64> = store
            .heap_history
            .iter()
            .map(|h| (h.used_bytes / 1024 / 1024) as u64)
            .collect();

        let max_heap = heap_data.iter().max().copied().unwrap_or(1);

        let latest_heap = store.heap_history.iter().last();
        let sparkline_title = if latest_heap.is_some() {
            format!("Heap Usage Timeline (max: {} MB)", max_heap)
        } else {
            "Heap Usage Timeline".to_string()
        };

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(sparkline_title),
            )
            .data(&heap_data)
            .max(max_heap)
            .style(Style::default().fg(theme.chart_line_primary()));

        frame.render_widget(sparkline, area);
    }

    fn render_memory_pools(frame: &mut Frame, area: Rect, store: &MetricsStore, theme: &Theme) {
        let latest_heap = store.heap_history.iter().last();

        if let Some(heap) = latest_heap {
            let pool_count = heap.pools.len();
            let constraints: Vec<Constraint> =
                (0..pool_count).map(|_| Constraint::Length(4)).collect();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);

            for (i, pool) in heap.pools.iter().enumerate() {
                if i < chunks.len() {
                    let ratio = if pool.max_bytes > 0 {
                        pool.used_bytes as f64 / pool.max_bytes as f64
                    } else {
                        0.0
                    };

                    let gauge_color = if ratio > 0.9 {
                        theme.memory_critical()
                    } else if ratio > 0.7 {
                        theme.memory_high()
                    } else {
                        theme.success()
                    };

                    let label = format!(
                        "{}: {} / {} MB ({:.1}%)",
                        pool.name,
                        pool.used_bytes / 1024 / 1024,
                        pool.max_bytes / 1024 / 1024,
                        ratio * 100.0
                    );

                    let gauge = Gauge::default()
                        .block(Block::default().borders(Borders::ALL))
                        .gauge_style(
                            Style::default()
                                .fg(gauge_color)
                                .bg(theme.gauge_background()),
                        )
                        .label(label)
                        .ratio(ratio);

                    frame.render_widget(gauge, chunks[i]);
                }
            }
        } else {
            let no_data = Paragraph::new("No memory pool data available")
                .block(Block::default().borders(Borders::ALL).title("Memory Pools"));
            frame.render_widget(no_data, area);
        }
    }
}
