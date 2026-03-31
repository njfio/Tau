use std::sync::Arc;

use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

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

    missions.sort_by(|left, right| right.updated_unix_ms.cmp(&left.updated_unix_ms));
    missions.truncate(limit);

    state.record_ui_telemetry_event("missions", "list", "mission_list_requested");
    (
        StatusCode::OK,
        Json(json!({
            "missions": missions,
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
    if !mission_path.exists() {
        return OpenResponsesApiError::not_found(
            "mission_not_found",
            format!("mission '{mission_id}' does not exist"),
        )
        .into_response();
    }

    let mission = match load_gateway_mission_state(&mission_path) {
        Ok(mission) => mission,
        Err(error) => return error.into_response(),
    };

    state.record_ui_telemetry_event("missions", "detail", "mission_detail_requested");
    (
        StatusCode::OK,
        Json(json!({
            "mission": mission,
            "path": mission_path.display().to_string(),
        })),
    )
        .into_response()
}
