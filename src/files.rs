use crate::items::TodoItem;
use std::fs;

const TODO_PATH: &str = ".todos";

pub fn write(items: &Vec<TodoItem>) {
    let data = serde_json::to_string(items).expect("serialize json");
    fs::write(TODO_PATH, data).expect("write data");
}

pub fn read() -> Vec<TodoItem> {
    let items = fs::read_to_string(TODO_PATH).unwrap_or("[]".into());
    serde_json::from_str(&items).expect("parse json")
}
