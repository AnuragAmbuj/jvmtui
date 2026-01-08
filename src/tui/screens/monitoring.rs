use crate::app::{App, AppMode, Tab};
use crate::metrics::store::MetricsStore;
use crate::tui::views::{
    classes::ClassesView, gc::GcView, memory::MemoryView, overview::OverviewView,
    threads::ThreadsView,
};
use crate::tui::widgets::{
    confirmation_dialog::ConfirmationDialog, error_screen::ErrorScreen, help_overlay::HelpOverlay,
    loading_screen::LoadingScreen, search_bar::SearchBar,
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

        match &app.mode {
            AppMode::Help => {
                HelpOverlay::render(frame, frame.area(), &app.theme);
            }
            AppMode::ConfirmGc => {
                ConfirmationDialog::render(
                    frame,
                    frame.area(),
                    "Trigger Garbage Collection",
                    "Are you sure you want to trigger a garbage collection?\n\nThis may pause the JVM briefly.",
                    &app.theme,
                );
            }
            AppMode::ConfirmExport => {
                let message = match app.current_tab {
                    Tab::Threads => "Export thread dump to file?",
                    _ => "Export current metrics to JSON file?",
                };
                ConfirmationDialog::render(frame, frame.area(), "Export Data", message, &app.theme);
            }
            AppMode::ExportSuccess(path) => {
                ConfirmationDialog::render(
                    frame,
                    frame.area(),
                    "Export Successful",
                    &format!("Data exported to:\n\n{}\n\nPress Enter to continue", path),
                    &app.theme,
                );
            }
            AppMode::Error(message) => {
                ErrorScreen::render(frame, frame.area(), message, &app.theme);
            }
            AppMode::Loading(message) => {
                LoadingScreen::render(frame, frame.area(), message, &app.theme);
            }
            AppMode::Search => {
                SearchBar::render(
                    frame,
                    frame.area(),
                    &app.search_query,
                    app.search_results.len(),
                    app.search_index,
                    &app.theme,
                );
            }
            AppMode::Normal => {}
        }
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
                    .fg(app.theme.primary())
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
                            .fg(app.theme.highlight())
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Line::from(title).style(Style::default().fg(app.theme.text_dim()))
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
                OverviewView::render(frame, area, store, &app.theme);
            }
            Tab::Memory => {
                MemoryView::render(frame, area, store, &app.theme);
            }
            Tab::Threads => {
                ThreadsView::render_with_scroll(frame, area, store, app.scroll_offset, &app.theme);
            }
            Tab::GC => {
                GcView::render(frame, area, store, &app.theme);
            }
            Tab::Classes => {
                ClassesView::render_with_scroll(frame, area, store, app.scroll_offset, &app.theme);
            }
        }
    }

    fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
        let footer_text = match app.current_tab {
            Tab::Overview => {
                "1-5: Switch Tab | h/l/←/→: Prev/Next | g: Trigger GC | r: Reset | ?: Help | q: Quit"
            }
            Tab::Memory => {
                "1-5: Switch Tab | h/l/←/→: Prev/Next | g: Trigger GC | r: Reset | ?: Help | q: Quit"
            }
            Tab::Threads => {
                "1-5: Switch Tab | j/k/↑/↓: Scroll | g: Trigger GC | r: Reset | ?: Help | q: Quit"
            }
            Tab::GC => {
                "1-5: Switch Tab | h/l/←/→: Prev/Next | g: Trigger GC | r: Reset | ?: Help | q: Quit"
            }
            Tab::Classes => {
                "1-5: Switch Tab | j/k/↑/↓: Scroll | g: Trigger GC | r: Reset | ?: Help | q: Quit"
            }
        };

        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(app.theme.text_dim()))
            .block(Block::default().borders(Borders::ALL).title("Controls"));

        frame.render_widget(footer, area);
    }
}
