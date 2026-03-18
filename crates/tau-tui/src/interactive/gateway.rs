use serde_json::Value;
use thiserror::Error;

const LOCAL_CODEX_MODEL_HINT: &str = "gpt-5.2-codex";
const CHATGPT_ACCOUNT_MODEL_ERROR: &str = "not supported when using Codex with a ChatGPT account";

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

pub fn drain_sse_frames(buffer: &mut String) -> Result<Vec<GatewayUiEvent>, GatewayParseError> {
    let mut events = Vec::new();
    while let Some(index) = buffer.find("\n\n") {
        let frame = buffer[..index].to_string();
        buffer.drain(..index + 2);
        if frame.trim().is_empty() {
            continue;
        }
        events.extend(parse_frame(&frame)?);
    }
    Ok(events)
}

pub fn parse_sse_frames(raw: &str) -> Result<Vec<GatewayUiEvent>, GatewayParseError> {
    let mut buffer = raw.to_string();
    let mut events = drain_sse_frames(&mut buffer)?;
    if !buffer.trim().is_empty() {
        events.extend(parse_frame(buffer.trim())?);
    }
    Ok(events)
}

fn parse_frame(frame: &str) -> Result<Vec<GatewayUiEvent>, GatewayParseError> {
    if is_comment_frame(frame) {
        return Ok(Vec::new());
    }

    let (event_name, data) = frame_parts(frame);
    if data.trim().is_empty() {
        return Err(GatewayParseError::MissingData { event_name });
    }
    if data.trim() == "[DONE]" {
        return Ok(Vec::new());
    }

    let payload =
        serde_json::from_str::<Value>(&data).map_err(|source| GatewayParseError::InvalidJson {
            event_name: event_name.clone(),
            source,
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
                events.push(GatewayUiEvent::Failure(normalize_codex_auth_failure(
                    message,
                )));
            }
        }
        _ => {}
    }
    Ok(events)
}

fn is_comment_frame(frame: &str) -> bool {
    frame
        .lines()
        .map(str::trim)
        .all(|line| line.is_empty() || line.starts_with(':'))
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

fn push_text_event(events: &mut Vec<GatewayUiEvent>, payload: &Value, key: &str, is_delta: bool) {
    let Some(text) = payload.get(key).and_then(Value::as_str) else {
        return;
    };
    if is_delta {
        events.push(GatewayUiEvent::AssistantDelta(text.to_string()));
        return;
    }
    events.push(GatewayUiEvent::AssistantDone(text.to_string()));
}

fn normalize_codex_auth_failure(message: &str) -> String {
    if message.contains("openai/gpt-5.2") && message.contains(CHATGPT_ACCOUNT_MODEL_ERROR) {
        return format!("{message} Use `{LOCAL_CODEX_MODEL_HINT}` instead.");
    }
    message.to_string()
}
