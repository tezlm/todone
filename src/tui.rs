pub use crossterm::event::KeyCode as Key;
use crossterm::event::{read, Event, KeyEvent};
use crossterm::{terminal as term, cursor, execute};
use std::io::stdout;

pub struct Terminal {
    // buffer: String,
}

impl Terminal {
    pub fn new() -> Self {
        Self {}
    }

    pub fn enable_raw(&self) {
        execute!(stdout(), term::EnterAlternateScreen).expect("enter alternate screen");
        term::enable_raw_mode().expect("enter raw mode");
    }

    pub fn disable_raw(&self) {
        execute!(stdout(), term::LeaveAlternateScreen).expect("leave alternate screen");
        term::disable_raw_mode().expect("leave raw mode");
    }

    pub fn read(&self) -> Option<Key> {
        if let Ok(event) = read() {
            match event {
                Event::Key(KeyEvent { code, .. }) => Some(code),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn write(&self, data: &str) {
        execute!(
            stdout(),
            cursor::MoveTo(0, 0),
            term::Clear(term::ClearType::FromCursorDown),
        ).expect("clear screen");
        print!("{}", data);
    }
}

