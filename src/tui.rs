pub use crossterm::event::KeyCode as Key;
use crossterm::event::{read, Event, KeyEvent};
use crossterm::{terminal as term, cursor, execute};
use std::io::{stdout, Write};

pub struct Terminal {}

impl Terminal {
    pub fn new() -> Self {
        execute!(stdout(), term::EnterAlternateScreen).expect("enter alternate screen");
        Self {}
    }

    pub fn drop(&self) {
        self.disable_raw();
        execute!(stdout(), term::LeaveAlternateScreen).expect("leave alternate screen");
    }

    pub fn enable_raw(&self) {
        execute!(stdout(), cursor::Hide).expect("hide cursor");
        term::enable_raw_mode().expect("enter raw mode");
    }

    pub fn disable_raw(&self) {
        execute!(stdout(), cursor::Show).expect("show cursor");
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

    pub fn write_at(&self, data: &str, pos: (u16, u16)) {
        execute!(stdout(), cursor::MoveTo(pos.0, pos.1)).expect("move cursor");
        print!("{}", data);
    }

    pub fn flush(&self) {
        stdout().flush().expect("flush stdout");
    }
}

