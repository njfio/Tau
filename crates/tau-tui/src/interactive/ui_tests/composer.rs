use crate::interactive::app::{App, AppConfig};

use super::helpers::render_app;

#[test]
fn red_spec_3582_composer_hints_expose_interrupt_retry_and_details_actions() {
    let mut app = App::new(AppConfig::default());
    let rendered = render_app(&mut app, 120, 24);

    assert!(rendered.contains("interrupt"));
    assert!(rendered.contains("retry"));
    assert!(rendered.contains("details"));
}

#[test]
fn red_spec_3582_composer_uses_action_chips_instead_of_instruction_sentence() {
    let mut app = App::new(AppConfig::default());
    let rendered = render_app(&mut app, 120, 24);

    assert!(rendered.contains("[/] commands"));
    assert!(rendered.contains("[Enter] send"));
    assert!(!rendered.contains("Press / for commands"));
}
