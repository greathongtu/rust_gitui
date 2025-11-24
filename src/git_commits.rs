use ratatui::{
    style::{Color, Style},
    widgets::{self, List, ListItem},
};
use std::process::{Command, Stdio};

pub fn load_commits() -> Vec<String> {
    let output = Command::new("git")
        .args(["log", "--oneline"])
        .output()
        .expect("failed to execute git status command.")
        .stdout;

    String::from_utf8(output)
        .unwrap_or_default()
        .lines()
        .map(|s| s.to_string())
        .collect()
}

pub fn widget(commits: &[String], focused: bool) -> List<'_> {
    let block = widgets::Block::bordered().title("Commits");

    let items: Vec<ListItem> = commits.iter().map(|s| ListItem::new(s.as_str())).collect();
    let list = widgets::List::new(items).block(block);
    if focused {
        return list.highlight_style(Style::new().bg(Color::Yellow));
    }
    list
}

pub fn commit(message: &str) -> std::io::Result<()> {
    if message.trim().is_empty() {
        return Err(std::io::Error::other("commit message cannot be empty"));
    }

    let status = if let Some((subject, body)) = message.split_once('\n') {
        Command::new("git")
            .args(["commit", "-q", "-m", subject.trim(), "-m", body.trim()])
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .status()?
    } else {
        Command::new("git")
            .args(["commit", "-q", "-m", message.trim()])
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .status()?
    };

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("git commit failed"))
    }
}

pub fn parse_commit_hash(line: &str) -> Option<String> {
    let mut parts = line.split_whitespace();
    parts.next().map(|s| s.to_string())
}

pub fn get_head_commit_message() -> std::io::Result<String> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%B"])
        .output()?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout).unwrap_or_default())
    } else {
        Err(std::io::Error::other("git log -1 failed"))
    }
}

pub fn amend_last_no_edit() -> std::io::Result<()> {
    let status = Command::new("git")
        .args(["commit", "--amend", "--no-edit", "-q"])
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("git commit --amend failed"))
    }
}

pub fn checkout_commit(hash: &str) -> std::io::Result<()> {
    if hash.trim().is_empty() {
        return Ok(());
    }
    let status = Command::new("git")
        .args(["checkout", "--quiet", hash.trim()])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("git checkout <hash> failed"))
    }
}

pub fn drop_commit(hash: &str) -> std::io::Result<()> {
    let parent = format!("{}^", hash.trim());
    let status = Command::new("git")
        .args(["rebase", "--quiet", "--onto", &parent, hash.trim()])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("git rebase --onto failed"))
    }
}

pub fn reword_last_commit(message: &str) -> std::io::Result<()> {
    if message.trim().is_empty() {
        return Err(std::io::Error::other("commit message cannot be empty"));
    }

    let status = if let Some((subject, body)) = message.split_once('\n') {
        Command::new("git")
            .args([
                "commit",
                "--allow-empty",
                "--amend",
                "--only",
                "-q",
                "-m",
                subject.trim(),
                "-m",
                body.trim(),
            ])
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .status()?
    } else {
        Command::new("git")
            .args([
                "commit",
                "--allow-empty",
                "--amend",
                "--only",
                "-q",
                "-m",
                message.trim(),
            ])
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .status()?
    };

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("git commit --amend failed"))
    }
}

pub fn reset_to(hash: &str, strength: &str) -> std::io::Result<()> {
    let strength = strength.trim().to_lowercase();
    if !["soft", "mixed", "hard"].contains(&strength.as_str()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid reset mode (must be soft/mixed/hard)",
        ));
    }
    if hash.trim().is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty commit hash",
        ));
    }

    let flag = format!("--{}", strength);
    let status = Command::new("git")
        .args(["reset", &flag, hash.trim(), "--quiet"])
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("git reset failed"))
    }
}
