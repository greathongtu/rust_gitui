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
}

pub struct ChangedFile {
    pub x: char,
    pub y: char,
    pub path: String,
}

pub fn refresh_states(app: &mut AppState) {
    if let Ok(changed_files) = load_changed_files() {
        app.changed_files = changed_files;
    }
    app.diff = load_diff();
    app.commits = load_commits();
    app.branches = load_branches();
}
