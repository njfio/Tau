//! OpenAI-compatible request/response adapters layered onto the OpenResponses runtime.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{json, Value};

use super::types::{OpenResponsesApiError, OpenResponsesRequest, OpenResponsesResponse};

const OPENAI_OWNER: &str = "tau-gateway";
const OPENAI_CHAT_COMPLETION_OBJECT: &str = "chat.completion";
const OPENAI_CHAT_COMPLETION_CHUNK_OBJECT: &str = "chat.completion.chunk";
const OPENAI_COMPLETION_OBJECT: &str = "text_completion";

#[derive(Debug, Deserialize)]
pub(super) struct OpenAiChatCompletionsRequest {
    pub(super) model: Option<String>,
    #[serde(default)]
    pub(super) messages: Value,
    #[serde(default)]
    pub(super) stream: bool,
    #[serde(default)]
    pub(super) user: Option<String>,
    #[serde(flatten)]
    pub(super) extra: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub(super) struct OpenAiCompletionsRequest {
    pub(super) model: Option<String>,
    #[serde(default)]
    pub(super) prompt: Value,
    #[serde(default)]
    pub(super) stream: bool,
    #[serde(default)]
    pub(super) user: Option<String>,
    #[serde(flatten)]
    pub(super) extra: BTreeMap<String, Value>,
}

#[derive(Debug)]
pub(super) struct OpenAiCompatRequestTranslation {
    pub(super) request: OpenResponsesRequest,
    pub(super) ignored_fields: Vec<String>,
    pub(super) requested_model: Option<String>,
    pub(super) stream: bool,
}

/// Translate OpenAI chat completions payload into OpenResponses runtime request envelope.
pub(super) fn translate_chat_completions_request(
    request: OpenAiChatCompletionsRequest,
) -> Result<OpenAiCompatRequestTranslation, OpenResponsesApiError> {
    let mut ignored_fields = Vec::new();
    let mut extra = request.extra;
    if extra.contains_key("tools") || extra.contains_key("tool_choice") {
        return Err(OpenResponsesApiError::bad_request(
            "unsupported_tools",
            "tools/tool_choice request fields are not supported by this compatibility surface",
        ));
    }

    validate_single_choice_request(&mut extra)?;
    let max_tokens = parse_optional_max_tokens(&mut extra)?;

    let messages = match request.messages {
        Value::Array(messages) => messages,
        _ => {
            return Err(OpenResponsesApiError::bad_request(
                "invalid_messages",
                "messages must be an array",
            ));
        }
    };
    if messages.is_empty() {
        return Err(OpenResponsesApiError::bad_request(
            "missing_messages",
            "messages must include at least one item",
        ));
    }

    let mut translated_messages = Vec::new();
    for (index, message) in messages.into_iter().enumerate() {
        match message {
            Value::String(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    ignored_fields.push(format!("messages[{index}]"));
                    continue;
                }
                translated_messages.push(json!({
                    "type": "message",
                    "role": "user",
                    "content": trimmed,
                }));
            }
            Value::Object(map) => {
                let role = map
                    .get("role")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("user")
                    .to_string();
                let content = map.get("content").cloned().unwrap_or(Value::Null);
                if is_effectively_empty_text(&content) {
                    ignored_fields.push(format!("messages[{index}].content"));
                    continue;
                }
                translated_messages.push(json!({
                    "type": "message",
                    "role": role,
                    "content": content,
                }));

                for unsupported in ["tool_calls", "function_call", "audio", "refusal", "name"] {
                    if map.contains_key(unsupported) {
                        ignored_fields.push(format!("messages[{index}].{unsupported}"));
                    }
                }
            }
            _ => ignored_fields.push(format!("messages[{index}]")),
        }
    }

    if translated_messages.is_empty() {
        return Err(OpenResponsesApiError::bad_request(
            "missing_messages",
            "messages did not include any textual content",
        ));
    }

    let session_user = non_empty_trimmed(request.user.as_deref()).map(str::to_string);
    let metadata = session_user
        .as_ref()
        .map(|user| json!({ "session_id": user }))
        .unwrap_or_else(|| json!({}));
    let requested_model = request.model.clone();

    Ok(OpenAiCompatRequestTranslation {
        stream: request.stream,
        requested_model,
        ignored_fields,
        request: OpenResponsesRequest {
            model: request.model,
            input: Value::Array(translated_messages),
            stream: request.stream,
            max_tokens,
            instructions: None,
            metadata,
            conversation: session_user,
            previous_response_id: None,
            extra,
        },
    })
}

pub(super) fn translate_completions_request(
    request: OpenAiCompletionsRequest,
) -> Result<OpenAiCompatRequestTranslation, OpenResponsesApiError> {
    let mut extra = request.extra;
    validate_single_choice_request(&mut extra)?;
    let max_tokens = parse_optional_max_tokens(&mut extra)?;

    if is_effectively_empty_text(&request.prompt) {
        return Err(OpenResponsesApiError::bad_request(
            "missing_prompt",
            "prompt must include textual content",
        ));
    }

    let session_user = non_empty_trimmed(request.user.as_deref()).map(str::to_string);
    let metadata = session_user
        .as_ref()
        .map(|user| json!({ "session_id": user }))
        .unwrap_or_else(|| json!({}));
    let requested_model = request.model.clone();

    Ok(OpenAiCompatRequestTranslation {
        stream: request.stream,
        requested_model,
        ignored_fields: Vec::new(),
        request: OpenResponsesRequest {
            model: request.model,
            input: request.prompt,
            stream: request.stream,
            max_tokens,
            instructions: None,
            metadata,
            conversation: session_user,
            previous_response_id: None,
            extra,
        },
    })
}

/// Build OpenAI chat.completions JSON payload from one OpenResponses result.
pub(super) fn build_chat_completions_payload(response: &OpenResponsesResponse) -> Value {
    let finish_reason = normalized_finish_reason(response);
    json!({
        "id": chat_completion_id(response.id.as_str()),
        "object": OPENAI_CHAT_COMPLETION_OBJECT,
        "created": response.created,
        "model": response.model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": response.output_text,
            },
            "finish_reason": finish_reason,
        }],
        "usage": {
            "prompt_tokens": response.usage.input_tokens,
            "completion_tokens": response.usage.output_tokens,
            "total_tokens": response.usage.total_tokens,
        }
    })
}

pub(super) fn build_completions_payload(response: &OpenResponsesResponse) -> Value {
    let finish_reason = normalized_finish_reason(response);
    json!({
        "id": completion_id(response.id.as_str()),
        "object": OPENAI_COMPLETION_OBJECT,
        "created": response.created,
        "model": response.model,
        "choices": [{
            "index": 0,
            "text": response.output_text,
            "logprobs": Value::Null,
            "finish_reason": finish_reason,
        }],
        "usage": {
            "prompt_tokens": response.usage.input_tokens,
            "completion_tokens": response.usage.output_tokens,
            "total_tokens": response.usage.total_tokens,
        }
    })
}

pub(super) fn build_chat_completions_stream_chunks(response: &OpenResponsesResponse) -> Vec<Value> {
    let finish_reason = normalized_finish_reason(response);
    vec![
        json!({
            "id": chat_completion_id(response.id.as_str()),
            "object": OPENAI_CHAT_COMPLETION_CHUNK_OBJECT,
            "created": response.created,
            "model": response.model,
            "choices": [{
                "index": 0,
                "delta": {
                    "role": "assistant",
                    "content": response.output_text,
                },
                "finish_reason": Value::Null,
            }],
        }),
        json!({
            "id": chat_completion_id(response.id.as_str()),
            "object": OPENAI_CHAT_COMPLETION_CHUNK_OBJECT,
            "created": response.created,
            "model": response.model,
            "choices": [{
                "index": 0,
                "delta": {},
                "finish_reason": finish_reason,
            }],
        }),
    ]
}

pub(super) fn build_completions_stream_chunks(response: &OpenResponsesResponse) -> Vec<Value> {
    let finish_reason = normalized_finish_reason(response);
    vec![
        json!({
            "id": completion_id(response.id.as_str()),
            "object": OPENAI_COMPLETION_OBJECT,
            "created": response.created,
            "model": response.model,
            "choices": [{
                "index": 0,
                "text": response.output_text,
                "logprobs": Value::Null,
                "finish_reason": Value::Null,
            }],
        }),
        json!({
            "id": completion_id(response.id.as_str()),
            "object": OPENAI_COMPLETION_OBJECT,
            "created": response.created,
            "model": response.model,
            "choices": [{
                "index": 0,
                "text": "",
                "logprobs": Value::Null,
                "finish_reason": finish_reason,
            }],
        }),
    ]
}

pub(super) fn build_models_payload(configured_model: &str, created: u64) -> Value {
    json!({
        "object": "list",
        "data": [{
            "id": configured_model,
            "object": "model",
            "created": created,
            "owned_by": OPENAI_OWNER,
        }],
    })
}

fn chat_completion_id(response_id: &str) -> String {
    let suffix = response_id.strip_prefix("resp_").unwrap_or(response_id);
    format!("chatcmpl_{suffix}")
}

fn completion_id(response_id: &str) -> String {
    let suffix = response_id.strip_prefix("resp_").unwrap_or(response_id);
    format!("cmpl_{suffix}")
}

fn normalized_finish_reason(response: &OpenResponsesResponse) -> &str {
    let finish_reason = response.finish_reason.trim();
    if finish_reason.is_empty() {
        "stop"
    } else {
        finish_reason
    }
}

fn non_empty_trimmed(raw: Option<&str>) -> Option<&str> {
    raw.map(str::trim).filter(|value| !value.is_empty())
}

fn validate_single_choice_request(
    extra: &mut BTreeMap<String, Value>,
) -> Result<(), OpenResponsesApiError> {
    let Some(raw_n) = extra.remove("n") else {
        return Ok(());
    };
    let Some(choice_count) = raw_n.as_u64() else {
        return Err(OpenResponsesApiError::bad_request(
            "invalid_n",
            "n must be a positive integer",
        ));
    };
    if choice_count == 0 {
        return Err(OpenResponsesApiError::bad_request(
            "invalid_n",
            "n must be greater than zero",
        ));
    }
    if choice_count > 1 {
        return Err(OpenResponsesApiError::bad_request(
            "unsupported_n",
            "n greater than 1 is not supported by this compatibility surface",
        ));
    }
    Ok(())
}

fn parse_optional_max_tokens(
    extra: &mut BTreeMap<String, Value>,
) -> Result<Option<u32>, OpenResponsesApiError> {
    let Some(raw_max_tokens) = extra.remove("max_tokens") else {
        return Ok(None);
    };
    let Some(max_tokens) = raw_max_tokens.as_u64() else {
        return Err(OpenResponsesApiError::bad_request(
            "invalid_max_tokens",
            "max_tokens must be a positive integer",
        ));
    };
    if max_tokens == 0 {
        return Err(OpenResponsesApiError::bad_request(
            "invalid_max_tokens",
            "max_tokens must be greater than zero",
        ));
    }
    let max_tokens = u32::try_from(max_tokens).map_err(|_| {
        OpenResponsesApiError::bad_request(
            "invalid_max_tokens",
            "max_tokens exceeds supported range",
        )
    })?;
    Ok(Some(max_tokens))
}

fn is_effectively_empty_text(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(text) => text.trim().is_empty(),
        Value::Array(values) => {
            if values.is_empty() {
                return true;
            }
            values.iter().all(is_effectively_empty_text)
        }
        Value::Object(map) => {
            if let Some(text) = map.get("text").and_then(Value::as_str) {
                return text.trim().is_empty();
            }
            false
        }
        _ => false,
    }
}
