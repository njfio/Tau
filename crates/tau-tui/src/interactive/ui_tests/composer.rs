use crate::interactive::app::{App, AppConfig};
use crate::interactive::chat::MessageRole;
use crate::interactive::gateway::{GatewayUiEvent, OperatorStateEvent};

use super::helpers::{render_app, submit_command};

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

#[test]
fn integration_spec_3582_thinking_command_opens_overlay_through_real_input_path() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "/thinking");

    let rendered = render_app(&mut app, 120, 28);

    assert!(rendered.contains("Thinking"));
    assert!(rendered.contains("No active turn context"));
}

#[test]
fn integration_spec_3582_bare_thinking_command_opens_overlay_through_real_input_path() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "thinking");

    let rendered = render_app(&mut app, 120, 28);

    assert!(rendered.contains("Thinking"));
    assert!(rendered.contains("No active turn context"));
}

#[test]
fn red_spec_3582_thinking_overlay_surfaces_operator_state_and_preview() {
    let mut app = App::new(AppConfig::default());
    app.apply_gateway_event(GatewayUiEvent::OperatorState(OperatorStateEvent {
        entity: "artifact".to_string(),
        status: "streaming".to_string(),
        phase: Some("stream".to_string()),
        artifact_kind: Some("assistant_output_text".to_string()),
        response_id: Some("resp_overlay".to_string()),
        reason_code: None,
    }));
    app.push_message(
        MessageRole::Assistant,
        "Blue is a color often associated with calm.".to_string(),
    );
    submit_command(&mut app, "/thinking");

    let rendered = render_app(&mut app, 120, 28);

    assert!(rendered.contains("artifact:stream"));
    assert!(rendered.contains("assistant_output_text"));
    assert!(rendered.contains("Blue is a color"));
}
