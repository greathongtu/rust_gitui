mod git_branch;
mod git_commits;
mod git_diff;
mod git_status;
mod state;

use crate::state::AppState;
use crate::state::refresh_states;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let mut app = AppState::default();
    refresh_states(&mut app);
    let result = run_loop(terminal, &mut app);
    ratatui::restore();
    result
}

fn run_loop(
    mut terminal: ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>>,
    app: &mut AppState,
) -> std::io::Result<()> {
    loop {
        terminal
            .draw(|frame| draw(frame, app))
            .expect("failed to draw frame");
        if handle_events(app)? {
            break Ok(());
        }
    }
}

fn handle_events(app: &mut AppState) -> std::io::Result<bool> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('r') => refresh_states(app),

            // handle other key events
            _ => {}
        },
        // handle other events
        _ => {}
    }
    Ok(false)
}

fn draw(frame: &mut Frame, app: &mut AppState) {
    let horizontal = Layout::horizontal([Constraint::Percentage(33), Constraint::Fill(1)]);
    let [left_area, right_area] = horizontal.areas(frame.area());
    let vertical = Layout::vertical([Constraint::Fill(1); 3]);
    let [left_top, left_middle, left_down] = vertical.areas(left_area);

    frame.render_widget(git_diff::widget(&app.diff), right_area);
    frame.render_widget(git_status::widget(&app.changed_files), left_top);
    frame.render_widget(git_branch::widget(&app.branches), left_middle);
    frame.render_widget(git_commits::widget(&app.commits), left_down);
}
