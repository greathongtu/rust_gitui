use ratatui::widgets::{self, ListItem};
use std::process::Command;

pub fn git_diff() -> impl widgets::Widget {
    let left_top_block = widgets::Block::bordered().title("Diff");
    let output = Command::new("git")
        .args(["diff"])
        .output()
        .expect("failed to execute git status command.")
        .stdout;
    let items: Vec<ListItem> = String::from_utf8(output)
        .expect("failed to convert output to String.")
        .lines()
        .map(|e| ListItem::new(e.to_string()))
        .collect();

    widgets::List::new(items).block(left_top_block)
}
