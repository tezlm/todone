pub use crossterm::event::KeyCode as Key;
use crossterm::event::{read, Event, KeyEvent};
use crossterm::{terminal as term, cursor, execute};
use std::io::{stdout, Write};

pub struct Terminal {}

impl Terminal {
    pub fn new() -> Self {
        execute!(stdout(), term::EnterAlternateScreen).expect("enter alternate screen");
        let term = Self {};
        term.enable_raw();
        term.hide_cursor();
        term
    }

    pub fn drop(&self) {
        self.disable_raw();
        self.show_cursor();
        execute!(stdout(), term::LeaveAlternateScreen).expect("leave alternate screen");
    }

    pub fn enable_raw(&self)  { term::enable_raw_mode().expect("enter raw mode") }
    pub fn disable_raw(&self) { term::disable_raw_mode().expect("leave raw mode") }
    pub fn show_cursor(&self) { execute!(stdout(), cursor::Show).expect("show cursor") }
    pub fn hide_cursor(&self) { execute!(stdout(), cursor::Hide).expect("hide cursor") }
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

pub struct Input {
    cursor: usize,
    data: String,
}

impl Input {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            data: String::new(),
        }
    }

    pub fn from(data: &str) -> Self {
        Self {
            cursor: data.len(),
            data: data.into(),
        }
    }

    fn cur_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn cur_right(&mut self) {
        if self.cursor < self.data.len() {
            self.cursor += 1;
        }
    }

    pub fn handle(&mut self, key: Key) {
        match key {
            Key::Char(c) => {
                self.data.insert(self.cursor, c);
                self.cur_right();
            },
            Key::Backspace => {
                if self.data.len() == 0 { return }
                self.data.remove(self.cursor - 1);
                self.cur_left();
            },
            Key::Left  => { self.cur_left() },
            Key::Right => { self.cur_right() },
            _ => {},
        };
    }

    pub fn data(&self)   -> String { self.data.clone() }
    pub fn cursor(&self) -> usize  { self.cursor }
}

