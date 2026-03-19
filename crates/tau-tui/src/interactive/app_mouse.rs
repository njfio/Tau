use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use super::app::{App, FocusPanel};
use super::ui_layout::active_panel_at;

pub(crate) fn handle_mouse(app: &mut App, mouse: MouseEvent, terminal_area: Rect) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => focus_panel_at_cursor(app, mouse, terminal_area),
        MouseEventKind::ScrollDown => scroll_panel_at_cursor(app, mouse, terminal_area, 1),
        MouseEventKind::ScrollUp => scroll_panel_at_cursor(app, mouse, terminal_area, -1),
        _ => {}
    }
}

fn focus_panel_at_cursor(app: &mut App, mouse: MouseEvent, terminal_area: Rect) {
    if let Some(panel) = active_panel_at(app, terminal_area, mouse.column, mouse.row) {
        app.focus = panel;
    }
}

fn scroll_panel_at_cursor(app: &mut App, mouse: MouseEvent, terminal_area: Rect, delta: i16) {
    match active_panel_at(app, terminal_area, mouse.column, mouse.row) {
        Some(FocusPanel::Chat) => {
            app.focus = FocusPanel::Chat;
            if delta > 0 {
                app.chat.scroll_down(delta as usize);
            } else {
                app.chat.scroll_up(delta.unsigned_abs() as usize);
            }
        }
        Some(FocusPanel::Tools) => {
            app.focus = FocusPanel::Tools;
            if delta > 0 {
                app.tools.scroll_down(delta as usize);
            } else {
                app.tools.scroll_up(delta.unsigned_abs() as usize);
            }
        }
        Some(FocusPanel::Input) | Some(FocusPanel::CommandPalette) | None => {}
    }
}
