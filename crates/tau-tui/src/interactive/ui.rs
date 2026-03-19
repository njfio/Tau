//! UI rendering with ratatui — multi-panel layout.

use ratatui::Frame;

use super::app::{App, FocusPanel};
use super::{ui_body, ui_input, ui_layout, ui_overlays, ui_status};

/// Render the full application UI.
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let layout = ui_layout::frame_layout(frame, app);

    ui_status::render_status_bar(frame, app, layout.status_bar);
    ui_body::render_body(frame, app, layout.chat, layout.tools);
    ui_input::render_input(frame, app, layout.input);
    ui_status::render_help_line(frame, app, layout.help);

    if app.show_help {
        ui_overlays::render_help_overlay(frame, size);
    }

    if app.focus == FocusPanel::CommandPalette {
        ui_overlays::render_command_palette(frame, app, size);
    }
}
