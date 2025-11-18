mod git_branch;
mod git_status;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::Frame;
use ratatui::{
    layout::{Constraint, Layout},
    widgets,
};

fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let result = run_loop(terminal);
    ratatui::restore();
    result
}

fn run_loop(
    mut terminal: ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>>,
) -> std::io::Result<()> {
    loop {
        terminal.draw(draw).expect("failed to draw frame");
        if handle_events()? {
            break Ok(());
        }
    }
}

fn handle_events() -> std::io::Result<bool> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') => return Ok(true),
            // handle other key events
            _ => {}
        },
        // handle other events
        _ => {}
    }
    Ok(false)
}

fn draw(frame: &mut Frame) {
    let horizontal = Layout::horizontal([Constraint::Percentage(33), Constraint::Fill(1)]);
    let [left_area, right_area] = horizontal.areas(frame.area());
    let vertical = Layout::vertical([Constraint::Fill(1); 3]);
    let [left_top, left_middle, left_down] = vertical.areas(left_area);

    frame.render_widget(widgets::Block::bordered().title("Diff"), right_area);
    frame.render_widget(git_status::changed_files_widget(), left_top);
    frame.render_widget(git_branch::git_branch(), left_middle);
    frame.render_widget(widgets::Block::bordered().title("Commits"), left_down);
}
