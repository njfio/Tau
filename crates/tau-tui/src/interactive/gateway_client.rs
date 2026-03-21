use std::{sync::mpsc, thread, time::Duration};

use reqwest::blocking::Client;
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayRuntimeConfig {
    pub base_url: String,
    pub auth_token: Option<String>,
    pub session_key: String,
    pub request_timeout_ms: u64,
}

impl Default for GatewayRuntimeConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8791".to_string(),
            auth_token: None,
            session_key: "default".to_string(),
            request_timeout_ms: 180_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayTurnResult {
    pub output_text: String,
    pub total_tokens: u64,
}

pub type GatewayTurnResponse = Result<GatewayTurnResult, String>;

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

fn submit_gateway_turn(config: &GatewayRuntimeConfig, prompt: &str) -> GatewayTurnResponse {
    let client = Client::builder()
        .timeout(Duration::from_millis(config.request_timeout_ms))
        .build()
        .map_err(|error| format!("failed to build gateway client: {error}"))?;

    let url = format!("{}/v1/responses", config.base_url.trim_end_matches('/'));
    let payload = json!({
        "input": prompt,
        "metadata": {
            "session_id": config.session_key,
        }
    });

    let mut request = client.post(url).json(&payload);
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

    let value: Value = serde_json::from_str(&body)
        .map_err(|error| format!("failed to parse gateway response: {error}"))?;
    let output_text = value
        .get("output_text")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| "gateway response missing output_text".to_string())?
        .to_string();
    let total_tokens = value
        .get("usage")
        .and_then(|usage| usage.get("total_tokens"))
        .and_then(Value::as_u64)
        .unwrap_or(0);

    Ok(GatewayTurnResult {
        output_text,
        total_tokens,
    })
}

fn parse_gateway_error(body: &str) -> Option<String> {
    let value = serde_json::from_str::<Value>(body).ok()?;
    value
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .map(|message| message.to_string())
}
