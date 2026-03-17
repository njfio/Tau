use crossterm::event::KeyCode;

use crate::interactive::app::{App, AppConfig};

use super::helpers::{ctrl, key, render_app, type_text};

#[test]
fn red_spec_3582_command_palette_lists_common_commands_and_shortcuts() {
    let mut app = App::new(AppConfig::default());
    app.handle_key(ctrl('p'));

    let rendered = render_app(&mut app, 120, 28);

    assert!(rendered.contains("Command Palette"));
    assert!(rendered.contains("/thinking"));
    assert!(rendered.contains("/memory"));
    assert!(rendered.contains("Esc close"));
    assert!(rendered.contains("Enter run"));
}

#[test]
fn red_spec_3582_command_palette_filters_commands_from_live_input() {
    let mut app = App::new(AppConfig::default());
    app.handle_key(ctrl('p'));
    type_text(&mut app, "me");

    let rendered = render_app(&mut app, 120, 28);

    assert!(rendered.contains("/memory"));
    assert!(!rendered.contains("/thinking"));
}

#[test]
fn red_spec_3582_command_palette_escape_returns_focus_to_input() {
    let mut app = App::new(AppConfig::default());
    app.handle_key(ctrl('p'));
    app.handle_key(key(KeyCode::Esc));

    let rendered = render_app(&mut app, 120, 28);

    assert!(!rendered.contains("Command Palette"));
    assert!(rendered.contains("[/] commands"));
}
