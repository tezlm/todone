use crossterm::event::{read, Event, KeyEvent, KeyCode};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use serde::{Serialize, Deserialize};
use tui::{Terminal, backend::CrosstermBackend};
use tui::{widgets::*, layout::*};
use std::{io, fs};

const TODO_PATH: &str = ".todos";

type Term = Terminal<CrosstermBackend<io::Stdout>>;

#[derive(Serialize, Deserialize, Debug)]
struct TodoItem {
    name: String,
    done: bool,
    req: Vec<TodoItem>,
}

impl TodoItem {
    fn len(&self) -> usize {
        self.req
            .iter()
            .fold(1, |a, i| a + i.len())
    }

    fn remove(&mut self, index: usize, a: bool) -> bool {
        let mut i = 0;
        let mut pos = 0;
        for item in &mut self.req {
            if pos == index { 
                self.req.remove(i);
                return true;
            }
            if item.remove(index - i - 1, true) {
                return true
            }
            i += 1;
            pos += item.len();
        }
        false
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");
    execute!(io::stdout(), EnterAlternateScreen)?;

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let items = fs::read_to_string(TODO_PATH).unwrap_or("[]".into());
    let items: Vec<TodoItem> = serde_json::from_str(&items)?;
    let mut items = TodoItem { done: false, name: "dummy".into(), req: items };
    let mut selected: usize = 0;

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            if items.len() > 0 {
                let list = List::new(render_items(&items.req, 0))
                    .highlight_symbol("> ");

                let mut state = ListState::default();
                state.select(Some(selected));
                rect.render_stateful_widget(list, size, &mut state);
            } else {
                rect.render_widget(Paragraph::new("no items!"), size);
            }
        })?;

        let event = read()?;
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char('q') => break,
                KeyCode::Char('a') => {
                    items.req.push(TodoItem { name: "arst".into(), done: false, req: vec![] });
                },
                KeyCode::Char('e') => {},
                KeyCode::Char('d') => {
                    items.remove(selected, false);
                    // selected = selected.min(items.len().saturating_sub(1));
                },
                KeyCode::Char('x') => { 
                    items.req[selected].done ^= true;
                    write(&items.req)?;
                },
                KeyCode::Up        => selected = selected.saturating_sub(1),
                KeyCode::Down      => selected = selected.saturating_add(1).min(items.len().saturating_sub(2)),
                KeyCode::Char('?') => {
                    help(&mut terminal)?;
                    read()?;
                },
                _ => {},
            }
        }
    }

    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

