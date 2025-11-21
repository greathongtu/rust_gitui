use ratatui::widgets::{self, List, ListItem};
use std::process::Command;

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

pub fn widget(commits: &[String]) -> List<'_> {
    let block = widgets::Block::bordered().title("Commits");

    let items: Vec<ListItem> = commits.iter().map(|s| ListItem::new(s.as_str())).collect();
    widgets::List::new(items)
        .highlight_style(ratatui::style::Color::Green)
        .block(block)
}
