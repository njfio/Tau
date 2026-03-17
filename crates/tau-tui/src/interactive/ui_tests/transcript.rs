use crate::interactive::app::{App, AppConfig};
use crate::interactive::chat::MessageRole;
use crate::interactive::gateway::{GatewayInteractiveConfig, GatewayUiEvent, OperatorStateEvent};

use super::helpers::{render_app, submit_command};

#[test]
fn red_spec_3582_status_bar_surfaces_session_and_approval_context() {
    let mut app = App::new(AppConfig::default());
    let rendered = render_app(&mut app, 120, 10);

    assert!(rendered.contains("session"));
    assert!(rendered.contains("approval"));
}

#[test]
fn red_spec_3582_transcript_shows_live_activity_summary_above_messages() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::Thinking;
    let rendered = render_app(&mut app, 120, 24);

    assert!(rendered.contains("Live activity"));
    assert!(rendered.contains("Thinking through the next step"));
}

#[test]
fn red_spec_3582_live_activity_surfaces_active_tool_count_and_name() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::ToolExec;
    app.push_tool_event(
        "bash".to_string(),
        crate::interactive::tools::ToolStatus::Running,
        "Inspecting repository layout".to_string(),
    );
    app.push_tool_event(
        "http".to_string(),
        crate::interactive::tools::ToolStatus::Running,
        "Fetching page data".to_string(),
    );

    let rendered = render_app(&mut app, 120, 28);

    assert!(rendered.contains("2 active"));
    assert!(rendered.contains("http"));
}

#[test]
fn red_spec_3582_transcript_surfaces_active_turn_card_with_prompt_context() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::Thinking;
    app.last_submitted_input = Some("Research Aleo private apps platform".to_string());

    let rendered = render_app(&mut app, 120, 28);

    assert!(rendered.contains("Thinking"));
    assert!(rendered.contains("Research Aleo private apps platform"));
    assert!(rendered.contains("thinking"));
}

#[test]
fn red_spec_3582_streaming_turn_card_surfaces_assistant_preview() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::Streaming;
    app.last_submitted_input = Some("summarize the company".to_string());
    app.push_message(
        MessageRole::Assistant,
        "Aleo builds infrastructure for private applications and zero-knowledge workflows."
            .to_string(),
    );

    let rendered = render_app(&mut app, 120, 30);

    assert!(rendered.contains("streaming"));
    assert!(rendered.contains("assistant output arriving"));
    assert!(rendered.contains("Aleo builds infrastructure"));
}

#[test]
fn red_spec_3582_thinking_run_state_card_uses_phase_specific_title() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::Thinking;
    app.last_submitted_input = Some("Research Aleo private apps platform".to_string());
    app.apply_gateway_event(GatewayUiEvent::OperatorState(OperatorStateEvent {
        entity: "turn".to_string(),
        status: "in_progress".to_string(),
        phase: Some("model".to_string()),
        artifact_kind: None,
        response_id: Some("resp_think".to_string()),
        reason_code: None,
    }));

    let rendered = render_app(&mut app, 120, 28);

    assert!(rendered.contains("Thinking"));
    assert!(rendered.contains("turn:model"));
}

#[test]
fn red_spec_3582_streaming_run_state_card_surfaces_artifact_context() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::Streaming;
    app.last_submitted_input = Some("summarize the company".to_string());
    app.apply_gateway_event(GatewayUiEvent::OperatorState(OperatorStateEvent {
        entity: "artifact".to_string(),
        status: "streaming".to_string(),
        phase: Some("stream".to_string()),
        artifact_kind: Some("assistant_output_text".to_string()),
        response_id: Some("resp_stream".to_string()),
        reason_code: None,
    }));
    app.push_message(
        MessageRole::Assistant,
        "Aleo builds infrastructure for private applications and zero-knowledge workflows."
            .to_string(),
    );

    let rendered = render_app(&mut app, 120, 30);

    assert!(rendered.contains("Streaming reply"));
    assert!(rendered.contains("artifact:stream"));
    assert!(rendered.contains("assistant_output_text"));
}

#[test]
fn red_spec_3582_failed_run_state_card_surfaces_reason_code() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::Error;
    app.last_submitted_input = Some("research the company".to_string());
    app.apply_gateway_event(GatewayUiEvent::OperatorState(OperatorStateEvent {
        entity: "turn".to_string(),
        status: "failed".to_string(),
        phase: Some("post_tool".to_string()),
        artifact_kind: None,
        response_id: Some("resp_failed".to_string()),
        reason_code: Some("rate_limited".to_string()),
    }));

    let rendered = render_app(&mut app, 120, 28);

    assert!(rendered.contains("Turn failed"));
    assert!(rendered.contains("rate_limited"));
    assert!(rendered.contains("turn:post_tool"));
}

#[test]
fn red_spec_3582_transcript_surfaces_running_tool_card_without_opening_drawer() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::ToolExec;
    app.push_tool_event(
        "bash".to_string(),
        crate::interactive::tools::ToolStatus::Running,
        "Inspecting repository layout".to_string(),
    );

    let rendered = render_app(&mut app, 120, 28);

    assert!(rendered.contains("Running tool"));
    assert!(rendered.contains("bash"));
    assert!(rendered.contains("Inspecting repository layout"));
}

#[test]
fn integration_spec_3582_prompt_submission_surfaces_last_turn_summary_card() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "plan the tui redesign");

    let rendered = render_app(&mut app, 120, 30);

    assert!(rendered.contains("Last turn"));
    assert!(rendered.contains("plan the tui redesign"));
    assert!(rendered.contains("assistant reply ready"));
}

#[test]
fn integration_spec_3582_prompt_submission_uses_compact_transcript_headers() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "what is blue?");

    let rendered = render_app(&mut app, 120, 30);

    assert!(rendered.contains("You ·"));
    assert!(rendered.contains("Tau ·"));
    assert!(!rendered.contains("╭─"));
}

#[test]
fn red_spec_3582_transcript_uses_compact_headers_without_box_chrome() {
    let mut app = App::new(AppConfig::default());
    app.push_message(MessageRole::User, "Build me something useful.".to_string());
    app.push_message(
        MessageRole::Assistant,
        "I can do that. First I need repository context.".to_string(),
    );

    let rendered = render_app(&mut app, 120, 28);

    assert!(rendered.contains("You ·"));
    assert!(rendered.contains("Tau ·"));
    assert!(rendered.contains("  I can do that."));
    assert!(!rendered.contains("╭─"));
    assert!(!rendered.contains("╰─"));
}

#[test]
fn red_spec_3582_compact_transcript_keeps_multiline_assistant_output_aligned() {
    let mut app = App::new(AppConfig::default());
    app.push_message(
        MessageRole::Assistant,
        "First line.\nSecond line.".to_string(),
    );

    let rendered = render_app(&mut app, 120, 20);

    assert!(rendered.contains("  First line."));
    assert!(rendered.contains("  Second line."));
}

#[test]
fn red_spec_3582_status_bar_surfaces_operator_phase_context() {
    let mut app = App::new(AppConfig::default());
    app.apply_gateway_event(GatewayUiEvent::OperatorState(OperatorStateEvent {
        entity: "turn".to_string(),
        status: "in_progress".to_string(),
        phase: Some("model".to_string()),
        artifact_kind: None,
        response_id: Some("resp_status".to_string()),
        reason_code: None,
    }));

    let rendered = render_app(&mut app, 140, 10);

    assert!(rendered.contains("turn:model"));
}

#[test]
fn red_spec_3582_live_activity_surfaces_operator_entity_and_artifact_kind() {
    let mut app = App::new(AppConfig::default());
    app.apply_gateway_event(GatewayUiEvent::OperatorState(OperatorStateEvent {
        entity: "artifact".to_string(),
        status: "streaming".to_string(),
        phase: Some("stream".to_string()),
        artifact_kind: Some("assistant_output_text".to_string()),
        response_id: Some("resp_stream".to_string()),
        reason_code: None,
    }));

    let rendered = render_app(&mut app, 140, 24);

    assert!(rendered.contains("artifact"));
    assert!(rendered.contains("assistant_output_text"));
}

#[test]
fn red_spec_3582_status_bar_surfaces_gateway_transport_posture() {
    let mut app = App::new(AppConfig {
        gateway: Some(GatewayInteractiveConfig {
            base_url: "http://127.0.0.1:8791".to_string(),
            auth_token: None,
            session_key: "default".to_string(),
            request_timeout_ms: 45_000,
        }),
        ..AppConfig::default()
    });
    app.apply_gateway_event(GatewayUiEvent::OperatorState(OperatorStateEvent {
        entity: "turn".to_string(),
        status: "in_progress".to_string(),
        phase: Some("model".to_string()),
        artifact_kind: None,
        response_id: Some("resp_transport".to_string()),
        reason_code: None,
    }));

    let rendered = render_app(&mut app, 140, 10);

    assert!(rendered.contains("transport=gateway"));
    assert!(rendered.contains("turn:model"));
}

#[test]
fn red_spec_3582_status_bar_surfaces_local_transport_when_gateway_missing() {
    let mut app = App::new(AppConfig::default());

    let rendered = render_app(&mut app, 140, 10);

    assert!(rendered.contains("transport=local"));
}
