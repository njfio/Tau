use std::convert::Infallible;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures_util::StreamExt;
use serde_json::json;
use tau_core::current_unix_timestamp;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use super::openai_compat::{
    build_chat_completions_payload, build_chat_completions_stream_chunks,
    build_completions_payload, build_completions_stream_chunks, build_models_payload,
    translate_chat_completions_request, translate_completions_request,
    OpenAiChatCompletionsRequest, OpenAiCompletionsRequest,
};
use super::{
    authorize_and_enforce_gateway_limits, execute_openresponses_request, parse_gateway_json_body,
    validate_gateway_request_body_size, GatewayOpenAiCompatSurface,
    GatewayOpenResponsesServerState, OpenResponsesRequest,
};

pub(super) async fn handle_openai_chat_completions(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(error) = authorize_and_enforce_gateway_limits(&state, &headers) {
        return error.into_response();
    }
    state.record_openai_compat_reason("openai_chat_completions_request_received");

    if let Err(error) = validate_gateway_request_body_size(&state, &body) {
        state.increment_openai_compat_translation_failures();
        state.record_openai_compat_reason("openai_chat_completions_body_too_large");
        return error.into_response();
    }

    let request = match parse_gateway_json_body::<OpenAiChatCompletionsRequest>(&body) {
        Ok(request) => request,
        Err(error) => {
            state.increment_openai_compat_translation_failures();
            state.record_openai_compat_reason("openai_chat_completions_malformed_json");
            return error.into_response();
        }
    };

    let translated = match translate_chat_completions_request(request) {
        Ok(translated) => translated,
        Err(error) => {
            state.increment_openai_compat_translation_failures();
            state.record_openai_compat_reason("openai_chat_completions_translation_failed");
            return error.into_response();
        }
    };

    state.record_openai_compat_request(
        GatewayOpenAiCompatSurface::ChatCompletions,
        translated.stream,
    );

    if translated
        .requested_model
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
    {
        state.record_openai_compat_reason("openai_chat_completions_model_override_ignored");
    }
    state.record_openai_compat_ignored_fields(&translated.ignored_fields);

    if translated.stream {
        return stream_openai_chat_completions(
            state,
            translated.request,
            translated.ignored_fields,
        )
        .await;
    }

    match execute_openresponses_request(state.clone(), translated.request, None).await {
        Ok(result) => {
            let mut ignored_fields = translated.ignored_fields;
            ignored_fields.extend(result.response.ignored_fields.clone());
            if !ignored_fields.is_empty() {
                state.record_openai_compat_reason("openai_chat_completions_ignored_fields");
            }
            state.record_openai_compat_ignored_fields(&ignored_fields);
            state.record_openai_compat_reason("openai_chat_completions_succeeded");
            (
                StatusCode::OK,
                Json(build_chat_completions_payload(&result.response)),
            )
                .into_response()
        }
        Err(error) => {
            state.increment_openai_compat_execution_failures();
            state.record_openai_compat_reason("openai_chat_completions_execution_failed");
            error.into_response()
        }
    }
}

pub(super) async fn handle_openai_completions(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(error) = authorize_and_enforce_gateway_limits(&state, &headers) {
        return error.into_response();
    }
    state.record_openai_compat_reason("openai_completions_request_received");

    if let Err(error) = validate_gateway_request_body_size(&state, &body) {
        state.increment_openai_compat_translation_failures();
        state.record_openai_compat_reason("openai_completions_body_too_large");
        return error.into_response();
    }

    let request = match parse_gateway_json_body::<OpenAiCompletionsRequest>(&body) {
        Ok(request) => request,
        Err(error) => {
            state.increment_openai_compat_translation_failures();
            state.record_openai_compat_reason("openai_completions_malformed_json");
            return error.into_response();
        }
    };

    let translated = match translate_completions_request(request) {
        Ok(translated) => translated,
        Err(error) => {
            state.increment_openai_compat_translation_failures();
            state.record_openai_compat_reason("openai_completions_translation_failed");
            return error.into_response();
        }
    };

    state.record_openai_compat_request(GatewayOpenAiCompatSurface::Completions, translated.stream);

    if translated
        .requested_model
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
    {
        state.record_openai_compat_reason("openai_completions_model_override_ignored");
    }
    state.record_openai_compat_ignored_fields(&translated.ignored_fields);

    if translated.stream {
        return stream_openai_completions(state, translated.request, translated.ignored_fields)
            .await;
    }

    match execute_openresponses_request(state.clone(), translated.request, None).await {
        Ok(result) => {
            let mut ignored_fields = translated.ignored_fields;
            ignored_fields.extend(result.response.ignored_fields.clone());
            if !ignored_fields.is_empty() {
                state.record_openai_compat_reason("openai_completions_ignored_fields");
            }
            state.record_openai_compat_ignored_fields(&ignored_fields);
            state.record_openai_compat_reason("openai_completions_succeeded");
            (
                StatusCode::OK,
                Json(build_completions_payload(&result.response)),
            )
                .into_response()
        }
        Err(error) => {
            state.increment_openai_compat_execution_failures();
            state.record_openai_compat_reason("openai_completions_execution_failed");
            error.into_response()
        }
    }
}

pub(super) async fn handle_openai_models(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(error) = authorize_and_enforce_gateway_limits(&state, &headers) {
        return error.into_response();
    }

    state.record_openai_compat_request(GatewayOpenAiCompatSurface::Models, false);
    state.record_openai_compat_reason("openai_models_listed");

    let payload = build_models_payload(&state.config.model, current_unix_timestamp());
    (StatusCode::OK, Json(payload)).into_response()
}

async fn stream_openai_chat_completions(
    state: Arc<GatewayOpenResponsesServerState>,
    request: OpenResponsesRequest,
    compat_ignored_fields: Vec<String>,
) -> Response {
    let (tx, rx) = mpsc::unbounded_channel::<Event>();
    tokio::spawn(async move {
        match execute_openresponses_request(state.clone(), request, None).await {
            Ok(result) => {
                let mut ignored_fields = compat_ignored_fields;
                ignored_fields.extend(result.response.ignored_fields.clone());
                if !ignored_fields.is_empty() {
                    state.record_openai_compat_reason(
                        "openai_chat_completions_stream_ignored_fields",
                    );
                }
                state.record_openai_compat_ignored_fields(&ignored_fields);
                for chunk in build_chat_completions_stream_chunks(&result.response) {
                    let _ = tx.send(Event::default().data(chunk.to_string()));
                }
                let _ = tx.send(Event::default().data("[DONE]"));
                state.record_openai_compat_reason("openai_chat_completions_stream_succeeded");
            }
            Err(error) => {
                state.increment_openai_compat_execution_failures();
                state.record_openai_compat_reason("openai_chat_completions_stream_failed");
                let _ = tx.send(
                    Event::default().data(
                        json!({
                            "error": {
                                "type": "server_error",
                                "code": error.code,
                                "message": error.message,
                            }
                        })
                        .to_string(),
                    ),
                );
                let _ = tx.send(Event::default().data("[DONE]"));
            }
        }
    });

    let stream = UnboundedReceiverStream::new(rx).map(Ok::<Event, Infallible>);
    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

async fn stream_openai_completions(
    state: Arc<GatewayOpenResponsesServerState>,
    request: OpenResponsesRequest,
    compat_ignored_fields: Vec<String>,
) -> Response {
    let (tx, rx) = mpsc::unbounded_channel::<Event>();
    tokio::spawn(async move {
        match execute_openresponses_request(state.clone(), request, None).await {
            Ok(result) => {
                let mut ignored_fields = compat_ignored_fields;
                ignored_fields.extend(result.response.ignored_fields.clone());
                if !ignored_fields.is_empty() {
                    state.record_openai_compat_reason("openai_completions_stream_ignored_fields");
                }
                state.record_openai_compat_ignored_fields(&ignored_fields);
                for chunk in build_completions_stream_chunks(&result.response) {
                    let _ = tx.send(Event::default().data(chunk.to_string()));
                }
                let _ = tx.send(Event::default().data("[DONE]"));
                state.record_openai_compat_reason("openai_completions_stream_succeeded");
            }
            Err(error) => {
                state.increment_openai_compat_execution_failures();
                state.record_openai_compat_reason("openai_completions_stream_failed");
                let _ = tx.send(
                    Event::default().data(
                        json!({
                            "error": {
                                "type": "server_error",
                                "code": error.code,
                                "message": error.message,
                            }
                        })
                        .to_string(),
                    ),
                );
                let _ = tx.send(Event::default().data("[DONE]"));
            }
        }
    });

    let stream = UnboundedReceiverStream::new(rx).map(Ok::<Event, Infallible>);
    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}
