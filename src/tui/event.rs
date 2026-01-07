use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Quit,
    Tab(usize),
    NextTab,
    PrevTab,
    Up,
    Down,
    Enter,
    Refresh,
    Help,
    None,
}

pub fn poll_event(timeout: Duration) -> std::io::Result<Event> {
    if event::poll(timeout)? {
        if let CrosstermEvent::Key(key) = event::read()? {
            return Ok(map_key_event(key));
        }
    }
    Ok(Event::None)
}

fn map_key_event(key: KeyEvent) -> Event {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => Event::Quit,
        (KeyCode::Char('1'), _) => Event::Tab(0),
        (KeyCode::Char('2'), _) => Event::Tab(1),
        (KeyCode::Char('3'), _) => Event::Tab(2),
        (KeyCode::Char('4'), _) => Event::Tab(3),
        (KeyCode::Char('5'), _) => Event::Tab(4),
        (KeyCode::Char('l'), _) | (KeyCode::Tab, _) => Event::NextTab,
        (KeyCode::Char('h'), _) | (KeyCode::BackTab, _) => Event::PrevTab,
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => Event::Up,
        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => Event::Down,
        (KeyCode::Enter, _) => Event::Enter,
        (KeyCode::Char('r'), _) => Event::Refresh,
        (KeyCode::Char('?'), _) => Event::Help,
        _ => Event::None,
    }
}
