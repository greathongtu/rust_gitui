mod events;
mod git_branch;
mod git_commits;
mod git_diff;
mod git_status;
mod state;

use crate::state::AppState;
use crate::state::CurrentPanel;
use crate::state::refresh_all_states;
use events::handle_events;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;

fn main() -> std::io::Result<()> {
    if let Some(path) = std::env::args().nth(1)
        && !path.is_empty()
    {
        let _ = std::env::set_current_dir(&path);
    }
    let terminal = ratatui::init();
    let mut app = AppState::default();
    refresh_all_states(&mut app);
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
        git_status::widget(
            &app.changed_files,
            matches!(app.current_panel, CurrentPanel::Status),
        ),
        left_top,
        &mut app.status_state,
    );
    frame.render_stateful_widget(
        git_branch::widget(
            &app.branches,
            matches!(app.current_panel, CurrentPanel::Branch),
        ),
        left_middle,
        &mut app.branch_state,
    );
    frame.render_stateful_widget(
        git_commits::widget(
            &app.commits,
            matches!(app.current_panel, CurrentPanel::Commit),
        ),
        left_down,
        &mut app.commit_state,
    );

    frame.render_stateful_widget(
        git_diff::widget(&app.diff, matches!(app.current_panel, CurrentPanel::Diff)),
        right_area,
        &mut app.diff_state,
    );

    render_commit_popup(frame, app);
    render_branch_popup(frame, app);
    render_reset_popup(frame, app);
    render_conflict_popup(frame, app);
    render_push_force_popup(frame, app);
}

fn render_commit_popup(frame: &mut Frame<'_>, app: &mut AppState) {
    if !app.commit_popup_open {
        return;
    }
    let v = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Length(7),
        Constraint::Percentage(40),
    ])
    .areas(frame.area());
    let [_, mid_area, _] = v;

    let h = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Percentage(60),
        Constraint::Percentage(20),
    ])
    .areas(mid_area);
    let [_, popup_area, _] = h;

    frame.render_widget(Clear, popup_area);
    let title = match app.commit_popup_mode {
        crate::state::CommitPopupMode::New => "Commit Message (Enter to submit, Esc to cancel)",
        crate::state::CommitPopupMode::Edit => {
            "Edit Commit Message (Enter to amend, Esc to cancel)"
        }
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    let para = Paragraph::new(app.commit_input.clone())
        .block(block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(para, popup_area);
}

fn render_branch_popup(frame: &mut Frame<'_>, app: &mut AppState) {
    if !app.branch_popup_open {
        return;
    }
    let v = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Length(5),
        Constraint::Percentage(40),
    ])
    .areas(frame.area());
    let [_, mid_area, _] = v;

    let h = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Percentage(60),
        Constraint::Percentage(20),
    ])
    .areas(mid_area);
    let [_, popup_area, _] = h;

    frame.render_widget(Clear, popup_area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title("New/Checkout Branch (Enter to confirm, Esc to cancel)");
    let para = Paragraph::new(app.branch_input.clone())
        .block(block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(para, popup_area);
}

fn render_reset_popup(frame: &mut Frame<'_>, app: &mut AppState) {
    if !app.reset_popup_open {
        return;
    }

    let v = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Length(9),
        Constraint::Percentage(40),
    ])
    .areas(frame.area());
    let [_, mid_area, _] = v;

    let h = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(50),
        Constraint::Percentage(25),
    ])
    .areas(mid_area);
    let [_, popup_area, _] = h;

    frame.render_widget(Clear, popup_area);

    let options = [
        "soft  (移动 HEAD 到指定提交，保留索引与工作区变更)",
        "mixed (默认：移动 HEAD，重置索引，保留工作区变更)",
        "hard  (移动 HEAD，重置索引与工作区到指定提交)",
    ];
    let items: Vec<ratatui::widgets::ListItem> = options
        .iter()
        .map(|s| ratatui::widgets::ListItem::new(*s))
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Reset to selected commit (↑/↓选择，space确定，Esc取消，s/m/h快速选择)");

    let list = ratatui::widgets::List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::Yellow));

    frame.render_stateful_widget(list, popup_area, &mut app.reset_state);
}

fn render_conflict_popup(frame: &mut Frame<'_>, app: &mut AppState) {
    if !app.conflict_popup_open {
        return;
    }
    let v = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Length(9),
        Constraint::Percentage(40),
    ])
    .areas(frame.area());
    let [_, mid_area, _] = v;

    let h = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(50),
        Constraint::Percentage(25),
    ])
    .areas(mid_area);
    let [_, popup_area, _] = h;

    frame.render_widget(Clear, popup_area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Conflict Detected (Esc关闭)");
    let para = Paragraph::new(app.conflict_message.clone())
        .block(block)
        .style(Style::default().fg(Color::Red));
    frame.render_widget(para, popup_area);
}

fn render_push_force_popup(frame: &mut Frame<'_>, app: &mut AppState) {
    if !app.push_force_popup_open {
        return;
    }

    let v = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Length(7),
        Constraint::Percentage(40),
    ])
    .areas(frame.area());
    let [_, mid_area, _] = v;

    let h = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(50),
        Constraint::Percentage(25),
    ])
    .areas(mid_area);
    let [_, popup_area, _] = h;

    frame.render_widget(Clear, popup_area);

    let options = ["Force push (with lease)", "Cancel"];
    let items: Vec<ratatui::widgets::ListItem> = options
        .iter()
        .map(|s| ratatui::widgets::ListItem::new(*s))
        .collect();

    let title = if app.push_force_message.is_empty() {
        "Push 被拒绝。是否强制推送？（回车/空格确认，Esc取消，上下选择，y/n快捷）"
    } else {
        &app.push_force_message
    };

    let block = Block::default().borders(Borders::ALL).title(title);

    let list = ratatui::widgets::List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::Yellow));

    frame.render_widget(list, popup_area);
}
