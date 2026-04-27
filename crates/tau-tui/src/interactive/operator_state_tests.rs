use super::{
    app::{App, AppConfig},
    chat::MessageRole,
    operator_state::apply_operator_turn_state,
    status::AgentStateDisplay,
    tools::ToolStatus,
};
use tau_contract::operator_state::{
    OperatorErrorContext, OperatorToolState, OperatorToolStatus, OperatorTurnEvent,
    OperatorTurnEventKind, OperatorTurnPhase, OperatorTurnState, OperatorTurnStatus,
    OPERATOR_TURN_STATE_SCHEMA_VERSION,
};

fn test_app() -> App {
    App::new(AppConfig::default())
}

fn base_state(status: OperatorTurnStatus, phase: OperatorTurnPhase) -> OperatorTurnState {
    OperatorTurnState {
        schema_version: OPERATOR_TURN_STATE_SCHEMA_VERSION,
        turn_id: "turn-3582".to_string(),
        task_id: Some("task-3582".to_string()),
        session_key: "session-3582".to_string(),
        mission_id: Some("mission-3582".to_string()),
        phase,
        status,
        assistant_text: String::new(),
        tools: Vec::new(),
        events: Vec::new(),
        error: None,
    }
}

fn last_message(app: &App, role: MessageRole) -> Option<&str> {
    app.chat
        .messages()
        .iter()
        .rev()
        .find(|message| message.role == role)
        .map(|message| message.content.as_str())
}

#[test]
fn operator_state_success_updates_transcript_first_assistant_text() {
    let mut app = test_app();
    let mut state = base_state(OperatorTurnStatus::Succeeded, OperatorTurnPhase::Completed);
    state.assistant_text = "The runtime is ready.".to_string();
    state.events.push(OperatorTurnEvent {
        event_id: "evt-final".to_string(),
        kind: OperatorTurnEventKind::FinalAnswer,
        summary: "final answer".to_string(),
        text_delta: None,
        tool_call_id: None,
        tool_name: None,
        reason_code: None,
        occurred_at_ms: Some(1),
    });

    apply_operator_turn_state(&mut app, &state);

    assert_eq!(
        last_message(&app, MessageRole::Assistant),
        Some("The runtime is ready.")
    );
    assert_eq!(app.status.agent_state, AgentStateDisplay::Idle);
    assert_eq!(
        app.status.active_mission_id.as_deref(),
        Some("mission-3582")
    );
}

#[test]
fn operator_state_tool_lifecycle_reconciles_rows_by_tool_call_id() {
    let mut app = test_app();
    let mut state = base_state(
        OperatorTurnStatus::ToolRunning,
        OperatorTurnPhase::WaitingForTool,
    );
    state.tools.push(OperatorToolState {
        tool_call_id: "call-1".to_string(),
        tool_name: "read_file".to_string(),
        status: OperatorToolStatus::Running,
        summary: Some("path=crates/tau-tui/Cargo.toml".to_string()),
        started_at_ms: Some(10),
        completed_at_ms: None,
    });

    apply_operator_turn_state(&mut app, &state);

    assert_eq!(app.tools.active_count(), 1);
    let running = app.tools.latest_running().expect("running tool");
    assert_eq!(running.tool_call_id.as_deref(), Some("call-1"));
    assert_eq!(running.name, "read_file");
    assert_eq!(running.detail, "path=crates/tau-tui/Cargo.toml");
    assert_eq!(app.status.agent_state, AgentStateDisplay::ToolExec);

    state.status = OperatorTurnStatus::Succeeded;
    state.phase = OperatorTurnPhase::Completed;
    state.tools[0].status = OperatorToolStatus::Completed;
    state.tools[0].summary = Some("latency_ms=12".to_string());
    state.tools[0].completed_at_ms = Some(22);
    state.assistant_text = "read complete".to_string();

    apply_operator_turn_state(&mut app, &state);

    assert_eq!(app.tools.active_count(), 0);
    let completed = app.tools.latest_entry().expect("completed tool");
    assert_eq!(completed.tool_call_id.as_deref(), Some("call-1"));
    assert_eq!(completed.status, ToolStatus::Success);
    assert_eq!(completed.detail, "latency_ms=12");
    assert_eq!(
        last_message(&app, MessageRole::Assistant),
        Some("read complete")
    );
}

#[test]
fn operator_state_timeout_surfaces_reason_code_and_error_status() {
    let mut app = test_app();
    let mut state = base_state(OperatorTurnStatus::TimedOut, OperatorTurnPhase::Completed);
    state.error = Some(OperatorErrorContext {
        reason_code: "gateway_read_timeout".to_string(),
        message: "gateway turn exceeded request timeout".to_string(),
        retryable: true,
    });
    state.events.push(OperatorTurnEvent {
        event_id: "evt-timeout".to_string(),
        kind: OperatorTurnEventKind::Timeout,
        summary: "turn timed out".to_string(),
        text_delta: None,
        tool_call_id: None,
        tool_name: None,
        reason_code: Some("gateway_read_timeout".to_string()),
        occurred_at_ms: Some(30),
    });

    apply_operator_turn_state(&mut app, &state);

    let system = last_message(&app, MessageRole::System).unwrap_or_default();
    assert!(system.contains("gateway_read_timeout"), "system={system}");
    assert!(
        system.contains("gateway turn exceeded request timeout"),
        "system={system}"
    );
    assert_eq!(app.status.agent_state, AgentStateDisplay::Error);
}

#[test]
fn operator_state_blocked_mission_surfaces_blocked_status() {
    let mut app = test_app();
    let mut state = base_state(
        OperatorTurnStatus::Blocked,
        OperatorTurnPhase::WaitingForVerifier,
    );
    state.error = Some(OperatorErrorContext {
        reason_code: "verifier_blocked".to_string(),
        message: "manual verification required".to_string(),
        retryable: false,
    });
    state.events.push(OperatorTurnEvent {
        event_id: "evt-blocked".to_string(),
        kind: OperatorTurnEventKind::MissionBlocked,
        summary: "mission blocked by verifier".to_string(),
        text_delta: None,
        tool_call_id: None,
        tool_name: None,
        reason_code: Some("verifier_blocked".to_string()),
        occurred_at_ms: Some(40),
    });

    apply_operator_turn_state(&mut app, &state);

    let system = last_message(&app, MessageRole::System).unwrap_or_default();
    assert!(system.contains("mission blocked"), "system={system}");
    assert!(system.contains("verifier_blocked"), "system={system}");
    assert_eq!(app.status.agent_state, AgentStateDisplay::Error);
    assert_eq!(
        app.status.active_mission_id.as_deref(),
        Some("mission-3582")
    );
}
