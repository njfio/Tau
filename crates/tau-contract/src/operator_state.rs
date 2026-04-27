use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::{ensure_unique_case_ids, parse_fixture_with_validation, validate_fixture_header};

pub const OPERATOR_TURN_STATE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorTurnFixture {
    pub schema_version: u32,
    pub name: String,
    pub cases: Vec<OperatorTurnState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorTurnState {
    pub schema_version: u32,
    pub turn_id: String,
    pub task_id: Option<String>,
    pub session_key: String,
    pub mission_id: Option<String>,
    pub phase: OperatorTurnPhase,
    pub status: OperatorTurnStatus,
    pub assistant_text: String,
    pub tools: Vec<OperatorToolState>,
    pub events: Vec<OperatorTurnEvent>,
    pub error: Option<OperatorErrorContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperatorTurnPhase {
    Created,
    Queued,
    Running,
    WaitingForTool,
    WaitingForVerifier,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperatorTurnStatus {
    Pending,
    Streaming,
    ToolRunning,
    Blocked,
    TimedOut,
    Failed,
    Succeeded,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorToolState {
    pub tool_call_id: String,
    pub tool_name: String,
    pub status: OperatorToolStatus,
    pub summary: Option<String>,
    pub started_at_ms: Option<u64>,
    pub completed_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperatorToolStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorTurnEvent {
    pub event_id: String,
    pub kind: OperatorTurnEventKind,
    pub summary: String,
    pub text_delta: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_name: Option<String>,
    pub reason_code: Option<String>,
    pub occurred_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatorTurnEventKind {
    #[serde(rename = "response.output_text.delta")]
    ResponseOutputTextDelta,
    #[serde(rename = "response.tool_execution.started")]
    ResponseToolExecutionStarted,
    #[serde(rename = "response.tool_execution.completed")]
    ResponseToolExecutionCompleted,
    #[serde(rename = "response.tool_execution.failed")]
    ResponseToolExecutionFailed,
    #[serde(rename = "response.completed")]
    ResponseCompleted,
    #[serde(rename = "response.failed")]
    ResponseFailed,
    #[serde(rename = "mission.checkpointed")]
    MissionCheckpointed,
    #[serde(rename = "mission.blocked")]
    MissionBlocked,
    #[serde(rename = "timeout")]
    Timeout,
    #[serde(rename = "final_answer")]
    FinalAnswer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorErrorContext {
    pub reason_code: String,
    pub message: String,
    pub retryable: bool,
}

pub fn parse_operator_turn_fixture(raw: &str) -> Result<OperatorTurnFixture> {
    parse_fixture_with_validation(
        raw,
        "failed to parse operator-state fixture",
        validate_operator_turn_fixture,
    )
}

pub fn validate_operator_turn_fixture(fixture: &OperatorTurnFixture) -> Result<()> {
    validate_fixture_header(
        "operator-state",
        fixture.schema_version,
        OPERATOR_TURN_STATE_SCHEMA_VERSION,
        &fixture.name,
        fixture.cases.len(),
    )?;
    ensure_unique_case_ids(fixture.cases.iter().map(|case| case.turn_id.as_str()))?;
    for case in &fixture.cases {
        validate_operator_turn_state(case)?;
    }
    Ok(())
}

pub fn validate_operator_turn_state(state: &OperatorTurnState) -> Result<()> {
    if state.schema_version != OPERATOR_TURN_STATE_SCHEMA_VERSION {
        bail!(
            "unsupported operator-state case schema version {} (expected {})",
            state.schema_version,
            OPERATOR_TURN_STATE_SCHEMA_VERSION
        );
    }
    if state.turn_id.trim().is_empty() {
        bail!("operator turn_id cannot be empty");
    }
    if state.session_key.trim().is_empty() {
        bail!("operator session_key cannot be empty");
    }
    for tool in &state.tools {
        validate_tool_state(tool)?;
    }
    for event in &state.events {
        validate_turn_event(event)?;
    }
    Ok(())
}

fn validate_tool_state(tool: &OperatorToolState) -> Result<()> {
    if tool.tool_call_id.trim().is_empty() {
        bail!("operator tool_call_id cannot be empty");
    }
    if tool.tool_name.trim().is_empty() {
        bail!("operator tool_name cannot be empty");
    }
    Ok(())
}

fn validate_turn_event(event: &OperatorTurnEvent) -> Result<()> {
    if event.event_id.trim().is_empty() {
        bail!("operator event_id cannot be empty");
    }
    if event.summary.trim().is_empty() {
        bail!("operator event summary cannot be empty");
    }
    Ok(())
}
