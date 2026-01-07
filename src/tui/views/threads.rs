use ratatui::{
    layout::Rect,
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub struct ThreadsView;

impl ThreadsView {
    pub fn render(frame: &mut Frame, area: Rect) {
        let content = Paragraph::new(
            "Thread View\n\n\
             Thread state summary and stack traces will be displayed here.\n\n\
             Features:\n\
             - Thread count by state (Runnable, Blocked, Waiting, etc.)\n\
             - Thread list with states\n\
             - Expandable stack traces\n\n\
             (Full implementation requires Thread.print parsing)",
        )
        .block(Block::default().borders(Borders::ALL).title("Threads"))
        .style(Style::default().fg(Color::White));

        frame.render_widget(content, area);
    }
}
