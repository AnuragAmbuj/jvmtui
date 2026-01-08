use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct Theme;

impl Theme {
    pub fn new() -> Self {
        Self
    }

    pub fn primary(&self) -> Color {
        Color::Cyan
    }

    pub fn secondary(&self) -> Color {
        Color::Yellow
    }

    pub fn text(&self) -> Color {
        Color::Reset
    }

    pub fn text_dim(&self) -> Color {
        Color::Indexed(8)
    }

    pub fn background(&self) -> Color {
        Color::Reset
    }

    pub fn success(&self) -> Color {
        Color::Green
    }

    pub fn warning(&self) -> Color {
        Color::Yellow
    }

    pub fn error(&self) -> Color {
        Color::Red
    }

    pub fn info(&self) -> Color {
        Color::Cyan
    }

    pub fn border(&self) -> Color {
        Color::Indexed(8)
    }

    pub fn border_focused(&self) -> Color {
        Color::Cyan
    }

    pub fn highlight(&self) -> Color {
        Color::Yellow
    }

    pub fn gauge_filled(&self) -> Color {
        Color::Cyan
    }

    pub fn gauge_background(&self) -> Color {
        Color::Reset
    }

    pub fn thread_state_runnable(&self) -> Color {
        Color::Green
    }

    pub fn thread_state_blocked(&self) -> Color {
        Color::Red
    }

    pub fn thread_state_waiting(&self) -> Color {
        Color::Yellow
    }

    pub fn thread_state_timed_waiting(&self) -> Color {
        Color::Cyan
    }

    pub fn thread_state_terminated(&self) -> Color {
        Color::Indexed(8)
    }

    pub fn thread_state_new(&self) -> Color {
        Color::Blue
    }

    pub fn memory_critical(&self) -> Color {
        Color::Red
    }

    pub fn memory_high(&self) -> Color {
        Color::Yellow
    }

    pub fn memory_normal(&self) -> Color {
        Color::Reset
    }

    pub fn chart_line_primary(&self) -> Color {
        Color::Cyan
    }

    pub fn chart_line_secondary(&self) -> Color {
        Color::Red
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}
