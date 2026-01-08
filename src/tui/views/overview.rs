use crate::metrics::store::MetricsStore;
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
};

pub struct OverviewView;

impl OverviewView {
    pub fn render(frame: &mut Frame, area: Rect, store: &MetricsStore, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Min(0),
            ])
            .split(area);

        Self::render_heap_section(frame, chunks[0], store, theme);
        Self::render_gc_section(frame, chunks[1], store, theme);
        Self::render_summary_section(frame, chunks[2], store, theme);
    }

    fn render_heap_section(frame: &mut Frame, area: Rect, store: &MetricsStore, theme: &Theme) {
        let inner = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        let heap_data: Vec<u64> = store
            .heap_history
            .iter()
            .map(|h| (h.used_bytes / 1024 / 1024) as u64)
            .collect();

        let latest_heap = store.heap_history.iter().last();

        let sparkline_title = if let Some(heap) = latest_heap {
            format!(
                "Heap Usage: {} / {} MB ({:.1}%)",
                heap.used_bytes / 1024 / 1024,
                heap.max_bytes / 1024 / 1024,
                (heap.used_bytes as f64 / heap.max_bytes as f64) * 100.0
            )
        } else {
            "Heap Usage".to_string()
        };

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(sparkline_title),
            )
            .data(&heap_data)
            .style(Style::default().fg(theme.chart_line_primary()));

        frame.render_widget(sparkline, inner[0]);

        if let Some(heap) = latest_heap {
            let ratio = heap.used_bytes as f64 / heap.max_bytes as f64;
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("Heap Gauge"))
                .gauge_style(
                    Style::default()
                        .fg(if ratio > 0.9 {
                            theme.memory_critical()
                        } else if ratio > 0.7 {
                            theme.memory_high()
                        } else {
                            theme.success()
                        })
                        .bg(theme.gauge_background()),
                )
                .ratio(ratio);

            frame.render_widget(gauge, inner[1]);
        }
    }

    fn render_gc_section(frame: &mut Frame, area: Rect, store: &MetricsStore, theme: &Theme) {
        let latest_gc = store.gc_history.iter().last();

        let gc_text = if let Some(gc) = latest_gc {
            format!(
                "Young GC: {} collections ({:.2}s total)\n\
                 Full GC: {} collections ({:.2}s total)\n\
                 Total GC Time: {:.2}s\n\
                 Avg Young GC: {:.2}ms\n\
                 Avg Full GC: {:.2}ms",
                gc.young_gc_count,
                gc.young_gc_time_ms as f64 / 1000.0,
                gc.old_gc_count,
                gc.old_gc_time_ms as f64 / 1000.0,
                (gc.young_gc_time_ms + gc.old_gc_time_ms) as f64 / 1000.0,
                if gc.young_gc_count > 0 {
                    gc.young_gc_time_ms as f64 / gc.young_gc_count as f64
                } else {
                    0.0
                },
                if gc.old_gc_count > 0 {
                    gc.old_gc_time_ms as f64 / gc.old_gc_count as f64
                } else {
                    0.0
                },
            )
        } else {
            "No GC data available".to_string()
        };

        let gc_widget = Paragraph::new(gc_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("GC Statistics"),
            )
            .style(Style::default().fg(theme.text()));

        frame.render_widget(gc_widget, area);
    }

    fn render_summary_section(frame: &mut Frame, area: Rect, store: &MetricsStore, theme: &Theme) {
        let latest_heap = store.heap_history.iter().last();

        let summary_text = if let Some(heap) = latest_heap {
            let metaspace = heap
                .pools
                .iter()
                .find(|p| p.name == "Metaspace")
                .map(|p| {
                    format!(
                        "Metaspace: {} / {} MB",
                        p.used_bytes / 1024 / 1024,
                        p.max_bytes / 1024 / 1024
                    )
                })
                .unwrap_or_else(|| "Metaspace: N/A".to_string());

            format!(
                "Memory Pools:\n\n\
                 {}\n\
                 Total Pools: {}\n\
                 \n\
                 Samples Collected: {} heap, {} GC",
                metaspace,
                heap.pools.len(),
                store.heap_history.len(),
                store.gc_history.len()
            )
        } else {
            "No memory data available".to_string()
        };

        let summary = Paragraph::new(summary_text)
            .block(Block::default().borders(Borders::ALL).title("Memory Pools"))
            .style(Style::default().fg(theme.text()));

        frame.render_widget(summary, area);
    }
}
