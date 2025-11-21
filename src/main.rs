mod events;
mod git_branch;
mod git_commits;
mod git_diff;
mod git_status;
mod state;

use crate::state::AppState;
use crate::state::refresh_states;
use events::handle_events;
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

fn draw(frame: &mut Frame, app: &mut AppState) {
    let horizontal = Layout::horizontal([Constraint::Percentage(33), Constraint::Fill(1)]);
    let [left_area, right_area] = horizontal.areas(frame.area());
    let vertical = Layout::vertical([Constraint::Fill(1); 3]);
    let [left_top, left_middle, left_down] = vertical.areas(left_area);

    frame.render_stateful_widget(
        git_status::widget(&app.changed_files),
        left_top,
        &mut app.status_state,
    );
    frame.render_stateful_widget(
        git_branch::widget(&app.branches),
        left_middle,
        &mut app.branch_state,
    );
    frame.render_stateful_widget(
        git_commits::widget(&app.commits),
        left_down,
        &mut app.commit_state,
    );

    frame.render_stateful_widget(git_diff::widget(&app.diff), right_area, &mut app.diff_state);

    // let status = format!(
    //     "status selected={:?} len={}",
    //     app.status_state.selected(),
    //     app.changed_files.len()
    // );
    // let block = ratatui::widgets::Block::default()
    //     .borders(Borders::ALL)
    //     .title("Debug");
    // let para = Paragraph::new(Line::raw(status))
    //     .block(block)
    //     .style(Style::default().fg(Color::Yellow));

    // let [_, debug_area] =
    //     Layout::vertical([Constraint::Fill(1), Constraint::Length(5)]).areas(frame.area());
    // frame.render_widget(para, debug_area);
}
