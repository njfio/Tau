use crossterm::event::KeyCode;

use crate::interactive::app::{App, AppConfig};

use super::helpers::{key, render_app, submit_command};

#[test]
fn integration_spec_3582_approve_command_resolves_pending_approval_through_real_input_path() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "/approval-needed");
    submit_command(&mut app, "/approve");

    let rendered = render_app(&mut app, 120, 28);

    assert!(!rendered.contains("Approval required"));
    assert!(rendered.contains("Approval approved"));
}

#[test]
fn integration_spec_3582_approval_shortcuts_resolve_request_without_leaving_insert_mode() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "/approval-needed");
    app.handle_key(key(KeyCode::Char('y')));

    let rendered = render_app(&mut app, 120, 28);

    assert!(!rendered.contains("Approval required"));
    assert!(rendered.contains("Approval approved"));
}

#[test]
fn red_spec_3582_approval_attention_strip_exposes_approve_and_reject_actions() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "/approval-needed");

    let rendered = render_app(&mut app, 120, 24);

    assert!(rendered.contains("Approval required"));
    assert!(rendered.contains("Approve"));
    assert!(rendered.contains("Reject"));
    assert!(rendered.contains("[Y] approve"));
    assert!(rendered.contains("[N] reject"));
}

#[test]
fn red_spec_3582_error_attention_strip_exposes_retry_and_details_actions() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::Error;
    let rendered = render_app(&mut app, 120, 24);

    assert!(rendered.contains("Retry turn"));
    assert!(rendered.contains("Open details"));
}
