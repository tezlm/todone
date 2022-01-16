mod tui;
use tui::{Terminal, Key};
use serde::{Serialize, Deserialize};
use std::{io, fs};

const TODO_PATH: &str = ".todos";

#[derive(PartialEq)]
enum InputState { Normal, Append, Insert, Edit }

struct App {
    state: InputState,
    terminal: Terminal,
    cursor: usize,
    items: Vec<TodoItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TodoItem {
    name: String,
    done: bool,
    req: Vec<TodoItem>,
}

trait Recursive {
    fn items(&self) -> &Vec<TodoItem>;
    fn mut_items(&mut self) -> &mut Vec<TodoItem>;

    fn len(&self) -> usize {
        1 + self.items().iter().fold(0, |sum, i| sum + i.len())
    }

    fn get(&self, index: usize) -> Option<&TodoItem> {
        let mut pos = 0;
        for i in self.items() {
            if pos == index {
                return Some(&i);
            }

            let len = i.len();
            pos += len - 1;

            let found = i.get(index.saturating_sub(pos));
            if found.is_some() {
                return found;
            }
            pos += 1;
        }
        None
    }

    fn mut_get(&mut self, index: usize) -> Option<&mut TodoItem> {
        let mut pos = 0;
        for i in self.mut_items() {
            if pos == index {
                return Some(i);
            }

            let len = i.len();
            pos += len - 1;

            let found = i.mut_get(index.saturating_sub(pos));
            if found.is_some() {
                return found;
            }
            pos += 1;
        }
        None
    }

    fn remove(&mut self, index: usize) -> bool {
        let mut pos = 0;
        for (i, node) in self.mut_items().iter_mut().enumerate() {
            if pos == index {
                self.mut_items().remove(i);
                return true;
            }

            let len = node.len();
            pos += len - 1;

            if node.remove(index.saturating_sub(pos)) {
                return true;
            }
            pos += 1;
        }
        return false;
    }
}

impl Recursive for TodoItem {
    fn items(&self) -> &Vec<TodoItem> { &self.req }
    fn mut_items(&mut self) -> &mut Vec<TodoItem> { &mut self.req }
}

impl Recursive for App {
    fn items(&self) -> &Vec<TodoItem> { &self.items }
    fn mut_items(&mut self) -> &mut Vec<TodoItem> { &mut self.items }
}

impl App {
    pub fn default() -> Self {
        Self {
            terminal: Terminal::new(),
            cursor: 0,
            state: InputState::Normal,
            items: Vec::new(),
        }
    }

    fn ui(&self) {
        if self.state != InputState::Normal {
            // f.set_cursor(1 + input_width, 1);
        }

        if self.len() > 1 {
            self.terminal.write(&render_items(&self.items(), 1));
            self.terminal.write_at("> ", (0, self.cursor as u16));
            self.terminal.flush();
        } else {
            self.terminal.write("no items!");
            self.terminal.flush();
        }
    }

    fn input(&mut self, key: Key) {
        // if self.state == InputState::Normal {
        match key {
            Key::Char('a') => {
                // self.input = Some(Input::default());
                // self.state = InputState::Append;
            },
            // Key::Char('i') => {
            //     self.input = Some(Input::default());
            //     self.state = InputState::Insert;
            // },
            Key::Char('e') => {},
            Key::Char('d') => {
                self.remove(self.cursor);
                self.cursor = self.cursor.min(self.len().saturating_sub(2));
            },
            Key::Char('x') => { 
                self.mut_get(self.cursor).unwrap().done ^= true;
                write(&self.items).expect("write data");
            },
            Key::Up        => self.cursor = self.cursor.saturating_sub(1),
            Key::Down      => self.cursor = self.cursor.saturating_add(1).min(self.len().saturating_sub(2)),
            _ => {},

        }
        // } else {
        //     let input = self.input.as_mut().unwrap();
        //     if let Some(res) = to_input_request(event).and_then(|r| input.handle(r)) {
        //         match res {
        //             InputResponse::Submitted => {
        //                 self.items.push(TodoItem { name: input.value().to_string(), done: false, req: vec![] });
        //                 self.state = InputState::Normal;
        //             },
        //             InputResponse::Escaped => {
        //                 self.state = InputState::Normal;
        //             },
        //             _ => {},
        //         };
        //     }
        // }
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

