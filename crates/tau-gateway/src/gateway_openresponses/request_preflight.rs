//! Shared request preflight helpers for gateway routes.

use super::*;

pub(super) fn authorize_and_enforce_gateway_limits(
    state: &Arc<GatewayOpenResponsesServerState>,
    headers: &HeaderMap,
) -> Result<String, OpenResponsesApiError> {
    let principal = authorize_gateway_request(state, headers)?;
    enforce_gateway_rate_limit(state, principal.as_str())?;
    Ok(principal)
}

pub(super) fn validate_gateway_request_body_size(
    state: &Arc<GatewayOpenResponsesServerState>,
    body: &Bytes,
) -> Result<(), OpenResponsesApiError> {
    let body_limit = state
        .config
        .max_input_chars
        .saturating_mul(INPUT_BODY_SIZE_MULTIPLIER)
        .max(state.config.max_input_chars);
    if body.len() > body_limit {
        return Err(OpenResponsesApiError::payload_too_large(format!(
            "request body exceeds max size of {} bytes",
            body_limit
        )));
    }
    Ok(())
}

pub(super) fn parse_gateway_json_body<T: DeserializeOwned>(
    body: &Bytes,
) -> Result<T, OpenResponsesApiError> {
    serde_json::from_slice::<T>(body).map_err(|error| {
        OpenResponsesApiError::bad_request(
            "malformed_json",
            format!("failed to parse request body: {error}"),
        )
    })
}

pub(super) fn enforce_policy_gate(
    provided: Option<&str>,
    required: &'static str,
) -> Result<(), OpenResponsesApiError> {
    let Some(gate) = provided.map(str::trim).filter(|value| !value.is_empty()) else {
        return Err(OpenResponsesApiError::forbidden(
            "policy_gate_required",
            format!("set policy_gate='{required}' to perform this operation"),
        ));
    };
    if gate != required {
        return Err(OpenResponsesApiError::forbidden(
            "policy_gate_mismatch",
            format!("policy_gate must equal '{required}'"),
        ));
    }
    Ok(())
}

pub(super) fn system_time_to_unix_ms(time: std::time::SystemTime) -> Option<u64> {
    let duration = time.duration_since(std::time::UNIX_EPOCH).ok()?;
    u64::try_from(duration.as_millis()).ok()
}
