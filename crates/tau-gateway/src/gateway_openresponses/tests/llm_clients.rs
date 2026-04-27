use super::*;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::time::Duration;
use tau_ai::{ChatRequest, ChatResponse, ChatUsage, Message, TauAiError};
use tokio::sync::Mutex as AsyncMutex;

#[derive(Clone, Default)]
pub(super) struct MockGatewayLlmClient {
    request_message_counts: Arc<Mutex<Vec<usize>>>,
}

#[async_trait]
impl LlmClient for MockGatewayLlmClient {
    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        self.complete_with_stream(request, None).await
    }

    async fn complete_with_stream(
        &self,
        request: ChatRequest,
        on_delta: Option<StreamDeltaHandler>,
    ) -> Result<ChatResponse, TauAiError> {
        let message_count = request.messages.len();
        if let Ok(mut counts) = self.request_message_counts.lock() {
            counts.push(message_count);
        }
        if let Some(handler) = on_delta {
            handler("messages=".to_string());
            handler(message_count.to_string());
        }
        let reply = format!("messages={message_count}");
        Ok(ChatResponse {
            message: Message::assistant_text(reply),
            finish_reason: Some("stop".to_string()),
            usage: ChatUsage {
                input_tokens: message_count as u64,
                output_tokens: 2,
                total_tokens: message_count as u64 + 2,
                cached_input_tokens: 0,
            },
        })
    }
}

#[derive(Clone, Default)]
pub(super) struct PanicGatewayLlmClient;

#[async_trait]
impl LlmClient for PanicGatewayLlmClient {
    async fn complete(&self, _request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        panic!("provider should not be invoked when gateway preflight blocks request");
    }

    async fn complete_with_stream(
        &self,
        _request: ChatRequest,
        _on_delta: Option<StreamDeltaHandler>,
    ) -> Result<ChatResponse, TauAiError> {
        panic!("provider should not be invoked when gateway preflight blocks request");
    }
}

#[derive(Clone)]
pub(super) struct CaptureGatewayLlmClient {
    reply_text: String,
    captured_requests: Arc<Mutex<Vec<ChatRequest>>>,
}

impl CaptureGatewayLlmClient {
    pub(super) fn new(reply_text: &str) -> Self {
        Self {
            reply_text: reply_text.to_string(),
            captured_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(super) fn captured_requests(&self) -> Vec<ChatRequest> {
        self.captured_requests
            .lock()
            .map(|requests| requests.clone())
            .unwrap_or_default()
    }
}

#[async_trait]
impl LlmClient for CaptureGatewayLlmClient {
    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        if let Ok(mut requests) = self.captured_requests.lock() {
            requests.push(request);
        }
        Ok(ChatResponse {
            message: Message::assistant_text(self.reply_text.clone()),
            finish_reason: Some("stop".to_string()),
            usage: ChatUsage::default(),
        })
    }
}

#[derive(Clone, Default)]
pub(super) struct ErrorGatewayLlmClient;

#[async_trait]
impl LlmClient for ErrorGatewayLlmClient {
    async fn complete(&self, _request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        Err(TauAiError::InvalidResponse(
            "forced cortex provider failure".to_string(),
        ))
    }
}

#[derive(Clone)]
pub(super) struct ScriptedGatewayLlmClient {
    responses: Arc<AsyncMutex<VecDeque<ChatResponse>>>,
    captured_requests: Arc<AsyncMutex<Vec<ChatRequest>>>,
}

impl ScriptedGatewayLlmClient {
    pub(super) fn new(responses: Vec<ChatResponse>) -> Self {
        Self {
            responses: Arc::new(AsyncMutex::new(VecDeque::from(responses))),
            captured_requests: Arc::new(AsyncMutex::new(Vec::new())),
        }
    }

    pub(super) async fn captured_requests(&self) -> Vec<ChatRequest> {
        self.captured_requests.lock().await.clone()
    }
}

#[async_trait]
impl LlmClient for ScriptedGatewayLlmClient {
    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        self.captured_requests.lock().await.push(request);
        let mut responses = self.responses.lock().await;
        responses
            .pop_front()
            .ok_or_else(|| TauAiError::InvalidResponse("scripted response queue exhausted".into()))
    }

    async fn complete_with_stream(
        &self,
        request: ChatRequest,
        on_delta: Option<StreamDeltaHandler>,
    ) -> Result<ChatResponse, TauAiError> {
        let response = self.complete(request).await?;
        if let Some(handler) = on_delta {
            let text = response.message.text_content();
            if !text.is_empty() {
                handler(text);
            }
        }
        Ok(response)
    }
}

pub(super) fn scripted_gateway_response(text: &str) -> ChatResponse {
    ChatResponse {
        message: Message::assistant_text(text.to_string()),
        finish_reason: Some("stop".to_string()),
        usage: ChatUsage::default(),
    }
}

#[derive(Clone)]
pub(super) struct DelayedScriptedGatewayLlmClient {
    responses: Arc<AsyncMutex<VecDeque<(u64, ChatResponse)>>>,
    captured_requests: Arc<AsyncMutex<Vec<ChatRequest>>>,
}

impl DelayedScriptedGatewayLlmClient {
    pub(super) fn new(responses: Vec<(u64, ChatResponse)>) -> Self {
        Self {
            responses: Arc::new(AsyncMutex::new(VecDeque::from(responses))),
            captured_requests: Arc::new(AsyncMutex::new(Vec::new())),
        }
    }

    pub(super) async fn captured_requests(&self) -> Vec<ChatRequest> {
        self.captured_requests.lock().await.clone()
    }
}

#[async_trait]
impl LlmClient for DelayedScriptedGatewayLlmClient {
    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        self.captured_requests.lock().await.push(request);
        let delay_ms = {
            let responses = self.responses.lock().await;
            responses
                .front()
                .map(|(delay_ms, _)| *delay_ms)
                .ok_or_else(|| {
                    TauAiError::InvalidResponse("scripted response queue exhausted".into())
                })?
        };
        if delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
        let mut responses = self.responses.lock().await;
        responses
            .pop_front()
            .map(|(_, response)| response)
            .ok_or_else(|| TauAiError::InvalidResponse("scripted response queue exhausted".into()))
    }

    async fn complete_with_stream(
        &self,
        request: ChatRequest,
        on_delta: Option<StreamDeltaHandler>,
    ) -> Result<ChatResponse, TauAiError> {
        let response = self.complete(request).await?;
        if let Some(handler) = on_delta {
            let text = response.message.text_content();
            if !text.is_empty() {
                handler(text);
            }
        }
        Ok(response)
    }
}

#[derive(Clone, Copy)]
pub(super) struct SlowGatewayLlmClient {
    pub(super) delay_ms: u64,
}

#[async_trait]
impl LlmClient for SlowGatewayLlmClient {
    async fn complete(&self, _request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
        Ok(ChatResponse {
            message: Message::assistant_text("slow-complete"),
            finish_reason: Some("stop".to_string()),
            usage: ChatUsage::default(),
        })
    }
}
