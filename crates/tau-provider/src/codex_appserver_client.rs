//! WebSocket client for the codex app-server protocol.
//!
//! Connects to a running `codex app-server --listen ws://IP:PORT` instance and
//! implements `LlmClient` with real streaming, session continuity, and tool evidence.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tau_ai::{
    ChatRequest, ChatResponse, ChatUsage, ContentBlock, Message, MessageRole, StreamDeltaHandler,
    TauAiError,
};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite;

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Configuration for the codex app-server WebSocket client.
pub struct CodexAppServerConfig {
    pub url: String,
    pub timeout_ms: u64,
    pub approval_policy: String,
    pub sandbox: String,
}

/// Persistent WebSocket connection state.
struct AppServerConnection {
    ws: WsStream,
    thread_id: String,
}

/// WebSocket-based LLM client that talks to `codex app-server`.
pub struct CodexAppServerClient {
    config: CodexAppServerConfig,
    connection: Arc<Mutex<Option<AppServerConnection>>>,
    next_id: AtomicU64,
}

impl CodexAppServerClient {
    pub fn new(config: CodexAppServerConfig) -> Self {
        Self {
            config,
            connection: Arc::new(Mutex::new(None)),
            next_id: AtomicU64::new(1),
        }
    }

    fn next_request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    async fn send_json(ws: &mut WsStream, value: &Value) -> Result<(), TauAiError> {
        let text: tungstenite::Message = value.to_string().into();
        ws.send(text)
            .await
            .map_err(|e| TauAiError::InvalidResponse(format!("WS send failed: {e}")))
    }

    fn parse_ws_text(msg: tungstenite::Message) -> Option<String> {
        match msg {
            tungstenite::Message::Text(t) => Some(t.to_string()),
            _ => None,
        }
    }
    async fn connect(&self) -> Result<AppServerConnection, TauAiError> {
        let (ws, _response) = tokio_tungstenite::connect_async(&self.config.url)
            .await
            .map_err(|e| {
                TauAiError::InvalidResponse(format!(
                    "codex app-server WebSocket connect failed: {e}"
                ))
            })?;

        let mut ws = ws;

        // initialize
        let init_id = self.next_request_id();
        Self::send_json(
            &mut ws,
            &json!({
                "jsonrpc": "2.0",
                "id": init_id,
                "method": "initialize",
                "params": {
                    "clientInfo": {
                        "name": "tau",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            }),
        )
        .await?;

        self.wait_for_response(&mut ws, init_id).await?;

        // thread/start
        let thread_req_id = self.next_request_id();
        Self::send_json(
            &mut ws,
            &json!({
                "jsonrpc": "2.0",
                "id": thread_req_id,
                "method": "thread/start",
                "params": {
                    "approvalPolicy": self.config.approval_policy,
                    "sandbox": self.config.sandbox
                }
            }),
        )
        .await?;

        let thread_response = self.wait_for_response(&mut ws, thread_req_id).await?;
        let tid = thread_response["thread"]["id"]
            .as_str()
            .ok_or_else(|| {
                TauAiError::InvalidResponse("thread/start response missing thread.id".to_string())
            })?
            .to_string();

        tracing::info!(
            thread_id = %tid,
            url = %self.config.url,
            "codex app-server connection established"
        );

        Ok(AppServerConnection { ws, thread_id: tid })
    }

    /// Wait for a JSON-RPC response with a matching id, skipping notifications.
    async fn wait_for_response(&self, ws: &mut WsStream, id: u64) -> Result<Value, TauAiError> {
        let timeout = std::time::Duration::from_millis(self.config.timeout_ms);
        loop {
            let msg = tokio::time::timeout(timeout, ws.next())
                .await
                .map_err(|_| {
                    TauAiError::InvalidResponse(format!(
                        "codex app-server timed out waiting for response id={id}"
                    ))
                })?
                .ok_or_else(|| {
                    TauAiError::InvalidResponse(
                        "codex app-server WebSocket closed unexpectedly".to_string(),
                    )
                })?
                .map_err(|e| TauAiError::InvalidResponse(format!("WS read error: {e}")))?;

            let Some(text) = Self::parse_ws_text(msg) else {
                continue;
            };

            let parsed: Value = serde_json::from_str(&text)?;

            if let Some(error) = parsed.get("error") {
                let msg = error["message"].as_str().unwrap_or("unknown error");
                let code = error["code"].as_i64().unwrap_or(-1);
                return Err(TauAiError::InvalidResponse(format!(
                    "codex app-server error (code={code}): {msg}"
                )));
            }

            if parsed.get("id").and_then(|v| v.as_u64()) == Some(id) {
                if let Some(result) = parsed.get("result") {
                    return Ok(result.clone());
                }
            }
        }
    }

    /// Send turn/start and stream events until turn/completed.
    async fn execute_turn(
        &self,
        conn: &mut AppServerConnection,
        request: &ChatRequest,
        on_delta: Option<&StreamDeltaHandler>,
    ) -> Result<ChatResponse, TauAiError> {
        let turn_req_id = self.next_request_id();
        let prompt = Self::extract_prompt(request);

        Self::send_json(
            &mut conn.ws,
            &json!({
                "jsonrpc": "2.0",
                "id": turn_req_id,
                "method": "turn/start",
                "params": {
                    "threadId": conn.thread_id,
                    "input": [{"type": "text", "text": prompt}]
                }
            }),
        )
        .await?;

        let mut response_text = String::new();
        let mut usage = ChatUsage::default();
        let timeout = std::time::Duration::from_millis(self.config.timeout_ms);

        loop {
            let msg = tokio::time::timeout(timeout, conn.ws.next())
                .await
                .map_err(|_| {
                    TauAiError::InvalidResponse("codex app-server turn timed out".to_string())
                })?
                .ok_or_else(|| {
                    TauAiError::InvalidResponse(
                        "codex app-server WebSocket closed during turn".to_string(),
                    )
                })?
                .map_err(|e| TauAiError::InvalidResponse(format!("WS read error: {e}")))?;

            let Some(text) = Self::parse_ws_text(msg) else {
                continue;
            };

            let parsed: Value = serde_json::from_str(&text)?;

            // Check for turn/start response (id match)
            if parsed.get("id").and_then(|v| v.as_u64()) == Some(turn_req_id) {
                if let Some(error) = parsed.get("error") {
                    let msg = error["message"].as_str().unwrap_or("unknown");
                    return Err(TauAiError::InvalidResponse(format!(
                        "turn/start failed: {msg}"
                    )));
                }
                continue;
            }

            let Some(method) = parsed["method"].as_str() else {
                continue;
            };
            let params = &parsed["params"];

            match method {
                "item/agentMessage/delta" => {
                    if let Some(delta) = params["delta"].as_str() {
                        response_text.push_str(delta);
                        if let Some(handler) = on_delta {
                            handler(delta.to_string());
                        }
                    }
                }
                "thread/tokenUsage/updated" => {
                    if let Some(total) = params.get("tokenUsage").and_then(|tu| tu.get("total")) {
                        usage.input_tokens =
                            total["inputTokens"].as_u64().unwrap_or(usage.input_tokens);
                        usage.output_tokens = total["outputTokens"]
                            .as_u64()
                            .unwrap_or(usage.output_tokens);
                        usage.total_tokens = total["totalTokens"]
                            .as_u64()
                            .unwrap_or(usage.input_tokens + usage.output_tokens);
                        usage.cached_input_tokens =
                            total["cachedInputTokens"].as_u64().unwrap_or(0);
                    }
                }
                "turn/completed" => {
                    let status = params["turn"]["status"].as_str().unwrap_or("unknown");
                    if status == "failed" {
                        let error_msg = params["turn"]["error"]["message"]
                            .as_str()
                            .unwrap_or("turn failed");
                        return Err(TauAiError::InvalidResponse(format!(
                            "codex turn failed: {error_msg}"
                        )));
                    }
                    break;
                }
                _ => {}
            }
        }

        if response_text.is_empty() {
            return Err(TauAiError::InvalidResponse(
                "codex app-server returned empty response".to_string(),
            ));
        }

        Ok(ChatResponse {
            message: Message::assistant_text(response_text),
            finish_reason: Some("stop".to_string()),
            usage,
        })
    }

    fn extract_prompt(request: &ChatRequest) -> String {
        for msg in request.messages.iter().rev() {
            if msg.role == MessageRole::User {
                for block in &msg.content {
                    if let ContentBlock::Text { text } = block {
                        return text.clone();
                    }
                }
            }
        }
        String::new()
    }
}

#[async_trait]
impl tau_ai::LlmClient for CodexAppServerClient {
    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        self.complete_with_stream(request, None).await
    }

    async fn complete_with_stream(
        &self,
        request: ChatRequest,
        on_delta: Option<StreamDeltaHandler>,
    ) -> Result<ChatResponse, TauAiError> {
        let mut guard = self.connection.lock().await;

        if guard.is_none() {
            let conn = self.connect().await?;
            *guard = Some(conn);
        }

        let conn = guard.as_mut().expect("connection just established");
        let result = self.execute_turn(conn, &request, on_delta.as_ref()).await;

        // On error, drop the connection so next call reconnects
        if result.is_err() {
            *guard = None;
        }

        result
    }
}
