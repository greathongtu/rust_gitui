use crate::state::{AppState, CommitPopupMode, CurrentPanel, RefreshScope, refresh_scopes};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

// todo: output command
pub fn handle_events(app: &mut AppState) -> std::io::Result<bool> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            if app.branch_popup_open {
                match key.code {
                    KeyCode::Esc => {
                        app.branch_popup_open = false;
                        app.branch_input.clear();
                    }
                    KeyCode::Char(' ') => {
                        let _ = crate::git_branch::checkout_or_create_branch(&app.branch_input);
                        app.branch_popup_open = false;
                        app.branch_input.clear();
                        refresh_scopes(
                            app,
                            &[
                                RefreshScope::Branches,
                                RefreshScope::Commits,
                                RefreshScope::Status,
                                RefreshScope::Diff,
                            ],
                        );
                    }
                    KeyCode::Backspace => {
                        app.branch_input.pop();
                    }
                    KeyCode::Char(c) => {
                        app.branch_input.push(c);
                    }
                    KeyCode::Tab => {
                        app.branch_input.push('\t');
                    }
                    _ => {}
                }
                return Ok(false);
            }
            if app.reset_popup_open {
                match key.code {
                    KeyCode::Esc => {
                        app.reset_popup_open = false;
                        app.pending_reset_hash = None;
                        app.reset_state.select(None);
                    }
                    KeyCode::Char(' ') => {
                        let sel = app.reset_state.selected().unwrap_or(1);
                        let mode = match sel {
                            0 => "soft",
                            1 => "mixed",
                            2 => "hard",
                            _ => "mixed",
                        };
                        if let Some(hash) = app.pending_reset_hash.clone() {
                            let _ = crate::git_commits::reset_to(&hash, mode);
                        }
                        app.reset_popup_open = false;
                        app.pending_reset_hash = None;
                        refresh_scopes(
                            app,
                            &[
                                RefreshScope::Branches,
                                RefreshScope::Commits,
                                RefreshScope::Status,
                                RefreshScope::Diff,
                            ],
                        );
                    }

                    KeyCode::Up | KeyCode::Char('k') => {
                        let i = app.reset_state.selected().unwrap_or(1);
                        let i = i.saturating_sub(1);
                        app.reset_state.select(Some(i));
                    }
                    KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let i = app.reset_state.selected().unwrap_or(1);
                        let i = i.saturating_sub(1);
                        app.reset_state.select(Some(i));
                    }

                    KeyCode::Down | KeyCode::Char('j') => {
                        let i = app.reset_state.selected().unwrap_or(1);
                        let i = (i + 1).min(2);
                        app.reset_state.select(Some(i));
                    }
                    KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let i = app.reset_state.selected().unwrap_or(1);
                        let i = (i + 1).min(2);
                        app.reset_state.select(Some(i));
                    }

                    KeyCode::Char('s') => app.reset_state.select(Some(0)),
                    KeyCode::Char('m') => app.reset_state.select(Some(1)),
                    KeyCode::Char('h') => app.reset_state.select(Some(2)),
                    _ => {}
                }
                return Ok(false);
            }
            if app.commit_popup_open {
                match key.code {
                    KeyCode::Esc => {
                        app.commit_popup_open = false;
                        app.commit_input.clear();
                    }
                    KeyCode::Enter => {
                        match app.commit_popup_mode {
                            CommitPopupMode::New => {
                                let _ = crate::git_commits::commit(&app.commit_input);
                            }
                            CommitPopupMode::Edit => {
                                let _ = crate::git_commits::reword_last_commit(&app.commit_input);
                            }
                        }
                        app.commit_popup_open = false;
                        app.commit_input.clear();
                        refresh_scopes(
                            app,
                            &[
                                RefreshScope::Commits,
                                RefreshScope::Status,
                                RefreshScope::Diff,
                            ],
                        );
                    }
                    KeyCode::Backspace => {
                        app.commit_input.pop();
                    }
                    KeyCode::Char(c) => {
                        app.commit_input.push(c);
                    }
                    KeyCode::Tab => {
                        app.commit_input.push('\t');
                    }

                    _ => {}
                }
                return Ok(false);
            }
            if app.conflict_popup_open {
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        app.conflict_popup_open = false;
                        app.conflict_message.clear();
                    }
                    _ => {}
                }
                return Ok(false);
            }

            match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('p') => {
                    let _ = crate::git_status::pull();
                    refresh_scopes(
                        app,
                        &[
                            RefreshScope::Branches,
                            RefreshScope::Commits,
                            RefreshScope::Status,
                            RefreshScope::Diff,
                        ],
                    );
                }
                KeyCode::Char('P') => {
                    let _ = crate::git_status::push();
                    refresh_scopes(app, &[RefreshScope::Branches, RefreshScope::Commits]);
                }

                KeyCode::Char('A') => {
                    if matches!(app.current_panel, CurrentPanel::Status) {
                        let _ = crate::git_commits::amend_last_no_edit();
                        refresh_scopes(
                            app,
                            &[
                                RefreshScope::Commits,
                                RefreshScope::Status,
                                RefreshScope::Diff,
                            ],
                        );
                    }
                }

                KeyCode::Char('c') => {
                    app.commit_popup_open = true;
                    app.commit_input.clear();
                    app.commit_popup_mode = CommitPopupMode::New;
                }
                KeyCode::Char('R') => {
                    if matches!(app.current_panel, CurrentPanel::Commit) {
                        app.commit_popup_open = true;
                        app.commit_input =
                            crate::git_commits::get_head_commit_message().unwrap_or_default();
                        app.commit_popup_mode = CommitPopupMode::Edit;
                        return Ok(false);
                    }
                }
                KeyCode::Char('d') => {
                    if matches!(app.current_panel, CurrentPanel::Commit) {
                        if let Some(idx) = app.commit_state.selected() {
                            if let Some(line) = app.commits.get(idx) {
                                if let Some(hash) = crate::git_commits::parse_commit_hash(line) {
                                    let _ = crate::git_commits::drop_commit(&hash);
                                    refresh_scopes(
                                        app,
                                        &[
                                            RefreshScope::Commits,
                                            RefreshScope::Status,
                                            RefreshScope::Diff,
                                        ],
                                    );
                                }
                            }
                        }
                    }
                }

                KeyCode::Char('j') | KeyCode::Down => match app.current_panel {
                    CurrentPanel::Status => app.status_state.scroll_down_by(1),
                    CurrentPanel::Branch => app.branch_state.scroll_down_by(1),
                    CurrentPanel::Commit => app.commit_state.scroll_down_by(1),
                    CurrentPanel::Diff => app.diff_state.scroll_down_by(1),
                },
                KeyCode::Char('k') | KeyCode::Up => match app.current_panel {
                    CurrentPanel::Status => app.status_state.scroll_up_by(1),
                    CurrentPanel::Branch => app.branch_state.scroll_up_by(1),
                    CurrentPanel::Commit => app.commit_state.scroll_up_by(1),
                    CurrentPanel::Diff => app.diff_state.scroll_up_by(1),
                },

                KeyCode::Char('1') => {
                    app.current_panel = CurrentPanel::Status;
                }
                KeyCode::Char('2') => {
                    app.current_panel = CurrentPanel::Branch;
                }
                KeyCode::Char('3') => {
                    app.current_panel = CurrentPanel::Commit;
                }
                KeyCode::Char('4') => {
                    app.current_panel = CurrentPanel::Diff;
                }
                KeyCode::Char('n') => {
                    if matches!(app.current_panel, CurrentPanel::Branch) {
                        app.branch_popup_open = true;
                        app.branch_input.clear();
                        return Ok(false);
                    }
                }
                KeyCode::Char('M') => {
                    if matches!(app.current_panel, CurrentPanel::Branch) {
                        if let Some(idx) = app.branch_state.selected() {
                            if let Some(branch_line) = app.branches.get(idx) {
                                let target = crate::git_branch::normalize_branch_name(branch_line);
                                let _ = crate::git_branch::merge_branch(&target);
                                if crate::git_branch::has_conflicts().unwrap_or(false) {
                                    app.conflict_popup_open = true;
                                    app.conflict_message =
                                        format!("检测到合并冲突。\n请手动解决冲突");
                                    return Ok(false);
                                } else {
                                    refresh_scopes(
                                        app,
                                        &[
                                            RefreshScope::Branches,
                                            RefreshScope::Commits,
                                            RefreshScope::Status,
                                            RefreshScope::Diff,
                                        ],
                                    );
                                }
                            }
                        }
                    }
                }
                // rebase：在分支面板选中目标分支后按 r
                KeyCode::Char('r') => {
                    if matches!(app.current_panel, CurrentPanel::Branch) {
                        if let Some(idx) = app.branch_state.selected() {
                            if let Some(branch_line) = app.branches.get(idx) {
                                let target = crate::git_branch::normalize_branch_name(branch_line);
                                let _ = crate::git_branch::rebase_onto_branch(&target);
                                if crate::git_branch::has_conflicts().unwrap_or(false) {
                                    app.conflict_popup_open = true;
                                    app.conflict_message =
                                        String::from("检测到 rebase 冲突。\n请手动解决冲突");
                                    return Ok(false);
                                } else {
                                    refresh_scopes(
                                        app,
                                        &[
                                            RefreshScope::Branches,
                                            RefreshScope::Commits,
                                            RefreshScope::Status,
                                            RefreshScope::Diff,
                                        ],
                                    );
                                }
                            }
                        }
                    }
                }
                KeyCode::Char(' ') => match app.current_panel {
                    CurrentPanel::Branch => {
                        if let Some(idx) = app.branch_state.selected() {
                            if let Some(branch) = app.branches.get(idx) {
                                let _ = crate::git_branch::checkout_branch(branch);
                                refresh_scopes(
                                    app,
                                    &[
                                        RefreshScope::Branches,
                                        RefreshScope::Commits,
                                        RefreshScope::Status,
                                        RefreshScope::Diff,
                                    ],
                                );
                            }
                        }
                    }
                    CurrentPanel::Status => {
                        if let Some(idx) = app.status_state.selected() {
                            if let Some(file) = app.changed_files.get(idx) {
                                if file.x == ' ' || file.x == '?' {
                                    let _ = crate::git_status::add_file(&file.path);
                                } else {
                                    let _ = crate::git_status::unstage_file(&file.path);
                                }

                                refresh_scopes(app, &[RefreshScope::Status, RefreshScope::Diff]);
                            }
                        }
                    }
                    CurrentPanel::Commit => {
                        if let Some(idx) = app.commit_state.selected() {
                            if let Some(line) = app.commits.get(idx) {
                                if let Some(hash) = crate::git_commits::parse_commit_hash(line) {
                                    let _ = crate::git_commits::checkout_commit(&hash);
                                    refresh_scopes(
                                        app,
                                        &[
                                            RefreshScope::Commits,
                                            RefreshScope::Status,
                                            RefreshScope::Diff,
                                        ],
                                    );
                                }
                            }
                        }
                    }
                    _ => {}
                },
                KeyCode::Char('a') => {
                    if matches!(app.current_panel, CurrentPanel::Status) {
                        let has_staged = app
                            .changed_files
                            .iter()
                            .any(|f| crate::git_status::is_staged_index_code(f.x));
                        if has_staged {
                            let _ = crate::git_status::unstage_all_file();
                        } else {
                            let _ = crate::git_status::add_all_file();
                        }

                        refresh_scopes(app, &[RefreshScope::Status, RefreshScope::Diff]);
                    }
                }
                KeyCode::Char('g') => {
                    if matches!(app.current_panel, CurrentPanel::Commit) {
                        if let Some(idx) = app.commit_state.selected() {
                            if let Some(line) = app.commits.get(idx) {
                                if let Some(hash) = crate::git_commits::parse_commit_hash(line) {
                                    app.pending_reset_hash = Some(hash);
                                    app.reset_popup_open = true;
                                    // 默认选中 mixed
                                    app.reset_state.select(Some(1));
                                    return Ok(false);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(false)
}
