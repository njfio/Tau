use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};
use tau_core::current_unix_timestamp_ms;

use super::types::GatewayConfigPatchRequest;
use super::{
    authorize_and_enforce_gateway_limits, parse_gateway_json_body, GatewayOpenResponsesServerState,
    OpenResponsesApiError,
};

pub(super) async fn handle_gateway_config_get(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(error) = authorize_and_enforce_gateway_limits(&state, &headers) {
        return error.into_response();
    }

    let overrides_path = gateway_config_overrides_path(&state.config.state_dir);
    let pending_overrides = match read_gateway_config_pending_overrides(&overrides_path) {
        Ok(overrides) => overrides,
        Err(error) => return error.into_response(),
    };
    let heartbeat_interval_ms =
        match u64::try_from(state.config.runtime_heartbeat.interval.as_millis()) {
            Ok(value) => value,
            Err(_) => u64::MAX,
        };
    let heartbeat_policy_path =
        gateway_runtime_heartbeat_policy_path(&state.config.runtime_heartbeat.state_path);
    let heartbeat_policy_exists = heartbeat_policy_path.is_file();

    state.record_ui_telemetry_event("configuration", "config_get", "config_get_requested");
    (
        StatusCode::OK,
        Json(json!({
            "active": {
                "model": state.config.model.clone(),
                "system_prompt": state.config.system_prompt.clone(),
                "max_turns": state.config.max_turns,
                "max_input_chars": state.config.max_input_chars,
                "turn_timeout_ms": state.config.turn_timeout_ms,
                "session_lock_wait_ms": state.config.session_lock_wait_ms,
                "session_lock_stale_ms": state.config.session_lock_stale_ms,
                "auth_mode": state.config.auth_mode.as_str(),
                "rate_limit_window_seconds": state.config.rate_limit_window_seconds,
                "rate_limit_max_requests": state.config.rate_limit_max_requests,
                "runtime_heartbeat_enabled": state.config.runtime_heartbeat.enabled,
                "runtime_heartbeat_interval_ms": heartbeat_interval_ms,
            },
            "pending_overrides": pending_overrides,
            "overrides_path": overrides_path.display().to_string(),
            "hot_reload_capabilities": {
                "runtime_heartbeat_interval_ms": {
                    "mode": "hot_reload",
                    "policy_path": heartbeat_policy_path.display().to_string(),
                    "policy_exists": heartbeat_policy_exists,
                },
                "model": { "mode": "restart_required" },
                "system_prompt": { "mode": "restart_required" },
                "max_turns": { "mode": "restart_required" },
                "max_input_chars": { "mode": "restart_required" },
            }
        })),
    )
        .into_response()
}

pub(super) async fn handle_gateway_config_patch(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(error) = authorize_and_enforce_gateway_limits(&state, &headers) {
        return error.into_response();
    }
    let request = match parse_gateway_json_body::<GatewayConfigPatchRequest>(&body) {
        Ok(request) => request,
        Err(error) => return error.into_response(),
    };

    let mut pending_overrides = match read_gateway_config_pending_overrides(
        &gateway_config_overrides_path(&state.config.state_dir),
    ) {
        Ok(overrides) => overrides,
        Err(error) => return error.into_response(),
    };
    let mut accepted = serde_json::Map::<String, Value>::new();
    let mut applied = serde_json::Map::<String, Value>::new();
    let mut restart_required_fields = BTreeSet::<String>::new();

    if let Some(model) = request.model {
        let trimmed = model.trim().to_string();
        if trimmed.is_empty() {
            return OpenResponsesApiError::bad_request("invalid_model", "model must be non-empty")
                .into_response();
        }
        accepted.insert("model".to_string(), json!(trimmed));
        pending_overrides.insert("model".to_string(), json!(trimmed));
        restart_required_fields.insert("model".to_string());
    }

    if let Some(system_prompt) = request.system_prompt {
        let trimmed = system_prompt.trim().to_string();
        if trimmed.is_empty() {
            return OpenResponsesApiError::bad_request(
                "invalid_system_prompt",
                "system_prompt must be non-empty",
            )
            .into_response();
        }
        accepted.insert("system_prompt".to_string(), json!(trimmed));
        pending_overrides.insert("system_prompt".to_string(), json!(trimmed));
        restart_required_fields.insert("system_prompt".to_string());
    }

    if let Some(max_turns) = request.max_turns {
        if max_turns == 0 {
            return OpenResponsesApiError::bad_request(
                "invalid_max_turns",
                "max_turns must be greater than zero",
            )
            .into_response();
        }
        accepted.insert("max_turns".to_string(), json!(max_turns));
        pending_overrides.insert("max_turns".to_string(), json!(max_turns));
        restart_required_fields.insert("max_turns".to_string());
    }

    if let Some(max_input_chars) = request.max_input_chars {
        if max_input_chars == 0 {
            return OpenResponsesApiError::bad_request(
                "invalid_max_input_chars",
                "max_input_chars must be greater than zero",
            )
            .into_response();
        }
        accepted.insert("max_input_chars".to_string(), json!(max_input_chars));
        pending_overrides.insert("max_input_chars".to_string(), json!(max_input_chars));
        restart_required_fields.insert("max_input_chars".to_string());
    }

    if let Some(runtime_heartbeat_interval_ms) = request.runtime_heartbeat_interval_ms {
        if runtime_heartbeat_interval_ms == 0 {
            return OpenResponsesApiError::bad_request(
                "invalid_runtime_heartbeat_interval_ms",
                "runtime_heartbeat_interval_ms must be greater than zero",
            )
            .into_response();
        }
        let clamped_interval_ms = runtime_heartbeat_interval_ms.clamp(100, 60_000);
        let policy_path =
            gateway_runtime_heartbeat_policy_path(&state.config.runtime_heartbeat.state_path);
        let policy_payload = format!("interval_ms = {clamped_interval_ms}\n");
        if let Some(parent) = policy_path.parent() {
            if !parent.as_os_str().is_empty() {
                if let Err(error) = std::fs::create_dir_all(parent) {
                    return OpenResponsesApiError::internal(format!(
                        "failed to create runtime heartbeat policy dir '{}': {error}",
                        parent.display()
                    ))
                    .into_response();
                }
            }
        }
        if let Err(error) = std::fs::write(&policy_path, policy_payload.as_bytes()) {
            return OpenResponsesApiError::internal(format!(
                "failed to write runtime heartbeat policy '{}': {error}",
                policy_path.display()
            ))
            .into_response();
        }

        accepted.insert(
            "runtime_heartbeat_interval_ms".to_string(),
            json!(clamped_interval_ms),
        );
        pending_overrides.insert(
            "runtime_heartbeat_interval_ms".to_string(),
            json!(clamped_interval_ms),
        );
        applied.insert(
            "runtime_heartbeat_interval_ms".to_string(),
            json!({
                "mode": "hot_reload",
                "value": clamped_interval_ms,
                "policy_path": policy_path.display().to_string(),
            }),
        );
    }

    if accepted.is_empty() {
        return OpenResponsesApiError::bad_request(
            "no_config_changes",
            "patch payload did not include any supported config fields",
        )
        .into_response();
    }

    let overrides_path = gateway_config_overrides_path(&state.config.state_dir);
    let updated_unix_ms = current_unix_timestamp_ms();
    let overrides_payload = json!({
        "schema_version": 1,
        "updated_unix_ms": updated_unix_ms,
        "pending_overrides": pending_overrides,
    });
    if let Some(parent) = overrides_path.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(error) = std::fs::create_dir_all(parent) {
                return OpenResponsesApiError::internal(format!(
                    "failed to create config override directory '{}': {error}",
                    parent.display()
                ))
                .into_response();
            }
        }
    }
    if let Err(error) = std::fs::write(&overrides_path, format!("{overrides_payload}\n").as_bytes())
    {
        return OpenResponsesApiError::internal(format!(
            "failed to write config overrides '{}': {error}",
            overrides_path.display()
        ))
        .into_response();
    }

    state.record_ui_telemetry_event("configuration", "config_patch", "config_patch_applied");
    (
        StatusCode::OK,
        Json(json!({
            "accepted": accepted,
            "applied": applied,
            "restart_required_fields": restart_required_fields.into_iter().collect::<Vec<_>>(),
            "pending_overrides": overrides_payload["pending_overrides"],
            "overrides_path": overrides_path.display().to_string(),
            "updated_unix_ms": updated_unix_ms,
        })),
    )
        .into_response()
}

fn gateway_config_overrides_path(state_dir: &Path) -> PathBuf {
    state_dir
        .join("openresponses")
        .join("config-overrides.json")
}

fn gateway_runtime_heartbeat_policy_path(state_path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.policy.toml", state_path.display()))
}

pub(super) fn read_gateway_config_pending_overrides(
    path: &Path,
) -> Result<serde_json::Map<String, Value>, OpenResponsesApiError> {
    if !path.exists() {
        return Ok(serde_json::Map::new());
    }
    let raw = std::fs::read_to_string(path).map_err(|error| {
        OpenResponsesApiError::internal(format!(
            "failed to read config overrides '{}': {error}",
            path.display()
        ))
    })?;
    let parsed = serde_json::from_str::<Value>(raw.as_str()).map_err(|error| {
        OpenResponsesApiError::internal(format!(
            "failed to parse config overrides '{}': {error}",
            path.display()
        ))
    })?;
    if let Some(overrides) = parsed.get("pending_overrides").and_then(Value::as_object) {
        return Ok(overrides.clone());
    }
    if let Some(overrides) = parsed.as_object() {
        return Ok(overrides.clone());
    }
    Ok(serde_json::Map::new())
}
