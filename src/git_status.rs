use crate::state::ChangedFile;
use ratatui::{
    style::{Color, Style},
    widgets::{self, List, ListItem},
};
use std::process::{Command, Stdio};

pub fn widget(files: &[ChangedFile], focused: bool) -> List<'_> {
    let block = widgets::Block::bordered().title("Status");

    let items: Vec<ListItem> = files
        .iter()
        .map(|file| {
            let text = format!("{}{} {}", file.x, file.y, file.path);

            let style = if is_staged_index_code(file.x) {
                Style::new().fg(Color::Green)
            } else if file.y != ' ' {
                Style::new().fg(Color::Red)
            } else {
                Style::new()
            };
            ListItem::new(text).style(style)
        })
        .collect();

    let list = widgets::List::new(items).block(block);
    if focused {
        return list.highlight_style(Style::new().bg(Color::Yellow));
    }

    list
}

pub fn is_staged_index_code(x: char) -> bool {
    matches!(x, 'A' | 'M' | 'D' | 'R' | 'C' | 'U')
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

    changed_files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(changed_files)
}

pub fn add_file(path: &str) -> std::io::Result<()> {
    let status = Command::new("git").args(["add", "--", path]).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("git add failed"))
    }
}

pub fn unstage_file(path: &str) -> std::io::Result<()> {
    let status = Command::new("git")
        .args(["restore", "--staged", "--", path])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("git restore staged failed"))
    }
}

pub fn add_all_file() -> std::io::Result<()> {
    let status = Command::new("git").args(["add", "-A"]).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("git add all failed"))
    }
}

pub fn unstage_all_file() -> std::io::Result<()> {
    let status = std::process::Command::new("git")
        .args(["reset", "--quiet"])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("git reset (unstage all) failed"))
    }
}

pub fn pull() -> std::io::Result<()> {
    let status = Command::new("git")
        .args(["pull"])
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("git pull failed"))
    }
}

pub fn push() -> std::io::Result<()> {
    let status = Command::new("git")
        .args(["push"])
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("git push failed"))
    }
}

pub fn force_push() -> std::io::Result<()> {
    let status = Command::new("git")
        .args(["push", "--force-with-lease"])
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("git push --force-with-lease failed"))
    }
}
