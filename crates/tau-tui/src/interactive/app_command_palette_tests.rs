use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{
    app::{App, AppConfig, FocusPanel},
    app_commands::handle_command_palette_key,
};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn char_key(character: char) -> KeyEvent {
    key(KeyCode::Char(character))
}

fn focused_palette_app() -> App {
    let mut app = App::new(AppConfig::default());
    app.focus = FocusPanel::CommandPalette;
    app
}

#[test]
fn placeholder_jump_command_palette_edits_at_interior_cursor() {
    let mut app = focused_palette_app();

    handle_command_palette_key(&mut app, char_key('a'));
    handle_command_palette_key(&mut app, char_key('c'));
    handle_command_palette_key(&mut app, key(KeyCode::Left));
    handle_command_palette_key(&mut app, char_key('b'));

    assert_eq!(app.command_input, "abc");
    assert_eq!(app.command_cursor_col, 2);

    handle_command_palette_key(&mut app, key(KeyCode::Backspace));

    assert_eq!(app.command_input, "ac");
    assert_eq!(app.command_cursor_col, 1);
}

#[test]
fn placeholder_jump_command_palette_moves_to_first_placeholder_start() {
    let mut app = focused_palette_app();
    app.command_input = "mission <mission-id>".to_string();
    app.command_cursor_col = 0;

    handle_command_palette_key(&mut app, key(KeyCode::Tab));

    assert_eq!(app.command_cursor_col, "mission <".chars().count());
}

#[test]
fn placeholder_jump_command_palette_advances_and_wraps_between_placeholders() {
    let mut app = focused_palette_app();
    app.command_input = "resume <mission-id> --profile {profile}".to_string();
    app.command_cursor_col = 0;

    handle_command_palette_key(&mut app, key(KeyCode::Tab));
    assert_eq!(app.command_cursor_col, "resume <".chars().count());

    handle_command_palette_key(&mut app, key(KeyCode::Tab));
    assert_eq!(
        app.command_cursor_col,
        "resume <mission-id> --profile {".chars().count()
    );

    handle_command_palette_key(&mut app, key(KeyCode::Tab));
    assert_eq!(app.command_cursor_col, "resume <".chars().count());
}

#[test]
fn placeholder_jump_command_palette_submit_preserves_existing_command_execution() {
    let mut app = focused_palette_app();
    app.command_input = "help".to_string();
    app.command_cursor_col = app.command_input.chars().count();

    handle_command_palette_key(&mut app, key(KeyCode::Enter));

    assert_eq!(app.focus, FocusPanel::Input);
    assert_eq!(app.command_input, "");
    assert_eq!(app.command_cursor_col, 0);
    assert!(app.show_help);
}
