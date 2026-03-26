use std::{io::BufRead, sync::mpsc, thread, time::Duration};

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

/// Events streamed from the gateway during a turn.
#[derive(Debug, Clone)]
pub enum GatewayStreamEvent {
    /// A text delta — append to the assistant message in real time.
    Delta(String),
    /// Token usage update (after each agent turn).
    UsageUpdate { input_tokens: u64, output_tokens: u64, total_tokens: u64 },
    /// Cost update.
    CostUpdate { cumulative_cost_cents: f64 },
    /// Tool execution started.
    ToolStart { tool_name: String, arguments_preview: String },
    /// Tool execution ended.
    ToolEnd { tool_name: String, success: bool, output_preview: String },
    /// The turn completed successfully.
    Done(GatewayTurnResult),
    /// The turn failed.
    Error(String),
}

pub type GatewayTurnResponse = Result<GatewayTurnResult, String>;

/// Spawn a gateway turn with SSE streaming.
/// Returns a channel that emits streaming deltas followed by a final Done or Error.
pub fn spawn_gateway_turn_streaming(
    config: GatewayRuntimeConfig,
    prompt: String,
) -> mpsc::Receiver<GatewayStreamEvent> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        submit_streaming_turn(&config, &prompt, &sender);
    });
    receiver
}

/// Legacy non-streaming spawn (kept for backward compatibility).
pub fn spawn_gateway_turn(
    config: GatewayRuntimeConfig,
    prompt: String,
) -> mpsc::Receiver<GatewayTurnResponse> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let result = submit_blocking_turn(&config, &prompt);
        let _ = sender.send(result);
    });
    receiver
}

fn submit_streaming_turn(
    config: &GatewayRuntimeConfig,
    prompt: &str,
    sender: &mpsc::Sender<GatewayStreamEvent>,
) {
    let client = match build_client(config) {
        Ok(c) => c,
        Err(e) => {
            let _ = sender.send(GatewayStreamEvent::Error(e));
            return;
        }
    };

    let response = match build_streaming_request(&client, config, prompt)
        .and_then(|r| r.send().map_err(|e| format!("gateway request failed: {e}")))
    {
        Ok(r) => r,
        Err(e) => {
            let _ = sender.send(GatewayStreamEvent::Error(e));
            return;
        }
    };

    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        let msg = parse_gateway_error(&body)
            .unwrap_or_else(|| format!("gateway request failed with status {status}"));
        let _ = sender.send(GatewayStreamEvent::Error(msg));
        return;
    }

    // Parse SSE stream line by line
    let reader = std::io::BufReader::new(response);
    let mut full_text = String::new();
    let mut total_tokens: u64 = 0;
    let mut got_done = false;

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => break,
        };

        // SSE format: "data: {...}" or "event: ..." lines
        let data = if let Some(stripped) = line.strip_prefix("data: ") {
            stripped
        } else {
            continue;
        };

        // [DONE] marker
        if data == "[DONE]" {
            got_done = true;
            break;
        }

        let Ok(event) = serde_json::from_str::<Value>(data) else {
            continue;
        };

        let event_type = event
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("");

        match event_type {
            "response.output_text.delta" => {
                if let Some(delta) = event.get("delta").and_then(Value::as_str) {
                    full_text.push_str(delta);
                    let _ = sender.send(GatewayStreamEvent::Delta(delta.to_string()));
                }
            }
            "response.usage.delta" => {
                if let Some(usage) = event.get("usage") {
                    let _ = sender.send(GatewayStreamEvent::UsageUpdate {
                        input_tokens: usage.get("input_tokens").and_then(Value::as_u64).unwrap_or(0),
                        output_tokens: usage.get("output_tokens").and_then(Value::as_u64).unwrap_or(0),
                        total_tokens: usage.get("total_tokens").and_then(Value::as_u64).unwrap_or(0),
                    });
                }
            }
            "response.cost.delta" => {
                if let Some(cost) = event.get("cumulative_cost_usd").and_then(Value::as_f64) {
                    let _ = sender.send(GatewayStreamEvent::CostUpdate {
                        cumulative_cost_cents: cost * 100.0,
                    });
                }
            }
            "response.tool.start" => {
                let name = event.get("tool_name").and_then(Value::as_str).unwrap_or("").to_string();
                let args = event.get("arguments_preview").and_then(Value::as_str).unwrap_or("").to_string();
                let _ = sender.send(GatewayStreamEvent::ToolStart {
                    tool_name: name,
                    arguments_preview: args,
                });
            }
            "response.tool.end" => {
                let name = event.get("tool_name").and_then(Value::as_str).unwrap_or("").to_string();
                let success = event.get("success").and_then(Value::as_bool).unwrap_or(false);
                let output = event.get("output_preview").and_then(Value::as_str).unwrap_or("").to_string();
                let _ = sender.send(GatewayStreamEvent::ToolEnd {
                    tool_name: name,
                    success,
                    output_preview: output,
                });
            }
            "response.completed" => {
                // Final response object
                if let Some(response_obj) = event.get("response") {
                    if let Some(text) = response_obj.get("output_text").and_then(Value::as_str) {
                        full_text = text.to_string();
                    }
                    total_tokens = response_obj
                        .get("usage")
                        .and_then(|u| u.get("total_tokens"))
                        .and_then(Value::as_u64)
                        .unwrap_or(0);
                }
                got_done = true;
                break;
            }
            "error" => {
                let msg = event
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(Value::as_str)
                    .unwrap_or("unknown streaming error");
                let _ = sender.send(GatewayStreamEvent::Error(msg.to_string()));
                return;
            }
            _ => {
                // Other event types (response.created, response.in_progress, etc.) — ignore
            }
        }
    }

    if got_done || !full_text.is_empty() {
        let _ = sender.send(GatewayStreamEvent::Done(GatewayTurnResult {
            output_text: full_text.trim().to_string(),
            total_tokens,
        }));
    } else {
        let _ = sender.send(GatewayStreamEvent::Error(
            "gateway stream ended without response".to_string(),
        ));
    }
}

fn submit_blocking_turn(config: &GatewayRuntimeConfig, prompt: &str) -> GatewayTurnResponse {
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
    Ok(request)
}

fn build_streaming_request(
    client: &Client,
    config: &GatewayRuntimeConfig,
    prompt: &str,
) -> Result<reqwest::blocking::RequestBuilder, String> {
    let url = format!("{}/v1/responses", config.base_url.trim_end_matches('/'));
    let payload = json!({
        "input": prompt,
        "stream": true,
        "metadata": {
            "session_id": config.session_key,
        }
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
