use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use super::app::App;
use super::{ui_chat, ui_tools};

pub(crate) fn render_body(frame: &mut Frame, app: &App, area: Rect) {
    if app.show_tool_panel {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        ui_chat::render_chat_panel(frame, app, chunks[0]);
        ui_tools::render_tool_panel(frame, app, chunks[1]);
        return;
    }

    ui_chat::render_chat_panel(frame, app, area);
}
