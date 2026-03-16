//! Shared OpenResponses payload builders for the initial operator-state stream contract.

use super::*;

pub(super) fn response_created_payload(response_id: &str, model: &str, created: u64) -> Value {
    json!({
        "type": "response.created",
        "response": {
            "id": response_id,
            "object": "response",
            "status": "in_progress",
            "model": model,
            "created": created,
        },
        "operator_state": turn_in_progress_state(response_id),
    })
}

pub(super) fn response_output_text_delta_payload(response_id: &str, delta: String) -> Value {
    json!({
        "type": "response.output_text.delta",
        "response_id": response_id,
        "delta": delta,
        "operator_state": assistant_text_streaming_state(response_id),
    })
}

pub(super) fn response_output_text_done_payload(response_id: &str, text: String) -> Value {
    json!({
        "type": "response.output_text.done",
        "response_id": response_id,
        "text": text,
        "operator_state": assistant_text_completed_state(response_id),
    })
}

pub(super) fn response_completed_payload(response: OpenResponsesResponse) -> Value {
    json!({
        "type": "response.completed",
        "response": response,
        "operator_state": turn_completed_state(),
    })
}

pub(super) fn response_failed_payload(error: &OpenResponsesApiError) -> Value {
    json!({
        "type": "response.failed",
        "error": {
            "code": error.code,
            "message": error.message,
        },
        "operator_state": turn_failed_state(error.code),
    })
}

fn turn_in_progress_state(response_id: &str) -> Value {
    json!({
        "entity": "turn",
        "status": "in_progress",
        "phase": "model",
        "response_id": response_id,
    })
}

fn turn_completed_state() -> Value {
    json!({
        "entity": "turn",
        "status": "completed",
        "phase": "done",
    })
}

fn turn_failed_state(reason_code: &str) -> Value {
    json!({
        "entity": "turn",
        "status": "failed",
        "phase": "failed",
        "reason_code": reason_code,
    })
}

fn assistant_text_streaming_state(response_id: &str) -> Value {
    json!({
        "entity": "artifact",
        "status": "streaming",
        "artifact_kind": "assistant_output_text",
        "response_id": response_id,
    })
}

fn assistant_text_completed_state(response_id: &str) -> Value {
    json!({
        "entity": "artifact",
        "status": "completed",
        "artifact_kind": "assistant_output_text",
        "response_id": response_id,
    })
}
