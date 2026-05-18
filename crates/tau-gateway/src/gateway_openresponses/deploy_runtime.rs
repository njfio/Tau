//! Gateway deploy/stop endpoint handlers.
//!
//! This module provides bounded runtime contracts for deploy requests and
//! stop actions backed by deterministic state persisted under gateway state dir.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tau_core::{current_unix_timestamp_ms, write_text_atomic};
use tau_dashboard_ui::{TauOpsDashboardDeployAgentRow, TauOpsDashboardDeploySnapshot};

use super::deploy_process_supervisor::GatewayDeployProcessStartRequest;
use super::{
    authorize_dashboard_request, parse_gateway_json_body, GatewayOpenResponsesServerState,
    OpenResponsesApiError,
};

const DEPLOY_STATE_FILE: &str = "deploy-agent-state.json";
const DEPLOY_STATE_SCHEMA_VERSION: u32 = 1;
const DEPLOY_STATUS_DEPLOYING: &str = "deploying";
const DEPLOY_STATUS_STOPPED: &str = "stopped";
const STOP_REASON_OPERATOR_REQUEST: &str = "operator_stop_request";

#[derive(Debug, Deserialize, Default)]
struct GatewayDeployRequest {
    #[serde(default)]
    agent_id: String,
    #[serde(default)]
    profile: String,
    #[serde(default)]
    model: String,
}

#[derive(Debug, Clone, Default)]
pub(super) struct GatewayDeployAgentInput {
    pub(super) agent_id: String,
    pub(super) profile: String,
    pub(super) model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GatewayDeployAgentResult {
    pub(super) agent_id: String,
    pub(super) status: String,
    pub(super) profile: String,
    pub(super) model: String,
    pub(super) accepted_unix_ms: u64,
    pub(super) process_id: String,
    pub(super) process_status: String,
    pub(super) process_pid: Option<u32>,
    pub(super) process_started_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GatewayDeployAgentStopResult {
    pub(super) agent_id: String,
    pub(super) status: String,
    pub(super) stopped_unix_ms: u64,
    pub(super) process_id: String,
    pub(super) process_status: String,
    pub(super) process_pid: Option<u32>,
    pub(super) process_stopped_unix_ms: u64,
    pub(super) process_stop_reason: String,
    pub(super) process_exit_status: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct GatewayDeployAgentRecord {
    agent_id: String,
    status: String,
    profile: String,
    model: String,
    created_unix_ms: u64,
    updated_unix_ms: u64,
    #[serde(default)]
    process_id: String,
    #[serde(default)]
    process_status: String,
    #[serde(default)]
    process_pid: Option<u32>,
    #[serde(default)]
    process_started_unix_ms: Option<u64>,
    #[serde(default)]
    process_stopped_unix_ms: Option<u64>,
    #[serde(default)]
    process_stop_reason: Option<String>,
    #[serde(default)]
    process_exit_status: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct GatewayDeployStateFile {
    #[serde(default = "deploy_state_schema_version")]
    schema_version: u32,
    #[serde(default)]
    agents: BTreeMap<String, GatewayDeployAgentRecord>,
}

impl Default for GatewayDeployStateFile {
    fn default() -> Self {
        Self {
            schema_version: DEPLOY_STATE_SCHEMA_VERSION,
            agents: BTreeMap::new(),
        }
    }
}

fn deploy_state_schema_version() -> u32 {
    DEPLOY_STATE_SCHEMA_VERSION
}

pub(super) async fn handle_gateway_deploy(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(error) = authorize_dashboard_request(&state, &headers) {
        return error.into_response();
    }
    let request = match parse_gateway_json_body::<GatewayDeployRequest>(&body) {
        Ok(request) => request,
        Err(error) => return error.into_response(),
    };
    let deploy_result = match deploy_gateway_agent(
        &state,
        GatewayDeployAgentInput {
            agent_id: request.agent_id,
            profile: request.profile,
            model: request.model,
        },
    ) {
        Ok(result) => result,
        Err(error) => return error.into_response(),
    };

    (
        StatusCode::OK,
        Json(json!({
            "schema_version": DEPLOY_STATE_SCHEMA_VERSION,
            "agent_id": deploy_result.agent_id,
            "status": deploy_result.status,
            "profile": deploy_result.profile,
            "model": deploy_result.model,
            "accepted_unix_ms": deploy_result.accepted_unix_ms,
            "process_id": deploy_result.process_id,
            "process_status": deploy_result.process_status,
            "process_pid": deploy_result.process_pid,
            "process_started_unix_ms": deploy_result.process_started_unix_ms,
        })),
    )
        .into_response()
}

pub(super) fn deploy_gateway_agent(
    state: &GatewayOpenResponsesServerState,
    request: GatewayDeployAgentInput,
) -> Result<GatewayDeployAgentResult, OpenResponsesApiError> {
    let normalized_agent_id = request.agent_id.trim();
    if normalized_agent_id.is_empty() {
        return Err(OpenResponsesApiError::bad_request(
            "invalid_agent_id",
            "agent_id must be non-empty",
        ));
    }

    let now_unix_ms = current_unix_timestamp_ms();
    let profile = normalize_non_empty(request.profile.as_str(), "default");
    let model = normalize_non_empty(request.model.as_str(), state.config.model.as_str());
    let state_path = gateway_deploy_state_path(&state.config.state_dir);
    let mut deploy_state = load_gateway_deploy_state(&state_path)?;

    let created_unix_ms = deploy_state
        .agents
        .get(normalized_agent_id)
        .map(|existing| existing.created_unix_ms)
        .unwrap_or(now_unix_ms);
    let process_start =
        match state
            .deploy_process_supervisor
            .start(GatewayDeployProcessStartRequest {
                agent_id: normalized_agent_id.to_string(),
                profile: profile.clone(),
                model: model.clone(),
                state_dir: state.config.state_dir.clone(),
            }) {
            Ok(result) => result,
            Err(error) => {
                return Err(OpenResponsesApiError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error.code(),
                    format!("failed to start deploy process for '{normalized_agent_id}': {error}"),
                ));
            }
        };
    if let Err(error) = crate::gateway_runtime::start_gateway_service_mode(&state.config.state_dir)
    {
        let _ = state
            .deploy_process_supervisor
            .stop(normalized_agent_id, "gateway_service_state_start_failed");
        return Err(OpenResponsesApiError::internal(format!(
            "failed to transition gateway service state for deploy request: {error}"
        )));
    }
    deploy_state.agents.insert(
        normalized_agent_id.to_string(),
        GatewayDeployAgentRecord {
            agent_id: normalized_agent_id.to_string(),
            status: DEPLOY_STATUS_DEPLOYING.to_string(),
            profile: profile.clone(),
            model: model.clone(),
            created_unix_ms,
            updated_unix_ms: now_unix_ms,
            process_id: process_start.process_id.clone(),
            process_status: process_start.status.clone(),
            process_pid: process_start.pid,
            process_started_unix_ms: Some(process_start.started_unix_ms),
            process_stopped_unix_ms: None,
            process_stop_reason: None,
            process_exit_status: None,
        },
    );
    if let Err(error) = save_gateway_deploy_state(&state_path, &deploy_state) {
        let _ = state
            .deploy_process_supervisor
            .stop(normalized_agent_id, "deploy_state_persist_failed");
        return Err(error);
    }

    Ok(GatewayDeployAgentResult {
        agent_id: normalized_agent_id.to_string(),
        status: DEPLOY_STATUS_DEPLOYING.to_string(),
        profile,
        model,
        accepted_unix_ms: now_unix_ms,
        process_id: process_start.process_id,
        process_status: process_start.status,
        process_pid: process_start.pid,
        process_started_unix_ms: process_start.started_unix_ms,
    })
}

pub(super) async fn handle_gateway_agent_stop(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    headers: HeaderMap,
    AxumPath(agent_id): AxumPath<String>,
) -> Response {
    if let Err(error) = authorize_dashboard_request(&state, &headers) {
        return error.into_response();
    }
    let stop_result =
        match stop_gateway_deploy_agent(&state, agent_id.as_str(), STOP_REASON_OPERATOR_REQUEST) {
            Ok(result) => result,
            Err(error) => return error.into_response(),
        };

    (
        StatusCode::OK,
        Json(json!({
            "schema_version": DEPLOY_STATE_SCHEMA_VERSION,
            "agent_id": stop_result.agent_id,
            "status": stop_result.status,
            "stopped_unix_ms": stop_result.stopped_unix_ms,
            "process_id": stop_result.process_id,
            "process_status": stop_result.process_status,
            "process_pid": stop_result.process_pid,
            "process_stopped_unix_ms": stop_result.process_stopped_unix_ms,
            "process_stop_reason": stop_result.process_stop_reason,
            "process_exit_status": stop_result.process_exit_status,
        })),
    )
        .into_response()
}

pub(super) fn stop_gateway_deploy_agent(
    state: &GatewayOpenResponsesServerState,
    agent_id: &str,
    stop_reason: &'static str,
) -> Result<GatewayDeployAgentStopResult, OpenResponsesApiError> {
    let normalized_agent_id = agent_id.trim();
    if normalized_agent_id.is_empty() {
        return Err(OpenResponsesApiError::bad_request(
            "invalid_agent_id",
            "agent_id must be non-empty",
        ));
    }

    let state_path = gateway_deploy_state_path(&state.config.state_dir);
    let mut deploy_state = load_gateway_deploy_state(&state_path)?;
    let now_unix_ms = current_unix_timestamp_ms();
    if !deploy_state.agents.contains_key(normalized_agent_id) {
        return Err(OpenResponsesApiError::not_found(
            "agent_not_found",
            format!("agent '{normalized_agent_id}' was not found"),
        ));
    };
    let process_stop = match state
        .deploy_process_supervisor
        .stop(normalized_agent_id, stop_reason)
    {
        Ok(result) => result,
        Err(error) => {
            return Err(OpenResponsesApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                error.code(),
                format!("failed to stop deploy process for '{normalized_agent_id}': {error}"),
            ));
        }
    };
    let Some(record) = deploy_state.agents.get_mut(normalized_agent_id) else {
        return Err(OpenResponsesApiError::not_found(
            "agent_not_found",
            format!("agent '{normalized_agent_id}' was not found"),
        ));
    };
    record.status = DEPLOY_STATUS_STOPPED.to_string();
    record.updated_unix_ms = now_unix_ms;
    record.process_id = process_stop.process_id.clone();
    record.process_status = process_stop.status.clone();
    record.process_pid = process_stop.pid;
    record.process_stopped_unix_ms = Some(process_stop.stopped_unix_ms);
    record.process_stop_reason = Some(process_stop.stop_reason.clone());
    record.process_exit_status = process_stop.exit_status;
    save_gateway_deploy_state(&state_path, &deploy_state)?;

    if let Err(error) = crate::gateway_runtime::stop_gateway_service_mode(
        &state.config.state_dir,
        Some(stop_reason),
    ) {
        return Err(OpenResponsesApiError::internal(format!(
            "failed to transition gateway service state for stop request: {error}"
        )));
    }

    Ok(GatewayDeployAgentStopResult {
        agent_id: normalized_agent_id.to_string(),
        status: DEPLOY_STATUS_STOPPED.to_string(),
        stopped_unix_ms: now_unix_ms,
        process_id: process_stop.process_id,
        process_status: process_stop.status,
        process_pid: process_stop.pid,
        process_stopped_unix_ms: process_stop.stopped_unix_ms,
        process_stop_reason: process_stop.stop_reason,
        process_exit_status: process_stop.exit_status,
    })
}

fn load_gateway_deploy_state(path: &Path) -> Result<GatewayDeployStateFile, OpenResponsesApiError> {
    if !path.exists() {
        return Ok(GatewayDeployStateFile::default());
    }
    let raw = std::fs::read_to_string(path).map_err(|error| {
        OpenResponsesApiError::internal(format!(
            "failed to read deploy runtime state '{}': {error}",
            path.display()
        ))
    })?;
    serde_json::from_str::<GatewayDeployStateFile>(&raw).map_err(|error| {
        OpenResponsesApiError::internal(format!(
            "failed to parse deploy runtime state '{}': {error}",
            path.display()
        ))
    })
}

fn save_gateway_deploy_state(
    path: &Path,
    state: &GatewayDeployStateFile,
) -> Result<(), OpenResponsesApiError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            OpenResponsesApiError::internal(format!(
                "failed to create deploy runtime state directory '{}': {error}",
                parent.display()
            ))
        })?;
    }
    let serialized = serde_json::to_string_pretty(state).map_err(|error| {
        OpenResponsesApiError::internal(format!(
            "failed to serialize deploy runtime state: {error}"
        ))
    })?;
    write_text_atomic(path, serialized.as_str()).map_err(|error| {
        OpenResponsesApiError::internal(format!(
            "failed to persist deploy runtime state '{}': {error}",
            path.display()
        ))
    })
}

fn gateway_deploy_state_path(state_dir: &Path) -> PathBuf {
    state_dir.join(DEPLOY_STATE_FILE)
}

pub(super) fn collect_tau_ops_dashboard_deploy_snapshot(
    state_dir: &Path,
) -> TauOpsDashboardDeploySnapshot {
    let state_path = gateway_deploy_state_path(state_dir);
    let state_source = state_path.display().to_string();
    if !state_path.exists() {
        return TauOpsDashboardDeploySnapshot {
            state_source,
            state_status: "missing".to_string(),
            ..TauOpsDashboardDeploySnapshot::default()
        };
    }

    let deploy_state = match load_gateway_deploy_state(&state_path) {
        Ok(state) => state,
        Err(error) => {
            return TauOpsDashboardDeploySnapshot {
                state_source,
                state_status: format!("error:{}", error.code),
                ..TauOpsDashboardDeploySnapshot::default()
            };
        }
    };
    let rows = deploy_state
        .agents
        .values()
        .map(|record| TauOpsDashboardDeployAgentRow {
            agent_id: record.agent_id.clone(),
            status: record.status.clone(),
            profile: record.profile.clone(),
            model: record.model.clone(),
            updated_unix_ms: record.updated_unix_ms,
            process_id: record.process_id.clone(),
            process_status: normalize_non_empty(record.process_status.as_str(), "unknown"),
            process_pid: record
                .process_pid
                .map(|pid| pid.to_string())
                .unwrap_or_else(|| "none".to_string()),
            process_started_unix_ms: record.process_started_unix_ms.unwrap_or(0),
            process_stopped_unix_ms: record.process_stopped_unix_ms.unwrap_or(0),
            process_stop_reason: record
                .process_stop_reason
                .clone()
                .unwrap_or_else(|| "none".to_string()),
            process_exit_status: record
                .process_exit_status
                .map(|status| status.to_string())
                .unwrap_or_else(|| "none".to_string()),
        })
        .collect::<Vec<_>>();
    let running_count = rows
        .iter()
        .filter(|row| row.process_status == "running")
        .count();
    let stopped_count = rows
        .iter()
        .filter(|row| row.process_status == "stopped")
        .count();
    TauOpsDashboardDeploySnapshot {
        state_source,
        state_status: if rows.is_empty() {
            "empty".to_string()
        } else {
            "loaded".to_string()
        },
        agent_count: rows.len(),
        running_count,
        stopped_count,
        rows,
    }
}

fn normalize_non_empty(raw: &str, fallback: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_normalize_non_empty_uses_trimmed_or_fallback() {
        assert_eq!(normalize_non_empty("  value  ", "fallback"), "value");
        assert_eq!(normalize_non_empty("   ", "fallback"), "fallback");
    }

    #[test]
    fn unit_gateway_deploy_state_path_uses_expected_filename() {
        let state_dir = PathBuf::from("/tmp/tau-gateway-tests");
        let path = gateway_deploy_state_path(&state_dir);
        assert!(path.ends_with(DEPLOY_STATE_FILE));
    }

    #[test]
    fn unit_collect_tau_ops_dashboard_deploy_snapshot_maps_process_rows() {
        let temp = tempfile::tempdir().expect("tempdir");
        let state_path = gateway_deploy_state_path(temp.path());
        let mut state = GatewayDeployStateFile::default();
        state.agents.insert(
            "agent-process".to_string(),
            GatewayDeployAgentRecord {
                agent_id: "agent-process".to_string(),
                status: DEPLOY_STATUS_DEPLOYING.to_string(),
                profile: "default".to_string(),
                model: "openai/gpt-5.3-codex".to_string(),
                created_unix_ms: 100,
                updated_unix_ms: 200,
                process_id: "gateway-deploy:agent-process:4242".to_string(),
                process_status: "running".to_string(),
                process_pid: Some(4242),
                process_started_unix_ms: Some(150),
                process_stopped_unix_ms: None,
                process_stop_reason: None,
                process_exit_status: None,
            },
        );
        save_gateway_deploy_state(&state_path, &state).expect("save deploy state");

        let snapshot = collect_tau_ops_dashboard_deploy_snapshot(temp.path());

        assert_eq!(snapshot.state_status, "loaded");
        assert_eq!(snapshot.agent_count, 1);
        assert_eq!(snapshot.running_count, 1);
        assert_eq!(snapshot.stopped_count, 0);
        assert_eq!(snapshot.rows[0].agent_id, "agent-process");
        assert_eq!(snapshot.rows[0].process_status, "running");
        assert_eq!(snapshot.rows[0].process_pid, "4242");
        assert_eq!(snapshot.rows[0].process_stop_reason, "none");
    }
}
