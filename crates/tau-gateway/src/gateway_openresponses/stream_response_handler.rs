//! Gateway stream response handler for `/v1/responses` stream mode.

use super::*;
use tau_contract::operator_state::{
    OperatorErrorContext, OperatorToolState, OperatorToolStatus, OperatorTurnEvent,
    OperatorTurnEventKind, OperatorTurnPhase, OperatorTurnState, OperatorTurnStatus,
    OPERATOR_TURN_STATE_SCHEMA_VERSION,
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
                let OpenResponsesExecutionResult {
                    response,
                    tool_executions,
                    completion_signal,
                } = result;
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
                            mission_id,
                            &tool_executions,
                            completion_signal.as_ref(),
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
    tool_executions: &[OpenResponsesObservedToolExecution],
    completion_signal: Option<&GatewayMissionCompletionSignalRecord>,
) -> OperatorTurnState {
    OperatorTurnState {
        schema_version: OPERATOR_TURN_STATE_SCHEMA_VERSION,
        turn_id: response.id.clone(),
        task_id: None,
        session_key,
        mission_id: Some(mission_id),
        phase: OperatorTurnPhase::Completed,
        status: build_operator_turn_status(completion_signal),
        assistant_text: response.output_text.clone(),
        tools: build_operator_tool_states(tool_executions),
        events: build_operator_turn_events(response, tool_executions, completion_signal),
        error: build_operator_error_context(completion_signal),
    }
}

fn build_operator_turn_status(
    completion_signal: Option<&GatewayMissionCompletionSignalRecord>,
) -> OperatorTurnStatus {
    match completion_signal.map(|signal| &signal.status) {
        Some(GatewayMissionCompletionStatus::Blocked) => OperatorTurnStatus::Blocked,
        _ => OperatorTurnStatus::Succeeded,
    }
}

fn build_operator_error_context(
    completion_signal: Option<&GatewayMissionCompletionSignalRecord>,
) -> Option<OperatorErrorContext> {
    let completion = completion_signal?;
    (completion.status == GatewayMissionCompletionStatus::Blocked).then(|| OperatorErrorContext {
        reason_code: "mission_completion_blocked".to_string(),
        message: completion.summary.clone(),
        retryable: false,
    })
}

fn build_operator_tool_states(
    tool_executions: &[OpenResponsesObservedToolExecution],
) -> Vec<OperatorToolState> {
    tool_executions
        .iter()
        .map(|tool| OperatorToolState {
            tool_call_id: tool.tool_call_id.clone(),
            tool_name: tool.tool_name.clone(),
            status: if tool.success {
                OperatorToolStatus::Completed
            } else {
                OperatorToolStatus::Failed
            },
            summary: (!tool.output_summary.trim().is_empty()).then(|| tool.output_summary.clone()),
            started_at_ms: Some(tool.timestamp_ms.saturating_sub(tool.latency_ms)),
            completed_at_ms: Some(tool.timestamp_ms),
        })
        .collect()
}

fn build_operator_turn_events(
    response: &OpenResponsesResponse,
    tool_executions: &[OpenResponsesObservedToolExecution],
    completion_signal: Option<&GatewayMissionCompletionSignalRecord>,
) -> Vec<OperatorTurnEvent> {
    let mut events = tool_executions
        .iter()
        .enumerate()
        .map(|(index, tool)| OperatorTurnEvent {
            event_id: format!("{}-tool-{}", response.id, index + 1),
            kind: if tool.success {
                OperatorTurnEventKind::ResponseToolExecutionCompleted
            } else {
                OperatorTurnEventKind::ResponseToolExecutionFailed
            },
            summary: tool.output_summary.clone(),
            text_delta: None,
            tool_call_id: Some(tool.tool_call_id.clone()),
            tool_name: Some(tool.tool_name.clone()),
            reason_code: (!tool.success).then(|| "tool_execution_failed".to_string()),
            occurred_at_ms: Some(tool.timestamp_ms),
        })
        .collect::<Vec<_>>();
    if let Some(completion) = completion_signal {
        match completion.status {
            GatewayMissionCompletionStatus::Partial => events.push(OperatorTurnEvent {
                event_id: format!("{}-mission-checkpointed", response.id),
                kind: OperatorTurnEventKind::MissionCheckpointed,
                summary: completion
                    .next_step
                    .as_ref()
                    .map(|next_step| format!("{} Next step: {next_step}", completion.summary))
                    .unwrap_or_else(|| completion.summary.clone()),
                text_delta: None,
                tool_call_id: None,
                tool_name: None,
                reason_code: Some("mission_completion_partial".to_string()),
                occurred_at_ms: Some(response.created.saturating_mul(1000)),
            }),
            GatewayMissionCompletionStatus::Blocked => events.push(OperatorTurnEvent {
                event_id: format!("{}-mission-blocked", response.id),
                kind: OperatorTurnEventKind::MissionBlocked,
                summary: completion.summary.clone(),
                text_delta: None,
                tool_call_id: None,
                tool_name: None,
                reason_code: Some("mission_completion_blocked".to_string()),
                occurred_at_ms: Some(response.created.saturating_mul(1000)),
            }),
            GatewayMissionCompletionStatus::Success => {}
        }
    }
    events.push(OperatorTurnEvent {
        event_id: format!("{}-final", response.id),
        kind: OperatorTurnEventKind::FinalAnswer,
        summary: "response completed".to_string(),
        text_delta: None,
        tool_call_id: None,
        tool_name: None,
        reason_code: None,
        occurred_at_ms: Some(response.created.saturating_mul(1000)),
    });
    events
}
