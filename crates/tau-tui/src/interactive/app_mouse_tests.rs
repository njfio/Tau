use crossterm::event::{MouseButton, MouseEvent, MouseEventKind, KeyModifiers};
use ratatui::layout::Rect;

use super::{
    app::{App, AppConfig, FocusPanel},
    app_mouse::handle_mouse,
    chat::MessageRole,
    tools::ToolStatus,
};

fn terminal_area() -> Rect {
    Rect::new(0, 0, 120, 24)
}

fn left_click(column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn scroll_down(column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

#[test]
fn red_spec_3596_left_click_chat_focuses_chat_panel() {
    let mut app = App::new(AppConfig::default());

    handle_mouse(&mut app, left_click(10, 5), terminal_area());

    assert_eq!(app.focus, FocusPanel::Chat);
}

#[test]
fn red_spec_3596_left_click_tools_focuses_tools_panel() {
    let mut app = App::new(AppConfig::default());
    app.show_tool_panel = true;

    handle_mouse(&mut app, left_click(100, 5), terminal_area());

    assert_eq!(app.focus, FocusPanel::Tools);
}

#[test]
fn red_spec_3596_scroll_down_over_chat_moves_chat_scroll_offset() {
    let mut app = App::new(AppConfig::default());
    for idx in 0..30 {
        app.push_message(MessageRole::Assistant, format!("message {idx}"));
    }
    app.chat.scroll_to_top();

    handle_mouse(&mut app, scroll_down(10, 5), terminal_area());

    assert!(
        app.chat.scroll_offset() > 0,
        "expected chat scroll offset to advance after mouse wheel"
    );
}

#[test]
fn red_spec_3596_scroll_down_over_tools_moves_tool_scroll_offset() {
    let mut app = App::new(AppConfig::default());
    app.show_tool_panel = true;
    for idx in 0..30 {
        app.push_tool_event(
            format!("bash-{idx}"),
            ToolStatus::Success,
            format!("detail {idx}"),
        );
    }

    handle_mouse(&mut app, scroll_down(100, 5), terminal_area());

    assert!(
        app.tools.scroll_offset() > 0,
        "expected tool scroll offset to advance after mouse wheel"
    );
}

#[test]
fn integration_spec_3596_mouse_event_outside_panels_does_not_corrupt_focus() {
    let mut app = App::new(AppConfig::default());
    let original_focus = app.focus;

    handle_mouse(&mut app, left_click(10, 0), terminal_area());

    assert_eq!(app.focus, original_focus);
}
