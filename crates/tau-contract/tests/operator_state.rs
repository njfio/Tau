use tau_contract::operator_state::{
    parse_operator_turn_fixture, OperatorToolStatus, OperatorTurnEventKind, OperatorTurnPhase,
    OperatorTurnStatus,
};

#[test]
fn operator_state_fixture_parses_success_with_tool_lifecycle() {
    let fixture = parse_operator_turn_fixture(
        r#"
        {
          "schema_version": 1,
          "name": "operator-state-success",
          "cases": [
            {
              "schema_version": 1,
              "turn_id": "turn-123",
              "task_id": "task-456",
              "session_key": "session-alpha",
              "mission_id": "mission-789",
              "phase": "completed",
              "status": "succeeded",
              "assistant_text": "Checked the repo and updated the contract.",
              "tools": [
                {
                  "tool_call_id": "call-1",
                  "tool_name": "shell",
                  "status": "completed",
                  "summary": "cargo test passed",
                  "started_at_ms": 1000,
                  "completed_at_ms": 1200
                }
              ],
              "events": [
                {
                  "event_id": "event-1",
                  "kind": "response.output_text.delta",
                  "summary": "assistant text streamed",
                  "text_delta": "Checked the repo",
                  "tool_call_id": null,
                  "tool_name": null,
                  "reason_code": null,
                  "occurred_at_ms": 1100
                },
                {
                  "event_id": "event-2",
                  "kind": "response.tool_execution.started",
                  "summary": "shell started",
                  "text_delta": null,
                  "tool_call_id": "call-1",
                  "tool_name": "shell",
                  "reason_code": null,
                  "occurred_at_ms": 1000
                },
                {
                  "event_id": "event-3",
                  "kind": "response.tool_execution.completed",
                  "summary": "shell completed",
                  "text_delta": null,
                  "tool_call_id": "call-1",
                  "tool_name": "shell",
                  "reason_code": null,
                  "occurred_at_ms": 1200
                }
              ],
              "error": null
            }
          ]
        }
        "#,
    )
    .expect("fixture should parse");

    let state = fixture.cases.first().expect("case should exist");
    assert_eq!(state.turn_id, "turn-123");
    assert_eq!(state.task_id.as_deref(), Some("task-456"));
    assert_eq!(state.phase, OperatorTurnPhase::Completed);
    assert_eq!(state.status, OperatorTurnStatus::Succeeded);
    assert_eq!(state.tools[0].status, OperatorToolStatus::Completed);
    assert_eq!(
        state.events[1].kind,
        OperatorTurnEventKind::ResponseToolExecutionStarted
    );
    assert_eq!(
        state.events[2].kind,
        OperatorTurnEventKind::ResponseToolExecutionCompleted
    );
}

#[test]
fn operator_state_fixture_rejects_unsupported_schema_version() {
    let error = parse_operator_turn_fixture(
        r#"
        {
          "schema_version": 2,
          "name": "operator-state-future",
          "cases": [
            {
              "schema_version": 2,
              "turn_id": "turn-future",
              "task_id": null,
              "session_key": "session-alpha",
              "mission_id": null,
              "phase": "completed",
              "status": "succeeded",
              "assistant_text": "future",
              "tools": [],
              "events": [],
              "error": null
            }
          ]
        }
        "#,
    )
    .expect_err("future schema versions should fail");

    assert!(error
        .to_string()
        .contains("unsupported operator-state contract schema version 2"));
}

#[test]
fn operator_state_fixture_covers_timeout_and_blocked_mission() {
    let fixture = parse_operator_turn_fixture(
        r#"
        {
          "schema_version": 1,
          "name": "operator-state-failures",
          "cases": [
            {
              "schema_version": 1,
              "turn_id": "turn-timeout",
              "task_id": null,
              "session_key": "session-beta",
              "mission_id": null,
              "phase": "completed",
              "status": "timed_out",
              "assistant_text": "",
              "tools": [],
              "events": [
                {
                  "event_id": "event-timeout",
                  "kind": "timeout",
                  "summary": "provider read timed out",
                  "text_delta": null,
                  "tool_call_id": null,
                  "tool_name": null,
                  "reason_code": "provider_read_timeout",
                  "occurred_at_ms": 3000
                }
              ],
              "error": {
                "reason_code": "provider_read_timeout",
                "message": "provider read timed out after 3000ms",
                "retryable": true
              }
            },
            {
              "schema_version": 1,
              "turn_id": "turn-blocked",
              "task_id": "task-blocked",
              "session_key": "session-gamma",
              "mission_id": "mission-blocked",
              "phase": "waiting_for_verifier",
              "status": "blocked",
              "assistant_text": "Need reviewer input before continuing.",
              "tools": [],
              "events": [
                {
                  "event_id": "event-blocked",
                  "kind": "mission.blocked",
                  "summary": "mission blocked on verifier",
                  "text_delta": null,
                  "tool_call_id": null,
                  "tool_name": null,
                  "reason_code": "verifier_required",
                  "occurred_at_ms": 4000
                }
              ],
              "error": {
                "reason_code": "verifier_required",
                "message": "mission requires verifier input",
                "retryable": false
              }
            }
          ]
        }
        "#,
    )
    .expect("failure fixture should parse");

    let timeout = &fixture.cases[0];
    assert_eq!(timeout.status, OperatorTurnStatus::TimedOut);
    assert_eq!(timeout.events[0].kind, OperatorTurnEventKind::Timeout);
    assert_eq!(
        timeout
            .error
            .as_ref()
            .map(|error| error.reason_code.as_str()),
        Some("provider_read_timeout")
    );

    let blocked = &fixture.cases[1];
    assert_eq!(blocked.phase, OperatorTurnPhase::WaitingForVerifier);
    assert_eq!(blocked.status, OperatorTurnStatus::Blocked);
    assert_eq!(
        blocked.events[0].kind,
        OperatorTurnEventKind::MissionBlocked
    );
}
