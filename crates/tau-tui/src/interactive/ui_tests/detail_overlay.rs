use crossterm::event::KeyCode;

use crate::interactive::app::{App, AppConfig};

use super::helpers::{key, render_app, submit_command};

#[test]
fn red_spec_3582_narrow_layout_uses_detail_overlay() {
    let mut app = App::new(AppConfig::default());
    app.show_tool_panel = true;

    let rendered = render_app(&mut app, 72, 22);

    assert!(rendered.contains("Quick details"));
    assert!(rendered.contains("[tools]"));
}

#[test]
fn red_spec_3582_narrow_detail_overlay_exposes_close_and_section_hints() {
    let mut app = App::new(AppConfig::default());
    app.show_tool_panel = true;

    let rendered = render_app(&mut app, 72, 22);

    assert!(rendered.contains("Esc close"));
    assert!(rendered.contains("[ ] section"));
}

#[test]
fn integration_spec_3582_narrow_detail_overlay_cycles_sections_from_real_key_path() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "/details");
    app.handle_key(key(KeyCode::Char(']')));

    let rendered = render_app(&mut app, 72, 22);

    assert!(rendered.contains("Quick details [memory]"));
}

#[test]
fn integration_spec_3582_escape_closes_narrow_detail_overlay_from_real_key_path() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "/details");
    app.handle_key(key(KeyCode::Esc));

    let rendered = render_app(&mut app, 72, 22);

    assert!(!rendered.contains("Quick details"));
}
