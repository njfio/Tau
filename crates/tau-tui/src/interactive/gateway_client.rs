use std::{
    io::{BufRead, BufReader, Read},
    sync::mpsc,
    thread,
    time::Duration,
};

use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayRuntimeConfig {
    pub base_url: String,
    pub auth_token: Option<String>,
    pub session_key: String,
    pub mission_id: Option<String>,
    pub request_timeout_ms: u64,
}

impl Default for GatewayRuntimeConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8791".to_string(),
            auth_token: None,
            session_key: "default".to_string(),
            mission_id: None,
            request_timeout_ms: 600_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayTurnResult {
    pub output_text: String,
    pub total_tokens: u64,
}

pub type GatewayTurnResponse = Result<GatewayTurnResult, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayToolStatus {
    Success,
    Failed,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayTurnEvent {
    TextDelta(String),
    ToolStarted {
        tool_call_id: String,
        tool_name: String,
        detail: String,
    },
    ToolCompleted {
        tool_call_id: String,
        tool_name: String,
        status: GatewayToolStatus,
        detail: String,
    },
    Finished(GatewayTurnResponse),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GatewayMissionVerifierSummary {
    pub status: String,
    pub reason_code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GatewayMissionCompletionSummary {
    pub status: String,
    pub summary: String,
    #[serde(default)]
    pub next_step: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GatewayMissionSnapshot {
    pub mission_id: String,
    pub session_key: String,
    pub status: String,
    pub goal_summary: String,
    pub latest_output_summary: String,
    pub iteration_count: usize,
    pub updated_unix_ms: u64,
    pub latest_verifier: GatewayMissionVerifierSummary,
    #[serde(default)]
    pub latest_completion: Option<GatewayMissionCompletionSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct GatewayMissionListEnvelope {
    missions: Vec<GatewayMissionSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct GatewayMissionDetailEnvelope {
    mission: GatewayMissionSnapshot,
}

pub fn spawn_gateway_turn(
    config: GatewayRuntimeConfig,
    prompt: String,
) -> mpsc::Receiver<GatewayTurnEvent> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let result = submit_gateway_turn(&config, &prompt, &sender);
        let _ = sender.send(GatewayTurnEvent::Finished(result));
    });
    receiver
}

pub fn fetch_gateway_missions(
    config: &GatewayRuntimeConfig,
    limit: usize,
) -> Result<Vec<GatewayMissionSnapshot>, String> {
    let client = build_client(config)?;
    let mut request = client.get(format!(
        "{}/gateway/missions?limit={}",
        config.base_url.trim_end_matches('/'),
        limit.clamp(1, 200)
    ));
    if let Some(token) = &config.auth_token {
        request = request.bearer_auth(token);
    }
    let response = request
        .send()
        .map_err(|error| format!("gateway request failed: {error}"))?;
    let status = response.status();
    let body = response
        .text()
        .map_err(|error| format!("failed to read gateway response body: {error}"))?;
    if !status.is_success() {
        return Err(parse_gateway_error(&body)
            .unwrap_or_else(|| format!("gateway request failed with status {status}")));
    }
    let parsed = serde_json::from_str::<GatewayMissionListEnvelope>(&body)
        .map_err(|error| format!("failed to parse gateway mission list: {error}"))?;
    Ok(parsed.missions)
}

pub fn fetch_gateway_mission_detail(
    config: &GatewayRuntimeConfig,
    mission_id: &str,
) -> Result<GatewayMissionSnapshot, String> {
    let client = build_client(config)?;
    let mut request = client.get(format!(
        "{}/gateway/missions/{}",
        config.base_url.trim_end_matches('/'),
        mission_id.trim()
    ));
    if let Some(token) = &config.auth_token {
        request = request.bearer_auth(token);
    }
    let response = request
        .send()
        .map_err(|error| format!("gateway request failed: {error}"))?;
    let status = response.status();
    let body = response
        .text()
        .map_err(|error| format!("failed to read gateway response body: {error}"))?;
    if !status.is_success() {
        return Err(parse_gateway_error(&body)
            .unwrap_or_else(|| format!("gateway request failed with status {status}")));
    }
    let parsed = serde_json::from_str::<GatewayMissionDetailEnvelope>(&body)
        .map_err(|error| format!("failed to parse gateway mission detail: {error}"))?;
    Ok(parsed.mission)
}

fn submit_gateway_turn(
    config: &GatewayRuntimeConfig,
    prompt: &str,
    sender: &mpsc::Sender<GatewayTurnEvent>,
) -> GatewayTurnResponse {
    let client = build_client(config)?;
    let mut response = build_request(&client, config, prompt)?
        .send()
        .map_err(|error| format!("gateway request failed: {error}"))?;
    let status = response.status();
    let is_stream = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|content_type| content_type.contains("text/event-stream"));

    if !status.is_success() {
        let mut body = String::new();
        response
            .read_to_string(&mut body)
            .map_err(|error| format!("failed to read gateway response body: {error}"))?;
        return Err(parse_gateway_error(&body)
            .unwrap_or_else(|| format!("gateway request failed with status {status}")));
    }

    if is_stream {
        parse_stream_response(response, sender)
    } else {
        let body = response
            .text()
            .map_err(|error| format!("failed to read gateway response body: {error}"))?;
        parse_success_response(&body)
    }
}

fn parse_gateway_error(body: &str) -> Option<String> {
    let value = serde_json::from_str::<Value>(body).ok()?;
    let error = value.get("error")?;
    let code = error
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .map(|message| message.to_string())?;
    Some(actionable_gateway_error_message(code, &message))
}

fn actionable_gateway_error_message(code: &str, message: &str) -> String {
    let normalized = message.to_ascii_lowercase();
    if code == "unsupported_codex_auth_model" || normalized.contains("unsupported codex auth model")
    {
        return format!(
            "{message}; select a supported Codex-auth model such as gpt-5-codex or gpt-5.3-codex, or change OpenAI auth mode to api-key"
        );
    }
    message.to_string()
}

fn build_client(config: &GatewayRuntimeConfig) -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_millis(config.request_timeout_ms))
        .build()
        .map_err(|error| format!("failed to build gateway client: {error}"))
}

fn build_request(
    client: &Client,
    config: &GatewayRuntimeConfig,
    prompt: &str,
) -> Result<reqwest::blocking::RequestBuilder, String> {
    let url = format!("{}/v1/responses", config.base_url.trim_end_matches('/'));
    let mut metadata = serde_json::Map::from_iter([(
        "session_id".to_string(),
        Value::String(config.session_key.clone()),
    )]);
    if let Some(mission_id) = config
        .mission_id
        .as_deref()
        .map(str::trim)
        .filter(|mission_id| !mission_id.is_empty())
    {
        metadata.insert(
            "mission_id".to_string(),
            Value::String(mission_id.to_string()),
        );
    }
    let payload = json!({
        "input": prompt,
        "stream": true,
        "metadata": Value::Object(metadata),
    });

    let mut request = client.post(url).json(&payload);
    if let Some(token) = &config.auth_token {
        request = request.bearer_auth(token);
    }
    Ok(request)
}

fn parse_success_response(body: &str) -> GatewayTurnResponse {
    let value: Value = serde_json::from_str(body)
        .map_err(|error| format!("failed to parse gateway response: {error}"))?;
    let output_text = parse_output_text(&value)?;
    let total_tokens = parse_total_tokens(&value);
    Ok(GatewayTurnResult {
        output_text,
        total_tokens,
    })
}

fn parse_stream_response(
    response: reqwest::blocking::Response,
    sender: &mpsc::Sender<GatewayTurnEvent>,
) -> GatewayTurnResponse {
    let mut reader = BufReader::new(response);
    let mut event_name = String::new();
    let mut data_lines = Vec::<String>::new();
    let mut output_text = String::new();
    let mut total_tokens = 0;

    loop {
        let mut line = String::new();
        let read = reader
            .read_line(&mut line)
            .map_err(|error| format!("failed to read gateway stream: {error}"))?;
        if read == 0 {
            break;
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            if let Some(result) = process_sse_frame(
                &event_name,
                &data_lines,
                sender,
                &mut output_text,
                &mut total_tokens,
            )? {
                return result;
            }
            event_name.clear();
            data_lines.clear();
            continue;
        }
        if let Some(rest) = line.strip_prefix("event:") {
            event_name = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim_start().to_string());
        }
    }

    if !event_name.is_empty() || !data_lines.is_empty() {
        if let Some(result) = process_sse_frame(
            &event_name,
            &data_lines,
            sender,
            &mut output_text,
            &mut total_tokens,
        )? {
            return result;
        }
    }

    if output_text.trim().is_empty() {
        Err("gateway stream ended without response.completed".to_string())
    } else {
        Ok(GatewayTurnResult {
            output_text,
            total_tokens,
        })
    }
}

fn process_sse_frame(
    event_name: &str,
    data_lines: &[String],
    sender: &mpsc::Sender<GatewayTurnEvent>,
    output_text: &mut String,
    total_tokens: &mut u64,
) -> Result<Option<GatewayTurnResponse>, String> {
    if data_lines.is_empty() {
        return Ok(None);
    }
    let data = data_lines.join("\n");
    if data.trim() == "[DONE]" {
        return Ok(None);
    }
    let value: Value = serde_json::from_str(&data)
        .map_err(|error| format!("failed to parse gateway stream frame {event_name}: {error}"))?;

    match event_name {
        "response.output_text.delta" => {
            if let Some(delta) = value.get("delta").and_then(Value::as_str) {
                if !delta.is_empty() {
                    output_text.push_str(delta);
                    let _ = sender.send(GatewayTurnEvent::TextDelta(delta.to_string()));
                }
            }
        }
        "response.output_text.done" => {
            if output_text.trim().is_empty() {
                if let Some(text) = value.get("text").and_then(Value::as_str) {
                    output_text.push_str(text);
                }
            }
        }
        "response.tool_execution.started" => {
            let tool_call_id = value
                .get("tool_call_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let tool_name = value
                .get("tool_name")
                .and_then(Value::as_str)
                .unwrap_or("tool")
                .to_string();
            let detail = value
                .get("arguments")
                .filter(|arguments| !arguments.is_null())
                .map(Value::to_string)
                .unwrap_or_default();
            let _ = sender.send(GatewayTurnEvent::ToolStarted {
                tool_call_id,
                tool_name,
                detail,
            });
        }
        "response.tool_execution.completed" => {
            let tool_call_id = value
                .get("tool_call_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let tool_name = value
                .get("tool_name")
                .and_then(Value::as_str)
                .unwrap_or("tool")
                .to_string();
            let status = if value
                .get("timed_out")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                GatewayToolStatus::Timeout
            } else if value
                .get("success")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                GatewayToolStatus::Success
            } else {
                GatewayToolStatus::Failed
            };
            let detail = value
                .get("latency_ms")
                .and_then(Value::as_u64)
                .map(|latency_ms| format!("latency_ms={latency_ms}"))
                .unwrap_or_default();
            let _ = sender.send(GatewayTurnEvent::ToolCompleted {
                tool_call_id,
                tool_name,
                status,
                detail,
            });
        }
        "response.completed" => {
            let response = value.get("response").unwrap_or(&value);
            *total_tokens = parse_total_tokens(response);
            let final_text = parse_output_text(response).unwrap_or_else(|_| output_text.clone());
            return Ok(Some(Ok(GatewayTurnResult {
                output_text: final_text,
                total_tokens: *total_tokens,
            })));
        }
        "response.failed" => {
            let message = value
                .get("error")
                .and_then(|error| error.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("gateway stream failed")
                .to_string();
            return Ok(Some(Err(message)));
        }
        _ => {}
    }

    Ok(None)
}

fn parse_output_text(value: &Value) -> Result<String, String> {
    value
        .get("output_text")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(|text| text.to_string())
        .ok_or_else(|| "gateway response missing output_text".to_string())
}

fn parse_total_tokens(value: &Value) -> u64 {
    value
        .get("usage")
        .and_then(|usage| usage.get("total_tokens"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}
