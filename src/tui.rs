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

    pub fn move_to(&self, x: usize, y: usize) {
        execute!(stdout(), cursor::MoveTo(x as u16, y as u16)).expect("move cursor");
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

    pub fn reset(&self) {
        execute!(
            stdout(),
            cursor::MoveTo(0, 0),
            term::Clear(term::ClearType::FromCursorDown),
        ).expect("reset");
    }

    pub fn write(&self, data: &str) {
        print!("{}", data);
    }

    pub fn clear_line(&self) {
        execute!(stdout(), term::Clear(term::ClearType::CurrentLine)).expect("clear line");
    }

    pub fn flush(&self) {
        stdout().flush().expect("flush stdout");
    }
}

