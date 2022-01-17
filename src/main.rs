mod tui;
mod items;
mod files;
use items::{TodoItem, Recursive};
use tui::{Terminal, Input, Key};

#[derive(PartialEq)]
enum InputState { Normal, Append, Insert, Edit }

struct App {
    state: InputState,
    input: Input,
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
            input: Input::new(),
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
            term.write(&format!("[ ] {}", self.input.data()));
            term.move_to(6 + self.input.cursor(), pos);
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
                    self.input = Input::new();
                    self.state = InputState::Append;
                    self.terminal.show_cursor();
                },
                // Key::Char('i') => {
                //     self.input = Some(Input::default());
                //     self.state = InputState::Insert;
                // },
                Key::Char('e') => {
                    self.input = Input::from(&self.items.get(self.cursor).unwrap().name);
                    self.state = InputState::Edit;
                    self.terminal.show_cursor();
                },
                Key::Char('d') => {
                    self.remove(self.cursor);
                    self.cursor = self.cursor.min(self.len().saturating_sub(2));
                    files::write(&self.items);
                },
                Key::Char('x') => { 
                    self.get_mut(self.cursor).unwrap().done ^= true;
                    files::write(&self.items);
                },
                Key::Up        => self.cursor = self.cursor.saturating_sub(1),
                Key::Down      => self.cursor = self.cursor.saturating_add(1).min(self.len().saturating_sub(2)),
                _ => {},

            };
        } else {
            self.input.handle(key);
            match key {
                Key::Enter | Key::Esc => {
                    if key == Key::Enter {
                        let name = self.input.data();
                        match self.state {
                            InputState::Append => { self.items.push(TodoItem { name, done: false, req: vec![] }); },
                            InputState::Edit => { self.items.get_mut(self.cursor).unwrap().name = name },
                            _ => {}
                        }
                    }
                    self.state = InputState::Normal;
                    self.terminal.hide_cursor();
                    files::write(&self.items);
                },
                _ => {},
            };
        }
    }

    pub fn run(&mut self) {
        self.items = files::read();
        loop {
            self.ui();
            if let Some(event) = self.terminal.read() {
                if self.state == InputState::Normal && event == Key::Char('q') {
                    break;
                }
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

fn main() {
    let mut app = App::default();
    app.run();
}

