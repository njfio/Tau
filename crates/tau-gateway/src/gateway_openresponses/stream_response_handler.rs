//! Gateway stream response handler for `/v1/responses` stream mode.

use super::*;
use tau_contract::operator_state::{
    OperatorTurnEvent, OperatorTurnEventKind, OperatorTurnPhase, OperatorTurnState,
    OperatorTurnStatus, OPERATOR_TURN_STATE_SCHEMA_VERSION,
};

pub(super) async fn stream_openresponses(
    state: Arc<GatewayOpenResponsesServerState>,
    request: OpenResponsesRequest,
) -> Response {
    let (tx, rx) = mpsc::unbounded_channel::<SseFrame>();
    tokio::spawn(async move {
        let operator_identity =
            translate_openresponses_request(&request, state.config.max_input_chars)
                .ok()
                .map(|prompt| (prompt.session_key, prompt.mission_id));
        match execute_openresponses_request(state, request, Some(tx.clone())).await {
            Ok(result) => {
                let response = result.response;
                let _ = tx.send(SseFrame::Json {
                    event: "response.output_text.done",
                    payload: json!({
                        "type": "response.output_text.done",
                        "response_id": response.id,
                        "text": response.output_text,
                    }),
                });
                if let Some((session_key, mission_id)) = operator_identity {
                    let _ = tx.send(SseFrame::Json {
                        event: "response.operator_turn_state.snapshot",
                        payload: json!(build_operator_turn_state_snapshot(
                            &response,
                            session_key,
                            mission_id
                        )),
                    });
                }
                let _ = tx.send(SseFrame::Json {
                    event: "response.completed",
                    payload: json!({
                        "type": "response.completed",
                        "response": response,
                    }),
                });
                let _ = tx.send(SseFrame::Done);
            }
            Err(error) => {
                let _ = tx.send(SseFrame::Json {
                    event: "response.failed",
                    payload: json!({
                        "type": "response.failed",
                        "error": {
                            "code": error.code,
                            "message": error.message,
                        }
                    }),
                });
                let _ = tx.send(SseFrame::Done);
            }
        }
    });

    let stream =
        UnboundedReceiverStream::new(rx).map(|frame| Ok::<Event, Infallible>(frame.into_event()));
    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

fn build_operator_turn_state_snapshot(
    response: &OpenResponsesResponse,
    session_key: String,
    mission_id: String,
) -> OperatorTurnState {
    OperatorTurnState {
        schema_version: OPERATOR_TURN_STATE_SCHEMA_VERSION,
        turn_id: response.id.clone(),
        task_id: None,
        session_key,
        mission_id: Some(mission_id),
        phase: OperatorTurnPhase::Completed,
        status: OperatorTurnStatus::Succeeded,
        assistant_text: response.output_text.clone(),
        tools: Vec::new(),
        events: vec![OperatorTurnEvent {
            event_id: format!("{}-final", response.id),
            kind: OperatorTurnEventKind::FinalAnswer,
            summary: "response completed".to_string(),
            text_delta: None,
            tool_call_id: None,
            tool_name: None,
            reason_code: None,
            occurred_at_ms: Some(response.created.saturating_mul(1000)),
        }],
        error: None,
    }
}
