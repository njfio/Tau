use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayInteractiveConfig {
    pub base_url: String,
    pub auth_token: Option<String>,
    pub session_key: String,
    pub request_timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorStateEvent {
    pub entity: String,
    pub status: String,
    pub phase: Option<String>,
    pub artifact_kind: Option<String>,
    pub response_id: Option<String>,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayUiEvent {
    OperatorState(OperatorStateEvent),
    AssistantDelta(String),
    AssistantDone(String),
    ResponseCompleted(Option<String>),
    Failure(String),
}

#[derive(Debug, Error)]
pub enum GatewayParseError {
    #[error("missing data for SSE event `{event_name}`")]
    MissingData { event_name: String },
    #[error("invalid JSON for SSE event `{event_name}`: {source}")]
    InvalidJson {
        event_name: String,
        #[source]
        source: serde_json::Error,
    },
}

pub fn parse_sse_frames(raw: &str) -> Result<Vec<GatewayUiEvent>, GatewayParseError> {
    raw.split("\n\n")
        .filter(|frame| !frame.trim().is_empty())
        .try_fold(Vec::new(), |mut events, frame| {
            events.extend(parse_frame(frame)?);
            Ok(events)
        })
}

fn parse_frame(frame: &str) -> Result<Vec<GatewayUiEvent>, GatewayParseError> {
    let (event_name, data) = frame_parts(frame);
    if data.trim().is_empty() {
        return Err(GatewayParseError::MissingData { event_name });
    }
    if data.trim() == "[DONE]" {
        return Ok(Vec::new());
    }

    let payload = serde_json::from_str::<Value>(&data).map_err(|source| {
        GatewayParseError::InvalidJson {
            event_name: event_name.clone(),
            source,
        }
    })?;
    let mut events = operator_state_event(&payload)
        .into_iter()
        .map(GatewayUiEvent::OperatorState)
        .collect::<Vec<_>>();

    match event_name.as_str() {
        "response.output_text.delta" => push_text_event(&mut events, &payload, "delta", true),
        "response.output_text.done" => push_text_event(&mut events, &payload, "text", false),
        "response.completed" => events.push(GatewayUiEvent::ResponseCompleted(
            payload
                .pointer("/response/output_text")
                .and_then(Value::as_str)
                .map(str::to_string),
        )),
        "response.failed" => {
            if let Some(message) = payload.pointer("/error/message").and_then(Value::as_str) {
                events.push(GatewayUiEvent::Failure(message.to_string()));
            }
        }
        _ => {}
    }
    Ok(events)
}

fn frame_parts(frame: &str) -> (String, String) {
    let mut event_name = "message".to_string();
    let mut data_lines = Vec::new();
    for line in frame.lines() {
        if let Some(value) = line.strip_prefix("event:") {
            event_name = value.trim().to_string();
        }
        if let Some(value) = line.strip_prefix("data:") {
            data_lines.push(value.trim_start().to_string());
        }
    }
    (event_name, data_lines.join("\n"))
}

fn operator_state_event(payload: &Value) -> Option<OperatorStateEvent> {
    let state = payload.get("operator_state")?;
    Some(OperatorStateEvent {
        entity: state.get("entity")?.as_str()?.to_string(),
        status: state.get("status")?.as_str()?.to_string(),
        phase: string_field(state, "phase"),
        artifact_kind: string_field(state, "artifact_kind"),
        response_id: string_field(state, "response_id"),
        reason_code: string_field(state, "reason_code"),
    })
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn push_text_event(
    events: &mut Vec<GatewayUiEvent>,
    payload: &Value,
    key: &str,
    is_delta: bool,
) {
    let Some(text) = payload.get(key).and_then(Value::as_str) else {
        return;
    };
    if is_delta {
        events.push(GatewayUiEvent::AssistantDelta(text.to_string()));
    } else {
        events.push(GatewayUiEvent::AssistantDone(text.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use crate::interactive::app::{App, AppConfig};
    use crate::interactive::chat::MessageRole;
    use crate::interactive::status::AgentStateDisplay;

    use super::{
        parse_sse_frames, GatewayInteractiveConfig, GatewayUiEvent, OperatorStateEvent,
    };

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
}
