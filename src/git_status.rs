use crate::state::ChangedFile;
use ratatui::widgets::{self, ListItem};
use std::process::Command;

pub fn widget(files: &[ChangedFile]) -> impl widgets::Widget {
    let block = widgets::Block::bordered().title("Status");

    let items: Vec<ListItem> = files
        .iter()
        .map(|file| ListItem::new(format!("{}{} {}", file.x, file.y, file.path)))
        .collect();

    widgets::List::new(items).block(block)
}

pub fn load_changed_files() -> std::io::Result<Vec<ChangedFile>> {
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

    let mut changed_files = Vec::new();
    let s = String::from_utf8_lossy(&output);
    let iter = s.split('\0').filter(|x| !x.is_empty());

    for line in iter {
        let mut chars = line.chars();
        let x = chars.next().unwrap_or(' ');
        let y = chars.next().unwrap_or(' ');
        let path = line.get(3..).unwrap_or(" ").trim();
        changed_files.push(ChangedFile {
            x,
            y,
            path: path.to_owned(),
        });
    }

    Ok(changed_files)
}
