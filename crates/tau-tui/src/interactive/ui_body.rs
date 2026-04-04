use ratatui::{layout::Rect, Frame};

use super::app::App;
use super::{ui_chat, ui_tools};

pub(crate) fn render_body(frame: &mut Frame, app: &mut App, chat: Rect, tools: Option<Rect>) {
    ui_chat::render_chat_panel(frame, app, chat);
    if let Some(area) = tools {
        ui_tools::render_tool_panel(frame, app, area);
    }
}
