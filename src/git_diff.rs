use ratatui::{
    style::{Color, Style},
    widgets::{self, List, ListItem},
};
use std::process::Command;

pub fn load_diff() -> String {
    let output = Command::new("git")
        .args(["diff"])
        .output()
        .expect("failed to execute git status command.")
        .stdout;

    String::from_utf8(output).unwrap_or_default()
}

pub fn widget(diff: &str, focused: bool) -> List<'_> {
    let block = widgets::Block::bordered().title("Diff");

    let items: Vec<ListItem> = diff.lines().map(|e| ListItem::new(e.to_string())).collect();

    let list = widgets::List::new(items).block(block);
    if focused {
        return list.highlight_style(Style::new().bg(Color::Yellow));
    }
    list
}
