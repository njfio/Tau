use std::sync::Arc;

use axum::extract::{Form, Path as AxumPath, State};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use tau_dashboard_ui::{TauOpsDashboardSidebarState, TauOpsDashboardTheme};

use super::{
    deploy_gateway_agent, sanitize_session_key, stop_gateway_deploy_agent, GatewayDeployAgentInput,
    GatewayOpenResponsesServerState, DEFAULT_SESSION_KEY, OPS_DASHBOARD_DEPLOY_ENDPOINT,
};

#[derive(Debug, Deserialize, Default)]
pub(super) struct OpsDashboardDeployForm {
    #[serde(default)]
    agent_id: String,
    #[serde(default)]
    profile: String,
    #[serde(default)]
    model: String,
    #[serde(default)]
    theme: String,
    #[serde(default)]
    sidebar: String,
    #[serde(default)]
    session: String,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct OpsDashboardDeployStopForm {
    #[serde(default)]
    theme: String,
    #[serde(default)]
    sidebar: String,
    #[serde(default)]
    session: String,
}

fn resolve_deploy_theme(theme: &str) -> TauOpsDashboardTheme {
    match theme.trim() {
        "light" => TauOpsDashboardTheme::Light,
        _ => TauOpsDashboardTheme::Dark,
    }
}

fn resolve_deploy_sidebar_state(sidebar: &str) -> TauOpsDashboardSidebarState {
    match sidebar.trim() {
        "collapsed" => TauOpsDashboardSidebarState::Collapsed,
        _ => TauOpsDashboardSidebarState::Expanded,
    }
}

impl OpsDashboardDeployForm {
    fn resolved_theme(&self) -> TauOpsDashboardTheme {
        resolve_deploy_theme(self.theme.as_str())
    }

    fn resolved_sidebar_state(&self) -> TauOpsDashboardSidebarState {
        resolve_deploy_sidebar_state(self.sidebar.as_str())
    }

    fn resolved_session_key(&self) -> String {
        let session = self.session.trim();
        if session.is_empty() {
            DEFAULT_SESSION_KEY.to_string()
        } else {
            sanitize_session_key(session)
        }
    }
}

impl OpsDashboardDeployStopForm {
    fn resolved_theme(&self) -> TauOpsDashboardTheme {
        resolve_deploy_theme(self.theme.as_str())
    }

    fn resolved_sidebar_state(&self) -> TauOpsDashboardSidebarState {
        resolve_deploy_sidebar_state(self.sidebar.as_str())
    }

    fn resolved_session_key(&self) -> String {
        let session = self.session.trim();
        if session.is_empty() {
            DEFAULT_SESSION_KEY.to_string()
        } else {
            sanitize_session_key(session)
        }
    }
}

fn normalize_ops_deploy_action_status_marker(status: &str) -> &'static str {
    match status {
        "deployed" => "deployed",
        "stopped" => "stopped",
        "missing" => "missing",
        "failed" => "failed",
        _ => "idle",
    }
}

fn sanitize_ops_deploy_marker(raw: &str) -> String {
    let sanitized = raw
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "none".to_string()
    } else {
        sanitized
    }
}

fn build_ops_deploy_redirect_path(
    theme: TauOpsDashboardTheme,
    sidebar_state: TauOpsDashboardSidebarState,
    session_key: &str,
    deploy_action_status: &str,
    deploy_agent_id: &str,
    deploy_action_reason: &str,
) -> String {
    let session_key = sanitize_session_key(session_key);
    let status = normalize_ops_deploy_action_status_marker(deploy_action_status);
    let agent_id = sanitize_ops_deploy_marker(deploy_agent_id);
    let reason = sanitize_ops_deploy_marker(deploy_action_reason);
    format!(
        "{OPS_DASHBOARD_DEPLOY_ENDPOINT}?theme={}&sidebar={}&session={session_key}&deploy_action_status={status}&deploy_agent_id={agent_id}&deploy_action_reason={reason}",
        theme.as_str(),
        sidebar_state.as_str()
    )
}

pub(super) async fn handle_ops_dashboard_deploy_submit(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    Form(form): Form<OpsDashboardDeployForm>,
) -> Response {
    let redirect_theme = form.resolved_theme();
    let redirect_sidebar_state = form.resolved_sidebar_state();
    let redirect_session_key = form.resolved_session_key();
    let requested_agent_id = form.agent_id.trim().to_string();
    if requested_agent_id.is_empty() {
        state.record_ui_telemetry_event("deploy", "submit", "deploy_form_missing_agent_id");
        let redirect_path = build_ops_deploy_redirect_path(
            redirect_theme,
            redirect_sidebar_state,
            redirect_session_key.as_str(),
            "missing",
            "none",
            "invalid_agent_id",
        );
        return Redirect::to(redirect_path.as_str()).into_response();
    }

    match deploy_gateway_agent(
        &state,
        GatewayDeployAgentInput {
            agent_id: form.agent_id,
            profile: form.profile,
            model: form.model,
        },
    ) {
        Ok(result) => {
            state.record_ui_telemetry_event("deploy", "submit", "deploy_process_started");
            let redirect_path = build_ops_deploy_redirect_path(
                redirect_theme,
                redirect_sidebar_state,
                redirect_session_key.as_str(),
                "deployed",
                result.agent_id.as_str(),
                "process_started",
            );
            Redirect::to(redirect_path.as_str()).into_response()
        }
        Err(error) => {
            state.record_ui_telemetry_event("deploy", "submit", "deploy_process_start_failed");
            let redirect_path = build_ops_deploy_redirect_path(
                redirect_theme,
                redirect_sidebar_state,
                redirect_session_key.as_str(),
                "failed",
                requested_agent_id.as_str(),
                error.code,
            );
            Redirect::to(redirect_path.as_str()).into_response()
        }
    }
}

pub(super) async fn handle_ops_dashboard_deploy_stop(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    AxumPath(agent_id): AxumPath<String>,
    Form(form): Form<OpsDashboardDeployStopForm>,
) -> Response {
    let redirect_theme = form.resolved_theme();
    let redirect_sidebar_state = form.resolved_sidebar_state();
    let redirect_session_key = form.resolved_session_key();
    let requested_agent_id = agent_id.trim().to_string();
    if requested_agent_id.is_empty() {
        state.record_ui_telemetry_event("deploy", "stop", "deploy_stop_form_missing_agent_id");
        let redirect_path = build_ops_deploy_redirect_path(
            redirect_theme,
            redirect_sidebar_state,
            redirect_session_key.as_str(),
            "missing",
            "none",
            "invalid_agent_id",
        );
        return Redirect::to(redirect_path.as_str()).into_response();
    }

    match stop_gateway_deploy_agent(&state, requested_agent_id.as_str(), "operator_stop_request") {
        Ok(result) => {
            state.record_ui_telemetry_event("deploy", "stop", "deploy_process_stopped");
            let redirect_path = build_ops_deploy_redirect_path(
                redirect_theme,
                redirect_sidebar_state,
                redirect_session_key.as_str(),
                "stopped",
                result.agent_id.as_str(),
                result.process_stop_reason.as_str(),
            );
            Redirect::to(redirect_path.as_str()).into_response()
        }
        Err(error) => {
            state.record_ui_telemetry_event("deploy", "stop", "deploy_process_stop_failed");
            let redirect_path = build_ops_deploy_redirect_path(
                redirect_theme,
                redirect_sidebar_state,
                redirect_session_key.as_str(),
                "failed",
                requested_agent_id.as_str(),
                error.code,
            );
            Redirect::to(redirect_path.as_str()).into_response()
        }
    }
}
