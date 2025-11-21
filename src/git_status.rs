use crate::state::ChangedFile;
use ratatui::{
    style::{Color, Style},
    widgets::{self, List, ListItem},
};
use std::process::Command;

pub fn widget(files: &[ChangedFile], focused: bool) -> List<'_> {
    let block = widgets::Block::bordered().title("Status");

    let items: Vec<ListItem> = files
        .iter()
        .map(|file| {
            let text = format!("{}{} {}", file.x, file.y, file.path);
            ListItem::new(text)
        })
        .collect();

    let list = widgets::List::new(items).block(block);
    if focused {
        return list.highlight_style(Style::new().bg(Color::Yellow));
    }

    list
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

pub fn add_file(path: &str) -> std::io::Result<()> {
    let status = Command::new("git").args(["add", "--", path]).status()?;
    if status.success() {
        return Ok(());
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "git add failed",
        ));
    }
}

pub fn unstage_file(path: &str) -> std::io::Result<()> {
    let status = Command::new("git")
        .args(["restore", "--staged", "--", path])
        .status()?;
    if status.success() {
        return Ok(());
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "git restore staged failed",
        ));
    }
}

pub fn add_all_file() -> std::io::Result<()> {
    let status = Command::new("git").args(["add", "-A"]).status()?;
    if status.success() {
        return Ok(());
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "git add all failed",
        ));
    }
}
