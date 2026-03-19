//! UI rendering with ratatui — multi-panel layout.

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use super::app::{App, FocusPanel};
use super::{ui_body, ui_input, ui_overlays, ui_status};

/// Render the full application UI.
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(ui_input::input_height(app)),
            Constraint::Length(1),
        ])
        .split(size);

    ui_status::render_status_bar(frame, app, main_chunks[0]);
    ui_body::render_body(frame, app, main_chunks[1]);
    ui_input::render_input(frame, app, main_chunks[2]);
    ui_status::render_help_line(frame, app, main_chunks[3]);

    if app.show_help {
        ui_overlays::render_help_overlay(frame, size);
    }

    if app.focus == FocusPanel::CommandPalette {
        ui_overlays::render_command_palette(frame, app, size);
    }
}
