use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

const SCROLL_LINES_PER_TICK: i16 = 3;

use super::app::{App, FocusPanel};
use super::ui_layout::active_panel_at;

pub(crate) fn handle_mouse(app: &mut App, mouse: MouseEvent, terminal_area: Rect) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => focus_panel_at_cursor(app, mouse, terminal_area),
        MouseEventKind::ScrollDown => scroll_panel_at_cursor(app, mouse, terminal_area, SCROLL_LINES_PER_TICK),
        MouseEventKind::ScrollUp => scroll_panel_at_cursor(app, mouse, terminal_area, -SCROLL_LINES_PER_TICK),
        _ => {}
    }
}

fn focus_panel_at_cursor(app: &mut App, mouse: MouseEvent, terminal_area: Rect) {
    if let Some(panel) = active_panel_at(app, terminal_area, mouse.column, mouse.row) {
        app.focus = panel;
    }
}

fn scroll_panel_at_cursor(app: &mut App, mouse: MouseEvent, terminal_area: Rect, delta: i16) {
    let (scroll_down, amount) = scroll_direction_and_amount(delta);
    match active_panel_at(app, terminal_area, mouse.column, mouse.row) {
        Some(FocusPanel::Chat) => {
            app.focus = FocusPanel::Chat;
            if scroll_down {
                app.chat.scroll_down(amount);
            } else {
                app.chat.scroll_up(amount);
            }
        }
        Some(FocusPanel::Tools) => {
            app.focus = FocusPanel::Tools;
            if scroll_down {
                app.tools.scroll_down(amount);
            } else {
                app.tools.scroll_up(amount);
            }
        }
        Some(FocusPanel::Input) | Some(FocusPanel::CommandPalette) | None => {}
    }
}

fn scroll_direction_and_amount(delta: i16) -> (bool, usize) {
    if delta > 0 {
        return (true, delta as usize);
    }

    (false, delta.unsigned_abs() as usize)
}
