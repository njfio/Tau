use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};

use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tau_agent_core::{
    coding_mission_state_path, load_coding_mission_state, CodingMissionPhase,
    CodingMissionPrPublicationStatus, CodingMissionState, CodingWorkspaceCommandEvidence,
    CodingWorkspaceCommandStatus,
};

use super::mission_supervisor_runtime::{
    gateway_mission_state_path, gateway_missions_root, load_gateway_mission_state,
    GatewayMissionState,
};
use super::{
    authorize_and_enforce_gateway_limits, sanitize_session_key, GatewayOpenResponsesServerState,
    OpenResponsesApiError,
};

#[derive(Debug, Clone, Deserialize, Default)]
pub(super) struct GatewayMissionsListQuery {
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct GatewayCodingMissionSummary {
    mission_id: String,
    phase: &'static str,
    repo_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verifier_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verifier_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_failure: Option<String>,
    changed_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resume_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pr_state: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pr_url: Option<String>,
}

pub(super) async fn handle_gateway_missions_list(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    headers: HeaderMap,
    Query(query): Query<GatewayMissionsListQuery>,
) -> Response {
    if let Err(error) = authorize_and_enforce_gateway_limits(&state, &headers) {
        return error.into_response();
    }

    let limit = query.limit.unwrap_or(20).clamp(1, 200);
    let missions_root = gateway_missions_root(&state.config.state_dir);
    let mut missions = Vec::<GatewayMissionState>::new();

    if missions_root.is_dir() {
        let dir_entries = match std::fs::read_dir(&missions_root) {
            Ok(entries) => entries,
            Err(error) => {
                return OpenResponsesApiError::internal(format!(
                    "failed to list missions directory {}: {error}",
                    missions_root.display()
                ))
                .into_response();
            }
        };

        for dir_entry in dir_entries.flatten() {
            let path = dir_entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            match load_gateway_mission_state(&path) {
                Ok(mission) => missions.push(mission),
                Err(error) => return error.into_response(),
            }
        }
    }

    missions.sort_by_key(|mission| std::cmp::Reverse(mission.updated_unix_ms));
    let harness_missions = missions
        .iter()
        .take(limit)
        .map(GatewayMissionState::to_shared_mission_snapshot)
        .collect::<Vec<_>>();
    let coding_states = match load_coding_mission_states(&state.config.state_dir) {
        Ok(states) => states,
        Err(error) => return error.into_response(),
    };
    let coding_states_by_id = coding_states
        .into_iter()
        .map(|coding_state| (coding_state.mission_id.clone(), coding_state))
        .collect::<BTreeMap<_, _>>();
    let coding_missions = coding_states_by_id
        .values()
        .map(coding_mission_summary)
        .collect::<Vec<_>>();
    let mut seen_gateway_mission_ids = BTreeSet::new();
    let mut mission_values = Vec::new();
    for mission in &missions {
        seen_gateway_mission_ids.insert(mission.mission_id.clone());
        match gateway_mission_response_value(
            mission,
            coding_states_by_id.get(mission.mission_id.as_str()),
        ) {
            Ok(value) => mission_values.push(value),
            Err(error) => return error.into_response(),
        }
    }
    for coding_state in coding_states_by_id.values() {
        if seen_gateway_mission_ids.contains(coding_state.mission_id.as_str()) {
            continue;
        }
        match coding_mission_response_value(coding_state) {
            Ok(value) => mission_values.push(value),
            Err(error) => return error.into_response(),
        }
    }
    mission_values.sort_by_key(|mission| {
        std::cmp::Reverse(
            mission
                .get("updated_unix_ms")
                .and_then(Value::as_u64)
                .unwrap_or_default(),
        )
    });
    mission_values.truncate(limit);

    state.record_ui_telemetry_event("missions", "list", "mission_list_requested");
    (
        StatusCode::OK,
        Json(json!({
            "missions": mission_values,
            "harness_missions": harness_missions,
            "coding_missions": coding_missions,
            "limit": limit,
        })),
    )
        .into_response()
}

pub(super) async fn handle_gateway_mission_detail(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    headers: HeaderMap,
    AxumPath(mission_id): AxumPath<String>,
) -> Response {
    if let Err(error) = authorize_and_enforce_gateway_limits(&state, &headers) {
        return error.into_response();
    }

    let mission_id = sanitize_session_key(mission_id.as_str());
    let mission_path = gateway_mission_state_path(&state.config.state_dir, &mission_id);
    let coding_state =
        match load_coding_mission_state_if_present(&state.config.state_dir, &mission_id) {
            Ok(state) => state,
            Err(error) => return error.into_response(),
        };
    let mission = if mission_path.exists() {
        match load_gateway_mission_state(&mission_path) {
            Ok(mission) => mission,
            Err(error) => return error.into_response(),
        }
    } else if let Some(coding_state) = coding_state.as_ref() {
        state.record_ui_telemetry_event("missions", "detail", "mission_detail_requested");
        return match coding_mission_response_value(coding_state) {
            Ok(mission_value) => (
                StatusCode::OK,
                Json(json!({
                    "mission": mission_value,
                    "harness_mission": Value::Null,
                    "coding_mission": coding_mission_summary(coding_state),
                    "path": coding_mission_state_path(&state.config.state_dir, &mission_id)
                        .display()
                        .to_string(),
                })),
            )
                .into_response(),
            Err(error) => error.into_response(),
        };
    } else {
        return OpenResponsesApiError::not_found(
            "mission_not_found",
            format!("mission '{mission_id}' does not exist"),
        )
        .into_response();
    };
    let harness_mission = mission.to_shared_mission_snapshot();
    let mission_value = match gateway_mission_response_value(&mission, coding_state.as_ref()) {
        Ok(value) => value,
        Err(error) => return error.into_response(),
    };

    state.record_ui_telemetry_event("missions", "detail", "mission_detail_requested");
    (
        StatusCode::OK,
        Json(json!({
            "mission": mission_value,
            "harness_mission": harness_mission,
            "coding_mission": coding_state.as_ref().map(coding_mission_summary),
            "path": mission_path.display().to_string(),
        })),
    )
        .into_response()
}

fn load_coding_mission_states(
    state_dir: &Path,
) -> Result<Vec<CodingMissionState>, OpenResponsesApiError> {
    let missions_root = state_dir.join("coding-missions");
    if !missions_root.is_dir() {
        return Ok(Vec::new());
    }
    let dir_entries = std::fs::read_dir(&missions_root).map_err(|error| {
        OpenResponsesApiError::internal(format!(
            "failed to list coding missions directory {}: {error}",
            missions_root.display()
        ))
    })?;
    let mut states = Vec::new();
    for dir_entry in dir_entries.flatten() {
        let path = dir_entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Some(mission_id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        states.push(
            load_coding_mission_state(state_dir, mission_id).map_err(|error| {
                OpenResponsesApiError::internal(format!(
                    "failed to load coding mission state {}: {error}",
                    path.display()
                ))
            })?,
        );
    }
    Ok(states)
}

fn load_coding_mission_state_if_present(
    state_dir: &Path,
    mission_id: &str,
) -> Result<Option<CodingMissionState>, OpenResponsesApiError> {
    let path = coding_mission_state_path(state_dir, mission_id);
    if !path.exists() {
        return Ok(None);
    }
    load_coding_mission_state(state_dir, mission_id)
        .map(Some)
        .map_err(|error| {
            OpenResponsesApiError::internal(format!(
                "failed to load coding mission state {}: {error}",
                path.display()
            ))
        })
}

fn gateway_mission_response_value(
    mission: &GatewayMissionState,
    coding_state: Option<&CodingMissionState>,
) -> Result<Value, OpenResponsesApiError> {
    let mut value = serde_json::to_value(mission).map_err(|error| {
        OpenResponsesApiError::internal(format!("failed to serialize gateway mission: {error}"))
    })?;
    if let Some(coding_state) = coding_state {
        value["coding_mission"] = serde_json::to_value(coding_mission_summary(coding_state))
            .map_err(|error| {
                OpenResponsesApiError::internal(format!(
                    "failed to serialize coding mission summary: {error}"
                ))
            })?;
    }
    Ok(value)
}

fn coding_mission_response_value(
    state: &CodingMissionState,
) -> Result<Value, OpenResponsesApiError> {
    let summary = coding_mission_summary(state);
    let latest_verifier = coding_latest_verifier_value(state);
    let latest_output = coding_latest_output_summary(state);
    let latest_completion = summary.resume_command.as_ref().map(|resume_command| {
        json!({
            "status": if state.phase == CodingMissionPhase::Blocked { "blocked" } else { "partial" },
            "summary": latest_output.clone(),
            "next_step": resume_command,
        })
    });
    serde_json::to_value(json!({
        "schema_version": 1,
        "mission_id": state.mission_id,
        "session_key": state.session_key,
        "response_id": format!("coding_{}", state.mission_id),
        "goal_summary": state.goal,
        "latest_output_summary": latest_output,
        "status": coding_phase_gateway_status(state.phase),
        "created_unix_ms": state.created_unix_ms,
        "updated_unix_ms": state.updated_unix_ms,
        "iteration_count": state.command_evidence.len(),
        "latest_verifier": latest_verifier,
        "latest_completion": latest_completion,
        "iterations": [],
        "coding_mission": summary,
    }))
    .map_err(|error| {
        OpenResponsesApiError::internal(format!("failed to serialize coding mission: {error}"))
    })
}

fn coding_mission_summary(state: &CodingMissionState) -> GatewayCodingMissionSummary {
    let latest_command = state.command_evidence.last();
    let latest_failed_command = state
        .command_evidence
        .iter()
        .rev()
        .find(|command| command.status != CodingWorkspaceCommandStatus::Succeeded);
    GatewayCodingMissionSummary {
        mission_id: state.mission_id.clone(),
        phase: coding_phase_label(state.phase),
        repo_path: state.repo_path.display().to_string(),
        branch_name: coding_branch_name(state),
        verifier_status: latest_command.map(coding_command_status_summary),
        verifier_command: latest_command.map(|command| command.argv.join(" ")),
        last_failure: latest_failed_command.map(coding_command_status_summary),
        changed_files: coding_changed_files(state),
        resume_command: state
            .resume_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.operator_resume_command.clone()),
        pr_state: state
            .pr_ready_bundle
            .as_ref()
            .map(|bundle| coding_pr_publication_label(bundle.status))
            .or_else(|| (state.phase == CodingMissionPhase::PrReady).then_some("pr_ready")),
        pr_url: state
            .pr_ready_bundle
            .as_ref()
            .and_then(|bundle| bundle.pr_url.clone()),
    }
}

fn coding_latest_verifier_value(state: &CodingMissionState) -> Value {
    if let Some(command) = state.command_evidence.last() {
        json!({
            "kind": "coding_mission_verifier",
            "status": match command.status {
                CodingWorkspaceCommandStatus::Succeeded => "passed",
                CodingWorkspaceCommandStatus::Failed | CodingWorkspaceCommandStatus::Denied => "failed",
            },
            "reason_code": command.reason_code,
            "message": coding_command_status_summary(command),
        })
    } else {
        json!({
            "kind": "coding_mission_verifier",
            "status": "continue",
            "reason_code": "coding_mission_pending_verifier",
            "message": "coding mission has not recorded verifier evidence yet",
        })
    }
}

fn coding_latest_output_summary(state: &CodingMissionState) -> String {
    state
        .resume_checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.latest_learning_summary.clone())
        .or_else(|| state.events.last().map(|event| event.message.clone()))
        .unwrap_or_else(|| "coding mission state persisted".to_string())
}

fn coding_branch_name(state: &CodingMissionState) -> Option<String> {
    state
        .resume_checkpoint
        .as_ref()
        .and_then(|checkpoint| checkpoint.branch_name.clone())
        .or_else(|| {
            state
                .pr_ready_bundle
                .as_ref()
                .map(|bundle| bundle.branch_name.clone())
        })
        .or_else(|| {
            state
                .git_evidence
                .iter()
                .rev()
                .find(|evidence| !evidence.branch_name.trim().is_empty())
                .map(|evidence| evidence.branch_name.clone())
        })
}

fn coding_changed_files(state: &CodingMissionState) -> Vec<String> {
    if let Some(bundle) = state
        .pr_ready_bundle
        .as_ref()
        .filter(|bundle| !bundle.changed_files.is_empty())
    {
        return bundle.changed_files.clone();
    }
    if let Some(evidence) = state
        .git_evidence
        .iter()
        .rev()
        .find(|evidence| !evidence.changed_files.is_empty())
    {
        return evidence.changed_files.clone();
    }
    state
        .resume_checkpoint
        .as_ref()
        .and_then(|checkpoint| checkpoint.mutation_fingerprint.as_ref())
        .map(|fingerprint| fingerprint.changed_files.clone())
        .unwrap_or_default()
}

fn coding_command_status_summary(command: &CodingWorkspaceCommandEvidence) -> String {
    format!(
        "{}:{}",
        coding_command_status_label(command.status),
        command.reason_code
    )
}

fn coding_command_status_label(status: CodingWorkspaceCommandStatus) -> &'static str {
    match status {
        CodingWorkspaceCommandStatus::Succeeded => "succeeded",
        CodingWorkspaceCommandStatus::Failed => "failed",
        CodingWorkspaceCommandStatus::Denied => "denied",
    }
}

fn coding_phase_gateway_status(phase: CodingMissionPhase) -> &'static str {
    match phase {
        CodingMissionPhase::Intake | CodingMissionPhase::Planned => "checkpointed",
        CodingMissionPhase::PreparingBranch
        | CodingMissionPhase::Executing
        | CodingMissionPhase::Verifying => "running",
        CodingMissionPhase::PrReady => "checkpointed",
        CodingMissionPhase::Blocked => "blocked",
        CodingMissionPhase::Completed => "completed",
    }
}

fn coding_phase_label(phase: CodingMissionPhase) -> &'static str {
    match phase {
        CodingMissionPhase::Intake => "intake",
        CodingMissionPhase::Planned => "planned",
        CodingMissionPhase::PreparingBranch => "preparing_branch",
        CodingMissionPhase::Executing => "executing",
        CodingMissionPhase::Verifying => "verifying",
        CodingMissionPhase::PrReady => "pr_ready",
        CodingMissionPhase::Blocked => "blocked",
        CodingMissionPhase::Completed => "completed",
    }
}

fn coding_pr_publication_label(status: CodingMissionPrPublicationStatus) -> &'static str {
    match status {
        CodingMissionPrPublicationStatus::ManualReady => "manual_ready",
        CodingMissionPrPublicationStatus::DraftCreated => "draft_created",
        CodingMissionPrPublicationStatus::DraftFailed => "draft_failed",
    }
}
