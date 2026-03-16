//! Live integration tests that hit real LLM provider APIs.
//!
//! Every test is `#[ignore]` so they never run in CI or local `cargo test`.
//! Run them explicitly via:
//!
//!     TAU_TEST_OPENAI_API_KEY=sk-... cargo test -p tau-ai --test provider_live_integration -- --ignored
//!     TAU_TEST_ANTHROPIC_API_KEY=sk-... cargo test -p tau-ai --test provider_live_integration -- --ignored
//!     TAU_TEST_GOOGLE_API_KEY=... cargo test -p tau-ai --test provider_live_integration -- --ignored

use serde_json::json;
use std::sync::{Arc, Mutex};
use tau_ai::{
    AnthropicClient, AnthropicConfig, ChatRequest, GoogleClient, GoogleConfig, LlmClient, Message,
    OpenAiAuthScheme, OpenAiClient, OpenAiConfig, ToolDefinition,
};

/// Read an environment variable, returning `None` when unset or empty.
fn require_key(var: &str) -> Option<String> {
    match std::env::var(var) {
        Ok(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

fn weather_tool() -> ToolDefinition {
    ToolDefinition {
        name: "get_weather".to_string(),
        description: "Get the current weather for a city".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "The city name"
                }
            },
            "required": ["city"]
        }),
    }
}

fn simple_prompt_request(model: &str) -> ChatRequest {
    ChatRequest {
        model: model.to_string(),
        messages: vec![Message::user("Reply with exactly the word: pong")],
        tools: vec![],
        tool_choice: None,
        json_mode: false,
        max_tokens: Some(20),
        temperature: Some(0.0),
        prompt_cache: Default::default(),
    }
}

fn tool_call_request(model: &str) -> ChatRequest {
    ChatRequest {
        model: model.to_string(),
        messages: vec![Message::user("What is the weather in Paris?")],
        tools: vec![weather_tool()],
        tool_choice: None,
        json_mode: false,
        max_tokens: Some(100),
        temperature: Some(0.0),
        prompt_cache: Default::default(),
    }
}

fn streaming_prompt_request(model: &str) -> ChatRequest {
    ChatRequest {
        model: model.to_string(),
        messages: vec![Message::user("Reply with exactly the word: pong")],
        tools: vec![],
        tool_choice: None,
        json_mode: false,
        max_tokens: Some(20),
        temperature: Some(0.0),
        prompt_cache: Default::default(),
    }
}

// ---------------------------------------------------------------------------
// OpenAI
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn live_openai_simple_prompt() {
    let Some(api_key) = require_key("TAU_TEST_OPENAI_API_KEY") else {
        eprintln!("TAU_TEST_OPENAI_API_KEY not set — skipping");
        return;
    };

    let client = OpenAiClient::new(OpenAiConfig {
        api_key,
        api_base: "https://api.openai.com/v1".to_string(),
        organization: None,
        request_timeout_ms: 30_000,
        max_retries: 2,
        retry_budget_ms: 0,
        retry_jitter: false,
        auth_scheme: OpenAiAuthScheme::Bearer,
        api_version: None,
    })
    .expect("openai client should be created");

    let response = client
        .complete(simple_prompt_request("gpt-4o-mini"))
        .await
        .expect("openai completion should succeed");

    let text = response.message.text_content().to_lowercase();
    assert!(
        text.contains("pong"),
        "expected response to contain 'pong', got: {text}"
    );
}

#[tokio::test]
#[ignore]
async fn live_openai_tool_call_extraction() {
    let Some(api_key) = require_key("TAU_TEST_OPENAI_API_KEY") else {
        eprintln!("TAU_TEST_OPENAI_API_KEY not set — skipping");
        return;
    };

    let client = OpenAiClient::new(OpenAiConfig {
        api_key,
        api_base: "https://api.openai.com/v1".to_string(),
        organization: None,
        request_timeout_ms: 30_000,
        max_retries: 2,
        retry_budget_ms: 0,
        retry_jitter: false,
        auth_scheme: OpenAiAuthScheme::Bearer,
        api_version: None,
    })
    .expect("openai client should be created");

    let response = client
        .complete(tool_call_request("gpt-4o-mini"))
        .await
        .expect("openai tool call completion should succeed");

    let tool_calls = response.message.tool_calls();
    assert!(
        !tool_calls.is_empty(),
        "expected at least one tool call, got none"
    );
    assert_eq!(
        tool_calls[0].name, "get_weather",
        "expected tool call name 'get_weather', got '{}'",
        tool_calls[0].name
    );
}

#[tokio::test]
#[ignore]
async fn live_openai_streaming() {
    let Some(api_key) = require_key("TAU_TEST_OPENAI_API_KEY") else {
        eprintln!("TAU_TEST_OPENAI_API_KEY not set — skipping");
        return;
    };

    let client = OpenAiClient::new(OpenAiConfig {
        api_key,
        api_base: "https://api.openai.com/v1".to_string(),
        organization: None,
        request_timeout_ms: 30_000,
        max_retries: 2,
        retry_budget_ms: 0,
        retry_jitter: false,
        auth_scheme: OpenAiAuthScheme::Bearer,
        api_version: None,
    })
    .expect("openai client should be created");

    let deltas: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let delta_sink = deltas.clone();
    let sink = Arc::new(move |delta: String| {
        delta_sink.lock().expect("delta lock").push(delta);
    });

    let response = client
        .complete_with_stream(streaming_prompt_request("gpt-4o-mini"), Some(sink))
        .await
        .expect("openai streaming completion should succeed");

    let collected: String = deltas.lock().expect("delta lock").join("");
    assert!(
        !collected.is_empty(),
        "expected non-empty streaming deltas"
    );

    let text = response.message.text_content().to_lowercase();
    assert!(
        text.contains("pong"),
        "expected streamed response to contain 'pong', got: {text}"
    );
}

// ---------------------------------------------------------------------------
// Anthropic
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn live_anthropic_simple_prompt() {
    let Some(api_key) = require_key("TAU_TEST_ANTHROPIC_API_KEY") else {
        eprintln!("TAU_TEST_ANTHROPIC_API_KEY not set — skipping");
        return;
    };

    let client = AnthropicClient::new(AnthropicConfig {
        api_key,
        api_base: "https://api.anthropic.com/v1".to_string(),
        request_timeout_ms: 30_000,
        max_retries: 2,
        retry_budget_ms: 0,
        retry_jitter: false,
    })
    .expect("anthropic client should be created");

    let response = client
        .complete(simple_prompt_request("claude-3-haiku-20240307"))
        .await
        .expect("anthropic completion should succeed");

    let text = response.message.text_content().to_lowercase();
    assert!(
        text.contains("pong"),
        "expected response to contain 'pong', got: {text}"
    );
}

#[tokio::test]
#[ignore]
async fn live_anthropic_tool_call_extraction() {
    let Some(api_key) = require_key("TAU_TEST_ANTHROPIC_API_KEY") else {
        eprintln!("TAU_TEST_ANTHROPIC_API_KEY not set — skipping");
        return;
    };

    let client = AnthropicClient::new(AnthropicConfig {
        api_key,
        api_base: "https://api.anthropic.com/v1".to_string(),
        request_timeout_ms: 30_000,
        max_retries: 2,
        retry_budget_ms: 0,
        retry_jitter: false,
    })
    .expect("anthropic client should be created");

    let response = client
        .complete(tool_call_request("claude-3-haiku-20240307"))
        .await
        .expect("anthropic tool call completion should succeed");

    let tool_calls = response.message.tool_calls();
    assert!(
        !tool_calls.is_empty(),
        "expected at least one tool call, got none"
    );
    assert_eq!(
        tool_calls[0].name, "get_weather",
        "expected tool call name 'get_weather', got '{}'",
        tool_calls[0].name
    );
}

#[tokio::test]
#[ignore]
async fn live_anthropic_streaming() {
    let Some(api_key) = require_key("TAU_TEST_ANTHROPIC_API_KEY") else {
        eprintln!("TAU_TEST_ANTHROPIC_API_KEY not set — skipping");
        return;
    };

    let client = AnthropicClient::new(AnthropicConfig {
        api_key,
        api_base: "https://api.anthropic.com/v1".to_string(),
        request_timeout_ms: 30_000,
        max_retries: 2,
        retry_budget_ms: 0,
        retry_jitter: false,
    })
    .expect("anthropic client should be created");

    let deltas: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let delta_sink = deltas.clone();
    let sink = Arc::new(move |delta: String| {
        delta_sink.lock().expect("delta lock").push(delta);
    });

    let response = client
        .complete_with_stream(
            streaming_prompt_request("claude-3-haiku-20240307"),
            Some(sink),
        )
        .await
        .expect("anthropic streaming completion should succeed");

    let collected: String = deltas.lock().expect("delta lock").join("");
    assert!(
        !collected.is_empty(),
        "expected non-empty streaming deltas"
    );

    let text = response.message.text_content().to_lowercase();
    assert!(
        text.contains("pong"),
        "expected streamed response to contain 'pong', got: {text}"
    );
}

// ---------------------------------------------------------------------------
// Google
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn live_google_simple_prompt() {
    let Some(api_key) = require_key("TAU_TEST_GOOGLE_API_KEY") else {
        eprintln!("TAU_TEST_GOOGLE_API_KEY not set — skipping");
        return;
    };

    let client = GoogleClient::new(GoogleConfig {
        api_key,
        api_base: "https://generativelanguage.googleapis.com".to_string(),
        request_timeout_ms: 30_000,
        max_retries: 2,
        retry_budget_ms: 0,
        retry_jitter: false,
    })
    .expect("google client should be created");

    let response = client
        .complete(simple_prompt_request("gemini-2.0-flash"))
        .await
        .expect("google completion should succeed");

    let text = response.message.text_content().to_lowercase();
    assert!(
        text.contains("pong"),
        "expected response to contain 'pong', got: {text}"
    );
}

#[tokio::test]
#[ignore]
async fn live_google_tool_call_extraction() {
    let Some(api_key) = require_key("TAU_TEST_GOOGLE_API_KEY") else {
        eprintln!("TAU_TEST_GOOGLE_API_KEY not set — skipping");
        return;
    };

    let client = GoogleClient::new(GoogleConfig {
        api_key,
        api_base: "https://generativelanguage.googleapis.com".to_string(),
        request_timeout_ms: 30_000,
        max_retries: 2,
        retry_budget_ms: 0,
        retry_jitter: false,
    })
    .expect("google client should be created");

    let response = client
        .complete(tool_call_request("gemini-2.0-flash"))
        .await
        .expect("google tool call completion should succeed");

    let tool_calls = response.message.tool_calls();
    assert!(
        !tool_calls.is_empty(),
        "expected at least one tool call, got none"
    );
    assert_eq!(
        tool_calls[0].name, "get_weather",
        "expected tool call name 'get_weather', got '{}'",
        tool_calls[0].name
    );
}

#[tokio::test]
#[ignore]
async fn live_google_streaming() {
    let Some(api_key) = require_key("TAU_TEST_GOOGLE_API_KEY") else {
        eprintln!("TAU_TEST_GOOGLE_API_KEY not set — skipping");
        return;
    };

    let client = GoogleClient::new(GoogleConfig {
        api_key,
        api_base: "https://generativelanguage.googleapis.com".to_string(),
        request_timeout_ms: 30_000,
        max_retries: 2,
        retry_budget_ms: 0,
        retry_jitter: false,
    })
    .expect("google client should be created");

    let deltas: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let delta_sink = deltas.clone();
    let sink = Arc::new(move |delta: String| {
        delta_sink.lock().expect("delta lock").push(delta);
    });

    let response = client
        .complete_with_stream(streaming_prompt_request("gemini-2.0-flash"), Some(sink))
        .await
        .expect("google streaming completion should succeed");

    let collected: String = deltas.lock().expect("delta lock").join("");
    assert!(
        !collected.is_empty(),
        "expected non-empty streaming deltas"
    );

    let text = response.message.text_content().to_lowercase();
    assert!(
        text.contains("pong"),
        "expected streamed response to contain 'pong', got: {text}"
    );
}
