use std::{sync::mpsc, thread, time::Duration};

use reqwest::blocking::Client;
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
) -> mpsc::Receiver<GatewayTurnResponse> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let result = submit_gateway_turn(&config, &prompt);
        let _ = sender.send(result);
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

fn submit_gateway_turn(config: &GatewayRuntimeConfig, prompt: &str) -> GatewayTurnResponse {
    let client = build_client(config)?;
    let response = build_request(&client, config, prompt)?
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

    parse_success_response(&body)
}

fn parse_gateway_error(body: &str) -> Option<String> {
    let value = serde_json::from_str::<Value>(body).ok()?;
    value
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .map(|message| message.to_string())
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
