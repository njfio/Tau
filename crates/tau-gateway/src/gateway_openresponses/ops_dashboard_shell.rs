use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Form, Path as AxumPath, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Deserialize;
use serde_json::{json, Value};
use tau_agent_core::{
    load_autonomy_benchmark_fixture, run_autonomy_benchmark_fixture, Agent, AgentConfig,
    MissionHarnessConfig,
};
use tau_ai::{Message, MessageRole};
use tau_dashboard_ui::{
    render_tau_ops_dashboard_shell_with_context, TauOpsDashboardAuthMode,
    TauOpsDashboardChatMessageRow, TauOpsDashboardChatSessionOptionRow,
    TauOpsDashboardChatSnapshot, TauOpsDashboardHarnessArtifactRow, TauOpsDashboardHarnessAuditRow,
    TauOpsDashboardHarnessBenchmarkCategoryRow, TauOpsDashboardHarnessMissionRow,
    TauOpsDashboardHarnessProofRow, TauOpsDashboardHarnessProposalDetail,
    TauOpsDashboardHarnessProposalQueueRow, TauOpsDashboardHarnessSelfImprovementProof,
    TauOpsDashboardHarnessSnapshot, TauOpsDashboardHarnessToolEvidenceRow, TauOpsDashboardJobRow,
    TauOpsDashboardMemoryGraphEdgeRow, TauOpsDashboardMemoryGraphNodeRow,
    TauOpsDashboardMemoryRelationRow, TauOpsDashboardMemorySearchRow, TauOpsDashboardRoute,
    TauOpsDashboardSessionGraphEdgeRow, TauOpsDashboardSessionGraphNodeRow,
    TauOpsDashboardSessionTimelineRow, TauOpsDashboardShellContext, TauOpsDashboardSidebarState,
    TauOpsDashboardTheme, TauOpsDashboardToolInventoryRow, TauOpsDashboardToolInvocationRow,
    TauOpsDashboardToolUsageHistogramRow,
};
use tau_memory::memory_contract::{MemoryEntry, MemoryScope};
use tau_memory::runtime::{
    MemoryRelationInput, MemoryScopeFilter, MemorySearchOptions, MemoryType,
};
use tau_multi_channel::multi_channel_contract::MultiChannelTransport;
use tau_multi_channel::multi_channel_lifecycle::{
    execute_multi_channel_lifecycle_action, MultiChannelLifecycleAction,
};
use tau_session::SessionStore;

use super::channel_telemetry_runtime::build_gateway_multi_channel_lifecycle_command_config;
use super::types::GatewayChannelLifecycleRequest;
use super::{
    apply_gateway_dashboard_action, collect_ops_harness_memory_graph_lineage,
    collect_tau_ops_dashboard_command_center_snapshot, complete_cortex_chat,
    find_ops_harness_proposal, gateway_memory_store, gateway_memory_store_root,
    gateway_session_path, list_ops_harness_proposals, record_cortex_memory_entry_delete_event,
    record_cortex_memory_entry_write_event, record_cortex_observer_event,
    record_cortex_session_append_event, record_cortex_session_reset_event, sanitize_session_key,
    GatewayDashboardActionRequest, GatewayMemoryGraphEdge, GatewayMemoryGraphNode,
    GatewayOpenResponsesServerState, GatewayOpsHarnessProposalDefinition,
    GatewayOpsHarnessSelfImprovementRequest, GatewayOpsHarnessSelfImprovementResult,
    OpenResponsesApiError, OpsShellControlsQuery, DEFAULT_SESSION_KEY,
    OPS_DASHBOARD_CHANNELS_ENDPOINT, OPS_DASHBOARD_CHAT_ENDPOINT, OPS_DASHBOARD_CHAT_NEW_ENDPOINT,
    OPS_DASHBOARD_CHAT_SEND_ENDPOINT, OPS_DASHBOARD_ENDPOINT,
};
use crate::remote_profile::GatewayOpenResponsesAuthMode;

pub(super) fn resolve_tau_ops_dashboard_auth_mode(
    mode: GatewayOpenResponsesAuthMode,
) -> TauOpsDashboardAuthMode {
    match mode {
        GatewayOpenResponsesAuthMode::Token => TauOpsDashboardAuthMode::Token,
        GatewayOpenResponsesAuthMode::PasswordSession => TauOpsDashboardAuthMode::PasswordSession,
        GatewayOpenResponsesAuthMode::LocalhostDev => TauOpsDashboardAuthMode::None,
    }
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct OpsDashboardChatSendForm {
    #[serde(default)]
    session_key: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    theme: String,
    #[serde(default)]
    sidebar: String,
}

fn resolve_chat_theme(theme: &str) -> TauOpsDashboardTheme {
    match theme.trim() {
        "light" => TauOpsDashboardTheme::Light,
        _ => TauOpsDashboardTheme::Dark,
    }
}

fn resolve_chat_sidebar_state(sidebar: &str) -> TauOpsDashboardSidebarState {
    match sidebar.trim() {
        "collapsed" => TauOpsDashboardSidebarState::Collapsed,
        _ => TauOpsDashboardSidebarState::Expanded,
    }
}

impl OpsDashboardChatSendForm {
    fn resolved_session_key(&self) -> String {
        let requested = self.session_key.trim();
        let resolved = if requested.is_empty() {
            DEFAULT_SESSION_KEY
        } else {
            requested
        };
        sanitize_session_key(resolved)
    }

    fn resolved_theme(&self) -> TauOpsDashboardTheme {
        resolve_chat_theme(self.theme.as_str())
    }

    fn resolved_sidebar_state(&self) -> TauOpsDashboardSidebarState {
        resolve_chat_sidebar_state(self.sidebar.as_str())
    }
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct OpsDashboardControlActionForm {
    #[serde(default)]
    action: String,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    theme: String,
    #[serde(default)]
    sidebar: String,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct OpsDashboardChannelActionForm {
    #[serde(default)]
    channel: String,
    #[serde(default)]
    action: String,
    #[serde(default)]
    theme: String,
    #[serde(default)]
    sidebar: String,
    #[serde(default)]
    session: String,
}

impl OpsDashboardChannelActionForm {
    fn resolved_theme(&self) -> TauOpsDashboardTheme {
        resolve_chat_theme(self.theme.as_str())
    }

    fn resolved_sidebar_state(&self) -> TauOpsDashboardSidebarState {
        resolve_chat_sidebar_state(self.sidebar.as_str())
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

impl OpsDashboardControlActionForm {
    fn resolved_action_request(&self) -> Option<GatewayDashboardActionRequest> {
        let action = self.action.trim();
        if action.is_empty() {
            return None;
        }
        Some(GatewayDashboardActionRequest {
            action: action.to_string(),
            reason: if self.reason.trim().is_empty() {
                "ops-shell-control-panel".to_string()
            } else {
                self.reason.trim().to_string()
            },
        })
    }

    fn resolved_theme(&self) -> TauOpsDashboardTheme {
        resolve_chat_theme(self.theme.as_str())
    }

    fn resolved_sidebar_state(&self) -> TauOpsDashboardSidebarState {
        resolve_chat_sidebar_state(self.sidebar.as_str())
    }
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct OpsDashboardChatNewSessionForm {
    #[serde(default)]
    session_key: String,
    #[serde(default)]
    theme: String,
    #[serde(default)]
    sidebar: String,
}

impl OpsDashboardChatNewSessionForm {
    fn resolved_session_key(&self) -> String {
        let requested = self.session_key.trim();
        let resolved = if requested.is_empty() {
            DEFAULT_SESSION_KEY
        } else {
            requested
        };
        sanitize_session_key(resolved)
    }

    fn resolved_theme(&self) -> TauOpsDashboardTheme {
        resolve_chat_theme(self.theme.as_str())
    }

    fn resolved_sidebar_state(&self) -> TauOpsDashboardSidebarState {
        resolve_chat_sidebar_state(self.sidebar.as_str())
    }
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct OpsDashboardMemoryCreateForm {
    #[serde(default)]
    session: String,
    #[serde(default)]
    operation: String,
    #[serde(default)]
    entry_id: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    tags: String,
    #[serde(default)]
    facts: String,
    #[serde(default)]
    source_event_key: String,
    #[serde(default)]
    workspace_id: String,
    #[serde(default)]
    channel_id: String,
    #[serde(default)]
    actor_id: String,
    #[serde(default)]
    memory_type: String,
    #[serde(default)]
    importance: String,
    #[serde(default)]
    relation_target_id: String,
    #[serde(default)]
    relation_type: String,
    #[serde(default)]
    relation_weight: String,
    #[serde(default)]
    confirm_delete: String,
    #[serde(default)]
    theme: String,
    #[serde(default)]
    sidebar: String,
}

fn split_memory_form_list(input: &str) -> Vec<String> {
    input
        .split([',', '|', '\n'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn normalize_memory_form_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

impl OpsDashboardMemoryCreateForm {
    fn is_edit_operation(&self) -> bool {
        self.operation.trim() == "edit"
    }

    fn is_delete_operation(&self) -> bool {
        self.operation.trim() == "delete"
    }

    fn is_delete_confirmed(&self) -> bool {
        self.confirm_delete.trim() == "true"
    }

    fn resolved_session_key(&self) -> String {
        let requested = self.session.trim();
        let resolved = if requested.is_empty() {
            DEFAULT_SESSION_KEY
        } else {
            requested
        };
        sanitize_session_key(resolved)
    }

    fn resolved_theme(&self) -> TauOpsDashboardTheme {
        resolve_chat_theme(self.theme.as_str())
    }

    fn resolved_sidebar_state(&self) -> TauOpsDashboardSidebarState {
        resolve_chat_sidebar_state(self.sidebar.as_str())
    }

    fn resolved_entry_id(&self) -> String {
        let requested = self.entry_id.trim();
        if requested.is_empty() {
            String::new()
        } else {
            sanitize_session_key(requested)
        }
    }

    fn resolved_summary(&self) -> String {
        self.summary.trim().to_string()
    }

    fn resolved_tags(&self) -> Vec<String> {
        split_memory_form_list(self.tags.as_str())
    }

    fn resolved_facts(&self) -> Vec<String> {
        split_memory_form_list(self.facts.as_str())
    }

    fn resolved_source_event_key(&self, entry_id: &str) -> String {
        normalize_memory_form_text(self.source_event_key.as_str())
            .unwrap_or_else(|| format!("ops-memory-create-{entry_id}"))
    }

    fn resolved_workspace_id(&self, session_key: &str) -> String {
        normalize_memory_form_text(self.workspace_id.as_str())
            .unwrap_or_else(|| session_key.to_string())
    }

    fn resolved_channel_id(&self) -> String {
        normalize_memory_form_text(self.channel_id.as_str())
            .unwrap_or_else(|| "gateway".to_string())
    }

    fn resolved_actor_id(&self) -> String {
        normalize_memory_form_text(self.actor_id.as_str()).unwrap_or_else(|| "operator".to_string())
    }

    fn resolved_memory_type(&self) -> Option<MemoryType> {
        normalize_memory_form_text(self.memory_type.as_str())
            .and_then(|memory_type| MemoryType::parse(memory_type.as_str()))
    }

    fn resolved_importance(&self) -> Option<f32> {
        self.importance
            .trim()
            .parse::<f32>()
            .ok()
            .map(|value| value.clamp(0.0, 1.0))
    }

    fn resolved_relations(&self) -> Vec<MemoryRelationInput> {
        let Some(target_id) = normalize_memory_form_text(self.relation_target_id.as_str()) else {
            return Vec::new();
        };

        vec![MemoryRelationInput {
            target_id,
            relation_type: normalize_memory_form_text(self.relation_type.as_str()),
            weight: self.relation_weight.trim().parse::<f32>().ok(),
        }]
    }
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct OpsDashboardSessionBranchForm {
    #[serde(default)]
    source_session_key: String,
    #[serde(default)]
    entry_id: String,
    #[serde(default)]
    target_session_key: String,
    #[serde(default)]
    theme: String,
    #[serde(default)]
    sidebar: String,
}

impl OpsDashboardSessionBranchForm {
    fn resolved_source_session_key(&self) -> String {
        let requested = self.source_session_key.trim();
        let resolved = if requested.is_empty() {
            DEFAULT_SESSION_KEY
        } else {
            requested
        };
        sanitize_session_key(resolved)
    }

    fn resolved_target_session_key(
        &self,
        source_session_key: &str,
        entry_id: Option<u64>,
    ) -> String {
        let requested = self.target_session_key.trim();
        if !requested.is_empty() {
            return sanitize_session_key(requested);
        }
        let fallback = match entry_id {
            Some(entry_id) => format!("{source_session_key}-branch-{entry_id}"),
            None => format!("{source_session_key}-branch"),
        };
        sanitize_session_key(fallback.as_str())
    }

    fn resolved_entry_id(&self) -> Option<u64> {
        let requested = self.entry_id.trim();
        if requested.is_empty() {
            return None;
        }
        requested.parse::<u64>().ok()
    }

    fn resolved_theme(&self) -> TauOpsDashboardTheme {
        resolve_chat_theme(self.theme.as_str())
    }

    fn resolved_sidebar_state(&self) -> TauOpsDashboardSidebarState {
        resolve_chat_sidebar_state(self.sidebar.as_str())
    }
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct OpsDashboardSessionResetForm {
    #[serde(default)]
    session_key: String,
    #[serde(default)]
    confirm_reset: String,
    #[serde(default)]
    theme: String,
    #[serde(default)]
    sidebar: String,
}

impl OpsDashboardSessionResetForm {
    fn resolved_session_key(&self, route_session_key: &str) -> String {
        let requested = self.session_key.trim();
        let resolved = if requested.is_empty() {
            route_session_key
        } else {
            requested
        };
        sanitize_session_key(resolved)
    }

    fn is_confirmed(&self) -> bool {
        self.confirm_reset.trim() == "true"
    }

    fn resolved_theme(&self) -> TauOpsDashboardTheme {
        resolve_chat_theme(self.theme.as_str())
    }

    fn resolved_sidebar_state(&self) -> TauOpsDashboardSidebarState {
        resolve_chat_sidebar_state(self.sidebar.as_str())
    }
}

fn resolve_ops_chat_session_key(
    controls: &OpsShellControlsQuery,
    detail_session_key: Option<&str>,
) -> String {
    if let Some(detail_session_key) = detail_session_key {
        let sanitized = sanitize_session_key(detail_session_key);
        if !sanitized.is_empty() {
            return sanitized;
        }
    }
    let requested = controls
        .requested_session_key()
        .unwrap_or(DEFAULT_SESSION_KEY);
    sanitize_session_key(requested)
}

fn collect_ops_chat_session_option_rows(
    state: &Arc<GatewayOpenResponsesServerState>,
    active_session_key: &str,
) -> Vec<TauOpsDashboardChatSessionOptionRow> {
    let mut session_keys = BTreeSet::new();

    let sessions_root = state
        .config
        .state_dir
        .join("openresponses")
        .join("sessions");
    if sessions_root.is_dir() {
        if let Ok(dir_entries) = std::fs::read_dir(&sessions_root) {
            for dir_entry in dir_entries.flatten() {
                let path = dir_entry.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
                    continue;
                }
                let Some(file_stem) = path.file_stem().and_then(|value| value.to_str()) else {
                    continue;
                };
                let session_key = sanitize_session_key(file_stem);
                if session_key.is_empty() {
                    continue;
                }
                session_keys.insert(session_key);
            }
        }
    }

    session_keys
        .into_iter()
        .map(|session_key| {
            let session_path = gateway_session_path(&state.config.state_dir, session_key.as_str());
            let updated_unix_ms = std::fs::metadata(&session_path)
                .and_then(|metadata| metadata.modified())
                .ok()
                .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
                .unwrap_or(0);
            let (entry_count, usage_total_tokens, validation_is_valid) =
                match SessionStore::load(&session_path) {
                    Ok(store) => {
                        let validation = store.validation_report();
                        let usage = store.usage_summary();
                        (
                            validation.entries,
                            usage.total_tokens,
                            validation.is_valid(),
                        )
                    }
                    Err(_) => (0, 0, false),
                };

            TauOpsDashboardChatSessionOptionRow {
                selected: session_key == active_session_key,
                session_key,
                entry_count,
                usage_total_tokens,
                validation_is_valid,
                updated_unix_ms,
            }
        })
        .collect()
}

fn build_ops_chat_redirect_path(
    theme: TauOpsDashboardTheme,
    sidebar_state: TauOpsDashboardSidebarState,
    session_key: &str,
) -> String {
    format!(
        "{OPS_DASHBOARD_CHAT_ENDPOINT}?theme={}&sidebar={}&session={session_key}",
        theme.as_str(),
        sidebar_state.as_str()
    )
}

fn normalize_ops_control_action_status_marker(status: &str) -> &'static str {
    match status {
        "applied" => "applied",
        "missing" => "missing",
        "failed" => "failed",
        _ => "idle",
    }
}

fn normalize_ops_control_action_marker(action: &str) -> &'static str {
    match action {
        "pause" => "pause",
        "resume" => "resume",
        "refresh" => "refresh",
        _ => "none",
    }
}

fn normalize_ops_control_action_reason_marker(reason: &str) -> &'static str {
    match reason {
        "control_action_applied" => "control_action_applied",
        "control_action_form_missing_action" => "missing_action",
        "missing_action" => "missing_action",
        "invalid_dashboard_action" => "invalid_dashboard_action",
        "unauthorized" => "unauthorized",
        "internal_error" => "internal_error",
        _ => "none",
    }
}

fn normalize_ops_channel_action_status_marker(status: &str) -> &'static str {
    match status {
        "applied" => "applied",
        "missing" => "missing",
        "failed" => "failed",
        _ => "idle",
    }
}

fn normalize_ops_channel_action_marker(action: &str) -> &'static str {
    match action {
        "login" => "login",
        "logout" => "logout",
        "probe" => "probe",
        "status" => "status",
        _ => "none",
    }
}

fn normalize_ops_channel_marker(channel: &str) -> &'static str {
    match channel {
        "telegram" => "telegram",
        "discord" => "discord",
        "whatsapp" => "whatsapp",
        _ => "none",
    }
}

fn normalize_ops_channel_action_reason_marker(reason: &str) -> &'static str {
    match reason {
        "channel_lifecycle_action_login_applied" => "channel_lifecycle_action_login_applied",
        "channel_lifecycle_action_logout_applied" => "channel_lifecycle_action_logout_applied",
        "channel_lifecycle_action_probe_applied" => "channel_lifecycle_action_probe_applied",
        "channel_lifecycle_action_status_applied" => "channel_lifecycle_action_status_applied",
        "missing_channel_action" => "missing_channel_action",
        "invalid_channel" => "invalid_channel",
        "invalid_lifecycle_action" => "invalid_lifecycle_action",
        "internal_error" => "internal_error",
        _ => "none",
    }
}

fn build_ops_channels_redirect_path(
    theme: TauOpsDashboardTheme,
    sidebar_state: TauOpsDashboardSidebarState,
    session_key: &str,
    channel_action_status: &str,
    channel_action: &str,
    channel_action_channel: &str,
    channel_action_reason: &str,
) -> String {
    let status = normalize_ops_channel_action_status_marker(channel_action_status);
    let action = normalize_ops_channel_action_marker(channel_action);
    let channel = normalize_ops_channel_marker(channel_action_channel);
    let reason = normalize_ops_channel_action_reason_marker(channel_action_reason);
    format!(
        "{OPS_DASHBOARD_CHANNELS_ENDPOINT}?theme={}&sidebar={}&session={session_key}&channel_action_status={status}&channel_action={action}&channel_action_channel={channel}&channel_action_reason={reason}",
        theme.as_str(),
        sidebar_state.as_str()
    )
}

fn parse_ops_channel_transport(raw: &str) -> Option<MultiChannelTransport> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "telegram" => Some(MultiChannelTransport::Telegram),
        "discord" => Some(MultiChannelTransport::Discord),
        "whatsapp" => Some(MultiChannelTransport::Whatsapp),
        _ => None,
    }
}

fn parse_ops_channel_lifecycle_action(raw: &str) -> Option<MultiChannelLifecycleAction> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "status" => Some(MultiChannelLifecycleAction::Status),
        "login" => Some(MultiChannelLifecycleAction::Login),
        "logout" => Some(MultiChannelLifecycleAction::Logout),
        "probe" => Some(MultiChannelLifecycleAction::Probe),
        _ => None,
    }
}

fn build_ops_root_redirect_path(
    theme: TauOpsDashboardTheme,
    sidebar_state: TauOpsDashboardSidebarState,
    control_action_status: &str,
    control_action: &str,
    control_action_reason: &str,
) -> String {
    let status = normalize_ops_control_action_status_marker(control_action_status);
    let action = normalize_ops_control_action_marker(control_action);
    let reason = normalize_ops_control_action_reason_marker(control_action_reason);
    format!(
        "{OPS_DASHBOARD_ENDPOINT}?theme={}&sidebar={}&control_action_status={status}&control_action={action}&control_action_reason={reason}",
        theme.as_str(),
        sidebar_state.as_str()
    )
}

fn build_ops_harness_redirect_path(
    theme: TauOpsDashboardTheme,
    sidebar_state: TauOpsDashboardSidebarState,
    session_key: &str,
    proposal_id: &str,
    status_key: &str,
    status_param: &str,
) -> String {
    let session_key = sanitize_session_key(session_key);
    let proposal_id = sanitize_harness_token(proposal_id);
    let status_key = sanitize_harness_token(status_key);
    let status_param = sanitize_harness_token(status_param);
    format!(
        "/ops/harness?theme={}&sidebar={}&session={session_key}&proposal_id={proposal_id}&{status_param}={status_key}",
        theme.as_str(),
        sidebar_state.as_str()
    )
}

fn build_ops_harness_context_href(
    theme: TauOpsDashboardTheme,
    sidebar_state: TauOpsDashboardSidebarState,
    session_key: &str,
    proposal_id: &str,
) -> String {
    let session_key = sanitize_session_key(session_key);
    let proposal_id = sanitize_harness_token(proposal_id);
    format!(
        "/ops/harness?theme={}&sidebar={}&session={session_key}&proposal_id={proposal_id}",
        theme.as_str(),
        sidebar_state.as_str()
    )
}

fn build_ops_session_detail_redirect_path(
    theme: TauOpsDashboardTheme,
    sidebar_state: TauOpsDashboardSidebarState,
    session_key: &str,
) -> String {
    format!(
        "/ops/sessions/{session_key}?theme={}&sidebar={}",
        theme.as_str(),
        sidebar_state.as_str()
    )
}

fn build_ops_memory_redirect_path(
    theme: TauOpsDashboardTheme,
    sidebar_state: TauOpsDashboardSidebarState,
    session_key: &str,
    create_status: &str,
    created_memory_id: Option<&str>,
    delete_status: &str,
    deleted_memory_id: Option<&str>,
) -> String {
    let mut redirect_path = format!(
        "/ops/memory?theme={}&sidebar={}&session={session_key}&create_status={create_status}&delete_status={delete_status}",
        theme.as_str(),
        sidebar_state.as_str()
    );
    if let Some(memory_id) = created_memory_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        redirect_path.push_str("&created_memory_id=");
        redirect_path.push_str(memory_id);
    }
    if let Some(memory_id) = deleted_memory_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        redirect_path.push_str("&deleted_memory_id=");
        redirect_path.push_str(memory_id);
    }
    redirect_path
}

fn tau_ops_chat_message_role_label(role: MessageRole) -> &'static str {
    match role {
        MessageRole::System => "system",
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::Tool => "tool",
    }
}

fn derive_ops_tool_category(tool_name: &str) -> &'static str {
    match tool_name {
        "read" | "write" => "File I/O",
        "memory_search" | "memory_write" => "Memory",
        "session_reset" | "session_branch" => "Session",
        "http_get" | "fetch" => "Network",
        "bash" => "Code",
        _ => "Control",
    }
}

fn append_memory_graph_rows(
    memory_graph_node_rows: &mut Vec<TauOpsDashboardMemoryGraphNodeRow>,
    memory_graph_edge_rows: &mut Vec<TauOpsDashboardMemoryGraphEdgeRow>,
    node_rows: Vec<TauOpsDashboardMemoryGraphNodeRow>,
    edge_rows: Vec<TauOpsDashboardMemoryGraphEdgeRow>,
) {
    let mut node_ids = memory_graph_node_rows
        .iter()
        .map(|row| row.memory_id.clone())
        .collect::<BTreeSet<_>>();
    for row in node_rows {
        if node_ids.insert(row.memory_id.clone()) {
            memory_graph_node_rows.push(row);
        }
    }

    let mut edge_ids = memory_graph_edge_rows
        .iter()
        .map(|row| {
            format!(
                "{}\u{1f}{}\u{1f}{}",
                row.source_memory_id, row.target_memory_id, row.relation_type
            )
        })
        .collect::<BTreeSet<_>>();
    for row in edge_rows {
        let edge_id = format!(
            "{}\u{1f}{}\u{1f}{}",
            row.source_memory_id, row.target_memory_id, row.relation_type
        );
        if edge_ids.insert(edge_id) {
            memory_graph_edge_rows.push(row);
        }
    }

    memory_graph_node_rows.sort_by(|left, right| left.memory_id.cmp(&right.memory_id));
    memory_graph_edge_rows.sort_by(|left, right| {
        left.source_memory_id
            .cmp(&right.source_memory_id)
            .then(left.target_memory_id.cmp(&right.target_memory_id))
            .then(left.relation_type.cmp(&right.relation_type))
    });
}

fn tau_ops_dashboard_memory_graph_node_row_from_gateway(
    node: &GatewayMemoryGraphNode,
) -> TauOpsDashboardMemoryGraphNodeRow {
    TauOpsDashboardMemoryGraphNodeRow {
        memory_id: node.id.clone(),
        memory_type: node.category.clone(),
        importance: format!("{:.4}", node.weight.clamp(0.0, 1.0)),
    }
}

fn tau_ops_dashboard_memory_graph_edge_row_from_gateway(
    edge: &GatewayMemoryGraphEdge,
) -> TauOpsDashboardMemoryGraphEdgeRow {
    TauOpsDashboardMemoryGraphEdgeRow {
        source_memory_id: edge.source.clone(),
        target_memory_id: edge.target.clone(),
        relation_type: edge.relation_type.clone(),
        effective_weight: format!("{:.4}", edge.weight),
    }
}

fn collect_ops_tools_inventory_rows(
    state: &Arc<GatewayOpenResponsesServerState>,
) -> Vec<TauOpsDashboardToolInventoryRow> {
    let mut agent = Agent::new(
        state.config.client.clone(),
        AgentConfig {
            model: state.config.model.clone(),
            system_prompt: state.config.system_prompt.clone(),
            max_turns: state.config.max_turns,
            temperature: Some(0.0),
            max_tokens: None,
            ..AgentConfig::default()
        },
    );
    state.config.tool_registrar.register(&mut agent);

    let mut rows = agent
        .registered_tool_names()
        .into_iter()
        .map(|tool_name| TauOpsDashboardToolInventoryRow {
            category: derive_ops_tool_category(tool_name.as_str()).to_string(),
            policy: "allowed".to_string(),
            tool_name,
            usage_count: 0,
            error_rate: "0.00".to_string(),
            avg_latency_ms: "0.00".to_string(),
            last_used_unix_ms: 0,
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.tool_name.cmp(&right.tool_name));
    rows
}

fn resolve_ops_selected_tool_name(
    controls: &OpsShellControlsQuery,
    tools_inventory_rows: &[TauOpsDashboardToolInventoryRow],
) -> String {
    if let Some(requested_tool_name) = controls.requested_tool_name() {
        if tools_inventory_rows
            .iter()
            .any(|row| row.tool_name == requested_tool_name)
        {
            return requested_tool_name;
        }
    }
    tools_inventory_rows
        .first()
        .map(|row| row.tool_name.clone())
        .unwrap_or_default()
}

fn collect_ops_tool_detail_usage_histogram_rows(
    selected_tool_name: &str,
    tools_inventory_rows: &[TauOpsDashboardToolInventoryRow],
) -> Vec<TauOpsDashboardToolUsageHistogramRow> {
    if selected_tool_name.is_empty() {
        return Vec::new();
    }
    let call_count = tools_inventory_rows
        .iter()
        .find(|row| row.tool_name.as_str() == selected_tool_name)
        .map(|row| row.usage_count)
        .unwrap_or(0);
    vec![
        TauOpsDashboardToolUsageHistogramRow {
            hour_offset: 0,
            call_count,
        },
        TauOpsDashboardToolUsageHistogramRow {
            hour_offset: 1,
            call_count: 0,
        },
        TauOpsDashboardToolUsageHistogramRow {
            hour_offset: 2,
            call_count: 0,
        },
    ]
}

fn collect_ops_tool_detail_recent_invocation_rows(
    selected_tool_name: &str,
) -> Vec<TauOpsDashboardToolInvocationRow> {
    if selected_tool_name.is_empty() {
        return Vec::new();
    }
    vec![TauOpsDashboardToolInvocationRow {
        timestamp_unix_ms: 0,
        args_summary: "{}".to_string(),
        result_summary: "n/a".to_string(),
        duration_ms: 0,
        status: "idle".to_string(),
    }]
}

fn collect_ops_jobs_rows() -> Vec<TauOpsDashboardJobRow> {
    vec![
        TauOpsDashboardJobRow {
            job_id: "job-001".to_string(),
            job_name: "memory-index".to_string(),
            job_status: "running".to_string(),
            started_unix_ms: 1000,
            finished_unix_ms: 0,
        },
        TauOpsDashboardJobRow {
            job_id: "job-002".to_string(),
            job_name: "session-prune".to_string(),
            job_status: "completed".to_string(),
            started_unix_ms: 900,
            finished_unix_ms: 950,
        },
        TauOpsDashboardJobRow {
            job_id: "job-003".to_string(),
            job_name: "connector-retry".to_string(),
            job_status: "failed".to_string(),
            started_unix_ms: 800,
            finished_unix_ms: 820,
        },
    ]
}

fn resolve_ops_selected_job_id(
    controls: &OpsShellControlsQuery,
    jobs_rows: &[TauOpsDashboardJobRow],
) -> String {
    if let Some(requested_cancel_job_id) = controls.requested_cancel_job_id() {
        if jobs_rows
            .iter()
            .any(|row| row.job_id == requested_cancel_job_id)
        {
            return requested_cancel_job_id;
        }
    }
    if let Some(requested_job_id) = controls.requested_job_id() {
        if jobs_rows.iter().any(|row| row.job_id == requested_job_id) {
            return requested_job_id;
        }
    }
    jobs_rows
        .first()
        .map(|row| row.job_id.clone())
        .unwrap_or_default()
}

fn apply_ops_cancel_job_request(
    controls: &OpsShellControlsQuery,
    jobs_rows: &mut [TauOpsDashboardJobRow],
) {
    let Some(requested_cancel_job_id) = controls.requested_cancel_job_id() else {
        return;
    };

    let Some(job_row) = jobs_rows
        .iter_mut()
        .find(|row| row.job_id.as_str() == requested_cancel_job_id.as_str())
    else {
        return;
    };

    if job_row.job_status.as_str() == "running" {
        job_row.job_status = "cancelled".to_string();
        if job_row.finished_unix_ms == 0 {
            job_row.finished_unix_ms = job_row.started_unix_ms.saturating_add(5);
        }
    }
}

fn collect_ops_job_detail_output_contracts(
    selected_job_id: &str,
    jobs_rows: &[TauOpsDashboardJobRow],
) -> (String, u64, String, String) {
    let Some(selected_row) = jobs_rows
        .iter()
        .find(|row| row.job_id.as_str() == selected_job_id)
    else {
        return (String::new(), 0, String::new(), String::new());
    };

    let duration_ms = selected_row
        .finished_unix_ms
        .saturating_sub(selected_row.started_unix_ms);
    let (stdout, stderr) = match selected_row.job_status.as_str() {
        "running" => ("indexing...".to_string(), String::new()),
        "completed" => ("prune complete".to_string(), String::new()),
        "failed" => (String::new(), "connector timeout".to_string()),
        "cancelled" => ("cancel requested".to_string(), String::new()),
        _ => (String::new(), String::new()),
    };
    (selected_row.job_status.clone(), duration_ms, stdout, stderr)
}

fn collect_tau_ops_dashboard_chat_snapshot(
    state: &Arc<GatewayOpenResponsesServerState>,
    controls: &OpsShellControlsQuery,
    detail_session_key: Option<&str>,
) -> TauOpsDashboardChatSnapshot {
    let active_session_key = resolve_ops_chat_session_key(controls, detail_session_key);
    let session_options = collect_ops_chat_session_option_rows(state, active_session_key.as_str());
    let session_path = gateway_session_path(&state.config.state_dir, active_session_key.as_str());
    let mut message_rows = Vec::new();
    let mut session_detail_validation_entries: usize = 0;
    let mut session_detail_validation_duplicates: usize = 0;
    let mut session_detail_validation_invalid_parent: usize = 0;
    let mut session_detail_validation_cycles: usize = 0;
    let mut session_detail_validation_is_valid = true;
    let mut session_detail_usage_input_tokens: u64 = 0;
    let mut session_detail_usage_output_tokens: u64 = 0;
    let mut session_detail_usage_total_tokens: u64 = 0;
    let mut session_detail_usage_estimated_cost_usd = "0.000000".to_string();
    let mut session_detail_timeline_rows = Vec::new();
    let mut session_graph_node_rows = Vec::new();
    let mut session_graph_edge_rows = Vec::new();
    let memory_search_query = controls
        .requested_memory_query()
        .map(str::to_string)
        .unwrap_or_default();
    let memory_search_workspace_id = controls.requested_memory_workspace_id().unwrap_or_default();
    let memory_search_channel_id = controls.requested_memory_channel_id().unwrap_or_default();
    let memory_search_actor_id = controls.requested_memory_actor_id().unwrap_or_default();
    let memory_search_memory_type = controls.requested_memory_type().unwrap_or_default();
    let memory_create_workspace_id = memory_search_workspace_id.clone();
    let memory_create_channel_id = memory_search_channel_id.clone();
    let memory_create_actor_id = memory_search_actor_id.clone();
    let memory_create_memory_type = memory_search_memory_type.clone();
    let memory_create_status = controls.requested_memory_create_status().to_string();
    let memory_create_created_entry_id = controls
        .requested_memory_created_entry_id()
        .unwrap_or_default();
    let memory_delete_status = controls.requested_memory_delete_status().to_string();
    let memory_delete_deleted_entry_id = controls
        .requested_memory_deleted_entry_id()
        .unwrap_or_default();
    let mut memory_search_rows = Vec::new();
    let mut memory_detail_visible = false;
    let mut memory_detail_selected_entry_id = controls
        .requested_memory_detail_entry_id()
        .unwrap_or_default();
    let mut memory_detail_summary = String::new();
    let mut memory_detail_memory_type = String::new();
    let mut memory_detail_embedding_source = String::new();
    let mut memory_detail_embedding_model = String::new();
    let mut memory_detail_embedding_reason_code = String::new();
    let mut memory_detail_embedding_dimensions = 0usize;
    let mut memory_detail_relation_rows = Vec::new();
    let mut memory_graph_node_rows = Vec::new();
    let mut memory_graph_edge_rows = Vec::new();
    let memory_graph_zoom_level = format!("{:.2}", controls.requested_memory_graph_zoom_level());
    let memory_graph_pan_x_level = format!("{:.2}", controls.requested_memory_graph_pan_x_level());
    let memory_graph_pan_y_level = format!("{:.2}", controls.requested_memory_graph_pan_y_level());
    let memory_graph_filter_memory_type = controls.requested_memory_graph_filter_memory_type();
    let memory_graph_filter_relation_type = controls.requested_memory_graph_filter_relation_type();
    let tools_inventory_rows = collect_ops_tools_inventory_rows(state);
    let tool_detail_selected_tool_name =
        resolve_ops_selected_tool_name(controls, tools_inventory_rows.as_slice());
    let tool_detail_description = if tool_detail_selected_tool_name.is_empty() {
        String::new()
    } else {
        format!("{tool_detail_selected_tool_name} tool is registered in gateway runtime.")
    };
    let tool_detail_parameter_schema = "{\"type\":\"object\",\"properties\":{}}".to_string();
    let tool_detail_policy_timeout_ms = 120_000u64;
    let tool_detail_policy_max_output_chars = 32_768u64;
    let tool_detail_policy_sandbox_mode = "default".to_string();
    let tool_detail_usage_histogram_rows = collect_ops_tool_detail_usage_histogram_rows(
        tool_detail_selected_tool_name.as_str(),
        tools_inventory_rows.as_slice(),
    );
    let tool_detail_recent_invocation_rows =
        collect_ops_tool_detail_recent_invocation_rows(tool_detail_selected_tool_name.as_str());
    let mut jobs_rows = collect_ops_jobs_rows();
    apply_ops_cancel_job_request(controls, jobs_rows.as_mut_slice());
    let job_detail_selected_job_id = resolve_ops_selected_job_id(controls, jobs_rows.as_slice());
    let (job_detail_status, job_detail_duration_ms, job_detail_stdout, job_detail_stderr) =
        collect_ops_job_detail_output_contracts(
            job_detail_selected_job_id.as_str(),
            jobs_rows.as_slice(),
        );
    let memory_scope_filter = MemoryScopeFilter {
        workspace_id: (!memory_search_workspace_id.is_empty())
            .then_some(memory_search_workspace_id.clone()),
        channel_id: (!memory_search_channel_id.is_empty())
            .then_some(memory_search_channel_id.clone()),
        actor_id: (!memory_search_actor_id.is_empty()).then_some(memory_search_actor_id.clone()),
    };
    let store = gateway_memory_store(&state.config.state_dir, active_session_key.as_str());

    if !memory_search_query.trim().is_empty() {
        let search_options = MemorySearchOptions {
            limit: controls.requested_memory_limit(),
            scope: memory_scope_filter.clone(),
            ..MemorySearchOptions::default()
        };
        if let Ok(search_result) = store.search(memory_search_query.as_str(), &search_options) {
            memory_search_rows = search_result
                .matches
                .iter()
                .filter(|entry| {
                    memory_search_memory_type.is_empty()
                        || entry.memory_type.as_str() == memory_search_memory_type.as_str()
                })
                .map(|entry| TauOpsDashboardMemorySearchRow {
                    memory_id: entry.memory_id.clone(),
                    summary: entry.summary.clone(),
                    memory_type: entry.memory_type.as_str().to_string(),
                    score: format!("{:.4}", entry.score),
                })
                .take(search_options.limit)
                .collect();
        }
    }

    if !memory_detail_selected_entry_id.trim().is_empty() {
        match store.read_entry(memory_detail_selected_entry_id.as_str(), None) {
            Ok(Some(record)) => {
                memory_detail_visible = true;
                memory_detail_summary = record.entry.summary.clone();
                memory_detail_memory_type = record.memory_type.as_str().to_string();
                memory_detail_embedding_source = record.embedding_source.clone();
                memory_detail_embedding_model = record.embedding_model.clone().unwrap_or_default();
                memory_detail_embedding_reason_code = record.embedding_reason_code.clone();
                memory_detail_embedding_dimensions = record.embedding_vector.len();
                memory_detail_relation_rows = record
                    .relations
                    .iter()
                    .map(|relation| TauOpsDashboardMemoryRelationRow {
                        target_id: relation.target_id.clone(),
                        relation_type: relation.relation_type.as_str().to_string(),
                        effective_weight: format!("{:.4}", relation.effective_weight),
                    })
                    .collect();
            }
            Ok(None) | Err(_) => {
                memory_detail_selected_entry_id.clear();
            }
        }
    }

    if let Ok(mut records) = store.list_latest_records(
        Some(&memory_scope_filter),
        controls.requested_memory_limit(),
    ) {
        records.sort_by(|left, right| left.entry.memory_id.cmp(&right.entry.memory_id));
        if !memory_search_memory_type.is_empty() {
            records
                .retain(|record| record.memory_type.as_str() == memory_search_memory_type.as_str());
        }

        let memory_ids = records
            .iter()
            .map(|record| record.entry.memory_id.clone())
            .collect::<BTreeSet<_>>();

        memory_graph_node_rows = records
            .iter()
            .map(|record| TauOpsDashboardMemoryGraphNodeRow {
                memory_id: record.entry.memory_id.clone(),
                memory_type: record.memory_type.as_str().to_string(),
                importance: format!("{:.4}", record.importance.clamp(0.0, 1.0)),
            })
            .collect();

        for record in &records {
            for relation in &record.relations {
                if memory_ids.contains(&relation.target_id) {
                    memory_graph_edge_rows.push(TauOpsDashboardMemoryGraphEdgeRow {
                        source_memory_id: record.entry.memory_id.clone(),
                        target_memory_id: relation.target_id.clone(),
                        relation_type: relation.relation_type.as_str().to_string(),
                        effective_weight: format!("{:.4}", relation.effective_weight),
                    });
                }
            }
        }
        memory_graph_edge_rows.sort_by(|left, right| {
            left.source_memory_id
                .cmp(&right.source_memory_id)
                .then(left.target_memory_id.cmp(&right.target_memory_id))
                .then(left.relation_type.cmp(&right.relation_type))
        });
    }

    let (harness_lineage_nodes, harness_lineage_edges) =
        collect_ops_harness_memory_graph_lineage(&state.config.state_dir, None, 0.0);
    append_memory_graph_rows(
        &mut memory_graph_node_rows,
        &mut memory_graph_edge_rows,
        harness_lineage_nodes
            .iter()
            .map(tau_ops_dashboard_memory_graph_node_row_from_gateway)
            .collect(),
        harness_lineage_edges
            .iter()
            .map(tau_ops_dashboard_memory_graph_edge_row_from_gateway)
            .collect(),
    );

    if let Ok(store) = SessionStore::load(&session_path) {
        let validation = store.validation_report();
        session_detail_validation_entries = validation.entries;
        session_detail_validation_duplicates = validation.duplicates;
        session_detail_validation_invalid_parent = validation.invalid_parent;
        session_detail_validation_cycles = validation.cycles;
        session_detail_validation_is_valid = validation.is_valid();

        let usage = store.usage_summary();
        session_detail_usage_input_tokens = usage.input_tokens;
        session_detail_usage_output_tokens = usage.output_tokens;
        session_detail_usage_total_tokens = usage.total_tokens;
        session_detail_usage_estimated_cost_usd = format!("{:.6}", usage.estimated_cost_usd);

        if let Ok(lineage_entries) = store.lineage_entries(store.head_id()) {
            for entry in lineage_entries {
                let role = tau_ops_chat_message_role_label(entry.message.role).to_string();
                session_graph_node_rows.push(TauOpsDashboardSessionGraphNodeRow {
                    entry_id: entry.id,
                    role: role.clone(),
                });
                if let Some(parent_id) = entry.parent_id {
                    session_graph_edge_rows.push(TauOpsDashboardSessionGraphEdgeRow {
                        source_entry_id: parent_id,
                        target_entry_id: entry.id,
                    });
                }

                let content = entry.message.text_content();
                if content.trim().is_empty() {
                    continue;
                }
                session_detail_timeline_rows.push(TauOpsDashboardSessionTimelineRow {
                    entry_id: entry.id,
                    role: role.clone(),
                    content: content.clone(),
                });
                if matches!(entry.message.role, MessageRole::System) {
                    continue;
                }
                message_rows.push(TauOpsDashboardChatMessageRow { role, content });
            }
        }
    }

    TauOpsDashboardChatSnapshot {
        active_session_key: active_session_key.clone(),
        new_session_form_action: OPS_DASHBOARD_CHAT_NEW_ENDPOINT.to_string(),
        new_session_form_method: "post".to_string(),
        send_form_action: OPS_DASHBOARD_CHAT_SEND_ENDPOINT.to_string(),
        send_form_method: "post".to_string(),
        control_action_status: controls.requested_control_action_status().to_string(),
        control_action: controls.requested_control_action().to_string(),
        control_action_reason: controls.requested_control_action_reason().to_string(),
        session_options,
        message_rows,
        session_detail_visible: detail_session_key.is_some(),
        session_detail_route: format!("/ops/sessions/{active_session_key}"),
        session_detail_validation_entries,
        session_detail_validation_duplicates,
        session_detail_validation_invalid_parent,
        session_detail_validation_cycles,
        session_detail_validation_is_valid,
        session_detail_usage_input_tokens,
        session_detail_usage_output_tokens,
        session_detail_usage_total_tokens,
        session_detail_usage_estimated_cost_usd,
        session_detail_timeline_rows,
        session_graph_node_rows,
        session_graph_edge_rows,
        memory_search_form_action: "/ops/memory".to_string(),
        memory_search_form_method: "get".to_string(),
        memory_search_query,
        memory_search_workspace_id,
        memory_search_channel_id,
        memory_search_actor_id,
        memory_search_memory_type,
        memory_search_rows,
        memory_create_form_action: "/ops/memory".to_string(),
        memory_create_form_method: "post".to_string(),
        memory_create_status,
        memory_create_created_entry_id,
        memory_create_entry_id: String::new(),
        memory_create_summary: String::new(),
        memory_create_tags: String::new(),
        memory_create_facts: String::new(),
        memory_create_source_event_key: String::new(),
        memory_create_workspace_id,
        memory_create_channel_id,
        memory_create_actor_id,
        memory_create_memory_type,
        memory_create_importance: String::new(),
        memory_create_relation_target_id: String::new(),
        memory_create_relation_type: String::new(),
        memory_create_relation_weight: String::new(),
        memory_delete_status,
        memory_delete_deleted_entry_id,
        memory_detail_visible,
        memory_detail_selected_entry_id,
        memory_detail_summary,
        memory_detail_memory_type,
        memory_detail_embedding_source,
        memory_detail_embedding_model,
        memory_detail_embedding_reason_code,
        memory_detail_embedding_dimensions,
        memory_detail_relation_rows,
        memory_graph_zoom_level,
        memory_graph_pan_x_level,
        memory_graph_pan_y_level,
        memory_graph_filter_memory_type,
        memory_graph_filter_relation_type,
        memory_graph_node_rows,
        memory_graph_edge_rows,
        tools_inventory_rows,
        tool_detail_selected_tool_name,
        tool_detail_description,
        tool_detail_parameter_schema,
        tool_detail_policy_timeout_ms,
        tool_detail_policy_max_output_chars,
        tool_detail_policy_sandbox_mode,
        tool_detail_usage_histogram_rows,
        tool_detail_recent_invocation_rows,
        jobs_rows,
        job_detail_selected_job_id,
        job_detail_status,
        job_detail_duration_ms,
        job_detail_stdout,
        job_detail_stderr,
    }
}

pub(super) fn render_tau_ops_dashboard_shell_for_route(
    state: &Arc<GatewayOpenResponsesServerState>,
    route: TauOpsDashboardRoute,
    controls: OpsShellControlsQuery,
    detail_session_key: Option<&str>,
) -> Html<String> {
    let mut command_center =
        collect_tau_ops_dashboard_command_center_snapshot(&state.config.state_dir);
    command_center.timeline_range = controls.timeline_range().to_string();
    command_center.channel_action_status = controls.requested_channel_action_status().to_string();
    command_center.channel_action = controls.requested_channel_action().to_string();
    command_center.channel_action_channel = controls.requested_channel_action_channel().to_string();
    command_center.channel_action_reason = controls.requested_channel_action_reason().to_string();
    let chat = collect_tau_ops_dashboard_chat_snapshot(state, &controls, detail_session_key);
    let requested_harness_audit_action = if controls.requested_harness_view() == Some("history") {
        controls.requested_harness_audit_action()
    } else {
        None
    };
    let requested_harness_audit_ref = if controls.requested_harness_view() == Some("history") {
        controls.requested_harness_audit_ref()
    } else {
        None
    };
    let mut harness = collect_tau_ops_dashboard_harness_snapshot(
        &state.config.state_dir,
        controls.requested_harness_proposal_id(),
        requested_harness_audit_action,
    );
    harness.audit_selected_ref = requested_harness_audit_ref.unwrap_or_default();
    harness.runtime_workspace_label = state.config.state_dir.display().to_string();
    harness.runtime_model_label = state.config.model.clone();
    harness.runtime_transport_label = "gateway".to_string();
    harness.runtime_health_key = command_center.health_state.clone();
    if let Some(mission_id) = controls.requested_harness_mission_id() {
        apply_harness_selected_mission_detail(&mut harness, &state.config.state_dir, mission_id);
    }
    if matches!(route, TauOpsDashboardRoute::Harness) {
        if let Some(mission_status) = controls.requested_harness_mission_status() {
            let mission_id = controls
                .requested_harness_mission_id()
                .map(sanitize_harness_token)
                .unwrap_or_else(|| "unknown".to_string());
            let (route_action_key, route_action_label) = match mission_status {
                "draft_created" => ("mission-draft", "Mission Draft Created"),
                "mission_started" => ("mission-start", "Mission Started"),
                "mission_completed" => ("mission-start", "Mission Completed"),
                "mission_blocked" => ("mission-start", "Mission Blocked"),
                "start_failed" => ("mission-start", "Mission Start Failed"),
                _ => ("mission-draft", "Mission Draft Failed"),
            };
            harness.route_action_key = route_action_key.to_string();
            harness.route_action_label = route_action_label.to_string();
            harness.route_action_detail = format!(
                "Mission {mission_id} | session {} | selected {}",
                chat.active_session_key.clone(),
                harness.selected_proposal_id
            );
            harness.route_action_count = harness.mission_rows.len();
        } else if controls.requested_harness_intent() == Some("new-mission") {
            harness.route_action_key = "new-mission".to_string();
            harness.route_action_label = "New Mission Draft".to_string();
            harness.route_action_detail = format!(
                "Session {} | selected {} | draft not submitted",
                chat.active_session_key.clone(),
                harness.selected_proposal_id
            );
            harness.route_action_count = harness.mission_rows.len();
        } else if controls.requested_harness_view() == Some("history") {
            harness.route_action_key = "history".to_string();
            harness.route_action_label = "Applied History".to_string();
            harness.route_action_detail = format!(
                "{} audit records loaded from {}",
                harness.audit_rows.len(),
                harness.audit_source
            );
            harness.route_action_count = harness.audit_rows.len();
        } else if let Some(proposal_status) = controls.requested_harness_proposal_status() {
            let (route_action_key, route_action_label) =
                harness_proposal_status_route_action(proposal_status);
            harness.route_action_key = route_action_key.to_string();
            harness.route_action_label = route_action_label.to_string();
            harness.route_action_detail = format!(
                "Proposal {} | session {} | selected {}",
                controls
                    .requested_harness_proposal_id()
                    .map(sanitize_harness_token)
                    .unwrap_or_else(|| harness.selected_proposal_id.clone()),
                chat.active_session_key.clone(),
                harness.selected_proposal_id
            );
            harness.route_action_count = harness.audit_rows.len();
        }
    }

    Html(render_tau_ops_dashboard_shell_with_context(
        TauOpsDashboardShellContext {
            auth_mode: resolve_tau_ops_dashboard_auth_mode(state.config.auth_mode),
            active_route: route,
            theme: controls.theme(),
            sidebar_state: controls.sidebar_state(),
            command_center,
            chat,
            harness,
        },
    ))
}

fn harness_proposal_status_route_action(proposal_status: &str) -> (&'static str, &'static str) {
    match proposal_status {
        "applied" => ("proposal-applied", "Proposal Applied"),
        "apply_failed" => ("proposal-apply-failed", "Proposal Apply Failed"),
        "approved" => ("proposal-approved", "Proposal Approved"),
        "dry_run_failed" => ("proposal-dry-run", "Dry Run Failed"),
        "dry_run_passed" => ("proposal-dry-run", "Dry Run Passed"),
        "rejected" => ("proposal-rejected", "Proposal Rejected"),
        _ => ("proposal-action", "Proposal Action"),
    }
}

fn harness_proposal_detail_from_definition(
    state_dir: &Path,
    proposal: &GatewayOpsHarnessProposalDefinition,
) -> TauOpsDashboardHarnessProposalDetail {
    let (test_evidence_href, test_evidence_label) =
        harness_proposal_test_evidence_link(state_dir, proposal);
    TauOpsDashboardHarnessProposalDetail {
        proposal_id: proposal.proposal_id.to_string(),
        learning_record_id: proposal.source_learning_record_id.to_string(),
        title: proposal.title.to_string(),
        target_type: proposal.target_type.to_string(),
        target_path: proposal.target_path.to_string(),
        dry_run_result_label: proposal.dry_run_result_label.to_string(),
        dry_run_result_key: proposal.dry_run_result_key.to_string(),
        safety_check_label: proposal.safety_check_label.to_string(),
        safety_check_key: proposal.safety_check_key.to_string(),
        rollback_plan: proposal.rollback_plan.to_string(),
        patch_summary: proposal.patch_summary.to_string(),
        failure_observed: proposal.failure_summary.to_string(),
        root_cause: proposal.root_cause.to_string(),
        test_evidence_href,
        test_evidence_label,
    }
}

fn harness_proposal_test_evidence_link(
    state_dir: &Path,
    proposal: &GatewayOpsHarnessProposalDefinition,
) -> (String, String) {
    let state_artifact_path = format!(
        "ops-harness/self-improvement/{}/dry-run-result.json",
        sanitize_harness_token(proposal.proposal_id)
    );
    if state_dir.join(&state_artifact_path).is_file() {
        (
            harness_state_artifact_href(&state_artifact_path),
            state_artifact_path,
        )
    } else {
        (
            proposal.test_evidence_href.to_string(),
            proposal.test_evidence_label.to_string(),
        )
    }
}

fn collect_tau_ops_dashboard_harness_snapshot(
    state_dir: &Path,
    requested_proposal_id: Option<&str>,
    requested_audit_action: Option<&str>,
) -> TauOpsDashboardHarnessSnapshot {
    let (proposal_queue_source, proposal_queue_rows) =
        collect_harness_proposal_queue_rows(state_dir);
    let mut snapshot = TauOpsDashboardHarnessSnapshot {
        proposal_queue_source,
        proposal_queue_rows,
        ..TauOpsDashboardHarnessSnapshot::default()
    };
    let selected_proposal = requested_proposal_id
        .and_then(find_ops_harness_proposal)
        .or_else(|| list_ops_harness_proposals().first());
    if let Some(selected_proposal) = selected_proposal {
        snapshot.selected_proposal_id = selected_proposal.proposal_id.to_string();
        snapshot.selected_proposal =
            harness_proposal_detail_from_definition(state_dir, selected_proposal);
        snapshot.self_improvement_proof =
            collect_harness_self_improvement_proof(state_dir, selected_proposal);
    }

    let proof_path = harness_artifact_dir(state_dir).join("latest.json");
    if let Ok(proof_json) = std::fs::read_to_string(&proof_path) {
        if let Ok(proof) = serde_json::from_str::<Value>(&proof_json) {
            if let Some(tasks) = proof.get("tasks").and_then(Value::as_array) {
                snapshot.proof_source = "state".to_string();
                snapshot.proof_artifact = proof_path.display().to_string();
                snapshot.benchmark_id = proof
                    .get("benchmark_id")
                    .and_then(Value::as_str)
                    .unwrap_or(snapshot.benchmark_id.as_str())
                    .to_string();
                snapshot.task_count = tasks.len();
                snapshot.pass_count = tasks
                    .iter()
                    .filter(|task| task.get("passed").and_then(Value::as_bool).unwrap_or(false))
                    .count();
                let failure_count = proof
                    .get("failure_reasons")
                    .and_then(Value::as_array)
                    .map(Vec::len)
                    .unwrap_or(0);
                snapshot.failed_gate_count = failure_count;
                snapshot.failed_gate_label = if failure_count == 0 {
                    "none".to_string()
                } else {
                    failure_count.to_string()
                };
                snapshot.latest_result = format!("{}/{}", snapshot.pass_count, snapshot.task_count);
                snapshot.latest_runtime = "state".to_string();
                snapshot.latest_summary = format!(
                    "Latest state-backed result: {}. Failed gates: {}.",
                    snapshot.latest_result, snapshot.failed_gate_label
                );

                let mut category_totals: BTreeMap<String, (usize, usize)> = BTreeMap::new();
                for task in tasks {
                    let category = task
                        .get("category")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                        .to_string();
                    let passed = task.get("passed").and_then(Value::as_bool).unwrap_or(false);
                    let entry = category_totals.entry(category).or_insert((0, 0));
                    entry.0 += 1;
                    if passed {
                        entry.1 += 1;
                    }
                }
                snapshot.benchmark_rows = category_totals
                    .into_iter()
                    .map(|(category, (total_count, pass_count))| {
                        let pass_rate = if total_count == 0 {
                            0
                        } else {
                            (pass_count * 100) / total_count
                        };
                        TauOpsDashboardHarnessBenchmarkCategoryRow {
                            category,
                            task_count: total_count,
                            pass_count,
                            total_count,
                            pass_rate: pass_rate.to_string(),
                        }
                    })
                    .collect();
                apply_harness_benchmark_detail_from_proof(
                    &mut snapshot,
                    state_dir,
                    &proof,
                    &proof_path,
                );
            }
        }
    }

    let durable_mission_rows = collect_harness_durable_mission_rows(state_dir);
    if !durable_mission_rows.is_empty() {
        let pending_durable_missions = durable_mission_rows
            .iter()
            .filter(|row| row.verification_state != "passed")
            .count();
        if snapshot.proof_source == "state" {
            snapshot.mission_rows.extend(durable_mission_rows);
        } else {
            snapshot.mission_rows = durable_mission_rows;
            snapshot.kpi_pending_verification_count = pending_durable_missions;
        }
        snapshot.mission_table_title = "Active Missions".to_string();
        snapshot.kpi_missions_title = "Active Missions".to_string();
        snapshot.kpi_missions_count = snapshot.mission_rows.len();
        snapshot.kpi_missions_detail = format!("{pending_durable_missions} draft/review");
        if pending_durable_missions > 0 {
            snapshot.kpi_pending_verification_detail =
                format!("{pending_durable_missions} draft gates pending");
        }
    }

    let audit_mission_index = collect_harness_audit_mission_index(state_dir);
    let audit_path = state_dir.join("ops-harness").join("audit.jsonl");
    if let Ok(audit_jsonl) = std::fs::read_to_string(&audit_path) {
        let all_audit_rows = audit_jsonl
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter_map(|record| {
                let proposal_id = record.get("proposal_id").and_then(Value::as_str)?;
                let action = record
                    .get("action")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                let result = record
                    .get("result")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                let benchmark_id = record
                    .get("benchmark_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let run_id = record
                    .get("run_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let timestamp = record
                    .get("timestamp_unix_ms")
                    .and_then(Value::as_u64)
                    .unwrap_or_default();
                let matched_mission =
                    if action == "start-mission" && record.get("mission_id").is_none() {
                        find_harness_audit_mission_match(
                            audit_mission_index.as_slice(),
                            proposal_id,
                            timestamp,
                        )
                    } else {
                        None
                    };
                let mission_id = record
                    .get("mission_id")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .or_else(|| matched_mission.map(|mission| mission.mission_id.clone()))
                    .unwrap_or_default();
                let record_proof_artifact = record
                    .get("proof_artifact")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .unwrap_or_default();
                let mission_proof_artifact = if record_proof_artifact.is_empty() {
                    matched_mission
                        .map(|mission| mission.proof_artifact.clone())
                        .unwrap_or_default()
                } else {
                    record_proof_artifact.clone()
                };
                let proposal_scope = find_ops_harness_proposal(proposal_id)
                    .map(|proposal| proposal.target_type.to_string())
                    .unwrap_or_else(|| "Proposal".to_string());
                let (scope, item, detail_label, detail_value, proof_artifact) = match action {
                    "start-mission" => (
                        "Mission".to_string(),
                        proposal_id.to_string(),
                        if mission_id.is_empty() {
                            String::new()
                        } else {
                            "Mission".to_string()
                        },
                        mission_id,
                        mission_proof_artifact,
                    ),
                    "run-benchmark" => (
                        "Benchmark".to_string(),
                        if benchmark_id.is_empty() {
                            proposal_id.to_string()
                        } else {
                            benchmark_id.to_string()
                        },
                        if run_id.is_empty() {
                            String::new()
                        } else {
                            "Run".to_string()
                        },
                        run_id.to_string(),
                        record_proof_artifact,
                    ),
                    _ => (
                        proposal_scope,
                        proposal_id.to_string(),
                        String::new(),
                        String::new(),
                        record_proof_artifact,
                    ),
                };
                Some(TauOpsDashboardHarnessAuditRow {
                    timestamp_label: format_harness_audit_timestamp(timestamp),
                    timestamp_unix_ms: timestamp.to_string(),
                    actor: "Gateway".to_string(),
                    action_label: humanize_harness_token(action),
                    action_key: action.to_string(),
                    scope,
                    item,
                    detail_label,
                    detail_value,
                    proof_artifact,
                    result_label: humanize_harness_token(result),
                    result_key: result.to_string(),
                })
            })
            .rev()
            .collect::<Vec<_>>();
        let audit_total_count = all_audit_rows.len();
        let audit_rows = all_audit_rows
            .into_iter()
            .filter(|row| requested_audit_action.is_none_or(|action| row.action_key == action))
            .take(4)
            .collect::<Vec<_>>();
        if !audit_rows.is_empty() || audit_total_count > 0 {
            snapshot.audit_source = "state".to_string();
            snapshot.audit_filter_action = requested_audit_action.unwrap_or("all").to_string();
            snapshot.audit_total_count = audit_total_count;
            snapshot.audit_rows = audit_rows;
        }
    }

    snapshot
}

fn collect_harness_proposal_queue_rows(
    state_dir: &Path,
) -> (String, Vec<TauOpsDashboardHarnessProposalQueueRow>) {
    let latest_audit_results = collect_latest_harness_proposal_audit_results(state_dir);
    let state_rows = list_ops_harness_proposals()
        .iter()
        .filter_map(|proposal| {
            let mission_status = read_harness_proposal_mission_status(state_dir, proposal);
            let latest_audit_result = latest_audit_results.get(proposal.proposal_id);
            if mission_status.is_none() && latest_audit_result.is_none() {
                return None;
            }
            Some(TauOpsDashboardHarnessProposalQueueRow {
                item_id: proposal.proposal_id.to_string(),
                status_key: harness_proposal_queue_status_key(
                    proposal,
                    mission_status.as_deref(),
                    latest_audit_result.map(String::as_str),
                ),
                label: proposal.queue_label.to_string(),
            })
        })
        .collect::<Vec<_>>();

    if !state_rows.is_empty() {
        return ("state".to_string(), state_rows);
    }

    (
        "registry".to_string(),
        list_ops_harness_proposals()
            .iter()
            .map(|proposal| TauOpsDashboardHarnessProposalQueueRow {
                item_id: proposal.proposal_id.to_string(),
                status_key: proposal.status_key.to_string(),
                label: proposal.queue_label.to_string(),
            })
            .collect(),
    )
}

fn collect_harness_durable_mission_rows(state_dir: &Path) -> Vec<TauOpsDashboardHarnessMissionRow> {
    let missions_dir = state_dir.join("ops-harness").join("missions");
    let Ok(entries) = std::fs::read_dir(missions_dir) else {
        return Vec::new();
    };

    let mut rows = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let mission_path = entry.path().join("mission.json");
            let mission_json = std::fs::read_to_string(mission_path).ok()?;
            let mission = serde_json::from_str::<Value>(&mission_json).ok()?;
            let mission_id = mission.get("mission_id").and_then(Value::as_str)?;
            let proposal_key = mission
                .get("proposal_id")
                .and_then(Value::as_str)
                .filter(|proposal_id| !proposal_id.trim().is_empty())
                .unwrap_or(mission_id)
                .to_string();
            let updated_unix_ms = mission
                .get("updated_unix_ms")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            let row = harness_durable_mission_row_from_value(&mission)?;
            Some((proposal_key, updated_unix_ms, row))
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| right.1.cmp(&left.1));

    let mut seen_proposals = BTreeSet::new();
    rows.into_iter()
        .filter_map(|(proposal_key, _, row)| seen_proposals.insert(proposal_key).then_some(row))
        .take(5)
        .collect()
}

#[derive(Debug, Clone)]
struct HarnessAuditMissionIndexRow {
    proposal_id: String,
    mission_id: String,
    updated_unix_ms: u64,
    proof_artifact: String,
}

fn collect_harness_audit_mission_index(state_dir: &Path) -> Vec<HarnessAuditMissionIndexRow> {
    let missions_dir = state_dir.join("ops-harness").join("missions");
    let Ok(entries) = std::fs::read_dir(missions_dir) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let mission_path = entry.path().join("mission.json");
            let mission_json = std::fs::read_to_string(&mission_path).ok()?;
            let mission = serde_json::from_str::<Value>(&mission_json).ok()?;
            let proposal_id = mission.get("proposal_id").and_then(Value::as_str)?;
            let mission_id = mission.get("mission_id").and_then(Value::as_str)?;
            let updated_unix_ms = mission
                .get("updated_unix_ms")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            Some(HarnessAuditMissionIndexRow {
                proposal_id: proposal_id.to_string(),
                mission_id: mission_id.to_string(),
                updated_unix_ms,
                proof_artifact: harness_mission_state_artifact(&mission, mission_id),
            })
        })
        .collect()
}

fn harness_mission_state_artifact(mission: &Value, mission_id: &str) -> String {
    mission
        .get("artifacts")
        .and_then(Value::as_array)
        .and_then(|artifacts| {
            artifacts.iter().find_map(|artifact| {
                let artifact_id = artifact.get("artifact_id").and_then(Value::as_str);
                let kind = artifact.get("kind").and_then(Value::as_str);
                let path = artifact.get("path").and_then(Value::as_str)?;
                if artifact_id == Some("mission-json") || kind == Some("mission-state") {
                    Some(path.to_string())
                } else {
                    None
                }
            })
        })
        .unwrap_or_else(|| format!("ops-harness/missions/{mission_id}/mission.json"))
}

fn find_harness_audit_mission_match<'a>(
    mission_index: &'a [HarnessAuditMissionIndexRow],
    proposal_id: &str,
    timestamp_unix_ms: u64,
) -> Option<&'a HarnessAuditMissionIndexRow> {
    if timestamp_unix_ms == 0 {
        return None;
    }
    mission_index
        .iter()
        .filter(|mission| mission.proposal_id == proposal_id)
        .filter_map(|mission| {
            let delta = mission.updated_unix_ms.abs_diff(timestamp_unix_ms);
            (delta <= 10_000).then_some((delta, mission))
        })
        .min_by_key(|(delta, _)| *delta)
        .map(|(_, mission)| mission)
}

fn harness_durable_mission_row_from_value(
    mission: &Value,
) -> Option<TauOpsDashboardHarnessMissionRow> {
    let mission_id = mission.get("mission_id").and_then(Value::as_str)?;
    let status_key = mission
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("draft");
    let acceptance_total = mission
        .get("acceptance_criteria")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default();
    let plan_nodes = mission
        .get("plan_dag")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let completed_plan_nodes = plan_nodes
        .iter()
        .filter(|node| {
            node.get("status")
                .and_then(Value::as_str)
                .is_some_and(|status| matches!(status, "completed" | "skipped"))
        })
        .count();
    let plan_progress = if plan_nodes.is_empty() {
        0
    } else {
        (completed_plan_nodes * 100) / plan_nodes.len()
    };
    let gates = mission
        .get("verification_gates")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let passed_gates = gates
        .iter()
        .filter(|gate| {
            gate.get("status")
                .and_then(Value::as_str)
                .is_some_and(|status| status == "passed")
        })
        .count();
    let failed_gates = gates
        .iter()
        .filter(|gate| {
            gate.get("status")
                .and_then(Value::as_str)
                .is_some_and(|status| status == "failed")
        })
        .count();
    let gate_total = gates.len();
    let passed_gate_ids = gates
        .iter()
        .filter(|gate| {
            gate.get("status")
                .and_then(Value::as_str)
                .is_some_and(|status| status == "passed")
        })
        .filter_map(|gate| gate.get("id").and_then(Value::as_str).map(str::to_string))
        .collect::<BTreeSet<_>>();
    let acceptance_met = mission
        .get("acceptance_criteria")
        .and_then(Value::as_array)
        .map(|criteria| {
            criteria
                .iter()
                .filter(|criterion| {
                    criterion
                        .get("verification_gate_ids")
                        .and_then(Value::as_array)
                        .is_some_and(|gate_ids| {
                            !gate_ids.is_empty()
                                && gate_ids.iter().all(|gate_id| {
                                    gate_id
                                        .as_str()
                                        .is_some_and(|id| passed_gate_ids.contains(id))
                                })
                        })
                })
                .count()
        })
        .unwrap_or_default();
    let verification_state = if failed_gates > 0 {
        "failed"
    } else if gate_total > 0 && passed_gates == gate_total {
        "passed"
    } else {
        "pending"
    };
    let tool_budget = mission.get("tool_budget").unwrap_or(&Value::Null);
    let consumed_tool_calls = tool_budget
        .get("consumed_tool_calls")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let max_tool_calls = tool_budget
        .get("max_tool_calls")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let memory_hits = mission
        .get("memory_hits")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default();
    let artifact_count = mission
        .get("artifacts")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default();
    let last_checkpoint = mission
        .get("checkpoints")
        .and_then(Value::as_array)
        .and_then(|checkpoints| checkpoints.last())
        .and_then(|checkpoint| checkpoint.get("summary"))
        .and_then(Value::as_str)
        .unwrap_or("draft checkpoint");

    Some(TauOpsDashboardHarnessMissionRow {
        mission_id: mission_id.to_string(),
        title: mission
            .get("goal")
            .and_then(Value::as_str)
            .unwrap_or(mission_id)
            .to_string(),
        status_key: status_key.to_string(),
        status_label: humanize_harness_token(status_key),
        gate_status_key: verification_state.to_string(),
        gate_label: format!("{passed_gates}/{gate_total} gates"),
        acceptance_label: format!("{acceptance_met}/{acceptance_total}"),
        plan_progress,
        tool_budget: format!("{consumed_tool_calls}/{max_tool_calls}"),
        memory_hits,
        verification_state: verification_state.to_string(),
        last_checkpoint: last_checkpoint.to_string(),
        artifact_count,
    })
}

fn read_harness_proposal_mission_status(
    state_dir: &Path,
    proposal: &GatewayOpsHarnessProposalDefinition,
) -> Option<String> {
    let mission_path = state_dir
        .join("ops-harness")
        .join("self-improvement")
        .join(proposal.proposal_id)
        .join("mission.json");
    let mission_json = std::fs::read_to_string(mission_path).ok()?;
    let mission = serde_json::from_str::<Value>(&mission_json).ok()?;
    mission
        .get("status")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn collect_latest_harness_proposal_audit_results(state_dir: &Path) -> BTreeMap<String, String> {
    let audit_path = state_dir.join("ops-harness").join("audit.jsonl");
    let Ok(audit_jsonl) = std::fs::read_to_string(audit_path) else {
        return BTreeMap::new();
    };

    let mut latest = BTreeMap::new();
    for record in audit_jsonl
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
    {
        let Some(proposal_id) = record.get("proposal_id").and_then(Value::as_str) else {
            continue;
        };
        if latest.contains_key(proposal_id) {
            continue;
        }
        let action = record
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let result = record
            .get("result")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        latest.insert(proposal_id.to_string(), format!("{action}:{result}"));
    }
    latest
}

fn harness_proposal_queue_status_key(
    proposal: &GatewayOpsHarnessProposalDefinition,
    mission_status: Option<&str>,
    latest_audit_result: Option<&str>,
) -> String {
    if mission_status == Some("completed") && latest_audit_result == Some("dry-run:passed") {
        return "completed".to_string();
    }
    match latest_audit_result {
        Some("apply:applied") => return "applied".to_string(),
        Some("approve:recorded") => return "approved".to_string(),
        Some("reject:rejected") => return "rejected".to_string(),
        Some("dry-run:passed") => return "dry-run-passed".to_string(),
        _ => {}
    }

    mission_status
        .filter(|status| !status.is_empty())
        .unwrap_or(proposal.status_key)
        .to_string()
}

fn normalize_harness_state_artifact_path(artifact_path: &str) -> Option<PathBuf> {
    let trimmed = artifact_path.trim().trim_start_matches('/');
    if trimmed.is_empty() || trimmed.contains('\\') || !trimmed.starts_with("ops-harness/") {
        return None;
    }
    let path = Path::new(trimmed);
    let has_only_safe_components = path
        .components()
        .all(|component| matches!(component, Component::Normal(_)));
    has_only_safe_components.then(|| path.to_path_buf())
}

fn harness_state_artifact_href(artifact_path: &str) -> String {
    normalize_harness_state_artifact_path(artifact_path)
        .and_then(|path| path.to_str().map(str::to_string))
        .map(|path| format!("/ops/harness/artifacts/view/{path}"))
        .unwrap_or_else(|| artifact_path.to_string())
}

fn harness_state_artifact_href_for_path(state_dir: &Path, artifact_path: &Path) -> String {
    artifact_path
        .strip_prefix(state_dir)
        .ok()
        .and_then(Path::to_str)
        .map(harness_state_artifact_href)
        .unwrap_or_else(|| artifact_path.display().to_string())
}

fn read_harness_state_artifact(
    state: &GatewayOpenResponsesServerState,
    artifact_path: &str,
) -> Option<(PathBuf, String)> {
    let relative_path = normalize_harness_state_artifact_path(artifact_path)?;
    let absolute_path = state.config.state_dir.join(relative_path);
    let payload = std::fs::read_to_string(&absolute_path).ok()?;
    Some((absolute_path, payload))
}

fn escape_harness_artifact_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn apply_harness_benchmark_detail_from_proof(
    snapshot: &mut TauOpsDashboardHarnessSnapshot,
    state_dir: &Path,
    proof: &Value,
    proof_path: &Path,
) {
    let Some(tasks) = proof.get("tasks").and_then(Value::as_array) else {
        return;
    };
    let run_id = proof
        .get("run_id")
        .and_then(Value::as_str)
        .unwrap_or(snapshot.detail_run_id.as_str());
    let passed = proof
        .get("passed")
        .and_then(Value::as_bool)
        .unwrap_or(snapshot.failed_gate_count == 0);
    let status = if passed { "completed" } else { "failed" };
    let first_mission = tasks.first().and_then(|task| task.get("mission"));
    let consumed_tool_calls = tasks
        .iter()
        .filter_map(|task| task.get("mission"))
        .filter_map(|mission| mission.get("tool_budget"))
        .filter_map(|budget| budget.get("consumed_tool_calls").and_then(Value::as_u64))
        .sum::<u64>();
    let max_tool_calls = tasks
        .iter()
        .filter_map(|task| task.get("mission"))
        .filter_map(|mission| mission.get("tool_budget"))
        .filter_map(|budget| budget.get("max_tool_calls").and_then(Value::as_u64))
        .sum::<u64>();
    let consumed_runtime_ms = tasks
        .iter()
        .filter_map(|task| task.get("mission"))
        .filter_map(|mission| mission.get("tool_budget"))
        .filter_map(|budget| budget.get("consumed_runtime_ms").and_then(Value::as_u64))
        .sum::<u64>();
    let consumed_cost = tasks
        .iter()
        .filter_map(|task| task.get("mission"))
        .filter_map(|mission| mission.get("tool_budget"))
        .filter_map(|budget| budget.get("consumed_cost_usd").and_then(Value::as_f64))
        .sum::<f64>();

    snapshot.detail_run_id = run_id.to_string();
    snapshot.detail_proof_artifact = proof_path.display().to_string();
    snapshot.detail_goal = format!(
        "Canonical {} benchmark proof run",
        humanize_harness_token(&snapshot.benchmark_id)
    );
    snapshot.detail_status = status.to_string();
    snapshot.detail_elapsed = format_harness_runtime_ms(consumed_runtime_ms);
    snapshot.detail_tool_budget = if max_tool_calls == 0 {
        consumed_tool_calls.to_string()
    } else {
        format!("{consumed_tool_calls}/{max_tool_calls}")
    };
    snapshot.detail_cost = format!("${consumed_cost:.2}");
    snapshot.detail_retry_count = tasks
        .iter()
        .filter_map(|task| task.get("operator_interventions_used"))
        .filter_map(Value::as_array)
        .map(Vec::len)
        .sum::<usize>()
        .to_string();

    if let Some(plan_nodes) = first_mission
        .and_then(|mission| mission.get("plan_dag"))
        .and_then(Value::as_array)
    {
        snapshot.detail_plan_rows = plan_nodes
            .iter()
            .filter_map(|node| {
                let item_id = node.get("id").and_then(Value::as_str)?;
                Some(TauOpsDashboardHarnessProofRow {
                    item_id: item_id.replace('_', "-"),
                    status_key: harness_detail_status_key(
                        node.get("status")
                            .and_then(Value::as_str)
                            .unwrap_or("pending"),
                    ),
                    label: humanize_harness_token(item_id),
                })
            })
            .collect();
        snapshot.detail_plan_current_node = snapshot
            .detail_plan_rows
            .iter()
            .find(|row| row.status_key != "passed")
            .or_else(|| snapshot.detail_plan_rows.last())
            .map(|row| row.item_id.clone())
            .unwrap_or_else(|| "plan".to_string());
    }

    if let Some(tool_evidence) = first_mission
        .and_then(|mission| mission.get("tool_evidence"))
        .and_then(Value::as_array)
    {
        let proof_artifact_href = harness_state_artifact_href_for_path(state_dir, proof_path);
        snapshot.detail_tool_call_count = tool_evidence.len();
        snapshot.detail_tool_rows = tool_evidence
            .iter()
            .filter_map(|tool| {
                let tool_name = tool.get("tool_name").and_then(Value::as_str)?;
                let call_id = tool
                    .get("tool_call_id")
                    .and_then(Value::as_str)
                    .unwrap_or(tool_name);
                let artifact_label = tool
                    .get("artifact_ids")
                    .and_then(Value::as_array)
                    .and_then(|ids| ids.first())
                    .and_then(Value::as_str)
                    .and_then(|artifact_id| artifact_id.rsplit(':').next())
                    .unwrap_or("proof");
                Some(TauOpsDashboardHarnessToolEvidenceRow {
                    tool_name: tool_name.to_string(),
                    call_id: compact_harness_call_id(call_id),
                    plan_node: humanize_harness_token(
                        tool.get("plan_node_id")
                            .and_then(Value::as_str)
                            .unwrap_or("execute"),
                    ),
                    runtime: format_harness_runtime_ms(
                        tool.get("runtime_ms").and_then(Value::as_u64).unwrap_or(0),
                    ),
                    status_key: harness_detail_status_key(
                        tool.get("status")
                            .and_then(Value::as_str)
                            .unwrap_or("pending"),
                    ),
                    artifact_label: artifact_label.to_string(),
                    artifact_href: proof_artifact_href.clone(),
                })
            })
            .collect();
    }

    let task_count = tasks.len();
    let pass_count = tasks
        .iter()
        .filter(|task| task.get("passed").and_then(Value::as_bool).unwrap_or(false))
        .count();
    let required_terminal_state = proof
        .get("required_terminal_state")
        .and_then(Value::as_str)
        .unwrap_or("completed");
    let unique_gate_rows = first_mission
        .and_then(|mission| mission.get("verification_gates"))
        .and_then(Value::as_array)
        .map(|gates| {
            gates
                .iter()
                .filter_map(|gate| {
                    let item_id = gate.get("id").and_then(Value::as_str)?;
                    Some(TauOpsDashboardHarnessProofRow {
                        item_id: item_id.to_string(),
                        status_key: harness_detail_status_key(
                            gate.get("status")
                                .and_then(Value::as_str)
                                .unwrap_or("pending"),
                        ),
                        label: gate
                            .get("description")
                            .and_then(Value::as_str)
                            .map(str::to_string)
                            .unwrap_or_else(|| humanize_harness_token(item_id)),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let passed_gate_count = unique_gate_rows
        .iter()
        .filter(|row| row.status_key == "passed")
        .count();

    snapshot.detail_acceptance_rows = vec![
        TauOpsDashboardHarnessProofRow {
            item_id: "benchmark-tasks".to_string(),
            status_key: if pass_count == task_count {
                "met"
            } else {
                "pending"
            }
            .to_string(),
            label: format!("Benchmark tasks passed {pass_count}/{task_count}"),
        },
        TauOpsDashboardHarnessProofRow {
            item_id: "terminal-state".to_string(),
            status_key: if status == required_terminal_state {
                "met"
            } else {
                "pending"
            }
            .to_string(),
            label: format!("Required terminal state {required_terminal_state}"),
        },
        TauOpsDashboardHarnessProofRow {
            item_id: "planning-tool-memory-verification-learning".to_string(),
            status_key: if passed_gate_count == unique_gate_rows.len() {
                "met"
            } else {
                "pending"
            }
            .to_string(),
            label: format!(
                "Planning/tool/memory/verification/learning gates passed {}/{}",
                passed_gate_count,
                unique_gate_rows.len()
            ),
        },
    ];
    snapshot.detail_acceptance_met_count = snapshot
        .detail_acceptance_rows
        .iter()
        .filter(|row| row.status_key == "met")
        .count();
    snapshot.detail_acceptance_total_count = snapshot.detail_acceptance_rows.len();
    if !unique_gate_rows.is_empty() {
        snapshot.detail_gate_failed_count = unique_gate_rows
            .iter()
            .filter(|row| row.status_key == "failed")
            .count();
        snapshot.detail_gate_rows = unique_gate_rows;
    }

    snapshot.detail_memory_hit_count = tasks
        .iter()
        .filter_map(|task| task.get("mission"))
        .filter_map(|mission| mission.get("memory_hits"))
        .filter_map(Value::as_array)
        .map(Vec::len)
        .sum();
    snapshot.detail_learning_record_count = tasks
        .iter()
        .filter_map(|task| task.get("mission"))
        .filter_map(|mission| mission.get("learning_records"))
        .filter_map(Value::as_array)
        .map(Vec::len)
        .sum();
    let latest_learning_ms = tasks
        .iter()
        .filter_map(|task| task.get("mission"))
        .filter_map(|mission| mission.get("learning_records"))
        .filter_map(Value::as_array)
        .flat_map(|records| records.iter())
        .filter_map(|record| record.get("created_unix_ms").and_then(Value::as_u64))
        .max()
        .unwrap_or_default();
    snapshot.detail_last_memory_write = if latest_learning_ms == 0 {
        "state".to_string()
    } else {
        format_harness_audit_timestamp(latest_learning_ms)
    };
    snapshot.detail_memory_evidence_label = format!(
        "{} benchmark memory hits used",
        snapshot.detail_memory_hit_count
    );
    snapshot.detail_artifact_rows = std::iter::once(TauOpsDashboardHarnessArtifactRow {
        item_id: "benchmark-proof".to_string(),
        status_key: "mission_harness_proof".to_string(),
        label: "Benchmark proof artifact".to_string(),
        href: harness_state_artifact_href_for_path(state_dir, proof_path),
    })
    .chain(tasks.iter().filter_map(|task| {
        let task_id = task.get("task_id").and_then(Value::as_str)?;
        Some(TauOpsDashboardHarnessArtifactRow {
            item_id: task_id.to_string(),
            status_key: "mission_task_proof".to_string(),
            label: format!("{} proof", humanize_harness_token(task_id)),
            href: harness_state_artifact_href_for_path(state_dir, proof_path),
        })
    }))
    .collect();

    let detail_gate_total = snapshot.detail_gate_rows.len();
    let detail_gate_label = format!("{passed_gate_count}/{detail_gate_total} gates");
    let detail_verification_state = if snapshot.detail_gate_failed_count > 0 {
        "failed"
    } else if detail_gate_total > 0 && passed_gate_count == detail_gate_total {
        "passed"
    } else {
        snapshot.detail_status.as_str()
    };
    let detail_plan_progress = if snapshot.detail_plan_rows.is_empty() {
        0
    } else {
        let passed_plan_count = snapshot
            .detail_plan_rows
            .iter()
            .filter(|row| row.status_key == "passed")
            .count();
        (passed_plan_count * 100) / snapshot.detail_plan_rows.len()
    };
    let open_gate_count = detail_gate_total.saturating_sub(passed_gate_count);

    snapshot.mission_table_title = "Benchmark Runs".to_string();
    snapshot.kpi_missions_title = "Benchmark Runs".to_string();
    snapshot.kpi_missions_count = 1;
    snapshot.kpi_missions_detail = snapshot.detail_status.clone();
    snapshot.kpi_pending_verification_count = open_gate_count;
    snapshot.kpi_pending_verification_detail = if open_gate_count == 0 {
        "none failed".to_string()
    } else {
        format!("{open_gate_count} open gates")
    };
    snapshot.kpi_memory_write_count = snapshot.detail_learning_record_count;
    snapshot.kpi_memory_write_detail = "learning records".to_string();
    snapshot.kpi_runtime_cost_today = snapshot.detail_cost.clone();
    snapshot.kpi_runtime_cost_detail = "Across 1 run".to_string();
    snapshot.mission_rows = vec![TauOpsDashboardHarnessMissionRow {
        mission_id: snapshot.detail_run_id.clone(),
        title: snapshot.detail_goal.clone(),
        status_key: snapshot.detail_status.clone(),
        status_label: humanize_harness_token(&snapshot.detail_status),
        gate_status_key: detail_verification_state.to_string(),
        gate_label: detail_gate_label,
        acceptance_label: format!(
            "{}/{}",
            snapshot.detail_acceptance_met_count, snapshot.detail_acceptance_total_count
        ),
        plan_progress: detail_plan_progress,
        tool_budget: snapshot.detail_tool_budget.clone(),
        memory_hits: snapshot.detail_memory_hit_count,
        verification_state: detail_verification_state.to_string(),
        last_checkpoint: snapshot.detail_last_memory_write.clone(),
        artifact_count: snapshot.detail_artifact_rows.len(),
    }];

    snapshot.detail_operator_log = format!(
        "state proof loaded: {}\nrun_id: {}\nbenchmark: {}\ntasks: {}/{} passed\nverification gates: {}/{} passed\nmemory hits: {}\nlearning records: {}\nstatus: {}",
        proof_path.display(),
        snapshot.detail_run_id,
        snapshot.benchmark_id,
        pass_count,
        task_count,
        passed_gate_count,
        snapshot.detail_gate_rows.len(),
        snapshot.detail_memory_hit_count,
        snapshot.detail_learning_record_count,
        snapshot.detail_status,
    );
}

fn apply_harness_selected_mission_detail(
    snapshot: &mut TauOpsDashboardHarnessSnapshot,
    state_dir: &Path,
    mission_id: &str,
) -> bool {
    let mission_id = sanitize_harness_token(mission_id);
    let mission_path = state_dir
        .join("ops-harness")
        .join("missions")
        .join(&mission_id)
        .join("mission.json");
    let Ok(mission_json) = std::fs::read_to_string(&mission_path) else {
        return false;
    };
    let Ok(mission) = serde_json::from_str::<Value>(&mission_json) else {
        return false;
    };

    let actual_mission_id = mission
        .get("mission_id")
        .and_then(Value::as_str)
        .map(sanitize_harness_token)
        .unwrap_or_else(|| mission_id.clone());
    let status = mission
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let created_unix_ms = mission
        .get("created_unix_ms")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let updated_unix_ms = mission
        .get("updated_unix_ms")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let elapsed_ms = updated_unix_ms.saturating_sub(created_unix_ms);
    let tool_budget = mission.get("tool_budget").unwrap_or(&Value::Null);
    let consumed_tool_calls = tool_budget
        .get("consumed_tool_calls")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let max_tool_calls = tool_budget
        .get("max_tool_calls")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let consumed_cost = tool_budget
        .get("consumed_cost_usd")
        .and_then(Value::as_f64)
        .unwrap_or_default();

    snapshot.detail_run_id = actual_mission_id.clone();
    snapshot.detail_proof_artifact = mission_path.display().to_string();
    snapshot.detail_goal = mission
        .get("goal")
        .and_then(Value::as_str)
        .unwrap_or(actual_mission_id.as_str())
        .to_string();
    snapshot.detail_status = status.to_string();
    snapshot.detail_elapsed = if elapsed_ms == 0 {
        "state".to_string()
    } else {
        format_harness_runtime_ms(elapsed_ms)
    };
    snapshot.detail_tool_budget = if max_tool_calls == 0 {
        consumed_tool_calls.to_string()
    } else {
        format!("{consumed_tool_calls}/{max_tool_calls}")
    };
    snapshot.detail_cost = format!("${consumed_cost:.2}");
    snapshot.detail_retry_count = mission
        .get("recovery_state")
        .and_then(|recovery| recovery.get("retry_count"))
        .and_then(Value::as_u64)
        .unwrap_or_default()
        .to_string();

    if let Some(plan_nodes) = mission.get("plan_dag").and_then(Value::as_array) {
        snapshot.detail_plan_rows = plan_nodes
            .iter()
            .filter_map(|node| {
                let item_id = node.get("id").and_then(Value::as_str)?;
                Some(TauOpsDashboardHarnessProofRow {
                    item_id: item_id.to_string(),
                    status_key: harness_detail_status_key(
                        node.get("status")
                            .and_then(Value::as_str)
                            .unwrap_or("pending"),
                    ),
                    label: node
                        .get("description")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .unwrap_or_else(|| humanize_harness_token(item_id)),
                })
            })
            .collect();
        snapshot.detail_plan_current_node = snapshot
            .detail_plan_rows
            .iter()
            .find(|row| row.status_key != "passed")
            .or_else(|| snapshot.detail_plan_rows.last())
            .map(|row| row.item_id.clone())
            .unwrap_or_else(|| "plan".to_string());
    }

    if let Some(tool_evidence) = mission.get("tool_evidence").and_then(Value::as_array) {
        let mission_artifact_href = harness_state_artifact_href_for_path(state_dir, &mission_path);
        snapshot.detail_tool_call_count = tool_evidence.len();
        snapshot.detail_tool_rows = tool_evidence
            .iter()
            .filter_map(|tool| {
                let tool_name = tool.get("tool_name").and_then(Value::as_str)?;
                let call_id = tool
                    .get("tool_call_id")
                    .and_then(Value::as_str)
                    .unwrap_or(tool_name);
                let artifact_label = tool
                    .get("artifact_ids")
                    .and_then(Value::as_array)
                    .and_then(|ids| ids.first())
                    .and_then(Value::as_str)
                    .unwrap_or("mission-json");
                Some(TauOpsDashboardHarnessToolEvidenceRow {
                    tool_name: tool_name.to_string(),
                    call_id: compact_harness_call_id(call_id),
                    plan_node: humanize_harness_token(
                        tool.get("plan_node_id")
                            .and_then(Value::as_str)
                            .unwrap_or("execute"),
                    ),
                    runtime: format_harness_runtime_ms(
                        tool.get("runtime_ms").and_then(Value::as_u64).unwrap_or(0),
                    ),
                    status_key: harness_detail_status_key(
                        tool.get("status")
                            .and_then(Value::as_str)
                            .unwrap_or("pending"),
                    ),
                    artifact_label: artifact_label.to_string(),
                    artifact_href: mission_artifact_href.clone(),
                })
            })
            .collect();
    }

    let gate_statuses = mission
        .get("verification_gates")
        .and_then(Value::as_array)
        .map(|gates| {
            gates
                .iter()
                .filter_map(|gate| {
                    Some((
                        gate.get("id").and_then(Value::as_str)?.to_string(),
                        gate.get("status")
                            .and_then(Value::as_str)
                            .unwrap_or("pending")
                            .to_string(),
                    ))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    if let Some(criteria) = mission.get("acceptance_criteria").and_then(Value::as_array) {
        let mut met_count = 0;
        snapshot.detail_acceptance_rows = criteria
            .iter()
            .enumerate()
            .map(|(index, criterion)| {
                let item_id = criterion
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("AC-{}", index + 1));
                let gate_ids = criterion
                    .get("verification_gate_ids")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let has_failed_gate = gate_ids.iter().any(|gate_id| {
                    gate_id
                        .as_str()
                        .and_then(|id| gate_statuses.get(id))
                        .is_some_and(|gate_status| gate_status == "failed")
                });
                let all_gates_passed = !gate_ids.is_empty()
                    && gate_ids.iter().all(|gate_id| {
                        gate_id
                            .as_str()
                            .and_then(|id| gate_statuses.get(id))
                            .is_some_and(|gate_status| gate_status == "passed")
                    });
                let status_key = if all_gates_passed {
                    met_count += 1;
                    "met"
                } else if has_failed_gate {
                    "failed"
                } else {
                    "pending"
                };
                TauOpsDashboardHarnessProofRow {
                    item_id,
                    status_key: status_key.to_string(),
                    label: criterion
                        .get("description")
                        .or_else(|| criterion.get("criterion"))
                        .or_else(|| criterion.get("summary"))
                        .and_then(Value::as_str)
                        .unwrap_or("Mission acceptance criterion")
                        .to_string(),
                }
            })
            .collect();
        snapshot.detail_acceptance_met_count = met_count;
        snapshot.detail_acceptance_total_count = snapshot.detail_acceptance_rows.len();
    }

    if let Some(gates) = mission.get("verification_gates").and_then(Value::as_array) {
        snapshot.detail_gate_rows = gates
            .iter()
            .filter_map(|gate| {
                let item_id = gate.get("id").and_then(Value::as_str)?;
                Some(TauOpsDashboardHarnessProofRow {
                    item_id: item_id.to_string(),
                    status_key: harness_detail_status_key(
                        gate.get("status")
                            .and_then(Value::as_str)
                            .unwrap_or("pending"),
                    ),
                    label: gate
                        .get("description")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .unwrap_or_else(|| humanize_harness_token(item_id)),
                })
            })
            .collect();
        snapshot.detail_gate_failed_count = snapshot
            .detail_gate_rows
            .iter()
            .filter(|row| row.status_key == "failed")
            .count();
    }

    snapshot.detail_memory_hit_count = mission
        .get("memory_hits")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default();
    let learning_record_count = mission
        .get("learning_records")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default()
        + mission
            .get("final_learning_output")
            .and_then(|output| output.get("records"))
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or_default();
    snapshot.detail_learning_record_count = learning_record_count;
    snapshot.detail_last_memory_write = if updated_unix_ms == 0 {
        "state".to_string()
    } else {
        format_harness_audit_timestamp(updated_unix_ms)
    };
    snapshot.detail_memory_evidence_label = mission
        .get("memory_recall")
        .and_then(|recall| recall.get("status"))
        .and_then(Value::as_str)
        .map(humanize_harness_token)
        .unwrap_or_else(|| format!("{} mission memory hits", snapshot.detail_memory_hit_count));

    if let Some(artifacts) = mission.get("artifacts").and_then(Value::as_array) {
        snapshot.detail_artifact_rows = artifacts
            .iter()
            .filter_map(|artifact| {
                let artifact_id = artifact.get("artifact_id").and_then(Value::as_str)?;
                let href = artifact
                    .get("path")
                    .and_then(Value::as_str)
                    .unwrap_or_else(|| mission_path.to_str().unwrap_or("mission.json"));
                Some(TauOpsDashboardHarnessArtifactRow {
                    item_id: artifact_id.to_string(),
                    status_key: artifact
                        .get("kind")
                        .and_then(Value::as_str)
                        .unwrap_or("artifact")
                        .to_string(),
                    label: artifact
                        .get("summary")
                        .or_else(|| artifact.get("path"))
                        .and_then(Value::as_str)
                        .unwrap_or(artifact_id)
                        .to_string(),
                    href: harness_state_artifact_href(href),
                })
            })
            .collect();
    }
    if snapshot.detail_artifact_rows.is_empty() {
        snapshot.detail_artifact_rows = vec![TauOpsDashboardHarnessArtifactRow {
            item_id: "mission-json".to_string(),
            status_key: "mission-state".to_string(),
            label: "Mission JSON".to_string(),
            href: harness_state_artifact_href_for_path(state_dir, &mission_path),
        }];
    }

    let passed_gate_count = snapshot
        .detail_gate_rows
        .iter()
        .filter(|row| row.status_key == "passed")
        .count();
    let gate_total = snapshot.detail_gate_rows.len();
    snapshot.kpi_memory_write_count = snapshot.detail_learning_record_count;
    snapshot.kpi_memory_write_detail = "learning outputs".to_string();
    snapshot.kpi_runtime_cost_today = snapshot.detail_cost.clone();
    snapshot.kpi_runtime_cost_detail = "Selected mission".to_string();
    snapshot.detail_operator_log = format!(
        "mission state loaded: {}\nmission_id: {}\nstatus: {}\nverification gates: {}/{} passed\ntool evidence: {} calls\nmemory hits: {}\nlearning records: {}\nfinal learning: {}",
        mission_path.display(),
        snapshot.detail_run_id,
        snapshot.detail_status,
        passed_gate_count,
        gate_total,
        snapshot.detail_tool_call_count,
        snapshot.detail_memory_hit_count,
        snapshot.detail_learning_record_count,
        mission
            .get("final_learning_output")
            .and_then(|output| output.get("summary"))
            .or_else(|| mission.get("latest_output_summary"))
            .and_then(Value::as_str)
            .unwrap_or("not written yet"),
    );

    true
}

fn harness_detail_status_key(status: &str) -> String {
    match status {
        "completed" | "succeeded" | "success" | "passed" => "passed".to_string(),
        "failed" | "failure" => "failed".to_string(),
        "running" | "in_progress" => "running".to_string(),
        _ => "pending".to_string(),
    }
}

fn compact_harness_call_id(call_id: &str) -> String {
    call_id
        .rsplit_once("-tool-")
        .map(|(_, suffix)| format!("tool-{suffix}"))
        .unwrap_or_else(|| call_id.to_string())
}

fn format_harness_runtime_ms(runtime_ms: u64) -> String {
    if runtime_ms < 1000 {
        return format!("{runtime_ms}ms");
    }
    let total_seconds = runtime_ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}

fn format_harness_audit_timestamp(timestamp_unix_ms: u64) -> String {
    if timestamp_unix_ms == 0 {
        return "unknown time".to_string();
    }

    let seconds = timestamp_unix_ms / 1_000;
    let days = seconds / 86_400;
    let seconds_of_day = seconds % 86_400;
    let (year, month, day) = utc_date_from_unix_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} UTC")
}

fn utc_date_from_unix_days(days_since_epoch: u64) -> (i64, u64, u64) {
    let mut days = match i64::try_from(days_since_epoch) {
        Ok(days) => days,
        Err(_) => return (9999, 12, 31),
    };
    days += 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    (year, month as u64, day as u64)
}

fn collect_harness_self_improvement_proof(
    state_dir: &Path,
    proposal: &GatewayOpsHarnessProposalDefinition,
) -> TauOpsDashboardHarnessSelfImprovementProof {
    let mission_path = state_dir
        .join("ops-harness")
        .join("self-improvement")
        .join(proposal.proposal_id)
        .join("mission.json");
    let Ok(mission_json) = std::fs::read_to_string(&mission_path) else {
        return TauOpsDashboardHarnessSelfImprovementProof::default();
    };
    let Ok(mission) = serde_json::from_str::<Value>(&mission_json) else {
        return TauOpsDashboardHarnessSelfImprovementProof::default();
    };

    let plan_rows = mission
        .get("plan_dag")
        .and_then(Value::as_array)
        .map(|nodes| {
            nodes
                .iter()
                .filter_map(|node| {
                    let item_id = node.get("id").and_then(Value::as_str)?;
                    Some(TauOpsDashboardHarnessProofRow {
                        item_id: item_id.to_string(),
                        status_key: node
                            .get("status")
                            .and_then(Value::as_str)
                            .unwrap_or("pending")
                            .to_string(),
                        label: node
                            .get("description")
                            .and_then(Value::as_str)
                            .unwrap_or(item_id)
                            .to_string(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let gate_rows = mission
        .get("verification_gates")
        .and_then(Value::as_array)
        .map(|gates| {
            gates
                .iter()
                .filter_map(|gate| {
                    let item_id = gate.get("id").and_then(Value::as_str)?;
                    Some(TauOpsDashboardHarnessProofRow {
                        item_id: item_id.to_string(),
                        status_key: gate
                            .get("status")
                            .and_then(Value::as_str)
                            .unwrap_or("pending")
                            .to_string(),
                        label: gate
                            .get("description")
                            .and_then(Value::as_str)
                            .unwrap_or(item_id)
                            .to_string(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let artifact_rows = mission
        .get("artifacts")
        .and_then(Value::as_array)
        .map(|artifacts| {
            artifacts
                .iter()
                .filter_map(|artifact| {
                    let item_id = artifact.get("artifact_id").and_then(Value::as_str)?;
                    Some(TauOpsDashboardHarnessProofRow {
                        item_id: item_id.to_string(),
                        status_key: artifact
                            .get("kind")
                            .and_then(Value::as_str)
                            .unwrap_or("artifact")
                            .to_string(),
                        label: artifact
                            .get("path")
                            .and_then(Value::as_str)
                            .or_else(|| artifact.get("summary").and_then(Value::as_str))
                            .unwrap_or(item_id)
                            .to_string(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let final_learning_output = mission.get("final_learning_output");
    let final_learning_records = final_learning_output
        .and_then(|output| output.get("records"))
        .and_then(Value::as_array)
        .map(|records| {
            records
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    TauOpsDashboardHarnessSelfImprovementProof {
        source: "state".to_string(),
        mission_id: mission
            .get("mission_id")
            .and_then(Value::as_str)
            .unwrap_or(proposal.mission_id)
            .to_string(),
        mission_status: mission
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        plan_completed_count: plan_rows
            .iter()
            .filter(|row| row.status_key == "completed")
            .count(),
        plan_total_count: plan_rows.len(),
        gate_passed_count: gate_rows
            .iter()
            .filter(|row| row.status_key == "passed")
            .count(),
        gate_total_count: gate_rows.len(),
        memory_hit_count: mission
            .get("memory_hits")
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or_default(),
        artifact_count: artifact_rows.len(),
        final_learning_summary: final_learning_output
            .and_then(|output| output.get("summary"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        final_learning_records,
        plan_rows,
        gate_rows,
        artifact_rows,
    }
}

pub(super) async fn handle_ops_dashboard_harness_run_benchmark(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    Query(controls): Query<OpsShellControlsQuery>,
) -> Response {
    let fixture_path = canonical_harness_fixture_path();
    let artifact_dir = harness_artifact_dir(&state.config.state_dir);
    let proof_path = artifact_dir.join("latest.json");
    let session_key = controls
        .requested_session_key()
        .map(sanitize_session_key)
        .unwrap_or_else(|| "default".to_string());
    let proposal_id = controls
        .requested_harness_proposal_id()
        .map(sanitize_harness_token)
        .filter(|proposal_id| find_ops_harness_proposal(proposal_id).is_some())
        .or_else(|| {
            list_ops_harness_proposals()
                .first()
                .map(|proposal| proposal.proposal_id.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());
    let memory_root = gateway_memory_store_root(&state.config.state_dir, session_key.as_str());
    let run_id = format!("gateway-harness-{}", now_unix_ms());
    let proof_artifact = "ops-harness/m334/latest.json";

    let redirect_path = match load_autonomy_benchmark_fixture(&fixture_path).and_then(|fixture| {
        run_autonomy_benchmark_fixture(
            &fixture,
            &MissionHarnessConfig {
                run_id: run_id.clone(),
                started_unix_ms: now_unix_ms(),
                memory_root,
                workspace_id: session_key.clone(),
            },
        )
    }) {
        Ok(proof) => {
            let write_result = std::fs::create_dir_all(&artifact_dir)
                .and_then(|()| serde_json::to_vec_pretty(&proof).map_err(std::io::Error::other))
                .and_then(|payload| std::fs::write(&proof_path, payload));
            if write_result.is_ok() {
                let status = if proof.passed { "passed" } else { "failed" };
                let task_count = proof.tasks.len();
                append_harness_audit_record_with_fields(
                    &state.config.state_dir,
                    proposal_id.as_str(),
                    "run-benchmark",
                    status,
                    &[
                        ("benchmark_id", proof.benchmark_id.as_str()),
                        ("run_id", proof.run_id.as_str()),
                        ("proof_artifact", proof_artifact),
                    ],
                );
                format!(
                    "{}&benchmark_tasks={task_count}",
                    build_ops_harness_redirect_path(
                        controls.theme(),
                        controls.sidebar_state(),
                        session_key.as_str(),
                        proposal_id.as_str(),
                        status,
                        "benchmark_status",
                    )
                )
            } else {
                append_harness_audit_record_with_fields(
                    &state.config.state_dir,
                    proposal_id.as_str(),
                    "run-benchmark",
                    "artifact_write_failed",
                    &[
                        ("benchmark_id", proof.benchmark_id.as_str()),
                        ("run_id", proof.run_id.as_str()),
                        ("proof_artifact", proof_artifact),
                    ],
                );
                build_ops_harness_redirect_path(
                    controls.theme(),
                    controls.sidebar_state(),
                    session_key.as_str(),
                    proposal_id.as_str(),
                    "artifact_write_failed",
                    "benchmark_status",
                )
            }
        }
        Err(_) => {
            append_harness_audit_record_with_fields(
                &state.config.state_dir,
                proposal_id.as_str(),
                "run-benchmark",
                "failed",
                &[
                    ("benchmark_id", "m334-tranche-one-autonomy"),
                    ("run_id", run_id.as_str()),
                    ("proof_artifact", proof_artifact),
                ],
            );
            build_ops_harness_redirect_path(
                controls.theme(),
                controls.sidebar_state(),
                session_key.as_str(),
                proposal_id.as_str(),
                "failed",
                "benchmark_status",
            )
        }
    };

    Redirect::to(redirect_path.as_str()).into_response()
}

pub(super) async fn handle_ops_dashboard_harness_artifact(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    AxumPath(artifact_path): AxumPath<String>,
) -> Response {
    let Some((absolute_path, payload)) = read_harness_state_artifact(&state, &artifact_path) else {
        return OpenResponsesApiError::not_found(
            "harness_artifact_not_found",
            "harness artifact could not be read",
        )
        .into_response();
    };
    let content_type = match absolute_path
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("json") => "application/json; charset=utf-8",
        Some("md") => "text/markdown; charset=utf-8",
        Some("txt" | "log") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    };
    (StatusCode::OK, [("content-type", content_type)], payload).into_response()
}

pub(super) async fn handle_ops_dashboard_harness_artifact_view(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    AxumPath(artifact_path): AxumPath<String>,
) -> Response {
    let Some((_absolute_path, payload)) = read_harness_state_artifact(&state, &artifact_path)
    else {
        return OpenResponsesApiError::not_found(
            "harness_artifact_not_found",
            "harness artifact could not be read",
        )
        .into_response();
    };
    let escaped_path = escape_harness_artifact_html(&artifact_path);
    let raw_href = format!("/ops/harness/artifacts/{escaped_path}");
    let escaped_payload = escape_harness_artifact_html(&payload);
    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Tau Harness Artifact - {escaped_path}</title>
  <style>
    body {{ margin: 0; background: #07131d; color: #d9e7ef; font: 14px/1.5 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }}
    main {{ padding: 24px; }}
    a {{ color: #74b9ff; }}
    pre {{ white-space: pre-wrap; overflow-wrap: anywhere; background: #0d1c28; border: 1px solid #263b4b; border-radius: 6px; padding: 16px; }}
  </style>
</head>
<body>
  <main id="tau-ops-harness-artifact-view" data-artifact-path="{escaped_path}">
    <p><a href="{raw_href}">Raw artifact</a></p>
    <pre>{escaped_payload}</pre>
  </main>
</body>
</html>"#
    ))
    .into_response()
}

pub(super) async fn handle_ops_dashboard_harness_create_mission_draft(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    Query(controls): Query<OpsShellControlsQuery>,
) -> Response {
    let session_key = controls
        .requested_session_key()
        .map(sanitize_session_key)
        .unwrap_or_else(|| DEFAULT_SESSION_KEY.to_string());
    let selected_proposal = controls
        .requested_harness_proposal_id()
        .and_then(find_ops_harness_proposal)
        .or_else(|| list_ops_harness_proposals().first());
    let proposal_id = selected_proposal
        .map(|proposal| proposal.proposal_id)
        .unwrap_or("unknown");
    let proposal_title = selected_proposal
        .map(|proposal| proposal.title)
        .unwrap_or("Operator defined harness mission");
    let proposal_goal = selected_proposal
        .map(|proposal| proposal.goal)
        .unwrap_or("Define, plan, execute, verify, and learn from an autonomous harness mission.");
    let now = now_unix_ms();
    let mission_id = format!("mission-draft-{now}");
    let relative_mission_path = format!("ops-harness/missions/{mission_id}/mission.json");
    let mission_dir = state
        .config
        .state_dir
        .join("ops-harness")
        .join("missions")
        .join(&mission_id);
    let mission_path = mission_dir.join("mission.json");
    let plan_node_ids = [
        "clarify-goal",
        "write-plan-dag",
        "execute-with-budget",
        "verify-gates",
        "write-final-learning",
    ];
    let mission = json!({
        "schema_version": 1,
        "mission_id": mission_id.as_str(),
        "session_key": session_key.as_str(),
        "proposal_id": proposal_id,
        "response_id": null,
        "goal": format!("{proposal_id} {proposal_title}: {proposal_goal}"),
        "latest_output_summary": "Draft mission created from the Tau harness New Mission action.",
        "status": "draft",
        "created_unix_ms": now,
        "updated_unix_ms": now,
        "acceptance_criteria": [
            {
                "id": "AC-PLAN",
                "description": "Mission has a goal, acceptance criteria, plan DAG, and tool budget before execution.",
                "verification_gate_ids": ["VG-PLAN"]
            },
            {
                "id": "AC-EXECUTE",
                "description": "Mission records tool execution, memory recall, checkpoints, artifacts, and recovery state.",
                "verification_gate_ids": ["VG-EXECUTE"]
            },
            {
                "id": "AC-LEARN",
                "description": "Mission ends with verification evidence and final learning output for curator review.",
                "verification_gate_ids": ["VG-LEARN"]
            }
        ],
        "plan_dag": [
            {
                "id": "clarify-goal",
                "description": "Normalize the selected proposal into a durable mission goal and constraints.",
                "depends_on": [],
                "status": "pending"
            },
            {
                "id": "write-plan-dag",
                "description": "Expand acceptance criteria into executable plan nodes and verification gates.",
                "depends_on": ["clarify-goal"],
                "status": "pending"
            },
            {
                "id": "execute-with-budget",
                "description": "Run the mission using approved tools within the configured budget.",
                "depends_on": ["write-plan-dag"],
                "status": "pending"
            },
            {
                "id": "verify-gates",
                "description": "Collect gate evidence, artifacts, memory writes, and recovery state before completion.",
                "depends_on": ["execute-with-budget"],
                "status": "pending"
            },
            {
                "id": "write-final-learning",
                "description": "Write the mission learning output and curator update proposal.",
                "depends_on": ["verify-gates"],
                "status": "pending"
            }
        ],
        "tool_budget": {
            "allowed_tools": ["repo.read", "memory.search", "tool.execute", "test.run", "report.write"],
            "max_tool_calls": 40,
            "max_runtime_ms": 1_800_000,
            "max_cost_usd": 12.0,
            "consumed_tool_calls": 0,
            "consumed_runtime_ms": 0,
            "consumed_cost_usd": null
        },
        "tool_evidence": [],
        "memory_hits": [],
        "memory_recall": {
            "query": format!("proposal:{proposal_id} session:{session_key} harness mission draft"),
            "status": "no_relevant_memory",
            "checked_unix_ms": now,
            "rationale": "Draft created before mission-specific memory retrieval executes.",
            "hit_keys": []
        },
        "verification_gates": [
            {
                "id": "VG-PLAN",
                "description": "Plan DAG and acceptance criteria are present.",
                "status": null,
                "evidence": {}
            },
            {
                "id": "VG-EXECUTE",
                "description": "Tool execution and artifacts are captured within budget.",
                "status": null,
                "evidence": {}
            },
            {
                "id": "VG-LEARN",
                "description": "Final learning output is written for curator review.",
                "status": null,
                "evidence": {}
            }
        ],
        "checkpoints": [
            {
                "checkpoint_id": "draft-created",
                "summary": "Draft mission saved before execution.",
                "created_unix_ms": now,
                "pending_plan_node_ids": plan_node_ids
            }
        ],
        "recovery_state": null,
        "artifacts": [
            {
                "artifact_id": "mission-json",
                "kind": "mission-state",
                "path": relative_mission_path,
                "summary": "Durable mission draft state."
            }
        ],
        "final_learning_output": null,
        "learning_records": [],
        "improvement_proposals": [],
        "iteration_count": 0,
        "latest_verifier": null,
        "latest_completion": null
    });

    let mission_status = std::fs::create_dir_all(&mission_dir)
        .and_then(|()| serde_json::to_vec_pretty(&mission).map_err(std::io::Error::other))
        .and_then(|payload| std::fs::write(&mission_path, payload))
        .map(|()| "draft_created")
        .unwrap_or("write_failed");
    let redirect_path = format!(
        "{}&mission_id={}",
        build_ops_harness_redirect_path(
            controls.theme(),
            controls.sidebar_state(),
            session_key.as_str(),
            proposal_id,
            mission_status,
            "mission_status",
        ),
        sanitize_harness_token(&mission_id)
    );
    Redirect::to(redirect_path.as_str()).into_response()
}

pub(super) async fn handle_ops_dashboard_harness_start_mission(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    AxumPath(mission_id): AxumPath<String>,
    Query(controls): Query<OpsShellControlsQuery>,
) -> Response {
    let mission_id = sanitize_harness_token(&mission_id);
    let session_key = controls
        .requested_session_key()
        .map(sanitize_session_key)
        .unwrap_or_else(|| DEFAULT_SESSION_KEY.to_string());
    let mission_path = state
        .config
        .state_dir
        .join("ops-harness")
        .join("missions")
        .join(&mission_id)
        .join("mission.json");
    let selected_proposal_id = match std::fs::read_to_string(&mission_path)
        .ok()
        .and_then(|mission_json| serde_json::from_str::<Value>(&mission_json).ok())
        .and_then(|mission| resolve_harness_mission_proposal_id(&mission, &controls))
    {
        Some(proposal_id) => proposal_id,
        None => controls
            .requested_harness_proposal_id()
            .map(sanitize_harness_token)
            .unwrap_or_else(|| "unknown".to_string()),
    };

    let mut mission = match std::fs::read_to_string(&mission_path)
        .ok()
        .and_then(|mission_json| serde_json::from_str::<Value>(&mission_json).ok())
    {
        Some(mission) => mission,
        None => {
            let redirect_path = format!(
                "{}&mission_id={mission_id}",
                build_ops_harness_redirect_path(
                    controls.theme(),
                    controls.sidebar_state(),
                    session_key.as_str(),
                    selected_proposal_id.as_str(),
                    "start_failed",
                    "mission_status",
                )
            );
            return Redirect::to(redirect_path.as_str()).into_response();
        }
    };

    let Some(proposal_id) = resolve_harness_mission_proposal_id(&mission, &controls) else {
        record_harness_mission_start_failure(
            &mut mission,
            "Mission is missing a known proposal id.",
            now_unix_ms(),
        );
        let _ = serde_json::to_vec_pretty(&mission)
            .map_err(std::io::Error::other)
            .and_then(|payload| std::fs::write(&mission_path, payload));
        let redirect_path = format!(
            "{}&mission_id={mission_id}",
            build_ops_harness_redirect_path(
                controls.theme(),
                controls.sidebar_state(),
                session_key.as_str(),
                selected_proposal_id.as_str(),
                "start_failed",
                "mission_status",
            )
        );
        return Redirect::to(redirect_path.as_str()).into_response();
    };

    let (mission_status, audit_result) = if let Some((result, final_learning_output)) =
        read_completed_harness_self_improvement_result(&state.config.state_dir, &proposal_id)
    {
        record_harness_mission_start_result(
            &mut mission,
            &proposal_id,
            &result,
            now_unix_ms(),
            true,
            final_learning_output,
        );
        ("mission_completed", "completed".to_string())
    } else {
        let request = build_harness_self_improvement_request(&state.config.state_dir, &proposal_id);
        match state.config.ops_harness_self_improvement.dry_run(request) {
            Ok(result) => {
                let passed = result.result_key == "passed";
                record_harness_mission_start_result(
                    &mut mission,
                    &proposal_id,
                    &result,
                    now_unix_ms(),
                    false,
                    None,
                );
                if passed {
                    ("mission_started", result.result_key)
                } else {
                    ("mission_blocked", result.result_key)
                }
            }
            Err(error) => {
                let error_summary = error.to_string();
                record_harness_mission_start_failure(&mut mission, &error_summary, now_unix_ms());
                ("start_failed", "failed".to_string())
            }
        }
    };

    let write_status = serde_json::to_vec_pretty(&mission)
        .map_err(std::io::Error::other)
        .and_then(|payload| std::fs::write(&mission_path, payload))
        .map(|()| mission_status)
        .unwrap_or("start_failed");
    let mission_proof_artifact = format!("ops-harness/missions/{mission_id}/mission.json");
    append_harness_audit_record_with_fields(
        &state.config.state_dir,
        &proposal_id,
        "start-mission",
        &audit_result,
        &[
            ("mission_id", mission_id.as_str()),
            ("proof_artifact", mission_proof_artifact.as_str()),
        ],
    );

    let redirect_path = format!(
        "{}&mission_id={mission_id}",
        build_ops_harness_redirect_path(
            controls.theme(),
            controls.sidebar_state(),
            session_key.as_str(),
            proposal_id.as_str(),
            write_status,
            "mission_status",
        )
    );
    Redirect::to(redirect_path.as_str()).into_response()
}

pub(super) async fn handle_ops_dashboard_harness_proposal_action(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    AxumPath((proposal_id, action)): AxumPath<(String, String)>,
    Query(controls): Query<OpsShellControlsQuery>,
) -> Response {
    let proposal_id = sanitize_harness_token(&proposal_id);
    let action = sanitize_harness_token(&action);
    if find_ops_harness_proposal(&proposal_id).is_none() {
        return render_harness_action_error(
            StatusCode::NOT_FOUND,
            &proposal_id,
            &action,
            "unknown_proposal",
            "Harness proposal is not registered.",
        );
    }

    let status = match action.as_str() {
        "approve" => {
            append_harness_audit_record(
                &state.config.state_dir,
                &proposal_id,
                "approve",
                "recorded",
            );
            "approved"
        }
        "reject" => {
            append_harness_audit_record(
                &state.config.state_dir,
                &proposal_id,
                "reject",
                "recorded",
            );
            "rejected"
        }
        "dry-run" => {
            let request =
                build_harness_self_improvement_request(&state.config.state_dir, &proposal_id);
            match state.config.ops_harness_self_improvement.dry_run(request) {
                Ok(result) => {
                    let proof_artifact = harness_self_improvement_result_proof_artifact(
                        &state.config.state_dir,
                        &proposal_id,
                        "dry-run",
                        &result,
                    );
                    let fields = proof_artifact
                        .as_deref()
                        .map(|artifact| vec![("proof_artifact", artifact)])
                        .unwrap_or_default();
                    append_harness_audit_record_with_fields(
                        &state.config.state_dir,
                        &proposal_id,
                        "dry-run",
                        result.result_key.as_str(),
                        fields.as_slice(),
                    );
                    if result.result_key == "passed" {
                        "dry_run_passed"
                    } else {
                        "dry_run_failed"
                    }
                }
                Err(error) => {
                    append_harness_audit_record(
                        &state.config.state_dir,
                        &proposal_id,
                        "dry-run",
                        "runner_unavailable",
                    );
                    return render_harness_action_error(
                        StatusCode::FAILED_DEPENDENCY,
                        &proposal_id,
                        "dry-run",
                        "runner_unavailable",
                        error.to_string().as_str(),
                    );
                }
            }
        }
        "apply" => {
            if !harness_latest_audit_action_is_approved(&state.config.state_dir, &proposal_id) {
                append_harness_audit_record(
                    &state.config.state_dir,
                    &proposal_id,
                    "apply",
                    "blocked_approval_required",
                );
                return (
                    StatusCode::FORBIDDEN,
                    Html(format!(
                        r#"<main id="tau-ops-harness-apply-blocked" data-proposal-id="{proposal_id}" data-result="blocked_approval_required">
<h1>Apply Requires Approval</h1>
<p>Harness self-improvement apply is intentionally approval-gated.</p>
<a href="/ops/harness">Back to Mission Harness</a>
</main>"#
                    )),
                )
                    .into_response();
            }

            let request =
                build_harness_self_improvement_request(&state.config.state_dir, &proposal_id);
            match state.config.ops_harness_self_improvement.apply(request) {
                Ok(result) => {
                    let proof_artifact = harness_self_improvement_result_proof_artifact(
                        &state.config.state_dir,
                        &proposal_id,
                        "apply",
                        &result,
                    );
                    let fields = proof_artifact
                        .as_deref()
                        .map(|artifact| vec![("proof_artifact", artifact)])
                        .unwrap_or_default();
                    append_harness_audit_record_with_fields(
                        &state.config.state_dir,
                        &proposal_id,
                        "apply",
                        result.result_key.as_str(),
                        fields.as_slice(),
                    );
                    if result.result_key == "applied" {
                        "applied"
                    } else {
                        "apply_failed"
                    }
                }
                Err(error) => {
                    append_harness_audit_record(
                        &state.config.state_dir,
                        &proposal_id,
                        "apply",
                        "runner_failed",
                    );
                    return render_harness_action_error(
                        StatusCode::FAILED_DEPENDENCY,
                        &proposal_id,
                        "apply",
                        "runner_failed",
                        error.to_string().as_str(),
                    );
                }
            }
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Html(format!(
                    r#"<main id="tau-ops-harness-action-invalid" data-proposal-id="{proposal_id}" data-action="{action}" data-result="invalid_action">
<h1>Invalid Harness Action</h1>
<p>Supported proposal actions are approve, reject, dry-run, and apply.</p>
<a href="/ops/harness">Back to Mission Harness</a>
</main>"#
                )),
            )
                .into_response();
        }
    };
    let redirect_session_key = controls
        .requested_session_key()
        .map(sanitize_session_key)
        .unwrap_or_else(|| DEFAULT_SESSION_KEY.to_string());
    let redirect_path = build_ops_harness_redirect_path(
        controls.theme(),
        controls.sidebar_state(),
        redirect_session_key.as_str(),
        proposal_id.as_str(),
        status,
        "proposal_status",
    );
    Redirect::to(redirect_path.as_str()).into_response()
}

fn build_harness_self_improvement_request(
    state_dir: &Path,
    proposal_id: &str,
) -> GatewayOpsHarnessSelfImprovementRequest {
    GatewayOpsHarnessSelfImprovementRequest {
        proposal_id: proposal_id.to_string(),
        state_dir: state_dir.to_path_buf(),
        workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        requested_unix_ms: now_unix_ms(),
    }
}

fn harness_self_improvement_result_proof_artifact(
    state_dir: &Path,
    proposal_id: &str,
    action: &str,
    result: &GatewayOpsHarnessSelfImprovementResult,
) -> Option<String> {
    let result_file_name = match action {
        "apply" => "apply-result.json",
        "dry-run" => "dry-run-result.json",
        _ => "",
    };
    if !result_file_name.is_empty() {
        let state_artifact_path = format!(
            "ops-harness/self-improvement/{}/{}",
            sanitize_harness_token(proposal_id),
            result_file_name
        );
        if state_dir.join(&state_artifact_path).is_file() {
            return Some(state_artifact_path);
        }
    }
    result
        .artifact_path
        .as_deref()
        .and_then(|path| harness_state_artifact_relative_path_for_path(state_dir, path))
}

fn harness_state_artifact_relative_path_for_path(
    state_dir: &Path,
    artifact_path: &Path,
) -> Option<String> {
    artifact_path
        .strip_prefix(state_dir)
        .ok()
        .and_then(Path::to_str)
        .and_then(normalize_harness_state_artifact_path)
        .and_then(|path| path.to_str().map(str::to_string))
}

fn render_harness_action_error(
    status: StatusCode,
    proposal_id: &str,
    action: &str,
    result: &str,
    message: &str,
) -> Response {
    (
        status,
        Html(format!(
            r#"<main id="tau-ops-harness-action-error" data-proposal-id="{proposal_id}" data-action="{action}" data-result="{result}">
<h1>Harness Action Failed</h1>
<p>{message}</p>
<a href="/ops/harness">Back to Mission Harness</a>
</main>"#
        )),
    )
        .into_response()
}

pub(super) async fn handle_ops_dashboard_harness_proposal_diff(
    AxumPath(proposal_id): AxumPath<String>,
    Query(controls): Query<OpsShellControlsQuery>,
) -> Response {
    let proposal_id = sanitize_harness_token(&proposal_id);
    let session_key = controls
        .requested_session_key()
        .map(sanitize_session_key)
        .unwrap_or_else(|| DEFAULT_SESSION_KEY.to_string());
    let back_href = build_ops_harness_context_href(
        controls.theme(),
        controls.sidebar_state(),
        session_key.as_str(),
        proposal_id.as_str(),
    );
    let Some(proposal) = find_ops_harness_proposal(&proposal_id) else {
        return (
            StatusCode::NOT_FOUND,
            Html(format!(
                r#"<main id="tau-ops-harness-diff-missing" data-proposal-id="{proposal_id}" data-result="unknown_proposal">
<h1>Harness Proposal Not Found</h1>
<p>The requested proposal is not registered.</p>
<a href="{back_href}">Back to Mission Harness</a>
</main>"#
            )),
        )
            .into_response();
    };
    let diff_lines = harness_proposal_diff_lines(proposal);
    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Harness Proposal Diff {proposal_id}</title>
<style>
:root {{
  color-scheme: dark;
  --bg: #061017;
  --panel: #0d1b25;
  --panel-2: #101f2c;
  --line: #264154;
  --text: #e7f0f6;
  --muted: #9fb4c2;
  --green: #56d075;
  --red: #ee746c;
  --blue: #66a6ff;
}}
* {{ box-sizing: border-box; }}
body {{
  margin: 0;
  min-height: 100vh;
  background: var(--bg);
  color: var(--text);
  font: 14px/1.45 Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}}
a {{ color: var(--blue); }}
#tau-ops-harness-diff {{
  width: min(1120px, calc(100vw - 32px));
  margin: 24px auto;
  display: grid;
  gap: 16px;
}}
.tau-harness-diff-header,
.tau-harness-diff-card {{
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--panel);
  box-shadow: 0 18px 48px rgba(0, 0, 0, .28);
}}
.tau-harness-diff-header {{
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 18px 20px;
}}
.tau-harness-diff-header p,
.tau-harness-diff-card p {{
  margin: 0;
  color: var(--muted);
}}
.tau-harness-diff-header h1 {{
  margin: 3px 0 0;
  font-size: 1.35rem;
  letter-spacing: 0;
}}
.tau-harness-diff-card {{
  padding: 18px 20px;
}}
.tau-harness-diff-meta {{
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
}}
.tau-harness-diff-meta div {{
  min-width: 0;
  padding: 10px 12px;
  border: 1px solid var(--line);
  border-radius: 6px;
  background: var(--panel-2);
}}
.tau-harness-diff-meta dt {{
  margin: 0 0 4px;
  color: var(--muted);
  font-size: .72rem;
  font-weight: 700;
  text-transform: uppercase;
}}
.tau-harness-diff-meta dd {{
  margin: 0;
  overflow-wrap: anywhere;
  font-weight: 700;
}}
.tau-harness-diff-policy {{
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  margin-top: 14px;
}}
.tau-harness-diff-policy section {{
  border: 1px solid var(--line);
  border-radius: 6px;
  padding: 12px;
  background: var(--panel-2);
}}
.tau-harness-diff-policy h2 {{
  margin: 0 0 8px;
  font-size: .86rem;
}}
.tau-harness-diff-chips {{
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}}
.tau-harness-diff-chip {{
  border: 1px solid var(--line);
  border-radius: 999px;
  padding: 3px 8px;
  font-size: .78rem;
  font-weight: 700;
}}
.tau-harness-diff-chip[data-scope="allowed"] {{ color: var(--green); }}
.tau-harness-diff-chip[data-scope="blocked"] {{ color: var(--red); }}
.tau-harness-diff-code {{
  margin: 0;
  max-height: 54vh;
  overflow: auto;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: #050b10;
  color: var(--text);
  font: 13px/1.5 "SFMono-Regular", Consolas, "Liberation Mono", monospace;
}}
.tau-harness-diff-line {{
  display: block;
  min-width: max-content;
  padding: 0 14px;
  white-space: pre;
}}
.tau-harness-diff-line:first-child {{ padding-top: 12px; }}
.tau-harness-diff-line:last-child {{ padding-bottom: 12px; }}
.tau-harness-diff-line-remove {{
  background: rgba(238, 116, 108, .14);
  color: #ffada8;
}}
.tau-harness-diff-line-add {{
  background: rgba(86, 208, 117, .13);
  color: #b6f0bf;
}}
.tau-harness-diff-actions {{
  display: flex;
  justify-content: flex-start;
}}
.tau-harness-diff-actions a {{
  display: inline-flex;
  min-height: 36px;
  align-items: center;
  border: 1px solid var(--line);
  border-radius: 6px;
  padding: 0 12px;
  background: var(--panel-2);
  color: var(--text);
  font-weight: 700;
  text-decoration: none;
}}
@media (max-width: 760px) {{
  #tau-ops-harness-diff {{ width: calc(100vw - 20px); margin: 10px auto; }}
  .tau-harness-diff-header,
  .tau-harness-diff-policy {{ grid-template-columns: minmax(0, 1fr); }}
  .tau-harness-diff-header {{ display: grid; }}
  .tau-harness-diff-meta {{ grid-template-columns: minmax(0, 1fr); }}
}}
</style>
</head>
<body>
<main id="tau-ops-harness-diff" data-proposal-id="{proposal_id}" data-diff-view="operator-review" data-target-path="{target_path}" data-dry-run-result="{dry_run_result_key}" data-safety-check="{safety_check_key}" data-policy-allowed="skill,config,prompt" data-policy-blocked="source-code,safety-policy">
  <header class="tau-harness-diff-header">
    <div>
      <p>Harness Proposal Diff</p>
      <h1>{proposal_id} {title}</h1>
    </div>
    <p>Operator review required before apply.</p>
  </header>
  <section class="tau-harness-diff-card" aria-labelledby="tau-harness-diff-summary">
    <h2 id="tau-harness-diff-summary">Change Summary</h2>
    <p>{patch_summary}</p>
    <dl class="tau-harness-diff-meta">
      <div><dt>Target Path</dt><dd>{target_path}</dd></div>
      <div><dt>Dry-run Result</dt><dd>{dry_run_result_label}</dd></div>
      <div><dt>Safety Check</dt><dd>{safety_check_label}</dd></div>
      <div><dt>Rollback Plan</dt><dd>{rollback_plan}</dd></div>
    </dl>
    <div class="tau-harness-diff-policy">
      <section>
        <h2>Allowed Scope</h2>
        <div class="tau-harness-diff-chips">
          <span class="tau-harness-diff-chip" data-scope="allowed">Skill</span>
          <span class="tau-harness-diff-chip" data-scope="allowed">Config</span>
          <span class="tau-harness-diff-chip" data-scope="allowed">Prompt</span>
        </div>
      </section>
      <section>
        <h2>Blocked Scope</h2>
        <div class="tau-harness-diff-chips">
          <span class="tau-harness-diff-chip" data-scope="blocked">Source code</span>
          <span class="tau-harness-diff-chip" data-scope="blocked">Safety policy</span>
        </div>
      </section>
    </div>
  </section>
  <section class="tau-harness-diff-card" aria-labelledby="tau-harness-diff-patch">
    <h2 id="tau-harness-diff-patch">Patch Preview</h2>
    <pre class="tau-harness-diff-code" data-diff-artifact="proposal-registry"><code>{diff_lines}</code></pre>
  </section>
  <nav class="tau-harness-diff-actions" aria-label="Harness diff actions">
    <a href="{back_href}">Back to Mission Harness</a>
  </nav>
</main>
</body>
 </html>"#,
        target_path = proposal.target_path,
        dry_run_result_key = proposal.dry_run_result_key,
        safety_check_key = proposal.safety_check_key,
        title = proposal.title,
        patch_summary = proposal.patch_summary,
        dry_run_result_label = proposal.dry_run_result_label,
        safety_check_label = proposal.safety_check_label,
        rollback_plan = proposal.rollback_plan,
    ))
    .into_response()
}

fn harness_proposal_diff_lines(proposal: &GatewayOpsHarnessProposalDefinition) -> String {
    let mut lines = vec![
        format!(
            r#"<span class="tau-harness-diff-line">--- {}</span>"#,
            proposal.target_path
        ),
        format!(
            r#"<span class="tau-harness-diff-line">+++ {}</span>"#,
            proposal.target_path
        ),
        r#"<span class="tau-harness-diff-line">@@</span>"#.to_string(),
    ];
    lines.extend(proposal.diff_removed_lines.iter().map(|line| {
        format!(r#"<span class="tau-harness-diff-line tau-harness-diff-line-remove">{line}</span>"#)
    }));
    lines.extend(proposal.diff_added_lines.iter().map(|line| {
        format!(r#"<span class="tau-harness-diff-line tau-harness-diff-line-add">{line}</span>"#)
    }));
    lines.join("\n")
}

fn canonical_harness_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tasks/fixtures/m334/tranche-one-autonomy-benchmark.json")
}

fn harness_artifact_dir(state_dir: &Path) -> PathBuf {
    state_dir.join("ops-harness").join("m334")
}

fn resolve_harness_mission_proposal_id(
    mission: &Value,
    controls: &OpsShellControlsQuery,
) -> Option<String> {
    mission
        .get("proposal_id")
        .and_then(Value::as_str)
        .or_else(|| controls.requested_harness_proposal_id())
        .or_else(|| {
            mission
                .get("goal")
                .and_then(Value::as_str)
                .and_then(|goal| goal.split_whitespace().find(|part| part.starts_with("PR-")))
        })
        .map(sanitize_harness_token)
        .filter(|proposal_id| find_ops_harness_proposal(proposal_id).is_some())
}

fn read_completed_harness_self_improvement_result(
    state_dir: &Path,
    proposal_id: &str,
) -> Option<(GatewayOpsHarnessSelfImprovementResult, Option<Value>)> {
    let mission_path = state_dir
        .join("ops-harness")
        .join("self-improvement")
        .join(proposal_id)
        .join("mission.json");
    let mission_json = std::fs::read_to_string(&mission_path).ok()?;
    let mission = serde_json::from_str::<Value>(&mission_json).ok()?;
    if mission.get("status").and_then(Value::as_str) != Some("completed") {
        return None;
    }
    let proposal = find_ops_harness_proposal(proposal_id)?;
    let final_learning_output = mission.get("final_learning_output").cloned();
    let summary = final_learning_output
        .as_ref()
        .and_then(|output| output.get("summary"))
        .and_then(Value::as_str)
        .or_else(|| mission.get("latest_output_summary").and_then(Value::as_str))
        .unwrap_or("Existing completed self-improvement mission proof linked.")
        .to_string();
    let mission_id = mission
        .get("mission_id")
        .and_then(Value::as_str)
        .unwrap_or(proposal.mission_id)
        .to_string();
    Some((
        GatewayOpsHarnessSelfImprovementResult {
            proposal_id: proposal_id.to_string(),
            mission_id,
            target_path: proposal.target_path.to_string(),
            result_key: "completed".to_string(),
            summary,
            artifact_path: Some(mission_path),
            applied: true,
        },
        final_learning_output,
    ))
}

fn record_harness_mission_start_result(
    mission: &mut Value,
    proposal_id: &str,
    result: &GatewayOpsHarnessSelfImprovementResult,
    now: u64,
    completed_proof: bool,
    final_learning_output: Option<Value>,
) {
    let passed = completed_proof || result.result_key == "passed";
    let checkpoint_id = format!("start-{now}");
    mission["proposal_id"] = json!(proposal_id);
    mission["linked_self_improvement_mission_id"] = json!(result.mission_id.as_str());
    mission["status"] = json!(if completed_proof {
        "completed"
    } else if passed {
        "awaiting_approval"
    } else {
        "blocked"
    });
    mission["latest_output_summary"] = json!(if completed_proof {
        result.summary.clone()
    } else if passed {
        "Mission started through the coding-agent self-improvement dry-run; operator approval is required before apply.".to_string()
    } else {
        format!("Mission start dry-run blocked: {}", result.summary)
    });
    mission["updated_unix_ms"] = json!(now);
    if let Some(budget) = mission.get_mut("tool_budget") {
        let consumed = budget
            .get("consumed_tool_calls")
            .and_then(Value::as_u64)
            .unwrap_or_default()
            .saturating_add(1);
        budget["consumed_tool_calls"] = json!(consumed);
    }
    set_harness_mission_plan_status(mission, "clarify-goal", "completed");
    set_harness_mission_plan_status(mission, "write-plan-dag", "completed");
    set_harness_mission_plan_status(
        mission,
        "execute-with-budget",
        if passed { "completed" } else { "blocked" },
    );
    if completed_proof {
        set_harness_mission_plan_status(mission, "verify-gates", "completed");
        set_harness_mission_plan_status(mission, "write-final-learning", "completed");
    }
    set_harness_mission_gate_status(
        mission,
        "VG-PLAN",
        "passed",
        json!({ "proposal_id": proposal_id, "mission_id": result.mission_id.as_str() }),
    );
    set_harness_mission_gate_status(
        mission,
        "VG-EXECUTE",
        if passed { "passed" } else { "failed" },
        json!({
            "proposal_id": proposal_id,
            "target_path": result.target_path.as_str(),
            "result_key": result.result_key.as_str(),
            "summary": result.summary.as_str(),
        }),
    );
    if completed_proof {
        set_harness_mission_gate_status(
            mission,
            "VG-LEARN",
            "passed",
            json!({
                "proposal_id": proposal_id,
                "linked_mission_id": result.mission_id.as_str(),
                "summary": result.summary.as_str(),
            }),
        );
        if let Some(final_learning_output) = final_learning_output {
            mission["final_learning_output"] = final_learning_output;
        }
    }
    upsert_harness_mission_array_object(
        mission,
        "memory_hits",
        "key",
        format!("learning:{proposal_id}").as_str(),
        json!({
            "key": format!("learning:{proposal_id}"),
            "summary": result.summary.as_str(),
            "score": 0.85,
            "source_event_key": proposal_id,
            "used_in_plan_node_ids": ["execute-with-budget"]
        }),
    );
    mission["memory_recall"] = json!({
        "query": format!("proposal:{proposal_id} harness mission start"),
        "status": "used_hits",
        "checked_unix_ms": now,
        "rationale": "Mission start used the selected proposal learning record and coding-agent dry-run result.",
        "hit_keys": [format!("learning:{proposal_id}")]
    });
    upsert_harness_mission_array_object(
        mission,
        "tool_evidence",
        "tool_call_id",
        format!("tool-{proposal_id}-dry-run-{now}").as_str(),
        json!({
            "tool_call_id": format!("tool-{proposal_id}-dry-run-{now}"),
            "plan_node_id": "execute-with-budget",
            "tool_name": if completed_proof {
                "self_improvement.completed_proof"
            } else {
                "self_modification.dry_run"
            },
            "status": if passed { "succeeded" } else { "blocked" },
            "started_unix_ms": now,
            "completed_unix_ms": now,
            "runtime_ms": 0,
            "cost_usd": null,
            "summary": result.summary.as_str(),
            "artifact_ids": ["self-improvement-dry-run"],
            "verification_gate_ids": ["VG-EXECUTE"]
        }),
    );
    if let Some(path) = result.artifact_path.as_ref() {
        upsert_harness_mission_array_object(
            mission,
            "artifacts",
            "artifact_id",
            if completed_proof {
                "self-improvement-proof"
            } else {
                "self-improvement-dry-run"
            },
            json!({
                "artifact_id": if completed_proof {
                    "self-improvement-proof"
                } else {
                    "self-improvement-dry-run"
                },
                "kind": if completed_proof {
                    "coding-agent-completed-proof"
                } else {
                    "coding-agent-dry-run"
                },
                "path": path.display().to_string(),
                "summary": if completed_proof {
                    "Existing completed self-improvement mission proof."
                } else {
                    "Coding-agent self-improvement dry-run evidence."
                }
            }),
        );
    }
    push_harness_mission_array_value(
        mission,
        "checkpoints",
        json!({
            "checkpoint_id": checkpoint_id,
            "summary": if completed_proof {
                "Started mission by linking existing completed self-improvement proof."
            } else if passed {
                "Started mission and completed coding-agent dry-run; waiting for operator approval."
            } else {
                "Started mission but coding-agent dry-run blocked execution."
            },
            "created_unix_ms": now,
            "pending_plan_node_ids": if completed_proof {
                json!([])
            } else if passed {
                json!(["verify-gates", "write-final-learning"])
            } else {
                json!(["execute-with-budget"])
            }
        }),
    );
    mission["recovery_state"] = if completed_proof {
        Value::Null
    } else {
        json!({
        "reason": if passed {
            "Coding-agent dry-run completed; operator approval is required before apply."
        } else {
            "Coding-agent dry-run blocked the mission start."
        },
        "next_action": if passed {
            "approve or reject the self-improvement proposal"
        } else {
            "inspect dry-run evidence before retry"
        },
        "retry_count": 0,
        "last_checkpoint_id": checkpoint_id,
        })
    };
    upsert_harness_mission_array_object(
        mission,
        "improvement_proposals",
        "proposal_id",
        proposal_id,
        json!({
            "proposal_id": proposal_id,
            "linked_mission_id": result.mission_id.as_str(),
            "target_path": result.target_path.as_str(),
            "status": if completed_proof {
                "applied"
            } else if passed {
                "dry_run_recorded"
            } else {
                "blocked"
            },
            "summary": result.summary.as_str()
        }),
    );
}

fn record_harness_mission_start_failure(mission: &mut Value, error_summary: &str, now: u64) {
    let checkpoint_id = format!("start-failed-{now}");
    mission["status"] = json!("blocked");
    mission["latest_output_summary"] = json!(format!("Mission start failed: {error_summary}"));
    mission["updated_unix_ms"] = json!(now);
    set_harness_mission_plan_status(mission, "clarify-goal", "completed");
    set_harness_mission_plan_status(mission, "write-plan-dag", "completed");
    set_harness_mission_plan_status(mission, "execute-with-budget", "blocked");
    set_harness_mission_gate_status(
        mission,
        "VG-PLAN",
        "passed",
        json!({ "checked_unix_ms": now }),
    );
    set_harness_mission_gate_status(
        mission,
        "VG-EXECUTE",
        "failed",
        json!({ "error": error_summary, "checked_unix_ms": now }),
    );
    push_harness_mission_array_value(
        mission,
        "checkpoints",
        json!({
            "checkpoint_id": checkpoint_id,
            "summary": "Mission start failed before coding-agent dry-run completed.",
            "created_unix_ms": now,
            "pending_plan_node_ids": ["execute-with-budget"]
        }),
    );
    mission["recovery_state"] = json!({
        "reason": error_summary,
        "next_action": "fix mission start blocker before retry",
        "retry_count": 1,
        "last_checkpoint_id": checkpoint_id,
    });
}

fn set_harness_mission_plan_status(mission: &mut Value, node_id: &str, status: &str) {
    if let Some(nodes) = mission.get_mut("plan_dag").and_then(Value::as_array_mut) {
        for node in nodes {
            if node.get("id").and_then(Value::as_str) == Some(node_id) {
                node["status"] = json!(status);
            }
        }
    }
}

fn set_harness_mission_gate_status(
    mission: &mut Value,
    gate_id: &str,
    status: &str,
    evidence: Value,
) {
    if let Some(gates) = mission
        .get_mut("verification_gates")
        .and_then(Value::as_array_mut)
    {
        for gate in gates {
            if gate.get("id").and_then(Value::as_str) == Some(gate_id) {
                gate["status"] = json!(status);
                gate["evidence"] = evidence.clone();
            }
        }
    }
}

fn push_harness_mission_array_value(mission: &mut Value, field: &str, value: Value) {
    if let Some(values) = mission.get_mut(field).and_then(Value::as_array_mut) {
        values.push(value);
        return;
    }
    if let Some(object) = mission.as_object_mut() {
        object.insert(field.to_string(), json!([value]));
    }
}

fn upsert_harness_mission_array_object(
    mission: &mut Value,
    field: &str,
    id_field: &str,
    id_value: &str,
    value: Value,
) {
    if let Some(values) = mission.get_mut(field).and_then(Value::as_array_mut) {
        if let Some(existing) = values
            .iter_mut()
            .find(|candidate| candidate.get(id_field).and_then(Value::as_str) == Some(id_value))
        {
            *existing = value;
        } else {
            values.push(value);
        }
        return;
    }
    if let Some(object) = mission.as_object_mut() {
        object.insert(field.to_string(), json!([value]));
    }
}

fn append_harness_audit_record(state_dir: &Path, proposal_id: &str, action: &str, result: &str) {
    append_harness_audit_record_with_fields(state_dir, proposal_id, action, result, &[]);
}

fn append_harness_audit_record_with_fields(
    state_dir: &Path,
    proposal_id: &str,
    action: &str,
    result: &str,
    fields: &[(&str, &str)],
) {
    let audit_dir = state_dir.join("ops-harness");
    let audit_path = audit_dir.join("audit.jsonl");
    if std::fs::create_dir_all(&audit_dir).is_err() {
        return;
    }
    let mut record = json!({
        "timestamp_unix_ms": now_unix_ms(),
        "proposal_id": proposal_id,
        "action": action,
        "result": result,
    });
    if let Some(object) = record.as_object_mut() {
        for (key, value) in fields {
            object.insert((*key).to_string(), json!(*value));
        }
    }
    let Ok(mut line) = serde_json::to_string(&record) else {
        return;
    };
    line.push('\n');
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(audit_path)
        .and_then(|mut file| std::io::Write::write_all(&mut file, line.as_bytes()));
}

fn harness_latest_audit_action_is_approved(state_dir: &Path, proposal_id: &str) -> bool {
    let audit_path = state_dir.join("ops-harness").join("audit.jsonl");
    let Ok(audit_jsonl) = std::fs::read_to_string(audit_path) else {
        return false;
    };
    audit_jsonl
        .lines()
        .rev()
        .find_map(|line| {
            let Ok(record) = serde_json::from_str::<Value>(line) else {
                return None;
            };
            if record
                .get("proposal_id")
                .and_then(Value::as_str)
                .is_none_or(|value| value != proposal_id)
            {
                return None;
            }
            let action = record.get("action").and_then(Value::as_str)?;
            if !matches!(action, "approve" | "reject" | "apply") {
                return None;
            }
            let result = record
                .get("result")
                .and_then(Value::as_str)
                .unwrap_or_default();
            Some(
                action == "approve"
                    && !result.starts_with("blocked")
                    && !result.ends_with("failed"),
            )
        })
        .unwrap_or(false)
}

fn sanitize_harness_token(token: &str) -> String {
    let sanitized = token
        .chars()
        .filter(|candidate| candidate.is_ascii_alphanumeric() || matches!(candidate, '-' | '_'))
        .collect::<String>();
    if sanitized.is_empty() {
        "unknown".to_string()
    } else {
        sanitized
    }
}

fn humanize_harness_token(token: &str) -> String {
    token
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut word = first.to_ascii_uppercase().to_string();
                    word.push_str(chars.as_str());
                    word
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}

pub(super) async fn handle_ops_dashboard_control_action(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    Form(form): Form<OpsDashboardControlActionForm>,
) -> Response {
    let redirect_theme = form.resolved_theme();
    let redirect_sidebar_state = form.resolved_sidebar_state();
    let Some(request) = form.resolved_action_request() else {
        state.record_ui_telemetry_event(
            "dashboard",
            "control-action",
            "control_action_form_missing_action",
        );
        let redirect_path = build_ops_root_redirect_path(
            redirect_theme,
            redirect_sidebar_state,
            "missing",
            "none",
            "missing_action",
        );
        return Redirect::to(redirect_path.as_str()).into_response();
    };

    let action_marker = normalize_ops_control_action_marker(request.action.as_str());
    match apply_gateway_dashboard_action(&state.config.state_dir, "ops-shell", request) {
        Ok(_) => {
            state.record_ui_telemetry_event(
                "dashboard",
                "control-action",
                "control_action_applied",
            );
            let redirect_path = build_ops_root_redirect_path(
                redirect_theme,
                redirect_sidebar_state,
                "applied",
                action_marker,
                "control_action_applied",
            );
            Redirect::to(redirect_path.as_str()).into_response()
        }
        Err(error) => {
            let reason_marker = normalize_ops_control_action_reason_marker(error.code);
            state.record_ui_telemetry_event(
                "dashboard",
                "control-action",
                format!("control_action_failed:{action_marker}").as_str(),
            );
            let redirect_path = build_ops_root_redirect_path(
                redirect_theme,
                redirect_sidebar_state,
                "failed",
                action_marker,
                reason_marker,
            );
            Redirect::to(redirect_path.as_str()).into_response()
        }
    }
}

pub(super) async fn handle_ops_dashboard_channel_action(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    Form(form): Form<OpsDashboardChannelActionForm>,
) -> Response {
    let redirect_theme = form.resolved_theme();
    let redirect_sidebar_state = form.resolved_sidebar_state();
    let redirect_session_key = form.resolved_session_key();
    let action_input = form.action.trim().to_ascii_lowercase();
    let channel_input = form.channel.trim().to_ascii_lowercase();

    if action_input.is_empty() || channel_input.is_empty() {
        state.record_ui_telemetry_event(
            "channels",
            "lifecycle-action",
            "channel_lifecycle_form_missing_action",
        );
        let redirect_path = build_ops_channels_redirect_path(
            redirect_theme,
            redirect_sidebar_state,
            redirect_session_key.as_str(),
            "missing",
            "none",
            "none",
            "missing_channel_action",
        );
        return Redirect::to(redirect_path.as_str()).into_response();
    }

    let Some(channel) = parse_ops_channel_transport(channel_input.as_str()) else {
        state.record_ui_telemetry_event(
            "channels",
            "lifecycle-action",
            "channel_lifecycle_form_invalid_channel",
        );
        let redirect_path = build_ops_channels_redirect_path(
            redirect_theme,
            redirect_sidebar_state,
            redirect_session_key.as_str(),
            "failed",
            action_input.as_str(),
            "none",
            "invalid_channel",
        );
        return Redirect::to(redirect_path.as_str()).into_response();
    };

    let Some(action) = parse_ops_channel_lifecycle_action(action_input.as_str()) else {
        state.record_ui_telemetry_event(
            "channels",
            "lifecycle-action",
            "channel_lifecycle_form_invalid_action",
        );
        let redirect_path = build_ops_channels_redirect_path(
            redirect_theme,
            redirect_sidebar_state,
            redirect_session_key.as_str(),
            "failed",
            "none",
            channel.as_str(),
            "invalid_lifecycle_action",
        );
        return Redirect::to(redirect_path.as_str()).into_response();
    };

    let lifecycle_request = GatewayChannelLifecycleRequest {
        action: action_input.clone(),
        ..GatewayChannelLifecycleRequest::default()
    };
    let command_config = build_gateway_multi_channel_lifecycle_command_config(
        &state.config.state_dir,
        &lifecycle_request,
    );
    match execute_multi_channel_lifecycle_action(&command_config, action, channel) {
        Ok(_) => {
            let reason_code = format!("channel_lifecycle_action_{}_applied", action_input);
            state.record_ui_telemetry_event("channels", "lifecycle-action", reason_code.as_str());
            let redirect_path = build_ops_channels_redirect_path(
                redirect_theme,
                redirect_sidebar_state,
                redirect_session_key.as_str(),
                "applied",
                action_input.as_str(),
                channel.as_str(),
                reason_code.as_str(),
            );
            Redirect::to(redirect_path.as_str()).into_response()
        }
        Err(_) => {
            state.record_ui_telemetry_event(
                "channels",
                "lifecycle-action",
                "channel_lifecycle_action_failed",
            );
            let redirect_path = build_ops_channels_redirect_path(
                redirect_theme,
                redirect_sidebar_state,
                redirect_session_key.as_str(),
                "failed",
                action_input.as_str(),
                channel.as_str(),
                "internal_error",
            );
            Redirect::to(redirect_path.as_str()).into_response()
        }
    }
}

pub(super) async fn handle_ops_dashboard_chat_new(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    Form(form): Form<OpsDashboardChatNewSessionForm>,
) -> Response {
    let session_key = form.resolved_session_key();
    let redirect_path = build_ops_chat_redirect_path(
        form.resolved_theme(),
        form.resolved_sidebar_state(),
        session_key.as_str(),
    );

    let session_path = gateway_session_path(&state.config.state_dir, session_key.as_str());
    let mut store = match SessionStore::load(&session_path) {
        Ok(store) => store,
        Err(error) => {
            return OpenResponsesApiError::internal(format!(
                "failed to load session '{}': {error}",
                session_path.display()
            ))
            .into_response();
        }
    };
    store.set_lock_policy(
        state.config.session_lock_wait_ms,
        state.config.session_lock_stale_ms,
    );

    let resolved_system_prompt = state.resolved_system_prompt();
    if let Err(error) = store.ensure_initialized(&resolved_system_prompt) {
        return OpenResponsesApiError::internal(format!(
            "failed to initialize session '{}': {error}",
            session_path.display()
        ))
        .into_response();
    }

    state.record_ui_telemetry_event("chat", "new-session", "chat_session_initialized");
    Redirect::to(redirect_path.as_str()).into_response()
}

pub(super) async fn handle_ops_dashboard_chat_send(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    Form(form): Form<OpsDashboardChatSendForm>,
) -> Response {
    let session_key = form.resolved_session_key();
    let redirect_path = build_ops_chat_redirect_path(
        form.resolved_theme(),
        form.resolved_sidebar_state(),
        session_key.as_str(),
    );
    let content = form.message.as_str();
    if content.trim().is_empty() {
        return Redirect::to(redirect_path.as_str()).into_response();
    }

    let session_path = gateway_session_path(&state.config.state_dir, session_key.as_str());
    let mut store = match SessionStore::load(&session_path) {
        Ok(store) => store,
        Err(error) => {
            return OpenResponsesApiError::internal(format!(
                "failed to load session '{}': {error}",
                session_path.display()
            ))
            .into_response();
        }
    };
    store.set_lock_policy(
        state.config.session_lock_wait_ms,
        state.config.session_lock_stale_ms,
    );

    let resolved_system_prompt = state.resolved_system_prompt();
    if let Err(error) = store.ensure_initialized(&resolved_system_prompt) {
        return OpenResponsesApiError::internal(format!(
            "failed to initialize session '{}': {error}",
            session_path.display()
        ))
        .into_response();
    }

    let parent_id = store.head_id();
    let message = Message::user(content);
    let new_head = match store.append_messages(parent_id, &[message]) {
        Ok(head) => head,
        Err(error) => {
            return OpenResponsesApiError::internal(format!(
                "failed to append session message '{}': {error}",
                session_path.display()
            ))
            .into_response();
        }
    };

    let assistant_output = complete_cortex_chat(&state, content.trim()).await;
    let assistant_message = Message::assistant_text(assistant_output.output_text.clone());
    let assistant_head = match store.append_messages(new_head, &[assistant_message]) {
        Ok(head) => head,
        Err(error) => {
            return OpenResponsesApiError::internal(format!(
                "failed to append assistant session message '{}': {error}",
                session_path.display()
            ))
            .into_response();
        }
    };

    let _ = record_cortex_observer_event(
        &state.config.state_dir,
        "cortex.chat.request",
        json!({
            "surface": "ops.chat",
            "session_key": session_key.as_str(),
            "input_chars": content.trim().chars().count(),
            "output_chars": assistant_output.output_text.chars().count(),
            "reason_code": assistant_output.reason_code,
            "fallback": assistant_output.fallback,
        }),
    );
    state.record_ui_telemetry_event("chat", "send", "chat_message_appended");
    state.record_ui_telemetry_event("chat", "send", "chat_assistant_message_appended");
    if write_ops_chat_turn_memory(
        &state,
        session_key.as_str(),
        assistant_head,
        content.trim(),
        assistant_output.output_text.as_str(),
        assistant_output.reason_code,
        assistant_output.fallback,
    )
    .is_ok()
    {
        state.record_ui_telemetry_event("memory", "entry_write", "ops_chat_turn_memory_written");
    } else {
        state.record_ui_telemetry_event("memory", "entry_write", "ops_chat_turn_memory_failed");
    }
    record_cortex_session_append_event(
        &state.config.state_dir,
        session_key.as_str(),
        assistant_head,
        store.entries().len(),
    );
    Redirect::to(redirect_path.as_str()).into_response()
}

fn write_ops_chat_turn_memory(
    state: &GatewayOpenResponsesServerState,
    session_key: &str,
    assistant_head: Option<u64>,
    user_input: &str,
    assistant_output: &str,
    reason_code: &str,
    fallback: bool,
) -> anyhow::Result<()> {
    let scope = MemoryScope {
        workspace_id: session_key.to_string(),
        channel_id: "ops.chat".to_string(),
        actor_id: "operator".to_string(),
    };
    let store = gateway_memory_store(&state.config.state_dir, session_key);
    let scope_filter = MemoryScopeFilter {
        workspace_id: Some(scope.workspace_id.clone()),
        channel_id: Some(scope.channel_id.clone()),
        actor_id: Some(scope.actor_id.clone()),
    };
    let previous_memory_id = store
        .list_latest_records(Some(&scope_filter), 1)
        .ok()
        .and_then(|records| {
            records
                .into_iter()
                .next()
                .map(|record| record.entry.memory_id)
        });
    let head_label = assistant_head
        .map(|head| head.to_string())
        .unwrap_or_else(|| now_unix_ms().to_string());
    let memory_id = format!("ops-chat-{session_key}-{head_label}");
    let summary = format!(
        "Ops chat turn: operator asked '{}' and assistant answered '{}'",
        truncate_ops_chat_memory_text(user_input, 160),
        truncate_ops_chat_memory_text(assistant_output, 220)
    );
    let mut facts = vec![
        format!("session_key={session_key}"),
        format!("assistant_head={head_label}"),
        format!("reason_code={reason_code}"),
        format!("fallback={fallback}"),
    ];
    if let Some(previous_memory_id) = previous_memory_id.as_deref() {
        facts.push(format!("previous_chat_memory_id={previous_memory_id}"));
    }
    let entry = MemoryEntry {
        memory_id: memory_id.clone(),
        summary,
        tags: vec![
            "ops_chat".to_string(),
            "automatic_memory".to_string(),
            format!("session:{session_key}"),
            format!("reason:{reason_code}"),
        ],
        facts,
        source_event_key: format!("ops-chat:{session_key}:{head_label}"),
        recency_weight_bps: 1_000,
        confidence_bps: 8_500,
    };
    let relations = previous_memory_id
        .filter(|previous| previous != &memory_id)
        .map(|target_id| {
            vec![MemoryRelationInput {
                target_id,
                relation_type: Some("updates".to_string()),
                weight: Some(0.65),
            }]
        })
        .unwrap_or_default();
    store.write_entry_with_metadata_and_relations(
        &scope,
        entry,
        Some(MemoryType::Event),
        Some(0.7),
        relations.as_slice(),
    )?;
    Ok(())
}

fn truncate_ops_chat_memory_text(value: &str, max_chars: usize) -> String {
    let collapsed = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= max_chars {
        return collapsed;
    }
    let mut truncated = collapsed.chars().take(max_chars).collect::<String>();
    truncated.push_str("...");
    truncated
}

pub(super) async fn handle_ops_dashboard_memory_create(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    Form(form): Form<OpsDashboardMemoryCreateForm>,
) -> Response {
    let session_key = form.resolved_session_key();
    let is_edit_operation = form.is_edit_operation();
    let is_delete_operation = form.is_delete_operation();
    let fallback_redirect_path = build_ops_memory_redirect_path(
        form.resolved_theme(),
        form.resolved_sidebar_state(),
        session_key.as_str(),
        "idle",
        None,
        "idle",
        None,
    );

    let entry_id = form.resolved_entry_id();
    if is_delete_operation {
        if entry_id.is_empty() || !form.is_delete_confirmed() {
            return Redirect::to(fallback_redirect_path.as_str()).into_response();
        }
        let store = gateway_memory_store(&state.config.state_dir, session_key.as_str());
        match store.soft_delete_entry(entry_id.as_str(), None) {
            Ok(Some(_)) => {
                state.record_ui_telemetry_event(
                    "memory",
                    "entry_delete",
                    "ops_memory_delete_form_submitted",
                );
                record_cortex_memory_entry_delete_event(
                    &state.config.state_dir,
                    session_key.as_str(),
                    entry_id.as_str(),
                    true,
                );
                let redirect_path = build_ops_memory_redirect_path(
                    form.resolved_theme(),
                    form.resolved_sidebar_state(),
                    session_key.as_str(),
                    "idle",
                    None,
                    "deleted",
                    Some(entry_id.as_str()),
                );
                return Redirect::to(redirect_path.as_str()).into_response();
            }
            Ok(None) => return Redirect::to(fallback_redirect_path.as_str()).into_response(),
            Err(error) => {
                return OpenResponsesApiError::internal(format!(
                    "failed to delete memory entry '{}' for session '{}': {error}",
                    entry_id, session_key
                ))
                .into_response();
            }
        }
    }

    let summary = form.resolved_summary();
    if entry_id.is_empty() || summary.is_empty() {
        return Redirect::to(fallback_redirect_path.as_str()).into_response();
    }

    let scope = MemoryScope {
        workspace_id: form.resolved_workspace_id(session_key.as_str()),
        channel_id: form.resolved_channel_id(),
        actor_id: form.resolved_actor_id(),
    };
    let entry = MemoryEntry {
        memory_id: entry_id.clone(),
        summary,
        tags: form.resolved_tags(),
        facts: form.resolved_facts(),
        source_event_key: form.resolved_source_event_key(entry_id.as_str()),
        recency_weight_bps: 0,
        confidence_bps: 1000,
    };
    let relation_inputs = form.resolved_relations();

    let store = gateway_memory_store(&state.config.state_dir, session_key.as_str());
    if is_edit_operation {
        match store.read_entry(entry_id.as_str(), None) {
            Ok(Some(_)) => {}
            Ok(None) => return Redirect::to(fallback_redirect_path.as_str()).into_response(),
            Err(error) => {
                return OpenResponsesApiError::internal(format!(
                    "failed to load memory entry '{}' for session '{}': {error}",
                    entry_id, session_key
                ))
                .into_response();
            }
        }
    }

    let write_result = match store.write_entry_with_metadata_and_relations(
        &scope,
        entry,
        form.resolved_memory_type(),
        form.resolved_importance(),
        relation_inputs.as_slice(),
    ) {
        Ok(result) => result,
        Err(error) => {
            return OpenResponsesApiError::internal(format!(
                "failed to upsert memory entry '{}' for session '{}': {error}",
                entry_id, session_key
            ))
            .into_response();
        }
    };

    let reason_code = if is_edit_operation {
        "ops_memory_edit_form_submitted"
    } else {
        "ops_memory_create_form_submitted"
    };
    state.record_ui_telemetry_event("memory", "entry_write", reason_code);
    record_cortex_memory_entry_write_event(
        &state.config.state_dir,
        session_key.as_str(),
        entry_id.as_str(),
        write_result.created,
    );
    let create_status = if write_result.created {
        "created"
    } else {
        "updated"
    };
    let redirect_path = build_ops_memory_redirect_path(
        form.resolved_theme(),
        form.resolved_sidebar_state(),
        session_key.as_str(),
        create_status,
        Some(entry_id.as_str()),
        "idle",
        None,
    );
    Redirect::to(redirect_path.as_str()).into_response()
}

pub(super) async fn handle_ops_dashboard_sessions_branch(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    Form(form): Form<OpsDashboardSessionBranchForm>,
) -> Response {
    let source_session_key = form.resolved_source_session_key();
    let selected_entry_id = form.resolved_entry_id();
    let redirect_theme = form.resolved_theme();
    let redirect_sidebar_state = form.resolved_sidebar_state();
    let target_session_key =
        form.resolved_target_session_key(source_session_key.as_str(), selected_entry_id);

    if target_session_key.trim().is_empty() {
        let source_redirect_path = build_ops_chat_redirect_path(
            redirect_theme,
            redirect_sidebar_state,
            source_session_key.as_str(),
        );
        return Redirect::to(source_redirect_path.as_str()).into_response();
    }

    let Some(selected_entry_id) = selected_entry_id else {
        let source_redirect_path = build_ops_chat_redirect_path(
            redirect_theme,
            redirect_sidebar_state,
            source_session_key.as_str(),
        );
        return Redirect::to(source_redirect_path.as_str()).into_response();
    };

    let source_session_path =
        gateway_session_path(&state.config.state_dir, source_session_key.as_str());
    let mut source_store = match SessionStore::load(&source_session_path) {
        Ok(store) => store,
        Err(error) => {
            return OpenResponsesApiError::internal(format!(
                "failed to load source session '{}': {error}",
                source_session_path.display()
            ))
            .into_response();
        }
    };
    source_store.set_lock_policy(
        state.config.session_lock_wait_ms,
        state.config.session_lock_stale_ms,
    );

    if !source_store.contains(selected_entry_id) {
        let source_redirect_path = build_ops_chat_redirect_path(
            redirect_theme,
            redirect_sidebar_state,
            source_session_key.as_str(),
        );
        return Redirect::to(source_redirect_path.as_str()).into_response();
    }

    let target_session_path =
        gateway_session_path(&state.config.state_dir, target_session_key.as_str());
    if let Err(error) = source_store.export_lineage(Some(selected_entry_id), &target_session_path) {
        return OpenResponsesApiError::internal(format!(
            "failed to export branch session '{}': {error}",
            target_session_path.display()
        ))
        .into_response();
    }

    state.record_ui_telemetry_event("sessions", "branch", "session_branch_created");
    let redirect_path = build_ops_chat_redirect_path(
        redirect_theme,
        redirect_sidebar_state,
        target_session_key.as_str(),
    );
    Redirect::to(redirect_path.as_str()).into_response()
}

pub(super) async fn handle_ops_dashboard_session_detail_reset(
    State(state): State<Arc<GatewayOpenResponsesServerState>>,
    AxumPath(route_session_key): AxumPath<String>,
    Form(form): Form<OpsDashboardSessionResetForm>,
) -> Response {
    let route_session_key = sanitize_session_key(route_session_key.as_str());
    let session_key = form.resolved_session_key(route_session_key.as_str());
    let redirect_path = build_ops_session_detail_redirect_path(
        form.resolved_theme(),
        form.resolved_sidebar_state(),
        session_key.as_str(),
    );

    if !form.is_confirmed() {
        state.record_ui_telemetry_event("sessions", "reset", "session_reset_confirmation_missing");
        return Redirect::to(redirect_path.as_str()).into_response();
    }

    let session_path = gateway_session_path(&state.config.state_dir, session_key.as_str());
    let lock_path = session_path.with_extension("lock");
    let mut reset = false;

    if session_path.exists() {
        if let Err(error) = std::fs::remove_file(&session_path) {
            return OpenResponsesApiError::internal(format!(
                "failed to remove session '{}': {error}",
                session_path.display()
            ))
            .into_response();
        }
        reset = true;
    }
    if lock_path.exists() {
        let _ = std::fs::remove_file(&lock_path);
    }

    state.record_ui_telemetry_event("sessions", "reset", "session_reset_applied");
    record_cortex_session_reset_event(&state.config.state_dir, session_key.as_str(), reset);
    Redirect::to(redirect_path.as_str()).into_response()
}
