use crate::state::{AppState, CurrentPanel, refresh_states};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::widgets::ListState;

pub fn handle_events(app: &mut AppState) -> std::io::Result<bool> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('r') => refresh_states(app),
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
                ensure_selection(&mut app.status_state, app.changed_files.len());
            }
            KeyCode::Char('2') => {
                app.current_panel = CurrentPanel::Branch;
                ensure_selection(&mut app.branch_state, app.branches.len());
            }
            KeyCode::Char('3') => {
                app.current_panel = CurrentPanel::Commit;
                ensure_selection(&mut app.commit_state, app.commits.len());
            }
            KeyCode::Char('4') => {
                app.current_panel = CurrentPanel::Diff;
            }
            KeyCode::Char(' ') => {
                if matches!(app.current_panel, CurrentPanel::Status) {
                    if let Some(idx) = app.status_state.selected() {
                        if let Some(file) = app.changed_files.get(idx) {
                            let _ = crate::git_status::add_file(&file.path);
                            // color to green
                        }
                    }
                }
            }
            _ => {}
        },
        _ => {}
    }
    Ok(false)
}

pub fn ensure_selection(state: &mut ListState, len: usize) {
    if state.selected().is_none() && len > 0 {
        state.select(Some(0));
    }
}
