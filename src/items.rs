use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct TodoItem {
    pub name: String,
    pub done: bool,
    pub req: Vec<TodoItem>,
}

pub trait Recursive {
    fn items(&self) -> &Vec<TodoItem>;
    fn items_mut(&mut self) -> &mut Vec<TodoItem>;

    fn len(&self) -> usize {
        1 + self.items().iter().fold(0, |sum, i| sum + i.len())
    }

    fn get(&self, index: usize) -> Option<&TodoItem> {
        let mut pos = 0;
        for item in self.items() {
            if pos == index {
                return Some(item);
            }
            pos += 1;

            let len = item.len();
            let found = item.get(index.saturating_sub(pos));
            if found.is_some() {
                return found;
            }
            pos += len - 1;
        }
        None
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut TodoItem> {
        let mut pos = 0;
        for item in self.items_mut() {
            if pos == index {
                return Some(item);
            }
            pos += 1;

            let len = item.len();
            let found = item.get_mut(index.saturating_sub(pos));
            if found.is_some() {
                return found;
            }
            pos += len - 1;
        }
        None
    }

    fn remove(&mut self, index: usize) -> bool {
        let mut pos = 0;
        for (i, item) in self.items_mut().iter_mut().enumerate() {
            if pos == index {
                self.items_mut().remove(i);
                return true;
            }
            pos += 1;

            let len = item.len();
            if item.remove(index.saturating_sub(pos)) {
                return true;
            }
            pos += len - 1;
        }
        false
    }
}

impl Recursive for TodoItem {
    fn items(&self) -> &Vec<TodoItem> { &self.req }
    fn items_mut(&mut self) -> &mut Vec<TodoItem> { &mut self.req }
}

