use std::process::Command;

use ratatui::widgets::{self, ListItem};

pub fn git_branch() -> impl widgets::Widget {
    let block = widgets::Block::bordered().title("Local Branches");
    let output = Command::new("git")
        .args(["branch"])
        .output()
        .expect("failed to execute git branch command.")
        .stdout;
    let items: Vec<ListItem> = String::from_utf8(output)
        .expect("failed to convert output to String.")
        .split('\0')
        .map(|e| ListItem::new(e.to_string()))
        .collect();

    widgets::List::new(items).block(block)
}
