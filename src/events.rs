use crate::state::{AppState, CurrentPanel, refresh_states};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

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

            KeyCode::Char('1') => app.current_panel = CurrentPanel::Status,
            KeyCode::Char('2') => app.current_panel = CurrentPanel::Branch,
            KeyCode::Char('3') => app.current_panel = CurrentPanel::Commit,
            KeyCode::Char('4') => app.current_panel = CurrentPanel::Diff,
            // handle other key events
            _ => {}
        },
        // handle other events
        _ => {}
    }
    Ok(false)
}
