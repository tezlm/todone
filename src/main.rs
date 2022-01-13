use crossterm::event::{read as input, Event, KeyCode};
use crossterm::terminal::*;
use crossterm::execute;
use serde::{Serialize, Deserialize};
use tui::{Terminal, Frame, backend::CrosstermBackend};
use tui::{widgets::*, layout::*};
use tui_input::{Input, InputResponse, backend::crossterm::to_input_request};
use std::{io, fs};

const TODO_PATH: &str = ".todos";

type Term = Terminal<CrosstermBackend<io::Stdout>>;

#[derive(PartialEq)]
enum InputState { Normal, Insert }

struct App {
    state: InputState,
    input: Option<Input>,
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
    fn get_terminal(&self) -> Term {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        Terminal::new(backend).expect("init terminal")
    }

    fn ui<B: tui::backend::Backend>(&self, f: &mut Frame<B>) {
        let size = f.size();
        let list_size = match self.state {
            InputState::Normal => size,
            _ => Rect { y: 3, height: size.height - 3, ..size },
        };

        match self.state {
            InputState::Normal => {},
            _ => {
                let input_size = Rect { height: 3, ..size };
                let input = self.input.as_ref().unwrap().value(); 
                let input_width = input.len() as u16;
                let input = Paragraph::new(input.to_owned()).block(Block::default().borders(Borders::all()));
                f.render_widget(input, input_size);
                f.set_cursor(1 + input_width, 1);
            },
        };

        if self.len() > 1 {
            let list = List::new(render_items(&self.items, 0))
                .highlight_symbol("> ");
            let mut state = ListState::default();
            state.select(Some(self.cursor));
            f.render_stateful_widget(list, list_size, &mut state);
        } else {
            f.render_widget(Paragraph::new("no items!"), list_size);
        }
    }

    pub fn default() -> Self {
        Self {
            input: None,
            cursor: 0,
            state: InputState::Normal,
            items: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        enable_raw_mode().expect("can run in raw mode");
        execute!(io::stdout(), EnterAlternateScreen).expect("enter alternate screen");
        self.items = read();
    }

    pub fn run(&mut self) {
        let mut terminal = self.get_terminal();
        terminal.show_cursor().unwrap();
        loop {
            terminal.draw(|frame| self.ui(frame)).unwrap();
            let event = input().expect("read input");
            if self.state == InputState::Normal {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('a') => {
                            self.input = Some(Input::default());
                            self.state = InputState::Insert;
                        },
                        KeyCode::Char('e') => {},
                        KeyCode::Char('d') => {
                            self.remove(self.cursor);
                            self.cursor = self.cursor.min(self.len().saturating_sub(2));
                        },
                        KeyCode::Char('x') => { 
                            self.mut_get(self.cursor).unwrap().done ^= true;
                            write(&self.items).expect("write data");
                        },
                        KeyCode::Up        => self.cursor = self.cursor.saturating_sub(1),
                        KeyCode::Down      => self.cursor = self.cursor.saturating_add(1).min(self.len().saturating_sub(2)),
                        KeyCode::Char('?') => {
                            help(&mut terminal).expect("render help");
                            input().expect("read input");
                        },
                        _ => {},
                    }
                }
            } else {
                let input = self.input.as_mut().unwrap();
                if let Some(res) = to_input_request(event).and_then(|r| input.handle(r)) {
                    match res {
                        InputResponse::Submitted => {
                            self.items.push(TodoItem { name: input.value().to_string(), done: false, req: vec![] });
                            self.state = InputState::Normal;
                        },
                        InputResponse::Escaped => {
                            self.state = InputState::Normal;
                        },
                        _ => {},
                    };
                }
            }
        }
    }

    pub fn cleanup(&self) {
        execute!(io::stdout(), LeaveAlternateScreen).expect("leave alternate screen");
        disable_raw_mode().expect("disable raw mode");
    }
}

fn help(term: &mut Term) -> Result<(), io::Error> {
    let text = vec![
        "q: Quit",
        "?: Help",
        "a: Add item",
        "e: Edit item",
        "d: Delete item",
        "x: Mark/unmark item as done",
    ];
    let block = Block::default()
        .borders(Borders::all())
        .title("help");
    let para = Paragraph::new(text.join("\n"))
        .block(block);
    term.draw(|rect| {
        let size = rect.size();
        let width = 30.max(size.width / 3);
        let height = text.len() as u16 + 2;
        let chunk = Rect {
            x: (size.width - width) / 2,
            y: (size.height - height) / 2,
            width,
            height, 
        };
        rect.render_widget(para, chunk);
    })?;
    Ok(())
}

fn state_to_char(state: bool) -> String {
    match state {
        true => "x".into(),
        false => " ".into(),
    }
}

fn render_items(items: &Vec<TodoItem>, depth: usize) -> Vec<ListItem> {
    fn pad(depth: usize) -> String {
        format!("{: ^1$}", "", depth * 4)
    }

    items
        .iter()
        .map(|i| vec![
             vec![ListItem::new(format!("{}[{}] {}", pad(depth), state_to_char(i.done), i.name))],
             render_items(&i.req, depth + 1)])
        .flatten()
        .flatten()
        .collect()
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
    app.init();
    app.run();
    app.cleanup();
}

