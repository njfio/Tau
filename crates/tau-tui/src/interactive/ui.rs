//! UI rendering with ratatui for the transcript-first interactive shell.

#[path = "ui_composer.rs"]
mod composer;
#[path = "ui_drawer.rs"]
mod drawer;
#[path = "ui_overlay.rs"]
mod overlay;
#[path = "ui_status.rs"]
mod status_bar;
#[cfg(test)]
#[path = "ui_tests.rs"]
mod tests;
#[path = "ui_transcript.rs"]
mod transcript;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use super::app::{App, FocusPanel};

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(composer::input_height(app)),
        ])
        .split(size);

    status_bar::render_status_bar(frame, app, main_chunks[0]);
    render_body(frame, app, main_chunks[1]);
    composer::render_input(frame, app, main_chunks[2]);

    if app.show_help {
        overlay::render_help_overlay(frame, size);
    }
    if app.focus == FocusPanel::CommandPalette {
        overlay::render_command_palette(frame, app, size);
    }
}

fn render_body(frame: &mut Frame, app: &App, area: Rect) {
    if app.show_tool_panel && area.width >= 96 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(74), Constraint::Percentage(26)])
            .split(area);
        transcript::render_transcript_shell(frame, app, chunks[0]);
        drawer::render_detail_drawer(frame, app, chunks[1]);
        return;
    }
    transcript::render_transcript_shell(frame, app, area);
}
