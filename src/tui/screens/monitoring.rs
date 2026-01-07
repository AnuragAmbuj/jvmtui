use crate::app::{App, Tab};
use crate::metrics::store::MetricsStore;
use crate::tui::views::{
    classes::ClassesView, gc::GcView, memory::MemoryView, overview::OverviewView,
    threads::ThreadsView,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Paragraph, Tabs},
};

pub struct MonitoringScreen;

impl MonitoringScreen {
    pub fn render(frame: &mut Frame, app: &App, store: &MetricsStore) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(frame.area());

        Self::render_header(frame, chunks[0], app);
        Self::render_tabs(frame, chunks[1], app);
        Self::render_content(frame, chunks[2], app, store);
        Self::render_footer(frame, chunks[3], app);
    }

    fn render_header(frame: &mut Frame, area: Rect, app: &App) {
        let header_text = if let Some(jvm_info) = &app.jvm_info {
            format!(
                "PID: {} │ JDK {} │ Uptime: {}h {}m",
                jvm_info.pid,
                jvm_info.version,
                jvm_info.uptime_seconds / 3600,
                (jvm_info.uptime_seconds % 3600) / 60
            )
        } else {
            "Loading JVM info...".to_string()
        };

        let header = Paragraph::new(header_text)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL).title("JVM Info"));

        frame.render_widget(header, area);
    }

    fn render_tabs(frame: &mut Frame, area: Rect, app: &App) {
        let titles: Vec<Line> = Tab::all()
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let num = i + 1;
                let title = format!("{}:{}", num, tab.title());
                if *tab == app.current_tab {
                    Line::from(format!("[{}]", title)).style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Line::from(title).style(Style::default().fg(Color::Gray))
                }
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Views"))
            .divider(" ");

        frame.render_widget(tabs, area);
    }

    fn render_content(frame: &mut Frame, area: Rect, app: &App, store: &MetricsStore) {
        match app.current_tab {
            Tab::Overview => {
                OverviewView::render(frame, area, store);
            }
            Tab::Memory => {
                MemoryView::render(frame, area, store);
            }
            Tab::Threads => {
                ThreadsView::render(frame, area, store);
            }
            Tab::GC => {
                GcView::render(frame, area, store);
            }
            Tab::Classes => {
                ClassesView::render(frame, area, store);
            }
        }
    }

    fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
        let footer_text = match app.current_tab {
            Tab::Overview => {
                "1-5: Switch Tab | h/l: Prev/Next | g: Trigger GC | r: Reset | q: Quit"
            }
            Tab::Memory => "1-5: Switch Tab | h/l: Prev/Next | g: Trigger GC | r: Reset | q: Quit",
            Tab::Threads => "1-5: Switch Tab | h/l: Prev/Next | g: Trigger GC | r: Reset | q: Quit",
            Tab::GC => "1-5: Switch Tab | h/l: Prev/Next | g: Trigger GC | r: Reset | q: Quit",
            Tab::Classes => "1-5: Switch Tab | h/l: Prev/Next | g: Trigger GC | r: Reset | q: Quit",
        };

        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Controls"));

        frame.render_widget(footer, area);
    }
}
