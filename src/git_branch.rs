use std::{
    io,
    process::{Command, Stdio},
};

use ratatui::{
    style::{Color, Style},
    widgets::{self, List, ListItem},
};

use crate::state::BranchInfo;

pub fn load_branches() -> Vec<BranchInfo> {
    let current = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let output = Command::new("git")
        .args([
            "for-each-ref",
            "--format=%(refname:short)\t%(upstream:short)\t%(upstream:track)",
            "refs/heads",
        ])
        .output()
        .expect("failed to execute git for-each-ref command.")
        .stdout;

    let s = String::from_utf8_lossy(&output);
    let mut res = Vec::new();

    for line in s.lines() {
        let mut parts = line.split('\t');
        let name = parts.next().unwrap_or("").to_string();
        let _upstream = parts.next().unwrap_or("");
        let track = parts.next().unwrap_or("");

        let mut ahead: u32 = 0;
        let mut behind: u32 = 0;

        // "[ahead 3]" "[behind 2]" "[ahead 3, behind 2]"
        if let Some(pos) = track.find("ahead ") {
            let num = track[pos + 6..]
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .unwrap_or("0");
            ahead = num.parse().unwrap_or(0);
        }
        if let Some(pos) = track.find("behind ") {
            let num = track[pos + 7..]
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .unwrap_or("0");
            behind = num.parse().unwrap_or(0);
        }

        res.push(BranchInfo {
            name: name.clone(),
            ahead,
            behind,
            is_current: !current.is_empty() && current == name,
        });
    }

    res
}

pub fn widget(branches: &[BranchInfo], focused: bool) -> List<'_> {
    let block = widgets::Block::bordered().title("Branches");

    let items: Vec<ListItem> = branches
        .iter()
        .map(|b| {
            let mut label = String::new();
            if b.is_current {
                label.push('*');
                label.push(' ');
            }
            label.push_str(&b.name);
            let mut counters = Vec::new();
            if b.ahead > 0 {
                counters.push(format!("↑{}", b.ahead));
            }
            if b.behind > 0 {
                counters.push(format!("↓{}", b.behind));
            }
            if !counters.is_empty() {
                label.push(' ');
                label.push_str(&counters.join(" "));
            }
            ListItem::new(label)
        })
        .collect();

    let list = widgets::List::new(items).block(block);
    if focused {
        return list.highlight_style(Style::new().bg(Color::Yellow));
    }
    list
}

pub fn checkout_branch(raw_name: &str) -> std::io::Result<()> {
    let name = normalize_branch_name(raw_name);
    if name.is_empty() {
        return Ok(());
    }
    let status = Command::new("git")
        .args(["checkout", "--quiet", &name])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "git checkout {} failed",
            name
        )))
    }
}

pub fn normalize_branch_name(s: &str) -> String {
    s.trim().trim_start_matches('*').trim().to_string()
}

pub fn checkout_or_create_branch(input: &str) -> io::Result<()> {
    let name = input.trim();
    if name.is_empty() {
        return Ok(());
    }

    if local_branch_exists(name)? {
        return checkout_local(name);
    }

    if let Some(remote) = find_remote_for_branch(name)? {
        if checkout_tracking(&remote).is_err() {
            if let Some((remote_name, short)) = split_remote_ref(&remote) {
                let _ = Command::new("git")
                    .args(["fetch", remote_name, &format!("{}:{}", short, short)])
                    .stderr(Stdio::null())
                    .stdout(Stdio::null())
                    .status();
            } else {
                let _ = Command::new("git")
                    .args(["fetch", "--all", "--prune", "--quiet"])
                    .stderr(Stdio::null())
                    .stdout(Stdio::null())
                    .status();
            }
            checkout_tracking(&remote)?;
        }
        return Ok(());
    }

    create_branch(name)
}

fn checkout_local(name: &str) -> io::Result<()> {
    let status = Command::new("git")
        .args(["checkout", "--quiet", name])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("git checkout {} failed", name)))
    }
}

fn create_branch(name: &str) -> io::Result<()> {
    let status = Command::new("git")
        .args(["checkout", "-b", name])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("git checkout -b {} failed", name)))
    }
}

fn checkout_tracking(remote_ref_short: &str) -> io::Result<()> {
    let status = Command::new("git")
        .args(["checkout", "--track", remote_ref_short])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "git checkout --track {} failed",
            remote_ref_short
        )))
    }
}

fn local_branch_exists(name: &str) -> io::Result<bool> {
    let status = Command::new("git")
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{}", name),
        ])
        .status()?;
    Ok(status.success())
}

fn find_remote_for_branch(name: &str) -> io::Result<Option<String>> {
    let output = Command::new("git")
        .args(["for-each-ref", "--format=%(refname:short)", "refs/remotes"])
        .output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let s = String::from_utf8_lossy(&output.stdout);
    // origin/main
    // origin/develop
    // upstream/main
    let mut candidates: Vec<&str> = s
        .lines()
        .filter(|line| line.ends_with(&format!("/{}", name)))
        .collect();

    if candidates.is_empty() {
        return Ok(None);
    }

    candidates.sort_by_key(|r| if r.starts_with("origin/") { 0 } else { 1 });
    Ok(candidates.first().map(|r| r.to_string()))
}

fn split_remote_ref(remote_ref: &str) -> Option<(&str, &str)> {
    let (remote, short) = remote_ref.split_once('/')?;
    Some((remote, short))
}

pub fn merge_branch(raw_target: &str) -> io::Result<()> {
    let target = normalize_branch_name(raw_target);
    if target.is_empty() {
        return Ok(());
    }
    let status = Command::new("git")
        .args(["merge", "--no-edit", &target])
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("git merge {} failed", target)))
    }
}

pub fn rebase_onto_branch(raw_target: &str) -> io::Result<()> {
    let target = normalize_branch_name(raw_target);
    if target.is_empty() {
        return Ok(());
    }
    let status = Command::new("git")
        .args(["rebase", &target])
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("git rebase {} failed", target)))
    }
}

pub fn has_conflicts() -> io::Result<bool> {
    let output = Command::new("git").args(["ls-files", "-u"]).output()?;
    if output.status.success() {
        return Ok(!output.stdout.is_empty());
    }

    let status_out = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;
    let s = String::from_utf8_lossy(&status_out.stdout);
    let conflicted = s.lines().any(|line| {
        let mut chars = line.chars();
        let x = chars.next().unwrap_or(' ');
        let y = chars.next().unwrap_or(' ');
        x == 'U' || y == 'U'
    });

    Ok(conflicted)
}
