use std::process::Command;

use ratatui::{
    style::{Color, Style},
    widgets::{self, List, ListItem},
};

pub fn load_branches() -> Vec<String> {
    let output = Command::new("git")
        .args(["branch"])
        .output()
        .expect("failed to execute git branch command.")
        .stdout;

    String::from_utf8(output)
        .unwrap_or_default()
        .lines()
        .map(|s| s.to_string())
        .collect()
}

pub fn widget(branches: &[String], focused: bool) -> List<'_> {
    let block = widgets::Block::bordered().title("Branches");

    let items: Vec<ListItem> = branches.iter().map(|s| ListItem::new(s.as_str())).collect();
    let list = widgets::List::new(items).block(block);
    if focused {
        return list.highlight_style(Style::new().bg(Color::Yellow));
    }
    list
}
