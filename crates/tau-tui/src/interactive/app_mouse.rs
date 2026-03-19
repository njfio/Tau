use crossterm::event::MouseEvent;
use ratatui::layout::Rect;

use super::app::App;

pub(crate) fn handle_mouse(_app: &mut App, _mouse: MouseEvent, _terminal_area: Rect) {}
