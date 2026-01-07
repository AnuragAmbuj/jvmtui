use crate::jvm::types::GcStats;
use crate::metrics::store::MetricsStore;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
};

pub struct GcView;

impl GcView {
    pub fn render(frame: &mut Frame, area: Rect, store: &MetricsStore) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(12),
                Constraint::Min(0),
            ])
            .split(area);

        Self::render_gc_summary(frame, chunks[0], store);
        Self::render_gc_timeline(frame, chunks[1], store);
        Self::render_gc_stats(frame, chunks[2], store);
    }

    fn render_gc_summary(frame: &mut Frame, area: Rect, store: &MetricsStore) {
        let latest_gc = store.gc_history.iter().last();

        let summary_text = if let Some(gc) = latest_gc {
            let total_gc_time = (gc.young_gc_time_ms + gc.old_gc_time_ms) as f64 / 1000.0;
            let avg_young = if gc.young_gc_count > 0 {
                gc.young_gc_time_ms as f64 / gc.young_gc_count as f64
            } else {
                0.0
            };
            let avg_old = if gc.old_gc_count > 0 {
                gc.old_gc_time_ms as f64 / gc.old_gc_count as f64
            } else {
                0.0
            };

            format!(
                "Total Collections: {}\n\
                 Young GC: {} collections, {:.2}s total (avg {:.2}ms)\n\
                 Full GC: {} collections, {:.2}s total (avg {:.2}ms)\n\
                 \n\
                 Total GC Time: {:.2}s\n\
                 GC Overhead: Calculating...",
                gc.young_gc_count + gc.old_gc_count,
                gc.young_gc_count,
                gc.young_gc_time_ms as f64 / 1000.0,
                avg_young,
                gc.old_gc_count,
                gc.old_gc_time_ms as f64 / 1000.0,
                avg_old,
                total_gc_time
            )
        } else {
            "No GC data available yet...".to_string()
        };

        let summary = Paragraph::new(summary_text)
            .block(Block::default().borders(Borders::ALL).title("GC Summary"))
            .style(Style::default().fg(Color::White));

        frame.render_widget(summary, area);
    }

    fn render_gc_timeline(frame: &mut Frame, area: Rect, store: &MetricsStore) {
        let gc_history: Vec<&GcStats> = store.gc_history.iter().collect();

        if gc_history.is_empty() {
            let placeholder = Paragraph::new("Waiting for GC data...")
                .block(Block::default().borders(Borders::ALL).title("GC Timeline"))
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(placeholder, area);
            return;
        }

        let young_data: Vec<(f64, f64)> = gc_history
            .iter()
            .enumerate()
            .map(|(i, gc)| (i as f64, gc.young_gc_count as f64))
            .collect();

        let old_data: Vec<(f64, f64)> = gc_history
            .iter()
            .enumerate()
            .map(|(i, gc)| (i as f64, gc.old_gc_count as f64))
            .collect();

        let max_young = gc_history
            .iter()
            .map(|gc| gc.young_gc_count)
            .max()
            .unwrap_or(1) as f64;

        let max_old = gc_history
            .iter()
            .map(|gc| gc.old_gc_count)
            .max()
            .unwrap_or(1) as f64;

        let max_count = max_young.max(max_old).max(10.0);

        let datasets = vec![
            Dataset::default()
                .name("Young GC")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Cyan))
                .data(&young_data),
            Dataset::default()
                .name("Full GC")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Red))
                .data(&old_data),
        ];

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("GC Event Timeline"),
            )
            .x_axis(
                Axis::default()
                    .title("Samples")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, gc_history.len() as f64]),
            )
            .y_axis(
                Axis::default()
                    .title("Count")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, max_count]),
            );

        frame.render_widget(chart, area);
    }

    fn render_gc_stats(frame: &mut Frame, area: Rect, store: &MetricsStore) {
        let gc_history: Vec<&GcStats> = store.gc_history.iter().collect();

        if gc_history.is_empty() {
            let placeholder = Paragraph::new("Waiting for GC statistics...")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("GC Statistics"),
                )
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(placeholder, area);
            return;
        }

        let latest = gc_history.last().unwrap();
        let first = gc_history.first().unwrap();

        let young_diff = latest.young_gc_count.saturating_sub(first.young_gc_count);
        let old_diff = latest.old_gc_count.saturating_sub(first.old_gc_count);
        let young_time_diff = latest
            .young_gc_time_ms
            .saturating_sub(first.young_gc_time_ms);
        let old_time_diff = latest.old_gc_time_ms.saturating_sub(first.old_gc_time_ms);

        let stats_text = format!(
            "Statistics (Last {} samples):\n\
             \n\
             Young GC Events: {} (Δ{})\n\
             Young GC Time: {:.2}s (Δ{:.2}s)\n\
             \n\
             Full GC Events: {} (Δ{})\n\
             Full GC Time: {:.2}s (Δ{:.2}s)\n\
             \n\
             Recent Avg Young GC: {:.2}ms\n\
             Recent Avg Full GC: {:.2}ms",
            gc_history.len(),
            latest.young_gc_count,
            young_diff,
            latest.young_gc_time_ms as f64 / 1000.0,
            young_time_diff as f64 / 1000.0,
            latest.old_gc_count,
            old_diff,
            latest.old_gc_time_ms as f64 / 1000.0,
            old_time_diff as f64 / 1000.0,
            if young_diff > 0 {
                young_time_diff as f64 / young_diff as f64
            } else {
                0.0
            },
            if old_diff > 0 {
                old_time_diff as f64 / old_diff as f64
            } else {
                0.0
            }
        );

        let stats = Paragraph::new(stats_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("GC Statistics"),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(stats, area);
    }
}
