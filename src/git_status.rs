use ratatui::widgets::{self, ListItem};
use std::process::Command;

pub fn changed_files_widget() -> impl widgets::Widget {
    let left_top_block = widgets::Block::bordered().title("Changed Files");
    let output = Command::new("git")
        .args([
            "status",
            "--porcelain",
            "-z",
            "--untracked-files=all",
            "--find-renames=50%",
        ])
        .output()
        .expect("failed to execute git status command.")
        .stdout;
    let items: Vec<ListItem> = String::from_utf8(output)
        .expect("failed to convert output to String.")
        .split('\0')
        .map(|e| ListItem::new(e.to_string()))
        .collect();

    widgets::List::new(items).block(left_top_block)
}
