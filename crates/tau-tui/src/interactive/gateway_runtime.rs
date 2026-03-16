use std::io::Read;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use reqwest::blocking::Client;
use serde_json::json;
use thiserror::Error;

use super::gateway::{drain_sse_frames, GatewayInteractiveConfig, GatewayUiEvent};

pub struct GatewayRuntime {
    prompt_tx: Sender<String>,
    event_rx: Receiver<GatewayUiEvent>,
}

impl GatewayRuntime {
    pub fn start(config: GatewayInteractiveConfig) -> Self {
        let (prompt_tx, prompt_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        thread::spawn(move || run_gateway_worker(config, prompt_rx, event_tx));
        Self {
            prompt_tx,
            event_rx,
        }
    }

    pub fn submit(&self, prompt: String) -> Result<(), GatewayRuntimeError> {
        self.prompt_tx
            .send(prompt)
            .map_err(|_| GatewayRuntimeError::ChannelClosed)
    }

    pub fn drain_events(&self) -> Vec<GatewayUiEvent> {
        let mut events = Vec::new();
        loop {
            match self.event_rx.try_recv() {
                Ok(event) => events.push(event),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return events,
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum GatewayRuntimeError {
    #[error("gateway request channel closed")]
    ChannelClosed,
    #[error("failed to build gateway client: {0}")]
    ClientBuild(String),
    #[error("gateway request failed: {0}")]
    Request(String),
}

fn run_gateway_worker(
    config: GatewayInteractiveConfig,
    prompt_rx: Receiver<String>,
    event_tx: Sender<GatewayUiEvent>,
) {
    let client = match build_client(config.request_timeout_ms) {
        Ok(client) => client,
        Err(error) => return emit_failure(&event_tx, error.to_string()),
    };
    while let Ok(prompt) = prompt_rx.recv() {
        if let Err(error) = stream_prompt(&client, &config, &prompt, &event_tx) {
            emit_failure(&event_tx, error.to_string());
        }
    }
}

fn build_client(timeout_ms: u64) -> Result<Client, GatewayRuntimeError> {
    Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .map_err(|error| GatewayRuntimeError::ClientBuild(error.to_string()))
}

fn stream_prompt(
    client: &Client,
    config: &GatewayInteractiveConfig,
    prompt: &str,
    event_tx: &Sender<GatewayUiEvent>,
) -> Result<(), GatewayRuntimeError> {
    let mut request = client
        .post(format!(
            "{}/v1/responses",
            trimmed_base_url(&config.base_url)
        ))
        .json(&json!({
            "input": prompt,
            "stream": true,
            "metadata": { "session_id": config.session_key },
        }));
    if let Some(token) = &config.auth_token {
        request = request.bearer_auth(token);
    }

    let mut response = request
        .send()
        .map_err(|error| GatewayRuntimeError::Request(error.to_string()))?;
    if !response.status().is_success() {
        return Err(GatewayRuntimeError::Request(read_failure_body(response)));
    }

    let mut chunk = [0_u8; 2048];
    let mut buffer = String::new();
    loop {
        let read = response
            .read(&mut chunk)
            .map_err(|error| GatewayRuntimeError::Request(error.to_string()))?;
        if read == 0 {
            return emit_buffered_events(&mut buffer, event_tx);
        }
        buffer.push_str(&String::from_utf8_lossy(&chunk[..read]));
        emit_buffered_events(&mut buffer, event_tx)?;
    }
}

fn emit_buffered_events(
    buffer: &mut String,
    event_tx: &Sender<GatewayUiEvent>,
) -> Result<(), GatewayRuntimeError> {
    for event in
        drain_sse_frames(buffer).map_err(|error| GatewayRuntimeError::Request(error.to_string()))?
    {
        let _ = event_tx.send(event);
    }
    Ok(())
}

fn emit_failure(event_tx: &Sender<GatewayUiEvent>, message: String) {
    let _ = event_tx.send(GatewayUiEvent::Failure(message));
}

fn trimmed_base_url(base_url: &str) -> &str {
    base_url.trim_end_matches('/')
}

fn read_failure_body(response: reqwest::blocking::Response) -> String {
    let status = response.status();
    let body = response.text().unwrap_or_else(|_| String::new());
    format!("status={} body={body}", status.as_u16())
}
