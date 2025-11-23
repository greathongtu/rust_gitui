use ratatui::widgets::ListState;

use crate::{
    git_branch::load_branches, git_commits::load_commits, git_diff::load_diff,
    git_status::load_changed_files,
};

#[derive(Default)]
pub struct AppState {
    pub branches: Vec<String>,
    pub commits: Vec<String>,
    pub diff: String,
    pub changed_files: Vec<ChangedFile>,
    pub status_state: ListState,
    pub branch_state: ListState,
    pub commit_state: ListState,
    pub diff_state: ListState,
    pub current_panel: CurrentPanel,

    // commit popup related
    pub commit_popup_open: bool,
    pub commit_input: String,
    pub commit_popup_mode: CommitPopupMode,

    // reset popup related
    pub reset_popup_open: bool,
    pub reset_state: ListState, // 选项列表选中项 (0: soft, 1: mixed, 2: hard)
    pub pending_reset_hash: Option<String>,

    // branch popup
    pub branch_popup_open: bool,
    pub branch_input: String,

    // conflict alert popup
    pub conflict_popup_open: bool,
    pub conflict_message: String,
}

#[derive(Default, Clone, Copy)]
pub enum CurrentPanel {
    #[default]
    Status,
    Branch,
    Commit,
    Diff,
}

#[derive(Default, Clone, Copy)]
pub enum CommitPopupMode {
    #[default]
    New,
    Edit,
}

pub struct ChangedFile {
    pub x: char,
    pub y: char,
    pub path: String,
}

pub fn refresh_states(app: &mut AppState) {
    let prev_panel = app.current_panel;
    app.current_panel = prev_panel;

    let prev_status_idx = app.status_state.selected();
    if let Ok(changed_files) = load_changed_files() {
        app.changed_files = changed_files;
        let len = app.changed_files.len();
        if len == 0 {
            app.status_state.select(None);
        } else {
            let idx = prev_status_idx.unwrap_or(0).min(len - 1);
            app.status_state.select(Some(idx));
        }
    }

    let prev_branch_idx = app.branch_state.selected();
    app.branches = load_branches();
    let branches_len = app.branches.len();
    if branches_len == 0 {
        app.branch_state.select(None);
    } else {
        let idx = prev_branch_idx.unwrap_or(0).min(branches_len - 1);
        app.branch_state.select(Some(idx));
    }

    let prev_commit_idx = app.commit_state.selected();
    app.commits = load_commits();
    let commits_len = app.commits.len();
    if commits_len == 0 {
        app.commit_state.select(None);
    } else {
        let idx = prev_commit_idx.unwrap_or(0).min(commits_len - 1);
        app.commit_state.select(Some(idx));
    }

    let prev_diff_idx = app.diff_state.selected();
    app.diff = load_diff();
    if let Some(idx) = prev_diff_idx {
        app.diff_state.select(Some(idx));
    } else {
        app.diff_state.select(None);
    }
}
