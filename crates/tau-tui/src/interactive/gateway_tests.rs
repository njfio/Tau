use super::app::{App, AppConfig};
use super::chat::MessageRole;
use super::gateway::{
    parse_sse_frames, GatewayInteractiveConfig, GatewayUiEvent, OperatorStateEvent,
};
use super::status::AgentStateDisplay;

fn sample_success_sse() -> &'static str {
    "event: response.created\n\
data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_1\"},\"operator_state\":{\"entity\":\"turn\",\"status\":\"in_progress\",\"phase\":\"model\",\"response_id\":\"resp_1\"}}\n\n\
event: response.output_text.delta\n\
data: {\"type\":\"response.output_text.delta\",\"response_id\":\"resp_1\",\"delta\":\"hello \",\"operator_state\":{\"entity\":\"artifact\",\"status\":\"streaming\",\"artifact_kind\":\"assistant_output_text\",\"response_id\":\"resp_1\"}}\n\n\
event: response.output_text.done\n\
data: {\"type\":\"response.output_text.done\",\"response_id\":\"resp_1\",\"text\":\"hello world\",\"operator_state\":{\"entity\":\"artifact\",\"status\":\"completed\",\"artifact_kind\":\"assistant_output_text\",\"response_id\":\"resp_1\"}}\n\n\
event: response.completed\n\
data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_1\",\"output_text\":\"hello world\"},\"operator_state\":{\"entity\":\"turn\",\"status\":\"completed\",\"phase\":\"done\"}}\n\n\
event: done\n\
data: [DONE]\n\n"
}

fn sample_failure_sse() -> &'static str {
    "event: response.created\n\
data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_2\"},\"operator_state\":{\"entity\":\"turn\",\"status\":\"in_progress\",\"phase\":\"model\",\"response_id\":\"resp_2\"}}\n\n\
event: response.failed\n\
data: {\"type\":\"response.failed\",\"error\":{\"code\":\"provider_timeout\",\"message\":\"model request exceeded budget\"},\"operator_state\":{\"entity\":\"turn\",\"status\":\"failed\",\"phase\":\"failed\",\"reason_code\":\"provider_timeout\"}}\n\n"
}

#[test]
fn red_spec_3581_gateway_sse_parser_preserves_response_operator_state_contract() {
    let events = parse_sse_frames(sample_success_sse()).expect("parse SSE");
    assert!(events.iter().any(|event| matches!(
        event,
        GatewayUiEvent::OperatorState(OperatorStateEvent { entity, status, phase, .. })
            if entity == "turn" && status == "in_progress" && phase.as_deref() == Some("model")
    )));
    assert!(events.iter().any(|event| {
        matches!(event, GatewayUiEvent::AssistantDone(text) if text == "hello world")
    }));
}

#[test]
fn red_spec_3581_gateway_response_events_drive_interactive_app_state() {
    let mut app = App::new(AppConfig {
        model: "openai/gpt-5.2".to_string(),
        profile: "local-dev".to_string(),
        tick_rate_ms: 100,
        gateway: Some(GatewayInteractiveConfig {
            base_url: "http://127.0.0.1:8791".to_string(),
            auth_token: None,
            session_key: "default".to_string(),
            request_timeout_ms: 45_000,
        }),
    });
    for event in parse_sse_frames(sample_success_sse()).expect("parse SSE") {
        app.apply_gateway_event(event);
    }

    assert_eq!(app.status.agent_state, AgentStateDisplay::Idle);
    assert!(app.chat.messages().iter().any(|message| {
        message.role == MessageRole::Assistant && message.content == "hello world"
    }));
    assert!(app.tools.entries().iter().any(|entry| entry.name == "turn"));
}

#[test]
fn red_spec_3581_gateway_failed_events_drive_interactive_error_state() {
    let mut app = App::new(AppConfig {
        model: "openai/gpt-5.2".to_string(),
        profile: "local-dev".to_string(),
        tick_rate_ms: 100,
        gateway: Some(GatewayInteractiveConfig {
            base_url: "http://127.0.0.1:8791".to_string(),
            auth_token: Some("token".to_string()),
            session_key: "default".to_string(),
            request_timeout_ms: 45_000,
        }),
    });
    for event in parse_sse_frames(sample_failure_sse()).expect("parse SSE") {
        app.apply_gateway_event(event);
    }

    assert_eq!(app.status.agent_state, AgentStateDisplay::Error);
    assert!(app.chat.messages().iter().any(|message| {
        message.role == MessageRole::System
            && message.content.contains("model request exceeded budget")
    }));
}
