mod tui;
mod items;
use items::{TodoItem, Recursive};
use tui::{Terminal, Key};
use std::{io, fs};

const TODO_PATH: &str = ".todos";

#[derive(PartialEq)]
enum InputState { Normal, Append, Insert, Edit }

struct App {
    state: InputState,
    input: String,
    terminal: Terminal,
    cursor: usize,
    items: Vec<TodoItem>,
}

impl Recursive for App {
    fn items(&self) -> &Vec<TodoItem> { &self.items }
    fn items_mut(&mut self) -> &mut Vec<TodoItem> { &mut self.items }
}

impl App {
    pub fn default() -> Self {
        Self {
            state: InputState::Normal,
            input: String::new(),
            terminal: Terminal::new(),
            cursor: 0,
            items: Vec::new(),
        }
    }

    fn ui(&self) {
        let term = &self.terminal;
        term.reset();

        if self.len() > 1 {
            term.write(&render_items(&self.items(), 1));
        } else {
            term.write("no items!");
        }

        if self.state != InputState::Normal {
            let pos = match self.state {
                InputState::Append => self.len() - 1,
                // InputState::Insert => self.cursor + 1,
                InputState::Edit   => self.cursor,
                _ => 0,
            };
            term.move_to(2, pos);
            term.clear_line();
            term.write(&format!("[ ] {}", self.input));
        } else if self.len() > 1 {
            term.move_to(0, self.cursor);
            term.write("> ");
        }

        term.flush();
    }

    fn input(&mut self, key: Key) {
        if self.state == InputState::Normal {
            match key {
                Key::Char('a') => {
                    self.input = String::new();
                    self.terminal.disable_raw();
                    self.state = InputState::Append;
                },
                // Key::Char('i') => {
                //     self.input = Some(Input::default());
                //     self.state = InputState::Insert;
                // },
                Key::Char('e') => {
                    // self.input = self.items.get(self.cursor).unwrap().name.to_string();
                    // self.terminal.disable_raw();
                    // self.state = InputState::Edit;
                },
                Key::Char('d') => {
                    self.remove(self.cursor);
                    self.cursor = self.cursor.min(self.len().saturating_sub(2));
                    write(&self.items).expect("write data");
                },
                Key::Char('x') => { 
                    self.get_mut(self.cursor).unwrap().done ^= true;
                    write(&self.items).expect("write data");
                },
                Key::Up        => self.cursor = self.cursor.saturating_sub(1),
                Key::Down      => self.cursor = self.cursor.saturating_add(1).min(self.len().saturating_sub(2)),
                _ => {},

            };
        } else {
            match key {
                Key::Char(c)   => { self.input.push(c); },
                Key::Backspace => { self.input.pop(); },
                Key::Enter | Key::Esc => {
                    if key == Key::Enter {
                        let name = self.input.to_string();
                        match self.state {
                            InputState::Append => { self.items.push(TodoItem { name, done: false, req: vec![] }); },
                            InputState::Edit => { self.items.get_mut(self.cursor).unwrap().name = name },
                            _ => {}
                        }
                    }
                    self.state = InputState::Normal;
                    self.terminal.enable_raw();
                    write(&self.items).expect("write data");
                },
                _ => {},
            };
        }
    }

    pub fn run(&mut self) {
        self.items = read();
        self.terminal.enable_raw();
        // terminal.show_cursor().unwrap();
        loop {
            // terminal.draw(|frame| self.ui(frame)).unwrap();
            self.ui();
            if let Some(event) = self.terminal.read() {
                if event == Key::Char('q') { break }
                self.input(event);
            }
        }
        self.terminal.drop();
    }
}

fn state_to_char(state: bool) -> String {
    match state {
        true => "x".into(),
        false => " ".into(),
    }
}

fn render_items(items: &Vec<TodoItem>, depth: usize) -> String {
    fn pad(depth: usize) -> String {
        format!("{: ^1$}", "", depth * 2)
    }

    let vec: Vec<String> = items
        .iter()
        .map(|i| vec![
             format!("{}[{}] {}\r\n", pad(depth), state_to_char(i.done), i.name),
             render_items(&i.req, depth + 2),
        ])
        .flatten()
        .collect();

    vec.join("")
}

fn write(items: &Vec<TodoItem>) -> Result<(), io::Error> {
    fs::write(TODO_PATH, serde_json::to_string(items)?)?;
    Ok(())
}

fn read() -> Vec<TodoItem> {
    let items = fs::read_to_string(TODO_PATH).unwrap_or("[]".into());
    serde_json::from_str(&items).expect("parse json")
}

fn main() {
    let mut app = App::default();
    app.run();
}

