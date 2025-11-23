use std::{
    io,
    process::{Command, Stdio},
};

use ratatui::{
    style::{Color, Style},
    widgets::{self, List, ListItem},
};

pub fn load_branches() -> Vec<String> {
    let output = Command::new("git")
        .args(["branch"])
        .output()
        .expect("failed to execute git branch command.")
        .stdout;

    String::from_utf8(output)
        .unwrap_or_default()
        .lines()
        .map(|s| s.to_string())
        .collect()
}

pub fn widget(branches: &[String], focused: bool) -> List<'_> {
    let block = widgets::Block::bordered().title("Branches");

    let items: Vec<ListItem> = branches.iter().map(|s| ListItem::new(s.as_str())).collect();
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
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("git checkout {} failed", name),
        ))
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
        // 先尝试直接 track checkout；失败则 fetch 后重试一次
        if checkout_tracking(&remote).is_err() {
            // 尝试只 fetch 该分支，避免太重
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

    // 本地不存在，远端也找不到：创建新分支
    create_branch(name)
}

fn checkout_local(name: &str) -> io::Result<()> {
    let status = Command::new("git")
        .args(["checkout", "--quiet", name])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("git checkout {} failed", name),
        ))
    }
}

fn create_branch(name: &str) -> io::Result<()> {
    let status = Command::new("git")
        .args(["checkout", "-b", name])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("git checkout -b {} failed", name),
        ))
    }
}

fn checkout_tracking(remote_ref_short: &str) -> io::Result<()> {
    let status = Command::new("git")
        .args(["checkout", "--track", remote_ref_short])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("git checkout --track {} failed", remote_ref_short),
        ))
    }
}

fn local_branch_exists(name: &str) -> io::Result<bool> {
    // 使用 show-ref 校验本地分支是否存在
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

// 找出任意一个匹配的远端分支（优先 origin/<name>，否则选第一个匹配）
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
    // "origin/feat" -> ("origin", "feat")
    let mut it = remote_ref.splitn(2, '/');
    let remote = it.next()?;
    let short = it.next()?;
    Some((remote, short))
}

pub fn merge_branch(raw_target: &str) -> io::Result<()> {
    let target = normalize_branch_name(raw_target);
    if target.is_empty() {
        return Ok(());
    }
    // 使用 --no-edit 避免打开编辑器；保留默认合并信息
    let status = Command::new("git")
        .args(["merge", "--no-edit", &target])
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        // 非 0 退出码可能表示冲突或其它错误，调用方可进一步检查冲突
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("git merge {} failed", target),
        ))
    }
}

/// 将当前分支 rebase 到选中的目标分支上
pub fn rebase_onto_branch(raw_target: &str) -> io::Result<()> {
    let target = normalize_branch_name(raw_target);
    if target.is_empty() {
        return Ok(());
    }
    let status = Command::new("git")
        .args(["rebase", &target])
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("git rebase {} failed", target),
        ))
    }
}

pub fn has_conflicts() -> io::Result<bool> {
    // git ls-files -u 输出非空即存在未合并条目
    let output = Command::new("git").args(["ls-files", "-u"]).output()?;
    if output.status.success() {
        return Ok(!output.stdout.is_empty());
    }
    // 如果命令出错，回退用 status 检测（更稳妥可再做增强）
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
    return Ok(conflicted);
}
