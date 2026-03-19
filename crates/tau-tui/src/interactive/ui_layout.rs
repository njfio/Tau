use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use super::app::{App, FocusPanel};
use super::ui_input;

#[derive(Debug, Clone, Copy)]
pub(crate) struct InteractiveLayout {
    pub status_bar: Rect,
    pub input: Rect,
    pub help: Rect,
    pub chat: Rect,
    pub tools: Option<Rect>,
}

pub(crate) fn frame_layout(frame: &Frame, app: &App) -> InteractiveLayout {
    compute_layout(app, frame.area())
}

pub(crate) fn active_panel_at(app: &App, area: Rect, column: u16, row: u16) -> Option<FocusPanel> {
    let layout = compute_layout(app, area);
    let point = (column, row);
    if rect_contains(layout.chat, point) {
        return Some(FocusPanel::Chat);
    }
    if layout.tools.is_some_and(|rect| rect_contains(rect, point)) {
        return Some(FocusPanel::Tools);
    }
    if rect_contains(layout.input, point) {
        return Some(FocusPanel::Input);
    }
    None
}

fn compute_layout(app: &App, area: Rect) -> InteractiveLayout {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(ui_input::input_height(app)),
            Constraint::Length(1),
        ])
        .split(area);

    let (chat, tools) = compute_body_layout(app, main_chunks[1]);
    InteractiveLayout {
        status_bar: main_chunks[0],
        input: main_chunks[2],
        help: main_chunks[3],
        chat,
        tools,
    }
}

fn compute_body_layout(app: &App, area: Rect) -> (Rect, Option<Rect>) {
    if !app.show_tool_panel {
        return (area, None);
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);
    (chunks[0], Some(chunks[1]))
}

fn rect_contains(rect: Rect, point: (u16, u16)) -> bool {
    let (column, row) = point;
    column >= rect.x && column < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}
