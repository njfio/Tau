//! Leptos SSR shell foundations for Tau Ops Dashboard.

use leptos::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Public enum `TauOpsDashboardAuthMode` in `tau-dashboard-ui`.
pub enum TauOpsDashboardAuthMode {
    None,
    Token,
    PasswordSession,
}

impl TauOpsDashboardAuthMode {
    /// Public `fn` `as_str` in `tau-dashboard-ui`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Token => "token",
            Self::PasswordSession => "password-session",
        }
    }

    /// Public `fn` `requires_authentication` in `tau-dashboard-ui`.
    pub fn requires_authentication(self) -> bool {
        !matches!(self, Self::None)
    }

    fn auth_input_label(self) -> &'static str {
        match self {
            Self::None => "Authentication disabled",
            Self::Token => "Bearer token",
            Self::PasswordSession => "Gateway password",
        }
    }

    fn auth_input_placeholder(self) -> &'static str {
        match self {
            Self::None => "No authentication required in localhost-dev mode",
            Self::Token => "Paste bearer token",
            Self::PasswordSession => "Enter gateway password",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Public enum `TauOpsDashboardRoute` in `tau-dashboard-ui`.
pub enum TauOpsDashboardRoute {
    Ops,
    Agents,
    AgentDetail,
    Chat,
    Sessions,
    Memory,
    MemoryGraph,
    ToolsJobs,
    Channels,
    Harness,
    Config,
    Training,
    Safety,
    Diagnostics,
    Deploy,
    Login,
}

impl TauOpsDashboardRoute {
    /// Public `fn` `as_str` in `tau-dashboard-ui`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ops => "ops",
            Self::Agents => "agents",
            Self::AgentDetail => "agent-detail",
            Self::Chat => "chat",
            Self::Sessions => "sessions",
            Self::Memory => "memory",
            Self::MemoryGraph => "memory-graph",
            Self::ToolsJobs => "tools-jobs",
            Self::Channels => "channels",
            Self::Harness => "harness",
            Self::Config => "config",
            Self::Training => "training",
            Self::Safety => "safety",
            Self::Diagnostics => "diagnostics",
            Self::Deploy => "deploy",
            Self::Login => "login",
        }
    }

    fn breadcrumb_token(self) -> &'static str {
        match self {
            Self::Ops => "command-center",
            Self::Agents => "agent-fleet",
            Self::AgentDetail => "agent-detail",
            Self::Chat => "chat",
            Self::Sessions => "sessions",
            Self::Memory => "memory",
            Self::MemoryGraph => "memory-graph",
            Self::ToolsJobs => "tools-jobs",
            Self::Channels => "channels",
            Self::Harness => "mission-harness",
            Self::Config => "config",
            Self::Training => "training",
            Self::Safety => "safety",
            Self::Diagnostics => "diagnostics",
            Self::Deploy => "deploy",
            Self::Login => "login",
        }
    }

    fn breadcrumb_label(self) -> &'static str {
        match self {
            Self::Ops => "Command Center",
            Self::Agents => "Agent Fleet",
            Self::AgentDetail => "Agent Detail",
            Self::Chat => "Conversation / Chat",
            Self::Sessions => "Sessions Explorer",
            Self::Memory => "Memory Explorer",
            Self::MemoryGraph => "Memory Graph",
            Self::ToolsJobs => "Tools & Jobs",
            Self::Channels => "Channels",
            Self::Harness => "Mission Harness",
            Self::Config => "Configuration",
            Self::Training => "Training & RL",
            Self::Safety => "Safety & Security",
            Self::Diagnostics => "Diagnostics & Audit",
            Self::Deploy => "Deploy Agent",
            Self::Login => "Login",
        }
    }

    fn shell_path(self) -> &'static str {
        match self {
            Self::Ops => "/ops",
            Self::Agents => "/ops/agents",
            Self::AgentDetail => "/ops/agents/default",
            Self::Chat => "/ops/chat",
            Self::Sessions => "/ops/sessions",
            Self::Memory => "/ops/memory",
            Self::MemoryGraph => "/ops/memory-graph",
            Self::ToolsJobs => "/ops/tools-jobs",
            Self::Channels => "/ops/channels",
            Self::Harness => "/ops/harness",
            Self::Config => "/ops/config",
            Self::Training => "/ops/training",
            Self::Safety => "/ops/safety",
            Self::Diagnostics => "/ops/diagnostics",
            Self::Deploy => "/ops/deploy",
            Self::Login => "/ops/login",
        }
    }

    fn from_shell_path(route: &str) -> Option<Self> {
        match route {
            "/ops" => Some(Self::Ops),
            "/ops/agents" => Some(Self::Agents),
            "/ops/agents/default" => Some(Self::AgentDetail),
            "/ops/chat" => Some(Self::Chat),
            "/ops/sessions" => Some(Self::Sessions),
            "/ops/memory" => Some(Self::Memory),
            "/ops/memory-graph" => Some(Self::MemoryGraph),
            "/ops/tools-jobs" => Some(Self::ToolsJobs),
            "/ops/channels" => Some(Self::Channels),
            "/ops/harness" => Some(Self::Harness),
            "/ops/config" => Some(Self::Config),
            "/ops/training" => Some(Self::Training),
            "/ops/safety" => Some(Self::Safety),
            "/ops/diagnostics" => Some(Self::Diagnostics),
            "/ops/deploy" => Some(Self::Deploy),
            "/ops/login" => Some(Self::Login),
            _ => None,
        }
    }
}

fn aria_current_for(
    active_route: TauOpsDashboardRoute,
    target_route: TauOpsDashboardRoute,
) -> &'static str {
    if active_route == target_route {
        "page"
    } else {
        "false"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Public enum `TauOpsDashboardTheme` in `tau-dashboard-ui`.
pub enum TauOpsDashboardTheme {
    Dark,
    Light,
}

impl TauOpsDashboardTheme {
    /// Public `fn` `as_str` in `tau-dashboard-ui`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Public enum `TauOpsDashboardSidebarState` in `tau-dashboard-ui`.
pub enum TauOpsDashboardSidebarState {
    Expanded,
    Collapsed,
}

impl TauOpsDashboardSidebarState {
    /// Public `fn` `as_str` in `tau-dashboard-ui`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Expanded => "expanded",
            Self::Collapsed => "collapsed",
        }
    }

    fn toggled(self) -> Self {
        match self {
            Self::Expanded => Self::Collapsed,
            Self::Collapsed => Self::Expanded,
        }
    }

    fn aria_expanded(self) -> &'static str {
        match self {
            Self::Expanded => "true",
            Self::Collapsed => "false",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardAlertFeedRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardAlertFeedRow {
    pub code: String,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardConnectorHealthRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardConnectorHealthRow {
    pub channel: String,
    pub mode: String,
    pub liveness: String,
    pub events_ingested: u64,
    pub provider_failures: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardChatMessageRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardChatMessageRow {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardMemorySearchRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardMemorySearchRow {
    pub memory_id: String,
    pub summary: String,
    pub memory_type: String,
    pub score: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardMemoryRelationRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardMemoryRelationRow {
    pub target_id: String,
    pub relation_type: String,
    pub effective_weight: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardMemoryGraphNodeRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardMemoryGraphNodeRow {
    pub memory_id: String,
    pub memory_type: String,
    pub importance: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardMemoryGraphEdgeRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardMemoryGraphEdgeRow {
    pub source_memory_id: String,
    pub target_memory_id: String,
    pub relation_type: String,
    pub effective_weight: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardChatSessionOptionRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardChatSessionOptionRow {
    pub session_key: String,
    pub selected: bool,
    pub entry_count: usize,
    pub usage_total_tokens: u64,
    pub validation_is_valid: bool,
    pub updated_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardSessionTimelineRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardSessionTimelineRow {
    pub entry_id: u64,
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardSessionGraphNodeRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardSessionGraphNodeRow {
    pub entry_id: u64,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardSessionGraphEdgeRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardSessionGraphEdgeRow {
    pub source_entry_id: u64,
    pub target_entry_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardToolInventoryRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardToolInventoryRow {
    pub tool_name: String,
    pub category: String,
    pub policy: String,
    pub usage_count: u64,
    pub error_rate: String,
    pub avg_latency_ms: String,
    pub last_used_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardToolUsageHistogramRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardToolUsageHistogramRow {
    pub hour_offset: u8,
    pub call_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardToolInvocationRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardToolInvocationRow {
    pub timestamp_unix_ms: u64,
    pub args_summary: String,
    pub result_summary: String,
    pub duration_ms: u64,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardJobRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardJobRow {
    pub job_id: String,
    pub job_name: String,
    pub job_status: String,
    pub started_unix_ms: u64,
    pub finished_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardChatSnapshot` in `tau-dashboard-ui`.
pub struct TauOpsDashboardChatSnapshot {
    pub active_session_key: String,
    pub new_session_form_action: String,
    pub new_session_form_method: String,
    pub send_form_action: String,
    pub send_form_method: String,
    pub control_action_status: String,
    pub control_action: String,
    pub control_action_reason: String,
    pub session_options: Vec<TauOpsDashboardChatSessionOptionRow>,
    pub message_rows: Vec<TauOpsDashboardChatMessageRow>,
    pub session_detail_visible: bool,
    pub session_detail_route: String,
    pub session_detail_validation_entries: usize,
    pub session_detail_validation_duplicates: usize,
    pub session_detail_validation_invalid_parent: usize,
    pub session_detail_validation_cycles: usize,
    pub session_detail_validation_is_valid: bool,
    pub session_detail_usage_input_tokens: u64,
    pub session_detail_usage_output_tokens: u64,
    pub session_detail_usage_total_tokens: u64,
    pub session_detail_usage_estimated_cost_usd: String,
    pub session_detail_timeline_rows: Vec<TauOpsDashboardSessionTimelineRow>,
    pub session_graph_node_rows: Vec<TauOpsDashboardSessionGraphNodeRow>,
    pub session_graph_edge_rows: Vec<TauOpsDashboardSessionGraphEdgeRow>,
    pub memory_search_form_action: String,
    pub memory_search_form_method: String,
    pub memory_search_query: String,
    pub memory_search_workspace_id: String,
    pub memory_search_channel_id: String,
    pub memory_search_actor_id: String,
    pub memory_search_memory_type: String,
    pub memory_search_rows: Vec<TauOpsDashboardMemorySearchRow>,
    pub memory_create_form_action: String,
    pub memory_create_form_method: String,
    pub memory_create_status: String,
    pub memory_create_created_entry_id: String,
    pub memory_create_entry_id: String,
    pub memory_create_summary: String,
    pub memory_create_tags: String,
    pub memory_create_facts: String,
    pub memory_create_source_event_key: String,
    pub memory_create_workspace_id: String,
    pub memory_create_channel_id: String,
    pub memory_create_actor_id: String,
    pub memory_create_memory_type: String,
    pub memory_create_importance: String,
    pub memory_create_relation_target_id: String,
    pub memory_create_relation_type: String,
    pub memory_create_relation_weight: String,
    pub memory_delete_status: String,
    pub memory_delete_deleted_entry_id: String,
    pub memory_detail_visible: bool,
    pub memory_detail_selected_entry_id: String,
    pub memory_detail_summary: String,
    pub memory_detail_memory_type: String,
    pub memory_detail_embedding_source: String,
    pub memory_detail_embedding_model: String,
    pub memory_detail_embedding_reason_code: String,
    pub memory_detail_embedding_dimensions: usize,
    pub memory_detail_relation_rows: Vec<TauOpsDashboardMemoryRelationRow>,
    pub memory_graph_zoom_level: String,
    pub memory_graph_pan_x_level: String,
    pub memory_graph_pan_y_level: String,
    pub memory_graph_filter_memory_type: String,
    pub memory_graph_filter_relation_type: String,
    pub memory_graph_node_rows: Vec<TauOpsDashboardMemoryGraphNodeRow>,
    pub memory_graph_edge_rows: Vec<TauOpsDashboardMemoryGraphEdgeRow>,
    pub tools_inventory_rows: Vec<TauOpsDashboardToolInventoryRow>,
    pub tool_detail_selected_tool_name: String,
    pub tool_detail_description: String,
    pub tool_detail_parameter_schema: String,
    pub tool_detail_policy_timeout_ms: u64,
    pub tool_detail_policy_max_output_chars: u64,
    pub tool_detail_policy_sandbox_mode: String,
    pub tool_detail_usage_histogram_rows: Vec<TauOpsDashboardToolUsageHistogramRow>,
    pub tool_detail_recent_invocation_rows: Vec<TauOpsDashboardToolInvocationRow>,
    pub jobs_rows: Vec<TauOpsDashboardJobRow>,
    pub job_detail_selected_job_id: String,
    pub job_detail_status: String,
    pub job_detail_duration_ms: u64,
    pub job_detail_stdout: String,
    pub job_detail_stderr: String,
}

impl Default for TauOpsDashboardChatSnapshot {
    fn default() -> Self {
        Self {
            active_session_key: "default".to_string(),
            new_session_form_action: "/ops/chat/new".to_string(),
            new_session_form_method: "post".to_string(),
            send_form_action: "/ops/chat/send".to_string(),
            send_form_method: "post".to_string(),
            control_action_status: "idle".to_string(),
            control_action: "none".to_string(),
            control_action_reason: "none".to_string(),
            session_options: vec![TauOpsDashboardChatSessionOptionRow {
                session_key: "default".to_string(),
                selected: true,
                entry_count: 0,
                usage_total_tokens: 0,
                validation_is_valid: true,
                updated_unix_ms: 0,
            }],
            message_rows: vec![],
            session_detail_visible: false,
            session_detail_route: "/ops/sessions/default".to_string(),
            session_detail_validation_entries: 0,
            session_detail_validation_duplicates: 0,
            session_detail_validation_invalid_parent: 0,
            session_detail_validation_cycles: 0,
            session_detail_validation_is_valid: true,
            session_detail_usage_input_tokens: 0,
            session_detail_usage_output_tokens: 0,
            session_detail_usage_total_tokens: 0,
            session_detail_usage_estimated_cost_usd: "0.000000".to_string(),
            session_detail_timeline_rows: vec![],
            session_graph_node_rows: vec![],
            session_graph_edge_rows: vec![],
            memory_search_form_action: "/ops/memory".to_string(),
            memory_search_form_method: "get".to_string(),
            memory_search_query: String::new(),
            memory_search_workspace_id: String::new(),
            memory_search_channel_id: String::new(),
            memory_search_actor_id: String::new(),
            memory_search_memory_type: String::new(),
            memory_search_rows: vec![],
            memory_create_form_action: "/ops/memory".to_string(),
            memory_create_form_method: "post".to_string(),
            memory_create_status: "idle".to_string(),
            memory_create_created_entry_id: String::new(),
            memory_create_entry_id: String::new(),
            memory_create_summary: String::new(),
            memory_create_tags: String::new(),
            memory_create_facts: String::new(),
            memory_create_source_event_key: String::new(),
            memory_create_workspace_id: String::new(),
            memory_create_channel_id: String::new(),
            memory_create_actor_id: String::new(),
            memory_create_memory_type: String::new(),
            memory_create_importance: String::new(),
            memory_create_relation_target_id: String::new(),
            memory_create_relation_type: String::new(),
            memory_create_relation_weight: String::new(),
            memory_delete_status: "idle".to_string(),
            memory_delete_deleted_entry_id: String::new(),
            memory_detail_visible: false,
            memory_detail_selected_entry_id: String::new(),
            memory_detail_summary: String::new(),
            memory_detail_memory_type: String::new(),
            memory_detail_embedding_source: String::new(),
            memory_detail_embedding_model: String::new(),
            memory_detail_embedding_reason_code: String::new(),
            memory_detail_embedding_dimensions: 0,
            memory_detail_relation_rows: vec![],
            memory_graph_zoom_level: "1.00".to_string(),
            memory_graph_pan_x_level: "0.00".to_string(),
            memory_graph_pan_y_level: "0.00".to_string(),
            memory_graph_filter_memory_type: "all".to_string(),
            memory_graph_filter_relation_type: "all".to_string(),
            memory_graph_node_rows: vec![],
            memory_graph_edge_rows: vec![],
            tools_inventory_rows: vec![],
            tool_detail_selected_tool_name: String::new(),
            tool_detail_description: String::new(),
            tool_detail_parameter_schema: "{}".to_string(),
            tool_detail_policy_timeout_ms: 120_000,
            tool_detail_policy_max_output_chars: 32_768,
            tool_detail_policy_sandbox_mode: "default".to_string(),
            tool_detail_usage_histogram_rows: vec![],
            tool_detail_recent_invocation_rows: vec![],
            jobs_rows: vec![],
            job_detail_selected_job_id: String::new(),
            job_detail_status: String::new(),
            job_detail_duration_ms: 0,
            job_detail_stdout: String::new(),
            job_detail_stderr: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardCommandCenterSnapshot` in `tau-dashboard-ui`.
pub struct TauOpsDashboardCommandCenterSnapshot {
    pub health_state: String,
    pub health_reason: String,
    pub rollout_gate: String,
    pub control_mode: String,
    pub control_paused: bool,
    pub action_pause_enabled: bool,
    pub action_resume_enabled: bool,
    pub action_refresh_enabled: bool,
    pub last_action_request_id: String,
    pub last_action_name: String,
    pub last_action_actor: String,
    pub last_action_reason: String,
    pub last_action_timestamp_unix_ms: u64,
    pub timeline_range: String,
    pub timeline_point_count: usize,
    pub timeline_last_timestamp_unix_ms: u64,
    pub queue_depth: usize,
    pub failure_streak: usize,
    pub processed_case_count: usize,
    pub alert_count: usize,
    pub widget_count: usize,
    pub timeline_cycle_count: usize,
    pub timeline_invalid_cycle_count: usize,
    pub primary_alert_code: String,
    pub primary_alert_severity: String,
    pub primary_alert_message: String,
    pub alert_feed_rows: Vec<TauOpsDashboardAlertFeedRow>,
    pub connector_health_rows: Vec<TauOpsDashboardConnectorHealthRow>,
    pub channel_action_status: String,
    pub channel_action: String,
    pub channel_action_channel: String,
    pub channel_action_reason: String,
}

impl Default for TauOpsDashboardCommandCenterSnapshot {
    fn default() -> Self {
        Self {
            health_state: "unknown".to_string(),
            health_reason: "dashboard snapshot unavailable".to_string(),
            rollout_gate: "hold".to_string(),
            control_mode: "running".to_string(),
            control_paused: false,
            action_pause_enabled: true,
            action_resume_enabled: false,
            action_refresh_enabled: true,
            last_action_request_id: "none".to_string(),
            last_action_name: "none".to_string(),
            last_action_actor: "none".to_string(),
            last_action_reason: "none".to_string(),
            last_action_timestamp_unix_ms: 0,
            timeline_range: "1h".to_string(),
            timeline_point_count: 0,
            timeline_last_timestamp_unix_ms: 0,
            queue_depth: 0,
            failure_streak: 0,
            processed_case_count: 0,
            alert_count: 0,
            widget_count: 0,
            timeline_cycle_count: 0,
            timeline_invalid_cycle_count: 0,
            primary_alert_code: "none".to_string(),
            primary_alert_severity: "info".to_string(),
            primary_alert_message: "No alerts loaded".to_string(),
            alert_feed_rows: vec![TauOpsDashboardAlertFeedRow {
                code: "none".to_string(),
                severity: "info".to_string(),
                message: "No alerts loaded".to_string(),
            }],
            connector_health_rows: vec![TauOpsDashboardConnectorHealthRow {
                channel: "none".to_string(),
                mode: "unknown".to_string(),
                liveness: "unknown".to_string(),
                events_ingested: 0,
                provider_failures: 0,
            }],
            channel_action_status: "idle".to_string(),
            channel_action: "none".to_string(),
            channel_action_channel: "none".to_string(),
            channel_action_reason: "none".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardHarnessBenchmarkCategoryRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardHarnessBenchmarkCategoryRow {
    pub category: String,
    pub task_count: usize,
    pub pass_count: usize,
    pub total_count: usize,
    pub pass_rate: String,
}

fn harness_benchmark_category_label(category: &str) -> String {
    let label = category
        .split(['_', '-'])
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if label.is_empty() {
        return "Uncategorized".to_string();
    }

    let mut chars = label.chars();
    let first = chars.next().expect("non-empty label").to_ascii_uppercase();
    format!("{first}{}", chars.as_str())
}

fn harness_queue_status_label(status: &str) -> String {
    match status {
        "applied" => "Applied".to_string(),
        "approved" => "Approved".to_string(),
        "blocked_approval_required" => "Blocked".to_string(),
        "completed" => "Completed".to_string(),
        "dry-run-passed" => "Dry Run Passed".to_string(),
        "needs-review" => "Needs Review".to_string(),
        "proposal" => "Proposal".to_string(),
        "rejected" => "Rejected".to_string(),
        other => harness_benchmark_category_label(other),
    }
}

fn harness_ops_artifact_href(proof_artifact: &str) -> Option<String> {
    let trimmed = proof_artifact.trim().trim_start_matches('/');
    let is_ops_harness_artifact = trimmed.starts_with("ops-harness/")
        && !trimmed.contains('\\')
        && trimmed
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..");

    is_ops_harness_artifact.then(|| format!("/ops/harness/artifacts/view/{trimmed}"))
}

fn harness_ops_artifact_href_with_query(
    proof_artifact: &str,
    query: Option<&str>,
) -> Option<String> {
    harness_ops_artifact_href(proof_artifact).map(|href| match query {
        Some(query) if !query.is_empty() => format!("{href}?{query}"),
        _ => href,
    })
}

fn sanitize_harness_audit_ref(raw: &str) -> String {
    raw.chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                Some(ch)
            } else if ch.is_ascii_whitespace() || matches!(ch, ':' | '/') {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn harness_audit_row_ref(row: &TauOpsDashboardHarnessAuditRow, index: usize) -> String {
    let timestamp = row.timestamp_unix_ms.trim();
    if !timestamp.is_empty() {
        return sanitize_harness_audit_ref(timestamp);
    }

    let fallback = format!("{}-{}-{}-{index}", row.action_key, row.item, row.result_key);
    let sanitized = sanitize_harness_audit_ref(&fallback);
    if sanitized.is_empty() {
        format!("audit-{index}")
    } else {
        sanitized
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardHarnessAuditRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardHarnessAuditRow {
    pub timestamp_label: String,
    pub timestamp_unix_ms: String,
    pub actor: String,
    pub action_label: String,
    pub action_key: String,
    pub scope: String,
    pub item: String,
    pub detail_label: String,
    pub detail_value: String,
    pub proof_artifact: String,
    pub result_label: String,
    pub result_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardHarnessMissionRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardHarnessMissionRow {
    pub mission_id: String,
    pub title: String,
    pub status_key: String,
    pub status_label: String,
    pub gate_status_key: String,
    pub gate_label: String,
    pub acceptance_label: String,
    pub plan_progress: usize,
    pub tool_budget: String,
    pub memory_hits: usize,
    pub verification_state: String,
    pub last_checkpoint: String,
    pub artifact_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardHarnessProposalQueueRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardHarnessProposalQueueRow {
    pub item_id: String,
    pub status_key: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardHarnessProposalDetail` in `tau-dashboard-ui`.
pub struct TauOpsDashboardHarnessProposalDetail {
    pub proposal_id: String,
    pub learning_record_id: String,
    pub title: String,
    pub target_type: String,
    pub target_path: String,
    pub dry_run_result_label: String,
    pub dry_run_result_key: String,
    pub safety_check_label: String,
    pub safety_check_key: String,
    pub rollback_plan: String,
    pub patch_summary: String,
    pub failure_observed: String,
    pub root_cause: String,
    pub test_evidence_href: String,
    pub test_evidence_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardHarnessProofRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardHarnessProofRow {
    pub item_id: String,
    pub status_key: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardHarnessToolEvidenceRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardHarnessToolEvidenceRow {
    pub tool_name: String,
    pub call_id: String,
    pub plan_node: String,
    pub runtime: String,
    pub status_key: String,
    pub artifact_label: String,
    pub artifact_href: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardHarnessArtifactRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardHarnessArtifactRow {
    pub item_id: String,
    pub status_key: String,
    pub label: String,
    pub href: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardHarnessSelfImprovementProof` in `tau-dashboard-ui`.
pub struct TauOpsDashboardHarnessSelfImprovementProof {
    pub source: String,
    pub mission_id: String,
    pub mission_status: String,
    pub plan_completed_count: usize,
    pub plan_total_count: usize,
    pub gate_passed_count: usize,
    pub gate_total_count: usize,
    pub memory_hit_count: usize,
    pub artifact_count: usize,
    pub final_learning_summary: String,
    pub final_learning_records: Vec<String>,
    pub plan_rows: Vec<TauOpsDashboardHarnessProofRow>,
    pub gate_rows: Vec<TauOpsDashboardHarnessProofRow>,
    pub artifact_rows: Vec<TauOpsDashboardHarnessProofRow>,
}

impl Default for TauOpsDashboardHarnessSelfImprovementProof {
    fn default() -> Self {
        Self {
            source: "missing".to_string(),
            mission_id: String::new(),
            mission_status: "unknown".to_string(),
            plan_completed_count: 0,
            plan_total_count: 0,
            gate_passed_count: 0,
            gate_total_count: 0,
            memory_hit_count: 0,
            artifact_count: 0,
            final_learning_summary: String::new(),
            final_learning_records: Vec::new(),
            plan_rows: Vec::new(),
            gate_rows: Vec::new(),
            artifact_rows: Vec::new(),
        }
    }
}

impl Default for TauOpsDashboardHarnessProposalDetail {
    fn default() -> Self {
        Self {
            proposal_id: "PR-044".to_string(),
            learning_record_id: "LR-044".to_string(),
            title: "Prompt compression for research tasks".to_string(),
            target_type: "Prompt".to_string(),
            target_path: "prompts/research_to_doc/system.md".to_string(),
            dry_run_result_label: "Tests passed (18/18)".to_string(),
            dry_run_result_key: "passed".to_string(),
            safety_check_label: "Passed".to_string(),
            safety_check_key: "passed".to_string(),
            rollback_plan: "Revert to previous prompt version".to_string(),
            patch_summary:
                "Compress system prompt by removing redundant instructions and examples."
                    .to_string(),
            failure_observed: "Token overrun during research-to-doc tasks".to_string(),
            root_cause: "Verbose prompts with redundant context".to_string(),
            test_evidence_href: "/evidence/pr-044-dryrun.json".to_string(),
            test_evidence_label: "evidence/pr-044-dryrun.json".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardHarnessSnapshot` in `tau-dashboard-ui`.
pub struct TauOpsDashboardHarnessSnapshot {
    pub runtime_workspace_label: String,
    pub runtime_model_label: String,
    pub runtime_transport_label: String,
    pub runtime_health_key: String,
    pub mission_table_title: String,
    pub kpi_missions_title: String,
    pub kpi_missions_count: usize,
    pub kpi_missions_detail: String,
    pub kpi_pending_verification_count: usize,
    pub kpi_pending_verification_detail: String,
    pub kpi_memory_write_count: usize,
    pub kpi_memory_write_detail: String,
    pub kpi_runtime_cost_today: String,
    pub kpi_runtime_cost_detail: String,
    pub proof_source: String,
    pub benchmark_id: String,
    pub proof_artifact: String,
    pub task_count: usize,
    pub pass_count: usize,
    pub failed_gate_count: usize,
    pub failed_gate_label: String,
    pub latest_result: String,
    pub latest_runtime: String,
    pub latest_cost: String,
    pub latest_summary: String,
    pub benchmark_rows: Vec<TauOpsDashboardHarnessBenchmarkCategoryRow>,
    pub detail_run_id: String,
    pub detail_proof_artifact: String,
    pub detail_goal: String,
    pub detail_status: String,
    pub detail_elapsed: String,
    pub detail_tool_budget: String,
    pub detail_cost: String,
    pub detail_retry_count: String,
    pub detail_plan_current_node: String,
    pub detail_plan_rows: Vec<TauOpsDashboardHarnessProofRow>,
    pub detail_tool_call_count: usize,
    pub detail_tool_rows: Vec<TauOpsDashboardHarnessToolEvidenceRow>,
    pub detail_operator_log: String,
    pub detail_acceptance_met_count: usize,
    pub detail_acceptance_total_count: usize,
    pub detail_acceptance_rows: Vec<TauOpsDashboardHarnessProofRow>,
    pub detail_gate_failed_count: usize,
    pub detail_gate_rows: Vec<TauOpsDashboardHarnessProofRow>,
    pub detail_memory_hit_count: usize,
    pub detail_learning_record_count: usize,
    pub detail_last_memory_write: String,
    pub detail_memory_evidence_label: String,
    pub detail_artifact_rows: Vec<TauOpsDashboardHarnessArtifactRow>,
    pub mission_rows: Vec<TauOpsDashboardHarnessMissionRow>,
    pub audit_source: String,
    pub audit_rows: Vec<TauOpsDashboardHarnessAuditRow>,
    pub audit_filter_action: String,
    pub audit_total_count: usize,
    pub audit_selected_ref: String,
    pub audit_selected_artifact_preview: String,
    pub audit_selected_artifact_preview_status: String,
    pub audit_selected_artifact_preview_bytes: usize,
    pub audit_selected_artifact_preview_truncated: bool,
    pub audit_selected_artifact_preview_limit: usize,
    pub selected_proposal_id: String,
    pub proposal_queue_source: String,
    pub proposal_queue_rows: Vec<TauOpsDashboardHarnessProposalQueueRow>,
    pub selected_proposal: TauOpsDashboardHarnessProposalDetail,
    pub self_improvement_proof: TauOpsDashboardHarnessSelfImprovementProof,
    pub route_action_key: String,
    pub route_action_label: String,
    pub route_action_detail: String,
    pub route_action_count: usize,
}

impl Default for TauOpsDashboardHarnessSnapshot {
    fn default() -> Self {
        Self {
            runtime_workspace_label: "/workspace/tau".to_string(),
            runtime_model_label: "gpt-5.4".to_string(),
            runtime_transport_label: "gateway".to_string(),
            runtime_health_key: "healthy".to_string(),
            mission_table_title: "Active Missions".to_string(),
            kpi_missions_title: "Active Missions".to_string(),
            kpi_missions_count: 5,
            kpi_missions_detail: "3 running".to_string(),
            kpi_pending_verification_count: 3,
            kpi_pending_verification_detail: "2 need review".to_string(),
            kpi_memory_write_count: 12,
            kpi_memory_write_detail: "Today".to_string(),
            kpi_runtime_cost_today: "$18.74".to_string(),
            kpi_runtime_cost_detail: "Across 5 runs".to_string(),
            proof_source: "fallback".to_string(),
            benchmark_id: "m334-tranche-one-autonomy".to_string(),
            proof_artifact: "/artifacts/bench/m334/latest.json".to_string(),
            task_count: 4,
            pass_count: 4,
            failed_gate_count: 0,
            failed_gate_label: "none".to_string(),
            latest_result: "4/4".to_string(),
            latest_runtime: "00:42:31".to_string(),
            latest_cost: "0.00".to_string(),
            latest_summary: "Latest deterministic result: 4/4. Failed gates: none.".to_string(),
            benchmark_rows: vec![
                TauOpsDashboardHarnessBenchmarkCategoryRow {
                    category: "repo_build".to_string(),
                    task_count: 1,
                    pass_count: 4,
                    total_count: 4,
                    pass_rate: "100".to_string(),
                },
                TauOpsDashboardHarnessBenchmarkCategoryRow {
                    category: "greenfield_build".to_string(),
                    task_count: 1,
                    pass_count: 4,
                    total_count: 4,
                    pass_rate: "100".to_string(),
                },
                TauOpsDashboardHarnessBenchmarkCategoryRow {
                    category: "research_design".to_string(),
                    task_count: 1,
                    pass_count: 4,
                    total_count: 4,
                    pass_rate: "100".to_string(),
                },
                TauOpsDashboardHarnessBenchmarkCategoryRow {
                    category: "data_to_deliverable".to_string(),
                    task_count: 1,
                    pass_count: 4,
                    total_count: 4,
                    pass_rate: "100".to_string(),
                },
            ],
            detail_run_id: "run_8f3a2".to_string(),
            detail_proof_artifact: "/artifacts/bench/m334/latest.json".to_string(),
            detail_goal: "Refactor plugin registry for safer hot reload".to_string(),
            detail_status: "running".to_string(),
            detail_elapsed: "01:42:18".to_string(),
            detail_tool_budget: "42/60".to_string(),
            detail_cost: "$3.82".to_string(),
            detail_retry_count: "1".to_string(),
            detail_plan_current_node: "verify".to_string(),
            detail_plan_rows: vec![
                TauOpsDashboardHarnessProofRow {
                    item_id: "plan".to_string(),
                    status_key: "passed".to_string(),
                    label: "Plan".to_string(),
                },
                TauOpsDashboardHarnessProofRow {
                    item_id: "execute".to_string(),
                    status_key: "passed".to_string(),
                    label: "Execute".to_string(),
                },
                TauOpsDashboardHarnessProofRow {
                    item_id: "memory-write".to_string(),
                    status_key: "passed".to_string(),
                    label: "Memory Write".to_string(),
                },
                TauOpsDashboardHarnessProofRow {
                    item_id: "verify".to_string(),
                    status_key: "running".to_string(),
                    label: "Verify".to_string(),
                },
                TauOpsDashboardHarnessProofRow {
                    item_id: "learn".to_string(),
                    status_key: "pending".to_string(),
                    label: "Learn".to_string(),
                },
            ],
            detail_tool_call_count: 8,
            detail_tool_rows: vec![
                TauOpsDashboardHarnessToolEvidenceRow {
                    tool_name: "repo.read".to_string(),
                    call_id: "c1a2bf3".to_string(),
                    plan_node: "Execute".to_string(),
                    runtime: "00:01:12".to_string(),
                    status_key: "passed".to_string(),
                    artifact_label: "/artifacts/repo-read.json".to_string(),
                    artifact_href: String::new(),
                },
                TauOpsDashboardHarnessToolEvidenceRow {
                    tool_name: "repo.edit".to_string(),
                    call_id: "c1a2b4c".to_string(),
                    plan_node: "Execute".to_string(),
                    runtime: "00:02:34".to_string(),
                    status_key: "passed".to_string(),
                    artifact_label: "/artifacts/edit.patch".to_string(),
                    artifact_href: String::new(),
                },
                TauOpsDashboardHarnessToolEvidenceRow {
                    tool_name: "test.run".to_string(),
                    call_id: "c1a2b6e".to_string(),
                    plan_node: "Execute".to_string(),
                    runtime: "00:08:42".to_string(),
                    status_key: "passed".to_string(),
                    artifact_label: "/artifacts/tests.json".to_string(),
                    artifact_href: String::new(),
                },
                TauOpsDashboardHarnessToolEvidenceRow {
                    tool_name: "memory.search".to_string(),
                    call_id: "c1a2b7f".to_string(),
                    plan_node: "Memory Write".to_string(),
                    runtime: "00:00:48".to_string(),
                    status_key: "passed".to_string(),
                    artifact_label: "/artifacts/memory.json".to_string(),
                    artifact_href: String::new(),
                },
                TauOpsDashboardHarnessToolEvidenceRow {
                    tool_name: "memory.write".to_string(),
                    call_id: "c1a2b8e".to_string(),
                    plan_node: "Memory Write".to_string(),
                    runtime: "00:00:36".to_string(),
                    status_key: "passed".to_string(),
                    artifact_label: "/artifacts/learning.json".to_string(),
                    artifact_href: String::new(),
                },
                TauOpsDashboardHarnessToolEvidenceRow {
                    tool_name: "report.write".to_string(),
                    call_id: "c1a2b9".to_string(),
                    plan_node: "Verify".to_string(),
                    runtime: "00:01:21".to_string(),
                    status_key: "running".to_string(),
                    artifact_label: "/artifacts/report.md".to_string(),
                    artifact_href: String::new(),
                },
            ],
            detail_operator_log: "10:18:22  Plan accepted
10:18:23  Executing plan with tool budget 42/60
10:18:25  repo.read call_id=c1a2b3 path=src/registry/**
10:18:37  repo.edit completed (42 files)
10:20:55  memory.write call_id=c1a2b8 record=learning
10:24:18  Verification started
10:25:52  verification gate VG-03 pending (collecting no-memory evidence)"
                .to_string(),
            detail_acceptance_met_count: 3,
            detail_acceptance_total_count: 5,
            detail_acceptance_rows: vec![
                TauOpsDashboardHarnessProofRow {
                    item_id: "VG-01".to_string(),
                    status_key: "met".to_string(),
                    label: "Registry loads plugins deterministically".to_string(),
                },
                TauOpsDashboardHarnessProofRow {
                    item_id: "VG-02".to_string(),
                    status_key: "met".to_string(),
                    label: "Hot reload preserves active sessions".to_string(),
                },
                TauOpsDashboardHarnessProofRow {
                    item_id: "VG-03".to_string(),
                    status_key: "met".to_string(),
                    label: "Added regression tests".to_string(),
                },
                TauOpsDashboardHarnessProofRow {
                    item_id: "VG-04".to_string(),
                    status_key: "pending".to_string(),
                    label: "Docs updated".to_string(),
                },
                TauOpsDashboardHarnessProofRow {
                    item_id: "VG-05".to_string(),
                    status_key: "pending".to_string(),
                    label: "Benchmark proof emitted".to_string(),
                },
            ],
            detail_gate_failed_count: 1,
            detail_gate_rows: vec![
                TauOpsDashboardHarnessProofRow {
                    item_id: "VG-01".to_string(),
                    status_key: "passed".to_string(),
                    label: "Planning proof".to_string(),
                },
                TauOpsDashboardHarnessProofRow {
                    item_id: "VG-02".to_string(),
                    status_key: "passed".to_string(),
                    label: "Tool execution proof".to_string(),
                },
                TauOpsDashboardHarnessProofRow {
                    item_id: "VG-03".to_string(),
                    status_key: "failed".to_string(),
                    label: "Memory proof".to_string(),
                },
                TauOpsDashboardHarnessProofRow {
                    item_id: "VG-04".to_string(),
                    status_key: "running".to_string(),
                    label: "Verification proof".to_string(),
                },
                TauOpsDashboardHarnessProofRow {
                    item_id: "VG-05".to_string(),
                    status_key: "pending".to_string(),
                    label: "Learning proof".to_string(),
                },
            ],
            detail_memory_hit_count: 12,
            detail_learning_record_count: 2,
            detail_last_memory_write: "10:20:55".to_string(),
            detail_memory_evidence_label: "Collected".to_string(),
            detail_artifact_rows: vec![
                TauOpsDashboardHarnessArtifactRow {
                    item_id: "code-changes".to_string(),
                    status_key: "artifact".to_string(),
                    label: "Code changes".to_string(),
                    href: "/artifacts/code.diff".to_string(),
                },
                TauOpsDashboardHarnessArtifactRow {
                    item_id: "docs".to_string(),
                    status_key: "artifact".to_string(),
                    label: "Docs".to_string(),
                    href: "/artifacts/docs.md".to_string(),
                },
                TauOpsDashboardHarnessArtifactRow {
                    item_id: "benchmark-proof".to_string(),
                    status_key: "artifact".to_string(),
                    label: "Benchmark proof".to_string(),
                    href: "/artifacts/proof.json".to_string(),
                },
            ],
            mission_rows: vec![
                TauOpsDashboardHarnessMissionRow {
                    mission_id: "run_linux_ci".to_string(),
                    title: "Port repo test harness to Linux CI".to_string(),
                    status_key: "running".to_string(),
                    status_label: "Running".to_string(),
                    gate_status_key: "needs-review".to_string(),
                    gate_label: "3/5 gates".to_string(),
                    acceptance_label: "4/5".to_string(),
                    plan_progress: 68,
                    tool_budget: "34/60".to_string(),
                    memory_hits: 12,
                    verification_state: "needs-review".to_string(),
                    last_checkpoint: "10:22:31 May 15".to_string(),
                    artifact_count: 5,
                },
                TauOpsDashboardHarnessMissionRow {
                    mission_id: "run_m334_flaky".to_string(),
                    title: "Investigate flaky benchmark on M334".to_string(),
                    status_key: "verifying".to_string(),
                    status_label: "Verifying".to_string(),
                    gate_status_key: "needs-review".to_string(),
                    gate_label: "2/5 gates".to_string(),
                    acceptance_label: "3/5".to_string(),
                    plan_progress: 72,
                    tool_budget: "28/60".to_string(),
                    memory_hits: 8,
                    verification_state: "needs-review".to_string(),
                    last_checkpoint: "10:25:52 May 15".to_string(),
                    artifact_count: 4,
                },
                TauOpsDashboardHarnessMissionRow {
                    mission_id: "run_research_brief".to_string(),
                    title: "Generate weekly research brief on model routing".to_string(),
                    status_key: "completed".to_string(),
                    status_label: "Completed".to_string(),
                    gate_status_key: "passed".to_string(),
                    gate_label: "5/5 gates".to_string(),
                    acceptance_label: "5/5".to_string(),
                    plan_progress: 100,
                    tool_budget: "18/60".to_string(),
                    memory_hits: 15,
                    verification_state: "passed".to_string(),
                    last_checkpoint: "09:55:11 May 15".to_string(),
                    artifact_count: 7,
                },
                TauOpsDashboardHarnessMissionRow {
                    mission_id: "run_receipts".to_string(),
                    title: "Automate receipt classification pipeline".to_string(),
                    status_key: "blocked".to_string(),
                    status_label: "Blocked".to_string(),
                    gate_status_key: "failed".to_string(),
                    gate_label: "1/5 gates".to_string(),
                    acceptance_label: "2/5".to_string(),
                    plan_progress: 36,
                    tool_budget: "16/60".to_string(),
                    memory_hits: 6,
                    verification_state: "failed".to_string(),
                    last_checkpoint: "09:48:03 May 15".to_string(),
                    artifact_count: 3,
                },
                TauOpsDashboardHarnessMissionRow {
                    mission_id: "run_8f3a2".to_string(),
                    title: "Refactor plugin registry for safer hot reload".to_string(),
                    status_key: "running".to_string(),
                    status_label: "Running".to_string(),
                    gate_status_key: "running".to_string(),
                    gate_label: "2/5 gates".to_string(),
                    acceptance_label: "3/5".to_string(),
                    plan_progress: 55,
                    tool_budget: "42/60".to_string(),
                    memory_hits: 12,
                    verification_state: "running".to_string(),
                    last_checkpoint: "10:24:18 May 15".to_string(),
                    artifact_count: 6,
                },
            ],
            audit_source: "fallback".to_string(),
            audit_rows: vec![
                TauOpsDashboardHarnessAuditRow {
                    timestamp_label: "May 15, 10:11".to_string(),
                    timestamp_unix_ms: "".to_string(),
                    actor: "Operator".to_string(),
                    action_label: "Dry Run".to_string(),
                    action_key: "dry-run".to_string(),
                    scope: "Prompt".to_string(),
                    item: "PR-044".to_string(),
                    detail_label: String::new(),
                    detail_value: String::new(),
                    proof_artifact: String::new(),
                    result_label: "Passed".to_string(),
                    result_key: "passed".to_string(),
                },
                TauOpsDashboardHarnessAuditRow {
                    timestamp_label: "May 15, 09:42".to_string(),
                    timestamp_unix_ms: "".to_string(),
                    actor: "Operator".to_string(),
                    action_label: "Apply".to_string(),
                    action_key: "apply".to_string(),
                    scope: "Config".to_string(),
                    item: "CL-031".to_string(),
                    detail_label: String::new(),
                    detail_value: String::new(),
                    proof_artifact: String::new(),
                    result_label: "Applied".to_string(),
                    result_key: "applied".to_string(),
                },
                TauOpsDashboardHarnessAuditRow {
                    timestamp_label: "May 15, 09:12".to_string(),
                    timestamp_unix_ms: "".to_string(),
                    actor: "Curator".to_string(),
                    action_label: "Apply".to_string(),
                    action_key: "apply".to_string(),
                    scope: "Skill".to_string(),
                    item: "SK-102".to_string(),
                    detail_label: String::new(),
                    detail_value: String::new(),
                    proof_artifact: String::new(),
                    result_label: "Applied".to_string(),
                    result_key: "applied".to_string(),
                },
                TauOpsDashboardHarnessAuditRow {
                    timestamp_label: "May 15, 08:33".to_string(),
                    timestamp_unix_ms: "".to_string(),
                    actor: "Operator".to_string(),
                    action_label: "Reject".to_string(),
                    action_key: "reject".to_string(),
                    scope: "Prompt".to_string(),
                    item: "PR-029".to_string(),
                    detail_label: String::new(),
                    detail_value: String::new(),
                    proof_artifact: String::new(),
                    result_label: "Rejected".to_string(),
                    result_key: "rejected".to_string(),
                },
            ],
            audit_filter_action: "all".to_string(),
            audit_total_count: 4,
            audit_selected_ref: String::new(),
            audit_selected_artifact_preview: String::new(),
            audit_selected_artifact_preview_status: "none".to_string(),
            audit_selected_artifact_preview_bytes: 0,
            audit_selected_artifact_preview_truncated: false,
            audit_selected_artifact_preview_limit: 2048,
            selected_proposal_id: "PR-044".to_string(),
            proposal_queue_source: "fallback".to_string(),
            proposal_queue_rows: vec![
                TauOpsDashboardHarnessProposalQueueRow {
                    item_id: "LR-219".to_string(),
                    status_key: "needs-review".to_string(),
                    label: "Retry storm in document synthesis".to_string(),
                },
                TauOpsDashboardHarnessProposalQueueRow {
                    item_id: "LR-220".to_string(),
                    status_key: "needs-review".to_string(),
                    label: "Missing memory write after verification".to_string(),
                },
                TauOpsDashboardHarnessProposalQueueRow {
                    item_id: "PR-044".to_string(),
                    status_key: "proposal".to_string(),
                    label: "Prompt compression for research tasks".to_string(),
                },
                TauOpsDashboardHarnessProposalQueueRow {
                    item_id: "PR-045".to_string(),
                    status_key: "proposal".to_string(),
                    label: "Skill patch for benchmark artifact naming".to_string(),
                },
            ],
            selected_proposal: TauOpsDashboardHarnessProposalDetail::default(),
            self_improvement_proof: TauOpsDashboardHarnessSelfImprovementProof::default(),
            route_action_key: "overview".to_string(),
            route_action_label: "Overview".to_string(),
            route_action_detail: String::new(),
            route_action_count: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardShellContext` in `tau-dashboard-ui`.
pub struct TauOpsDashboardShellContext {
    pub auth_mode: TauOpsDashboardAuthMode,
    pub active_route: TauOpsDashboardRoute,
    pub theme: TauOpsDashboardTheme,
    pub sidebar_state: TauOpsDashboardSidebarState,
    pub command_center: TauOpsDashboardCommandCenterSnapshot,
    pub chat: TauOpsDashboardChatSnapshot,
    pub harness: TauOpsDashboardHarnessSnapshot,
}

impl Default for TauOpsDashboardShellContext {
    fn default() -> Self {
        Self {
            auth_mode: TauOpsDashboardAuthMode::Token,
            active_route: TauOpsDashboardRoute::Ops,
            theme: TauOpsDashboardTheme::Dark,
            sidebar_state: TauOpsDashboardSidebarState::Expanded,
            command_center: TauOpsDashboardCommandCenterSnapshot::default(),
            chat: TauOpsDashboardChatSnapshot::default(),
            harness: TauOpsDashboardHarnessSnapshot::default(),
        }
    }
}

fn contains_markdown_contract_syntax(content: &str) -> bool {
    content.contains("```")
        || content.starts_with('#')
        || content.contains("\n#")
        || content.starts_with("- ")
        || content.contains("\n- ")
        || content.contains("](")
        || (content.contains('|') && content.contains("\n|---"))
}

fn extract_first_fenced_code_block(content: &str) -> Option<(String, String)> {
    let fence_start = content.find("```")?;
    let after_open_fence = &content[fence_start + 3..];
    let fence_end = after_open_fence.find("```")?;
    let fenced_block = &after_open_fence[..fence_end];
    let (language, code) = if let Some((language, code)) = fenced_block.split_once('\n') {
        (language.trim(), code.trim())
    } else {
        ("plain", fenced_block.trim())
    };
    if code.is_empty() {
        return None;
    }
    let language = if language.is_empty() {
        "plain"
    } else {
        language
    };
    Some((language.to_string(), code.to_string()))
}

fn extract_assistant_stream_tokens(content: &str) -> Vec<String> {
    content
        .split_whitespace()
        .map(ToString::to_string)
        .collect()
}

fn current_unix_ms() -> u64 {
    let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return 0;
    };
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

fn format_chat_session_updated_label_at(updated_unix_ms: u64, now_unix_ms: u64) -> String {
    if updated_unix_ms == 0 {
        return "never".to_string();
    }
    if updated_unix_ms >= now_unix_ms {
        return "just now".to_string();
    }

    let age_ms = now_unix_ms - updated_unix_ms;
    let age_seconds = age_ms / 1_000;
    if age_seconds < 60 {
        return "just now".to_string();
    }

    let age_minutes = age_seconds / 60;
    if age_minutes < 60 {
        return format!("{age_minutes}m ago");
    }

    let age_hours = age_minutes / 60;
    if age_hours < 24 {
        return format!("{age_hours}h ago");
    }

    let age_days = age_hours / 24;
    format!("{age_days}d ago")
}

fn derive_memory_graph_node_size_contracts(importance: &str) -> (&'static str, String) {
    let normalized_importance = importance
        .parse::<f32>()
        .ok()
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);
    let size_bucket = if normalized_importance < 0.34 {
        "small"
    } else if normalized_importance < 0.67 {
        "medium"
    } else {
        "large"
    };
    let size_px = format!("{:.2}", 12.0 + (normalized_importance * 16.0));
    (size_bucket, size_px)
}

fn derive_memory_graph_node_color_contracts(memory_type: &str) -> (&'static str, &'static str) {
    match memory_type.trim() {
        "goal" => ("goal", "#f59e0b"),
        "fact" => ("fact", "#2563eb"),
        "event" => ("event", "#7c3aed"),
        "observation" => ("observation", "#0d9488"),
        _ => ("unknown", "#6b7280"),
    }
}

fn derive_memory_graph_edge_style_contracts(relation_type: &str) -> (&'static str, &'static str) {
    let normalized = relation_type.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "related_to" | "relates_to" | "supports" | "references" | "part_of" => ("solid", "none"),
        "updates" | "caused_by" | "depends_on" | "result_of" => ("dashed", "6 4"),
        "contradicts" | "blocks" => ("dotted", "2 4"),
        _ => ("solid", "none"),
    }
}

/// Public `fn` `render_tau_ops_dashboard_shell` in `tau-dashboard-ui`.
pub fn render_tau_ops_dashboard_shell() -> String {
    render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext::default())
}

/// Public `fn` `render_tau_ops_dashboard_shell_for_route` in `tau-dashboard-ui`.
pub fn render_tau_ops_dashboard_shell_for_route(route: &str) -> String {
    let context = TauOpsDashboardShellContext {
        active_route: TauOpsDashboardRoute::from_shell_path(route)
            .unwrap_or(TauOpsDashboardRoute::Ops),
        ..TauOpsDashboardShellContext::default()
    };
    render_tau_ops_dashboard_shell_with_context(context)
}

/// Public `fn` `render_tau_ops_dashboard_shell_with_context` in `tau-dashboard-ui`.
pub fn render_tau_ops_dashboard_shell_with_context(context: TauOpsDashboardShellContext) -> String {
    let auth_mode = context.auth_mode;
    let login_required = auth_mode.requires_authentication();
    let auth_mode_attr = auth_mode.as_str();
    let active_route_attr = context.active_route.as_str();
    let active_shell_path = context.active_route.shell_path();
    let theme_attr = context.theme.as_str();
    let sidebar_state_attr = context.sidebar_state.as_str();
    let breadcrumb_current = context.active_route.breadcrumb_token();
    let breadcrumb_label = context.active_route.breadcrumb_label();
    let shell_subtitle = format!("{breadcrumb_label} operator control surface");
    let sidebar_toggle_target_state = context.sidebar_state.toggled().as_str();
    let sidebar_toggle_href =
        format!("{active_shell_path}?theme={theme_attr}&sidebar={sidebar_toggle_target_state}");
    let dark_theme_href = format!("{active_shell_path}?theme=dark&sidebar={sidebar_state_attr}");
    let light_theme_href = format!("{active_shell_path}?theme=light&sidebar={sidebar_state_attr}");
    let dark_theme_pressed = if matches!(context.theme, TauOpsDashboardTheme::Dark) {
        "true"
    } else {
        "false"
    };
    let light_theme_pressed = if matches!(context.theme, TauOpsDashboardTheme::Light) {
        "true"
    } else {
        "false"
    };
    let login_hidden = if matches!(context.active_route, TauOpsDashboardRoute::Login) {
        "false"
    } else {
        "true"
    };
    let protected_hidden = if matches!(context.active_route, TauOpsDashboardRoute::Login) {
        "true"
    } else {
        "false"
    };
    let chat_route_active = matches!(context.active_route, TauOpsDashboardRoute::Chat);
    let chat_panel_hidden = if chat_route_active { "false" } else { "true" };
    let chat_panel_visible = if chat_route_active { "true" } else { "false" };
    let sessions_route_active = matches!(context.active_route, TauOpsDashboardRoute::Sessions);
    let sessions_panel_hidden = if sessions_route_active {
        "false"
    } else {
        "true"
    };
    let sessions_panel_visible = if sessions_route_active {
        "true"
    } else {
        "false"
    };
    let memory_panel_hidden = if matches!(context.active_route, TauOpsDashboardRoute::Memory) {
        "false"
    } else {
        "true"
    };
    let memory_panel_visible = if matches!(context.active_route, TauOpsDashboardRoute::Memory) {
        "true"
    } else {
        "false"
    };
    let memory_graph_panel_hidden =
        if matches!(context.active_route, TauOpsDashboardRoute::MemoryGraph) {
            "false"
        } else {
            "true"
        };
    let memory_graph_panel_visible =
        if matches!(context.active_route, TauOpsDashboardRoute::MemoryGraph) {
            "true"
        } else {
            "false"
        };
    let tools_panel_hidden = if matches!(context.active_route, TauOpsDashboardRoute::ToolsJobs) {
        "false"
    } else {
        "true"
    };
    let tools_panel_visible = if matches!(context.active_route, TauOpsDashboardRoute::ToolsJobs) {
        "true"
    } else {
        "false"
    };
    let harness_panel_hidden = if matches!(context.active_route, TauOpsDashboardRoute::Harness) {
        "false"
    } else {
        "true"
    };
    let harness_panel_visible = if matches!(context.active_route, TauOpsDashboardRoute::Harness) {
        "true"
    } else {
        "false"
    };
    let harness_route_active = matches!(context.active_route, TauOpsDashboardRoute::Harness);
    let harness_history_view_active =
        harness_route_active && context.harness.route_action_key == "history";
    let harness_route_action_visible = if harness_route_active
        && context.harness.route_action_key != "overview"
        && !harness_history_view_active
    {
        "true"
    } else {
        "false"
    };
    let harness_route_action_hidden = harness_route_action_visible == "false";
    let harness_task_count = context.harness.task_count.to_string();
    let harness_pass_count = context.harness.pass_count.to_string();
    let harness_audit_row_count = context.harness.audit_rows.len().to_string();
    let harness_history_total_count = if context.harness.audit_total_count == 0 {
        context.harness.audit_rows.len()
    } else {
        context.harness.audit_total_count
    }
    .to_string();
    let harness_history_filter_action = if context.harness.audit_filter_action.trim().is_empty() {
        "all".to_string()
    } else {
        context.harness.audit_filter_action.clone()
    };
    let harness_history_proof_count = context
        .harness
        .audit_rows
        .iter()
        .filter(|row| !row.proof_artifact.is_empty())
        .count()
        .to_string();
    let harness_history_latest_action = context
        .harness
        .audit_rows
        .first()
        .map(|row| format!("{} {} {}", row.action_label, row.item, row.result_label))
        .unwrap_or_else(|| "No audit records".to_string());
    let harness_history_latest_timestamp = context
        .harness
        .audit_rows
        .first()
        .map(|row| row.timestamp_label.clone())
        .unwrap_or_else(|| "none".to_string());
    let harness_selected_proposal_id = context.harness.selected_proposal.proposal_id.clone();
    let harness_action_query = format!(
        "theme={theme_attr}&sidebar={sidebar_state_attr}&session={}",
        context.chat.active_session_key.clone()
    );
    let harness_benchmark_action = format!(
        "/ops/harness/run-benchmark?theme={theme_attr}&sidebar={sidebar_state_attr}&session={}&proposal_id={}",
        context.chat.active_session_key.clone(), harness_selected_proposal_id
    );
    let harness_selected_approve_action = format!(
        "/ops/harness/proposals/{}/approve?{}",
        harness_selected_proposal_id, harness_action_query
    );
    let harness_selected_reject_action = format!(
        "/ops/harness/proposals/{}/reject?{}",
        harness_selected_proposal_id, harness_action_query
    );
    let harness_selected_dry_run_action = format!(
        "/ops/harness/proposals/{}/dry-run?{}",
        harness_selected_proposal_id, harness_action_query
    );
    let harness_selected_apply_action = format!(
        "/ops/harness/proposals/{}/apply?{}",
        harness_selected_proposal_id, harness_action_query
    );
    let harness_selected_diff_href = format!(
        "/ops/harness/proposals/{}/diff?{}",
        harness_selected_proposal_id, harness_action_query
    );
    let harness_queue_theme = theme_attr.to_string();
    let harness_queue_sidebar = sidebar_state_attr.to_string();
    let harness_queue_session_key = context.chat.active_session_key.clone();
    let harness_new_mission_action = format!(
        "/ops/harness/missions/draft?theme={theme_attr}&sidebar={sidebar_state_attr}&session={}&proposal_id={}",
        context.chat.active_session_key, harness_selected_proposal_id
    );
    let harness_history_href = format!(
        "/ops/harness?theme={theme_attr}&sidebar={sidebar_state_attr}&session={}&proposal_id={}&view=history",
        context.chat.active_session_key, harness_selected_proposal_id
    );
    let harness_history_dry_run_href = format!("{harness_history_href}&audit_action=dry-run");
    let harness_history_apply_href = format!("{harness_history_href}&audit_action=apply");
    let harness_history_benchmark_href =
        format!("{harness_history_href}&audit_action=run-benchmark");
    let harness_history_start_href = format!("{harness_history_href}&audit_action=start-mission");
    let harness_history_filter_all_current = if harness_history_filter_action == "all" {
        "page"
    } else {
        "false"
    };
    let harness_history_filter_dry_run_current = if harness_history_filter_action == "dry-run" {
        "page"
    } else {
        "false"
    };
    let harness_history_filter_apply_current = if harness_history_filter_action == "apply" {
        "page"
    } else {
        "false"
    };
    let harness_history_filter_benchmark_current =
        if harness_history_filter_action == "run-benchmark" {
            "page"
        } else {
            "false"
        };
    let harness_history_filter_start_current = if harness_history_filter_action == "start-mission" {
        "page"
    } else {
        "false"
    };
    let harness_history_current_href = if harness_history_filter_action == "all" {
        harness_history_href.clone()
    } else {
        format!("{harness_history_href}&audit_action={harness_history_filter_action}")
    };
    let harness_history_scope_label = if harness_selected_proposal_id.trim().is_empty() {
        "current harness state".to_string()
    } else {
        format!("selected proposal {harness_selected_proposal_id}")
    };
    let harness_history_artifact_base_query = if harness_history_filter_action == "all" {
        format!(
            "theme={theme_attr}&sidebar={sidebar_state_attr}&session={}&proposal_id={}&view=history",
            context.chat.active_session_key, harness_selected_proposal_id
        )
    } else {
        format!(
            "theme={theme_attr}&sidebar={sidebar_state_attr}&session={}&proposal_id={}&view=history&audit_action={harness_history_filter_action}",
            context.chat.active_session_key, harness_selected_proposal_id
        )
    };
    let requested_audit_ref = sanitize_harness_audit_ref(&context.harness.audit_selected_ref);
    let harness_history_selected_audit_index = context
        .harness
        .audit_rows
        .iter()
        .enumerate()
        .find(|(index, row)| {
            !requested_audit_ref.is_empty()
                && harness_audit_row_ref(row, *index) == requested_audit_ref
        })
        .map(|(index, _)| index)
        .or({
            if context.harness.audit_rows.is_empty() {
                None
            } else {
                Some(0)
            }
        });
    let harness_history_selected_audit_ref = harness_history_selected_audit_index
        .and_then(|index| {
            context
                .harness
                .audit_rows
                .get(index)
                .map(|row| harness_audit_row_ref(row, index))
        })
        .unwrap_or_else(|| "none".to_string());
    let harness_overview_href = format!(
        "/ops/harness?theme={theme_attr}&sidebar={sidebar_state_attr}&session={}&proposal_id={}",
        context.chat.active_session_key, harness_selected_proposal_id
    );
    let harness_selected_latest_operator_decision = context.harness.audit_rows.iter().find(|row| {
        row.item == harness_selected_proposal_id
            && matches!(row.action_key.as_str(), "approve" | "reject" | "apply")
    });
    let harness_selected_queue_status = context
        .harness
        .proposal_queue_rows
        .iter()
        .find(|row| row.item_id == harness_selected_proposal_id)
        .map(|row| row.status_key.as_str());
    let harness_selected_completed_proof = context.harness.self_improvement_proof.source == "state"
        && context.harness.self_improvement_proof.mission_status == "completed";
    let harness_selected_apply_enabled =
        harness_selected_latest_operator_decision.is_some_and(|row| {
            row.action_key == "approve"
                && !row.result_key.starts_with("blocked")
                && !row.result_key.ends_with("failed")
        });
    let harness_selected_apply_disabled = if harness_selected_apply_enabled {
        "false"
    } else {
        "true"
    };
    let harness_selected_apply_aria_disabled = harness_selected_apply_disabled;
    let harness_selected_apply_state = harness_selected_latest_operator_decision
        .map(|row| match row.action_key.as_str() {
            "approve" if harness_selected_apply_enabled => "approved",
            "reject" => "rejected",
            "apply" if row.result_key == "applied" => "applied",
            _ => "approval-required",
        })
        .unwrap_or_else(|| {
            if harness_selected_completed_proof
                || harness_selected_queue_status == Some("completed")
            {
                "applied"
            } else {
                match harness_selected_queue_status {
                    Some("applied") => "applied",
                    Some("approved") => "approved",
                    Some("rejected") => "rejected",
                    _ => "approval-required",
                }
            }
        });
    let harness_selected_apply_label = match harness_selected_apply_state {
        "approved" => "Apply",
        "applied" => "Applied",
        "rejected" => "Rejected",
        _ => "Apply (Approval Required)",
    };
    let harness_selected_review_terminal =
        matches!(harness_selected_apply_state, "applied" | "rejected");
    let harness_selected_review_terminal_attr = if harness_selected_review_terminal {
        "true"
    } else {
        "false"
    };
    let harness_selected_decision_control_disabled = if harness_selected_review_terminal {
        "true"
    } else {
        "false"
    };
    let harness_selected_decision_label = match harness_selected_apply_state {
        "approved" => "Approved",
        "applied" => "Applied",
        "rejected" => "Rejected",
        _ => "Awaiting approval",
    };
    let harness_selected_decision_detail = match harness_selected_apply_state {
        "approved" => "Approval is recorded; apply remains available.",
        "applied" => {
            "Approval and rejection are closed; inspect diff, dry-run evidence, or audit history."
        }
        "rejected" => "This proposal was rejected; approval and apply are closed.",
        _ => "Approve or reject before apply can run.",
    };
    let harness_proposal_queue_rows = context
        .harness
        .proposal_queue_rows
        .iter()
        .map(|row| {
            let row_is_selected = row.item_id == harness_selected_proposal_id;
            let row_is_proposal = row.item_id.starts_with("PR-");
            let row_selected = if row_is_selected { "true" } else { "false" };
            let row_actionable = if row_is_proposal { "true" } else { "false" };
            let row_href = if row_is_proposal {
                format!(
                    "/ops/harness?theme={}&sidebar={}&session={}&proposal_id={}",
                    harness_queue_theme,
                    harness_queue_sidebar,
                    harness_queue_session_key,
                    row.item_id
                )
            } else {
                "#tau-ops-harness-learning-queue".to_string()
            };
            let row_current = if row_is_selected && harness_route_active {
                "page"
            } else {
                "false"
            };
            let row_status_label = harness_queue_status_label(&row.status_key);
            let row_content = if row_is_proposal {
                leptos::either::Either::Left(view! {
                    <a href=row_href data-proposal-link=row.item_id.clone() aria-current=row_current>
                        <span class="tau-harness-queue-label">{row.label.clone()}</span>
                        <span class="tau-harness-queue-status">{row_status_label}</span>
                    </a>
                })
            } else {
                leptos::either::Either::Right(view! {
                    <span class="tau-harness-queue-static">
                        <span class="tau-harness-queue-label">{row.label.clone()}</span>
                        <span class="tau-harness-queue-status">{row_status_label}</span>
                    </span>
                })
            };
            view! {
                <li
                    data-learning-id=row.item_id.clone()
                    data-status=row.status_key.clone()
                    data-selected=row_selected
                    data-actionable=row_actionable
                >
                    {row_content}
                </li>
            }
        })
        .collect_view();
    let harness_proposal_queue_count = context.harness.proposal_queue_rows.len().to_string();
    let harness_self_improvement_plan_rows = context
        .harness
        .self_improvement_proof
        .plan_rows
        .iter()
        .map(|row| {
            view! {
                <li data-proof-row="plan" data-proof-id=row.item_id.clone() data-proof-status=row.status_key.clone()>{row.label.clone()}</li>
            }
        })
        .collect_view();
    let harness_self_improvement_gate_rows = context
        .harness
        .self_improvement_proof
        .gate_rows
        .iter()
        .map(|row| {
            view! {
                <li data-proof-row="gate" data-proof-id=row.item_id.clone() data-proof-status=row.status_key.clone()>{format!("{}: {}", row.item_id, row.label)}</li>
            }
        })
        .collect_view();
    let harness_self_improvement_artifact_rows = context
        .harness
        .self_improvement_proof
        .artifact_rows
        .iter()
        .map(|row| {
            let artifact_content = if let Some(artifact_href) = harness_ops_artifact_href(&row.label)
            {
                view! {
                    <a
                        href=artifact_href
                        data-proof-artifact-href="true"
                        data-proof-artifact-path=row.label.clone()
                    >
                        {row.label.clone()}
                    </a>
                }
                .into_any()
            } else {
                view! { <span>{row.label.clone()}</span> }.into_any()
            };
            view! {
                <li data-proof-row="artifact" data-proof-id=row.item_id.clone() data-proof-status=row.status_key.clone()>{artifact_content}</li>
            }
        })
        .collect_view();
    let harness_self_improvement_records = context
        .harness
        .self_improvement_proof
        .final_learning_records
        .join(",");
    let harness_self_improvement_plan_completed = context
        .harness
        .self_improvement_proof
        .plan_completed_count
        .to_string();
    let harness_self_improvement_plan_total = context
        .harness
        .self_improvement_proof
        .plan_total_count
        .to_string();
    let harness_self_improvement_gate_passed = context
        .harness
        .self_improvement_proof
        .gate_passed_count
        .to_string();
    let harness_self_improvement_gate_total = context
        .harness
        .self_improvement_proof
        .gate_total_count
        .to_string();
    let harness_self_improvement_memory_hits = context
        .harness
        .self_improvement_proof
        .memory_hit_count
        .to_string();
    let harness_self_improvement_artifact_count = context
        .harness
        .self_improvement_proof
        .artifact_count
        .to_string();
    let harness_self_improvement_proof = if context.harness.self_improvement_proof.source == "state"
    {
        leptos::either::Either::Left(view! {
            <section
                id="tau-ops-harness-self-improvement-proof"
                data-proof-source=context.harness.self_improvement_proof.source.clone()
                data-mission-id=context.harness.self_improvement_proof.mission_id.clone()
                data-mission-status=context.harness.self_improvement_proof.mission_status.clone()
                data-plan-completed=harness_self_improvement_plan_completed
                data-plan-total=harness_self_improvement_plan_total
                data-gates-passed=harness_self_improvement_gate_passed
                data-gates-total=harness_self_improvement_gate_total
                data-memory-hits=harness_self_improvement_memory_hits
                data-artifact-count=harness_self_improvement_artifact_count
                data-final-learning-records=harness_self_improvement_records
            >
                <h4>"Mission Proof"</h4>
                <p data-proof-row="summary">{context.harness.self_improvement_proof.final_learning_summary.clone()}</p>
                <div class="tau-harness-self-improvement-proof-grid">
                    <section data-proof-group="plan">
                        <h5>"Plan"</h5>
                        <ul>{harness_self_improvement_plan_rows}</ul>
                    </section>
                    <section data-proof-group="gates">
                        <h5>"Gates"</h5>
                        <ul>{harness_self_improvement_gate_rows}</ul>
                    </section>
                    <section data-proof-group="artifacts">
                        <h5>"Artifacts"</h5>
                        <ul>{harness_self_improvement_artifact_rows}</ul>
                    </section>
                </div>
            </section>
        })
    } else {
        leptos::either::Either::Right(())
    };
    let harness_benchmark_rows = context
        .harness
        .benchmark_rows
        .iter()
        .map(|row| {
            let task_count = row.task_count.to_string();
            let last_run = format!("{}/{} pass", row.pass_count, row.total_count);
            let last_run_attr = last_run.clone();
            let category_label = harness_benchmark_category_label(&row.category);
            let category_label_attr = category_label.clone();
            view! {
                <tr
                    data-category=row.category.clone()
                    data-task-count=task_count
                    data-last-run=last_run_attr
                    data-pass-rate=row.pass_rate.clone()
                >
                    <td data-category-label=category_label_attr><span class="tau-harness-benchmark-category-label">{category_label}</span></td>
                    <td>{row.task_count}</td>
                    <td>{last_run}</td>
                    <td>{format!("{}%", row.pass_rate)}</td>
                </tr>
            }
        })
        .collect_view();
    let harness_audit_rows = context
        .harness
        .audit_rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let audit_row_ref = harness_audit_row_ref(row, index);
            let audit_selected = if Some(index) == harness_history_selected_audit_index {
                "true"
            } else {
                "false"
            };
            let audit_selected_current = if audit_selected == "true" {
                "page"
            } else {
                "false"
            };
            let audit_detail_href =
                format!("{harness_history_current_href}&audit_ref={audit_row_ref}");
            let audit_item_cell = if !harness_history_view_active
                && row.detail_value.is_empty()
                && row.proof_artifact.is_empty()
            {
                view! { <td>{row.item.clone()}</td> }.into_any()
            } else if row.detail_value.is_empty() && row.proof_artifact.is_empty() {
                view! {
                    <td data-audit-item-cell="item-proof">
                        <span>{row.item.clone()}</span>
                        <a
                            href=audit_detail_href.clone()
                            class="tau-harness-audit-inspect"
                            data-audit-inspect-link="true"
                            data-audit-ref=audit_row_ref.clone()
                            aria-current=audit_selected_current
                        >
                            "Inspect"
                        </a>
                    </td>
                }
                .into_any()
            } else {
                let audit_detail = if row.detail_value.is_empty() {
                    view! {
                        <span
                            class="tau-harness-audit-detail"
                            data-audit-detail-visible="false"
                            hidden
                        ></span>
                    }
                    .into_any()
                } else {
                    view! {
                        <span
                            class="tau-harness-audit-detail"
                            data-audit-detail-visible="true"
                            data-audit-detail-label=row.detail_label.clone()
                            data-audit-detail-value=row.detail_value.clone()
                        >
                            {format!("{} {}", row.detail_label, row.detail_value)}
                        </span>
                    }
                    .into_any()
                };
                let audit_proof = if row.proof_artifact.is_empty() {
                    view! {
                        <span
                            class="tau-harness-audit-proof"
                            data-audit-proof-visible="false"
                            hidden
                        ></span>
                    }
                    .into_any()
                } else {
                    let artifact_query = harness_history_view_active.then(|| {
                        format!("{harness_history_artifact_base_query}&audit_ref={audit_row_ref}")
                    });
                    let proof_href = harness_ops_artifact_href_with_query(
                        &row.proof_artifact,
                        artifact_query.as_deref(),
                    )
                    .unwrap_or_else(|| row.proof_artifact.clone());
                    view! {
                        <a
                            href=proof_href
                            class="tau-harness-audit-proof"
                            data-audit-proof-visible="true"
                            data-audit-proof-artifact=row.proof_artifact.clone()
                        >
                            {format!("Proof {}", row.proof_artifact)}
                        </a>
                    }
                    .into_any()
                };
                if harness_history_view_active {
                    let audit_inspect = view! {
                        <a
                            href=audit_detail_href.clone()
                            class="tau-harness-audit-inspect"
                            data-audit-inspect-link="true"
                            data-audit-ref=audit_row_ref.clone()
                            aria-current=audit_selected_current
                        >
                            "Inspect"
                        </a>
                    }
                    .into_any();
                    view! {
                        <td data-audit-item-cell="item-proof">
                            <span>{row.item.clone()}</span>
                            {audit_detail}
                            {audit_proof}
                            {audit_inspect}
                        </td>
                    }
                    .into_any()
                } else {
                    view! {
                        <td data-audit-item-cell="item-proof">
                            <span>{row.item.clone()}</span>
                            {audit_detail}
                            {audit_proof}
                        </td>
                    }
                    .into_any()
                }
            };
            view! {
                <tr
                    data-action=row.action_key.clone()
                    data-result=row.result_key.clone()
                    data-timestamp-unix-ms=row.timestamp_unix_ms.clone()
                    data-audit-detail-label=row.detail_label.clone()
                    data-audit-detail-value=row.detail_value.clone()
                    data-audit-proof-artifact=row.proof_artifact.clone()
                    data-audit-row="true"
                    data-audit-ref=audit_row_ref
                    data-audit-selected=audit_selected
                >
                    <td>{row.timestamp_label.clone()}</td>
                    <td>{row.actor.clone()}</td>
                    <td>{row.action_label.clone()}</td>
                    <td>{row.scope.clone()}</td>
                    {audit_item_cell}
                    <td>{row.result_label.clone()}</td>
                </tr>
            }
        })
        .collect_view();
    let harness_history_selected_audit_detail = if let Some(selected_index) =
        harness_history_selected_audit_index
    {
        let selected_row = &context.harness.audit_rows[selected_index];
        let selected_detail_value = if selected_row.detail_value.is_empty() {
            "none".to_string()
        } else {
            format!(
                "{} {}",
                selected_row.detail_label.clone(),
                selected_row.detail_value.clone()
            )
        };
        let selected_proof_label = if selected_row.proof_artifact.is_empty() {
            "none".to_string()
        } else {
            selected_row.proof_artifact.clone()
        };
        let selected_artifact_query = harness_history_view_active.then(|| {
            format!(
                "{harness_history_artifact_base_query}&audit_ref={harness_history_selected_audit_ref}"
            )
        });
        let selected_preview_visible =
            context.harness.audit_selected_artifact_preview_status == "loaded";
        let selected_preview_visible_attr = if selected_preview_visible {
            "true"
        } else {
            "false"
        };
        let selected_preview_bytes = context
            .harness
            .audit_selected_artifact_preview_bytes
            .to_string();
        let selected_preview_limit = context
            .harness
            .audit_selected_artifact_preview_limit
            .to_string();
        let selected_preview_truncated =
            if context.harness.audit_selected_artifact_preview_truncated {
                "true"
            } else {
                "false"
            };
        let selected_preview_label = if selected_preview_visible {
            if context.harness.audit_selected_artifact_preview_truncated {
                format!(
                    "{} bytes shown; capped at {} bytes",
                    context.harness.audit_selected_artifact_preview_bytes,
                    context.harness.audit_selected_artifact_preview_limit
                )
            } else {
                format!(
                    "{} bytes shown",
                    context.harness.audit_selected_artifact_preview_bytes
                )
            }
        } else {
            "No preview available".to_string()
        };
        let selected_preview_body = if selected_preview_visible {
            context.harness.audit_selected_artifact_preview.clone()
        } else if selected_row.proof_artifact.is_empty() {
            "Selected audit record has no proof artifact.".to_string()
        } else {
            "Proof artifact preview is unavailable for this record.".to_string()
        };
        let selected_proof_link = harness_ops_artifact_href_with_query(
            &selected_row.proof_artifact,
            selected_artifact_query.as_deref(),
        )
        .map(|proof_href| {
            view! {
                <a
                    href=proof_href
                    data-history-selected-proof-link="true"
                    data-history-selected-proof-artifact=selected_row.proof_artifact.clone()
                >
                    {selected_row.proof_artifact.clone()}
                </a>
            }
            .into_any()
        })
        .unwrap_or_else(|| {
            view! {
                <span
                    data-history-selected-proof-link="false"
                    data-history-selected-proof-artifact=selected_row.proof_artifact.clone()
                >
                    {selected_proof_label.clone()}
                </span>
            }
            .into_any()
        });
        leptos::either::Either::Left(view! {
            <section
                id="tau-ops-harness-history-detail"
                data-history-selected-audit="true"
                data-history-selected-audit-ref=harness_history_selected_audit_ref.clone()
                data-selected-action=selected_row.action_key.clone()
                data-selected-result=selected_row.result_key.clone()
                data-selected-proof-artifact=selected_row.proof_artifact.clone()
            >
                <header>
                    <div>
                        <h4>"Selected Audit Record"</h4>
                        <p>{format!("{} {} {}", selected_row.action_label, selected_row.item, selected_row.result_label)}</p>
                    </div>
                    {selected_proof_link}
                </header>
                <dl>
                    <div><dt>"Time"</dt><dd>{selected_row.timestamp_label.clone()}</dd></div>
                    <div><dt>"Actor"</dt><dd>{selected_row.actor.clone()}</dd></div>
                    <div><dt>"Action"</dt><dd>{selected_row.action_label.clone()}</dd></div>
                    <div><dt>"Scope"</dt><dd>{selected_row.scope.clone()}</dd></div>
                    <div><dt>"Item"</dt><dd>{selected_row.item.clone()}</dd></div>
                    <div><dt>"Result"</dt><dd>{selected_row.result_label.clone()}</dd></div>
                    <div><dt>"Detail"</dt><dd>{selected_detail_value}</dd></div>
                    <div><dt>"Proof"</dt><dd>{selected_proof_label}</dd></div>
                </dl>
                <section
                    class="tau-harness-history-preview"
                    data-history-selected-preview=selected_preview_visible_attr
                    data-history-selected-preview-status=context.harness.audit_selected_artifact_preview_status.clone()
                    data-history-selected-preview-bytes=selected_preview_bytes
                    data-history-selected-preview-limit=selected_preview_limit
                    data-history-selected-preview-truncated=selected_preview_truncated
                >
                    <header>
                        <h5>"Proof Preview"</h5>
                        <span>{selected_preview_label}</span>
                    </header>
                    <pre>{selected_preview_body}</pre>
                </section>
            </section>
        })
    } else {
        leptos::either::Either::Right(view! {
            <section
                id="tau-ops-harness-history-detail"
                data-history-selected-audit="false"
                data-history-selected-audit-ref="none"
                data-selected-action="none"
                data-selected-result="none"
                data-selected-proof-artifact=""
            >
                <h4>"Selected Audit Record"</h4>
                <p>"No audit record is available for this history filter."</p>
            </section>
        })
    };
    let harness_history_view = if harness_history_view_active {
        leptos::either::Either::Left(view! {
            <section
                id="tau-ops-harness-history-view"
                data-history-view="true"
                data-history-route-priority="primary"
                data-history-source=context.harness.audit_source.clone()
                data-history-row-count=harness_audit_row_count.clone()
                data-history-total-count=harness_history_total_count.clone()
                data-history-proof-count=harness_history_proof_count.clone()
                data-history-action-filter=harness_history_filter_action.clone()
                data-history-selected-proposal=harness_selected_proposal_id.clone()
                data-history-latest-action=harness_history_latest_action.clone()
                data-history-latest-timestamp=harness_history_latest_timestamp.clone()
            >
                <header>
                    <div>
                        <h4>"Applied History"</h4>
                        <p>{format!("Audit records loaded for {}.", harness_history_scope_label)}</p>
                    </div>
                    <a href=harness_overview_href.clone() data-history-overview-link="true">"Overview"</a>
                </header>
                <nav
                    aria-label="History filters"
                    data-history-filter-count="5"
                    data-history-filter-current=harness_history_filter_action.clone()
                >
                    <a
                        href=harness_history_href.clone()
                        data-history-filter-action="all"
                        aria-current=harness_history_filter_all_current
                    >
                        "All"
                    </a>
                    <a
                        href=harness_history_dry_run_href
                        data-history-filter-action="dry-run"
                        aria-current=harness_history_filter_dry_run_current
                    >
                        "Dry Run"
                    </a>
                    <a
                        href=harness_history_apply_href
                        data-history-filter-action="apply"
                        aria-current=harness_history_filter_apply_current
                    >
                        "Apply"
                    </a>
                    <a
                        href=harness_history_benchmark_href
                        data-history-filter-action="run-benchmark"
                        aria-current=harness_history_filter_benchmark_current
                    >
                        "Benchmark"
                    </a>
                    <a
                        href=harness_history_start_href
                        data-history-filter-action="start-mission"
                        aria-current=harness_history_filter_start_current
                    >
                        "Mission"
                    </a>
                </nav>
                <dl>
                    <div>
                        <dt>"Shown"</dt>
                        <dd>{harness_audit_row_count.clone()}</dd>
                    </div>
                    <div>
                        <dt>"Total"</dt>
                        <dd>{harness_history_total_count.clone()}</dd>
                    </div>
                    <div>
                        <dt>"Source"</dt>
                        <dd>{context.harness.audit_source.clone()}</dd>
                    </div>
                    <div>
                        <dt>"Proof Links"</dt>
                        <dd>{harness_history_proof_count.clone()}</dd>
                    </div>
                    <div>
                        <dt>"Proposal"</dt>
                        <dd>{harness_selected_proposal_id.clone()}</dd>
                    </div>
                </dl>
                <p data-history-latest="true">
                    {format!("Latest: {} at {}", harness_history_latest_action, harness_history_latest_timestamp)}
                </p>
                {harness_history_selected_audit_detail}
                <a href="#tau-ops-harness-audit-log" data-history-audit-anchor="true">"Open audit log"</a>
            </section>
        })
    } else {
        leptos::either::Either::Right(())
    };
    let harness_detail_plan_rows = context
        .harness
        .detail_plan_rows
        .iter()
        .map(|row| {
            view! {
                <li
                    id=format!("tau-ops-harness-dag-{}", row.item_id)
                    data-plan-node=row.label.clone()
                    data-node-status=row.status_key.clone()
                >
                    {row.label.clone()}
                </li>
            }
        })
        .collect_view();
    let harness_detail_tool_rows = context
        .harness
        .detail_tool_rows
        .iter()
        .map(|row| {
            if row.artifact_href.trim().is_empty() {
                leptos::either::Either::Left(view! {
                    <tr data-tool=row.tool_name.clone() data-status=row.status_key.clone()>
                        <td>{row.tool_name.clone()}</td>
                        <td>{row.call_id.clone()}</td>
                        <td>{row.plan_node.clone()}</td>
                        <td>{row.runtime.clone()}</td>
                        <td>{row.status_key.clone()}</td>
                        <td>{row.artifact_label.clone()}</td>
                    </tr>
                })
            } else {
                leptos::either::Either::Right(view! {
                <tr
                    data-tool=row.tool_name.clone()
                    data-status=row.status_key.clone()
                    data-tool-artifact-href=row.artifact_href.clone()
                >
                    <td>{row.tool_name.clone()}</td>
                    <td>{row.call_id.clone()}</td>
                    <td>{row.plan_node.clone()}</td>
                    <td>{row.runtime.clone()}</td>
                    <td>{row.status_key.clone()}</td>
                    <td>
                        <a
                            href=row.artifact_href.clone()
                            data-tool-proof-artifact-href="true"
                        >
                            {row.artifact_label.clone()}
                        </a>
                    </td>
                </tr>
                })
            }
        })
        .collect_view();
    let harness_detail_acceptance_rows = context
        .harness
        .detail_acceptance_rows
        .iter()
        .map(|row| {
            view! {
                <li data-ac-id=row.item_id.clone() data-ac-status=row.status_key.clone()>{row.label.clone()}</li>
            }
        })
        .collect_view();
    let harness_detail_gate_rows = context
        .harness
        .detail_gate_rows
        .iter()
        .map(|row| {
            let lower_label = row.label.to_ascii_lowercase();
            let gate_slug = if lower_label.contains("planning") {
                "planning".to_string()
            } else if lower_label.contains("tool") {
                "tool-exec".to_string()
            } else if lower_label.contains("memory") {
                "memory".to_string()
            } else if lower_label.contains("verification") {
                "verification".to_string()
            } else if lower_label.contains("learning") {
                "learning".to_string()
            } else {
                row.item_id.to_ascii_lowercase().replace('_', "-")
            };
            view! {
                <li
                    id=format!("tau-ops-harness-gate-{gate_slug}")
                    data-gate-id=row.item_id.clone()
                    data-gate-status=row.status_key.clone()
                >
                    {row.label.clone()}
                </li>
            }
        })
        .collect_view();
    let harness_detail_artifact_rows = context
        .harness
        .detail_artifact_rows
        .iter()
        .map(|row| {
            view! {
                <li data-artifact-id=row.item_id.clone() data-artifact-kind=row.status_key.clone()>
                    <a href=row.href.clone()>{row.label.clone()}</a>
                </li>
            }
        })
        .collect_view();
    let harness_mission_rows = context
        .harness
        .mission_rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let row_id = format!("tau-ops-harness-mission-row-{index}");
            let plan_progress = row.plan_progress.to_string();
            let row_selected = row.mission_id == context.harness.detail_run_id;
            let row_mission_status = match row.status_key.as_str() {
                "completed" => "mission_completed",
                "blocked" | "failed" => "mission_blocked",
                "draft" => "draft_created",
                "awaiting_approval" | "running" | "verifying" => "mission_started",
                _ => "mission_started",
            };
            let row_detail_href = format!(
                "/ops/harness?theme={theme_attr}&sidebar={sidebar_state_attr}&session={}&proposal_id={}&mission_status={row_mission_status}&mission_id={}",
                context.chat.active_session_key.clone(),
                harness_selected_proposal_id,
                row.mission_id,
            );
            let row_identity_label = if row_selected {
                format!("Selected proof {}", row.mission_id)
            } else {
                format!("Proof {}", row.mission_id)
            };
            let row_start_form_id = format!("tau-ops-harness-start-mission-form-{index}");
            let row_start_button_id = format!("tau-ops-harness-start-mission-{index}");
            let row_start_action = format!(
                "/ops/harness/missions/{}/start?theme={theme_attr}&sidebar={sidebar_state_attr}&session={}&proposal_id={}",
                row.mission_id,
                context.chat.active_session_key.clone(),
                harness_selected_proposal_id
            );
            let row_action = if row.status_key == "draft" {
                leptos::either::Either::Left(view! {
                    <form
                        id=row_start_form_id
                        action=row_start_action
                        method="post"
                        data-action-contract="mission-start-dry-run"
                        data-preserves-shell-context="true"
                    >
                        <button
                            id=row_start_button_id
                            type="submit"
                            data-action="start-mission"
                            data-mission-id=row.mission_id.clone()
                            data-action-contract="coding-agent-dry-run"
                        >
                            "Start"
                        </button>
                    </form>
                })
            } else {
                leptos::either::Either::Right(view! {
                    <span
                        class="tau-harness-row-action-status"
                        data-action="mission-state"
                        data-action-state=row.status_key.clone()
                    >
                        {row.status_label.clone()}
                    </span>
                })
            };
            view! {
                <tr
                    id=row_id
                    data-mission-id=row.mission_id.clone()
                    data-status=row.status_key.clone()
                    data-plan-progress=plan_progress.clone()
                    data-verification-state=row.verification_state.clone()
                    data-selected-proof=row_selected.to_string()
                >
                    <td data-mission-summary="inline-status">
                        <a
                            class="tau-harness-mission-title"
                            href=row_detail_href
                            data-mission-detail-link=row.mission_id.clone()
                            data-mission-detail-status=row_mission_status
                            data-selected-proof=row_selected.to_string()
                        >
                            {row.title.clone()}
                        </a>
                        <div
                            class="tau-harness-mission-identity"
                            data-mission-identity=row.mission_id.clone()
                            data-selected-proof=row_selected.to_string()
                            data-visible-proof-id="true"
                        >
                            {row_identity_label}
                        </div>
                        <div class="tau-harness-mission-meta" data-compact-mission-meta="status-gates">
                            <span class="tau-harness-status-chip" data-mission-state-chip=row.status_key.clone()>{row.status_label.clone()}</span>
                            <span class="tau-harness-status-chip" data-mission-gate-chip=row.gate_status_key.clone()>{row.gate_label.clone()}</span>
                            {row_action}
                        </div>
                    </td>
                    <td>{row.acceptance_label.clone()}</td>
                    <td><meter min="0" max="100" value=plan_progress.clone()>{format!("{plan_progress}%")}</meter></td>
                    <td>{row.tool_budget.clone()}</td>
                    <td>{row.memory_hits}</td>
                    <td><span class="tau-harness-status-chip" data-gate-status=row.verification_state.clone()>{row.gate_label.clone()}</span></td>
                    <td>{row.last_checkpoint.clone()}</td>
                    <td>{row.artifact_count}</td>
                </tr>
            }
        })
        .collect_view();
    let harness_kpi_missions_count = context.harness.kpi_missions_count.to_string();
    let harness_kpi_pending_verification_count =
        context.harness.kpi_pending_verification_count.to_string();
    let harness_kpi_memory_write_count = context.harness.kpi_memory_write_count.to_string();
    let harness_detail_tool_call_count = context.harness.detail_tool_call_count.to_string();
    let harness_detail_acceptance_met = context.harness.detail_acceptance_met_count.to_string();
    let harness_detail_acceptance_total = context.harness.detail_acceptance_total_count.to_string();
    let harness_detail_gate_count = context.harness.detail_gate_rows.len().to_string();
    let harness_detail_failed_gate_count = context.harness.detail_gate_failed_count.to_string();
    let harness_detail_passed_gate_count = context
        .harness
        .detail_gate_rows
        .iter()
        .filter(|row| row.status_key == "passed")
        .count()
        .to_string();
    let harness_detail_memory_hits = context.harness.detail_memory_hit_count.to_string();
    let harness_detail_learning_records = context.harness.detail_learning_record_count.to_string();
    let harness_detail_artifact_count = context.harness.detail_artifact_rows.len().to_string();
    let harness_selected_mission_acceptance_summary = format!(
        "{}/{}",
        harness_detail_acceptance_met, harness_detail_acceptance_total
    );
    let harness_selected_mission_gate_summary = format!(
        "{}/{}",
        harness_detail_passed_gate_count, harness_detail_gate_count
    );
    let harness_runtime_workspace_label = context.harness.runtime_workspace_label.clone();
    let harness_runtime_model_label = context.harness.runtime_model_label.clone();
    let harness_runtime_transport_label = context.harness.runtime_transport_label.clone();
    let harness_runtime_health_key = context.harness.runtime_health_key.clone();
    let harness_runtime_health_label = harness_queue_status_label(&harness_runtime_health_key);
    let harness_selected_mission_is_durable = context
        .harness
        .detail_proof_artifact
        .contains("/ops-harness/missions/");
    let harness_selected_mission_start_action = format!(
        "/ops/harness/missions/{}/start?theme={theme_attr}&sidebar={sidebar_state_attr}&session={}&proposal_id={}",
        context.harness.detail_run_id.clone(),
        context.chat.active_session_key.clone(),
        harness_selected_proposal_id
    );
    let harness_selected_mission_actions = if harness_selected_mission_is_durable {
        if context.harness.detail_status == "draft" {
            view! {
                <section
                    id="tau-ops-harness-selected-mission-actions"
                    data-selected-mission-id=context.harness.detail_run_id.clone()
                    data-selected-mission-status=context.harness.detail_status.clone()
                    data-selected-mission-action="start"
                    data-next-status="mission_started"
                    data-action-surface="proof-pane"
                    data-acceptance-met=harness_detail_acceptance_met.clone()
                    data-acceptance-total=harness_detail_acceptance_total.clone()
                    data-gates-passed=harness_detail_passed_gate_count.clone()
                    data-gates-total=harness_detail_gate_count.clone()
                    data-tool-calls=harness_detail_tool_call_count.clone()
                    data-memory-hits=harness_detail_memory_hits.clone()
                    data-learning-records=harness_detail_learning_records.clone()
                    data-transition-proof="pending-start"
                >
                    <div>
                        <h4>"Selected Mission"</h4>
                        <p>{format!("{} is ready to start from this proof view.", context.harness.detail_run_id)}</p>
                    </div>
                    <form
                        id="tau-ops-harness-start-selected-mission-form"
                        action=harness_selected_mission_start_action
                        method="post"
                        data-action-contract="mission-start-dry-run"
                        data-preserves-shell-context="true"
                    >
                        <button
                            id="tau-ops-harness-start-selected-mission"
                            type="submit"
                            data-action="start-selected-mission"
                            data-mission-id=context.harness.detail_run_id.clone()
                            data-start-result-target="mission-proof-refresh"
                            data-action-contract="coding-agent-dry-run"
                        >
                            "Start Selected"
                        </button>
                    </form>
                </section>
            }
            .into_any()
        } else {
            view! {
                <section
                    id="tau-ops-harness-selected-mission-actions"
                    data-selected-mission-id=context.harness.detail_run_id.clone()
                    data-selected-mission-status=context.harness.detail_status.clone()
                    data-selected-mission-action="inspect"
                    data-next-status="none"
                    data-action-surface="proof-pane"
                    data-acceptance-met=harness_detail_acceptance_met.clone()
                    data-acceptance-total=harness_detail_acceptance_total.clone()
                    data-gates-passed=harness_detail_passed_gate_count.clone()
                    data-gates-total=harness_detail_gate_count.clone()
                    data-tool-calls=harness_detail_tool_call_count.clone()
                    data-memory-hits=harness_detail_memory_hits.clone()
                    data-learning-records=harness_detail_learning_records.clone()
                    data-transition-proof="visible"
                >
                    <div>
                        <h4>"Selected Mission"</h4>
                        <p>{format!("{} is loaded in the proof pane.", context.harness.detail_run_id)}</p>
                        <dl
                            class="tau-harness-selected-mission-proof"
                            aria-label="Selected mission proof summary"
                        >
                            <div data-proof-metric="acceptance">
                                <dt>"Acceptance"</dt>
                                <dd>{harness_selected_mission_acceptance_summary.clone()}</dd>
                            </div>
                            <div data-proof-metric="gates">
                                <dt>"Gates"</dt>
                                <dd>{harness_selected_mission_gate_summary.clone()}</dd>
                            </div>
                            <div data-proof-metric="tool-calls">
                                <dt>"Tool calls"</dt>
                                <dd>{harness_detail_tool_call_count.clone()}</dd>
                            </div>
                            <div data-proof-metric="memory-hits">
                                <dt>"Memory hits"</dt>
                                <dd>{harness_detail_memory_hits.clone()}</dd>
                            </div>
                            <div data-proof-metric="learning-records">
                                <dt>"Learning records"</dt>
                                <dd>{harness_detail_learning_records.clone()}</dd>
                            </div>
                        </dl>
                    </div>
                    <span class="tau-harness-status-chip" data-selected-mission-state=context.harness.detail_status.clone()>
                        {harness_queue_status_label(&context.harness.detail_status)}
                    </span>
                </section>
            }
            .into_any()
        }
    } else {
        view! {
            <section
                id="tau-ops-harness-selected-mission-actions"
                data-selected-mission-id=context.harness.detail_run_id.clone()
                data-selected-mission-status=context.harness.detail_status.clone()
                data-selected-mission-action="benchmark-proof"
                data-next-status="none"
                data-action-surface="proof-pane"
                hidden
            >
                <div>
                    <h4>"Selected Mission"</h4>
                    <p>"Benchmark proof selected."</p>
                </div>
            </section>
        }
        .into_any()
    };
    let harness_tui_summary = if context
        .harness
        .detail_proof_artifact
        .contains("/ops-harness/missions/")
    {
        format!(
            "tau@harness:~$ tau status\nmission={}\ntransport=gateway\nstatus={}\ntool_budget={}\nproof={}\n\nMission Proof\nAcceptance: {}/{}\nGates: {}/{} passed\nMemory Hits: {}\nLearning Records: {}\nProof: {}",
            context.harness.detail_run_id.clone(),
            context.harness.detail_status.clone(),
            context.harness.detail_tool_budget.clone(),
            context.harness.detail_proof_artifact.clone(),
            context.harness.detail_acceptance_met_count,
            context.harness.detail_acceptance_total_count,
            harness_detail_passed_gate_count,
            harness_detail_gate_count,
            harness_detail_memory_hits,
            harness_detail_learning_records,
            context.harness.detail_proof_artifact.clone()
        )
    } else {
        format!(
            "tau@harness:~$ tau status\nmission={}\ntransport=gateway\nstatus={}\ntool_budget={}\nbench: {} pass; proof {}\n\nBenchmark M334\nPassed: {}\nFailed Gates:\n  {}\nProof: {}",
            context.harness.detail_run_id.clone(),
            context.harness.detail_status.clone(),
            context.harness.detail_tool_budget.clone(),
            context.harness.latest_result.clone(),
            context.harness.proof_artifact.clone(),
            context.harness.latest_result.clone(),
            context.harness.failed_gate_label.clone(),
            context.harness.proof_artifact.clone()
        )
    };
    let config_panel_hidden = if matches!(context.active_route, TauOpsDashboardRoute::Config) {
        "false"
    } else {
        "true"
    };
    let config_panel_visible = if matches!(context.active_route, TauOpsDashboardRoute::Config) {
        "true"
    } else {
        "false"
    };
    let training_panel_hidden = if matches!(context.active_route, TauOpsDashboardRoute::Training) {
        "false"
    } else {
        "true"
    };
    let training_panel_visible = if matches!(context.active_route, TauOpsDashboardRoute::Training) {
        "true"
    } else {
        "false"
    };
    let safety_panel_hidden = if matches!(context.active_route, TauOpsDashboardRoute::Safety) {
        "false"
    } else {
        "true"
    };
    let safety_panel_visible = if matches!(context.active_route, TauOpsDashboardRoute::Safety) {
        "true"
    } else {
        "false"
    };
    let diagnostics_panel_hidden =
        if matches!(context.active_route, TauOpsDashboardRoute::Diagnostics) {
            "false"
        } else {
            "true"
        };
    let diagnostics_panel_visible =
        if matches!(context.active_route, TauOpsDashboardRoute::Diagnostics) {
            "true"
        } else {
            "false"
        };
    let command_center_panel_hidden = if matches!(context.active_route, TauOpsDashboardRoute::Ops) {
        "false"
    } else {
        "true"
    };
    let deploy_panel_hidden = if matches!(context.active_route, TauOpsDashboardRoute::Deploy) {
        "false"
    } else {
        "true"
    };
    let deploy_panel_visible = if matches!(context.active_route, TauOpsDashboardRoute::Deploy) {
        "true"
    } else {
        "false"
    };
    let chat_message_rows = if !chat_route_active {
        Vec::new()
    } else if context.chat.message_rows.is_empty() {
        vec![TauOpsDashboardChatMessageRow {
            role: "system".to_string(),
            content: "No chat messages yet.".to_string(),
        }]
    } else {
        context.chat.message_rows.clone()
    };
    let sessions_row_options = context.chat.session_options.clone();
    let sessions_row_count_value = sessions_row_options.len().to_string();
    let chat_session_key = context.chat.active_session_key.clone();
    let rendered_chat_entry_count = context.chat.message_rows.len();
    let mut chat_session_options = sessions_row_options.clone();
    let mut active_session_marked = false;
    for option in &mut chat_session_options {
        if option.session_key == chat_session_key {
            option.selected = true;
            active_session_marked = true;
        }
    }
    if !active_session_marked {
        chat_session_options.push(TauOpsDashboardChatSessionOptionRow {
            session_key: chat_session_key.clone(),
            selected: true,
            entry_count: rendered_chat_entry_count,
            usage_total_tokens: 0,
            validation_is_valid: true,
            updated_unix_ms: 0,
        });
    }
    let sessions_rows_view = if sessions_row_options.is_empty() {
        leptos::either::Either::Left(view! {
            <li id="tau-ops-sessions-empty-state" data-empty-state="true">
                No sessions discovered yet.
            </li>
        })
    } else {
        leptos::either::Either::Right(
            sessions_row_options
                .iter()
                .enumerate()
                .map(|(index, session_option)| {
                    let row_id = format!("tau-ops-sessions-row-{index}");
                    let selected_attr = if session_option.selected {
                        "true"
                    } else {
                        "false"
                    };
                    let entry_count_attr = session_option.entry_count.to_string();
                    let total_tokens_attr = session_option.usage_total_tokens.to_string();
                    let is_valid_attr = if session_option.validation_is_valid {
                        "true"
                    } else {
                        "false"
                    };
                    let updated_unix_ms_attr = session_option.updated_unix_ms.to_string();
                    let open_chat_href = format!(
                        "/ops/chat?theme={theme_attr}&sidebar={sidebar_state_attr}&session={}",
                        session_option.session_key
                    );
                    view! {
                        <li
                            id=row_id
                            data-session-key=session_option.session_key.clone()
                            data-selected=selected_attr
                            data-entry-count=entry_count_attr
                            data-total-tokens=total_tokens_attr
                            data-is-valid=is_valid_attr
                            data-updated-unix-ms=updated_unix_ms_attr
                        >
                            <a
                                data-open-chat-session=session_option.session_key.clone()
                                href=open_chat_href
                            >
                                {session_option.session_key.clone()}
                            </a>
                        </li>
                    }
                })
                .collect_view(),
        )
    };
    let memory_search_form_action = context.chat.memory_search_form_action.clone();
    let memory_search_form_method = context.chat.memory_search_form_method.clone();
    let memory_search_query = context.chat.memory_search_query.clone();
    let memory_search_workspace_id = context.chat.memory_search_workspace_id.clone();
    let memory_search_channel_id = context.chat.memory_search_channel_id.clone();
    let memory_search_actor_id = context.chat.memory_search_actor_id.clone();
    let memory_search_memory_type = context.chat.memory_search_memory_type.clone();
    let memory_search_rows = context.chat.memory_search_rows.clone();
    let memory_result_count_value = memory_search_rows.len().to_string();
    let memory_query_panel_attr = memory_search_query.clone();
    let memory_query_input_value = memory_search_query.clone();
    let memory_workspace_id_panel_attr = memory_search_workspace_id.clone();
    let memory_channel_id_panel_attr = memory_search_channel_id.clone();
    let memory_actor_id_panel_attr = memory_search_actor_id.clone();
    let memory_workspace_id_input_value = memory_search_workspace_id.clone();
    let memory_channel_id_input_value = memory_search_channel_id.clone();
    let memory_actor_id_input_value = memory_search_actor_id.clone();
    let memory_type_panel_attr = memory_search_memory_type.clone();
    let memory_type_input_value = memory_search_memory_type.clone();
    let memory_result_count_panel_attr = memory_result_count_value.clone();
    let memory_result_count_list_attr = memory_result_count_value.clone();
    let memory_create_form_action = context.chat.memory_create_form_action.clone();
    let memory_create_form_method = context.chat.memory_create_form_method.clone();
    let memory_create_status = context.chat.memory_create_status.clone();
    let memory_create_created_entry_id = context.chat.memory_create_created_entry_id.clone();
    let memory_create_status_panel_attr = memory_create_status.clone();
    let memory_create_created_entry_id_panel_attr = memory_create_created_entry_id.clone();
    let memory_create_status_marker_attr = memory_create_status.clone();
    let memory_create_created_entry_id_marker_attr = memory_create_created_entry_id.clone();
    let memory_create_entry_id = context.chat.memory_create_entry_id.clone();
    let memory_create_summary = context.chat.memory_create_summary.clone();
    let memory_create_tags = context.chat.memory_create_tags.clone();
    let memory_create_facts = context.chat.memory_create_facts.clone();
    let memory_create_source_event_key = context.chat.memory_create_source_event_key.clone();
    let memory_create_workspace_id = context.chat.memory_create_workspace_id.clone();
    let memory_create_channel_id = context.chat.memory_create_channel_id.clone();
    let memory_create_actor_id = context.chat.memory_create_actor_id.clone();
    let memory_create_memory_type = context.chat.memory_create_memory_type.clone();
    let memory_create_importance = context.chat.memory_create_importance.clone();
    let memory_create_relation_target_id = context.chat.memory_create_relation_target_id.clone();
    let memory_create_relation_type = context.chat.memory_create_relation_type.clone();
    let memory_create_relation_weight = context.chat.memory_create_relation_weight.clone();
    let memory_create_status_message = match memory_create_status.as_str() {
        "created" => "Memory entry created.".to_string(),
        "updated" => "Memory entry updated.".to_string(),
        _ => "Create a memory entry.".to_string(),
    };
    let memory_edit_form_action = memory_create_form_action.clone();
    let memory_edit_form_method = memory_create_form_method.clone();
    let memory_edit_status_panel_attr = memory_create_status.clone();
    let memory_edit_edited_memory_id_panel_attr = memory_create_created_entry_id.clone();
    let memory_edit_status_marker_attr = memory_create_status.clone();
    let memory_edit_edited_memory_id_marker_attr = memory_create_created_entry_id.clone();
    let memory_edit_entry_id = memory_create_created_entry_id.clone();
    let memory_edit_summary = memory_create_summary.clone();
    let memory_edit_tags = memory_create_tags.clone();
    let memory_edit_facts = memory_create_facts.clone();
    let memory_edit_source_event_key = memory_create_source_event_key.clone();
    let memory_edit_workspace_id = memory_create_workspace_id.clone();
    let memory_edit_channel_id = memory_create_channel_id.clone();
    let memory_edit_actor_id = memory_create_actor_id.clone();
    let memory_edit_memory_type = memory_create_memory_type.clone();
    let memory_edit_importance = memory_create_importance.clone();
    let memory_edit_relation_target_id = memory_create_relation_target_id.clone();
    let memory_edit_relation_type = memory_create_relation_type.clone();
    let memory_edit_relation_weight = memory_create_relation_weight.clone();
    let memory_edit_status_message = match memory_create_status.as_str() {
        "updated" => "Memory entry updated.".to_string(),
        _ => "Edit an existing memory entry.".to_string(),
    };
    let memory_delete_form_action = memory_edit_form_action.clone();
    let memory_delete_form_method = memory_edit_form_method.clone();
    let memory_delete_status = context.chat.memory_delete_status.clone();
    let memory_delete_deleted_entry_id = context.chat.memory_delete_deleted_entry_id.clone();
    let memory_delete_status_panel_attr = memory_delete_status.clone();
    let memory_delete_deleted_entry_id_panel_attr = memory_delete_deleted_entry_id.clone();
    let memory_delete_status_marker_attr = memory_delete_status.clone();
    let memory_delete_deleted_entry_id_marker_attr = memory_delete_deleted_entry_id.clone();
    let memory_delete_entry_id = memory_delete_deleted_entry_id.clone();
    let memory_delete_status_message = match memory_delete_status.as_str() {
        "deleted" => "Memory entry deleted.".to_string(),
        _ => "Delete a memory entry.".to_string(),
    };
    let memory_detail_visible = if context.chat.memory_detail_visible {
        "true"
    } else {
        "false"
    };
    let memory_detail_selected_entry_id = context.chat.memory_detail_selected_entry_id.clone();
    let memory_detail_memory_type = context.chat.memory_detail_memory_type.clone();
    let memory_detail_embedding_source = context.chat.memory_detail_embedding_source.clone();
    let memory_detail_embedding_model = context.chat.memory_detail_embedding_model.clone();
    let memory_detail_embedding_reason_code =
        context.chat.memory_detail_embedding_reason_code.clone();
    let memory_detail_embedding_dimensions =
        context.chat.memory_detail_embedding_dimensions.to_string();
    let memory_detail_relation_rows = context.chat.memory_detail_relation_rows.clone();
    let memory_detail_relation_count = memory_detail_relation_rows.len().to_string();
    let memory_detail_embedding_source_panel_attr = memory_detail_embedding_source.clone();
    let memory_detail_embedding_model_panel_attr = memory_detail_embedding_model.clone();
    let memory_detail_embedding_reason_code_panel_attr =
        memory_detail_embedding_reason_code.clone();
    let memory_detail_embedding_dimensions_panel_attr = memory_detail_embedding_dimensions.clone();
    let memory_detail_relation_count_panel_attr = memory_detail_relation_count.clone();
    let memory_detail_summary = if context.chat.memory_detail_summary.is_empty() {
        "No selected memory detail.".to_string()
    } else {
        context.chat.memory_detail_summary.clone()
    };
    let memory_detail_relations_view = if memory_detail_relation_rows.is_empty() {
        leptos::either::Either::Left(view! {
            <li id="tau-ops-memory-relations-empty-state" data-empty-state="true">
                No connected entries.
            </li>
        })
    } else {
        leptos::either::Either::Right(
            memory_detail_relation_rows
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    let row_id = format!("tau-ops-memory-relation-row-{index}");
                    view! {
                        <li
                            id=row_id
                            data-target-id=row.target_id.clone()
                            data-relation-type=row.relation_type.clone()
                            data-relation-weight=row.effective_weight.clone()
                        >
                            {row.target_id.clone()}
                        </li>
                    }
                })
                .collect_view(),
        )
    };
    let memory_results_view = if memory_search_rows.is_empty() {
        leptos::either::Either::Left(view! {
            <li id="tau-ops-memory-empty-state" data-empty-state="true">
                No memory matches found.
            </li>
        })
    } else {
        leptos::either::Either::Right(
            memory_search_rows
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    let row_id = format!("tau-ops-memory-result-row-{index}");
                    view! {
                        <li
                            id=row_id
                            data-memory-id=row.memory_id.clone()
                            data-memory-type=row.memory_type.clone()
                            data-score=row.score.clone()
                            data-detail-memory-id=row.memory_id.clone()
                        >
                            {row.summary.clone()}
                        </li>
                    }
                })
                .collect_view(),
        )
    };
    let memory_graph_node_rows = context.chat.memory_graph_node_rows.clone();
    let memory_graph_edge_rows = context.chat.memory_graph_edge_rows.clone();
    let memory_graph_zoom_level_value = context
        .chat
        .memory_graph_zoom_level
        .parse::<f32>()
        .ok()
        .unwrap_or(1.0)
        .clamp(0.25, 2.0);
    let memory_graph_zoom_level = format!("{:.2}", memory_graph_zoom_level_value);
    let memory_graph_zoom_min = "0.25";
    let memory_graph_zoom_max = "2.00";
    let memory_graph_zoom_step = "0.10";
    let memory_graph_zoom_in_level =
        format!("{:.2}", (memory_graph_zoom_level_value + 0.10).min(2.0));
    let memory_graph_zoom_out_level =
        format!("{:.2}", (memory_graph_zoom_level_value - 0.10).max(0.25));
    let memory_graph_pan_x_value = context
        .chat
        .memory_graph_pan_x_level
        .parse::<f32>()
        .ok()
        .unwrap_or(0.0)
        .clamp(-500.0, 500.0);
    let memory_graph_pan_y_value = context
        .chat
        .memory_graph_pan_y_level
        .parse::<f32>()
        .ok()
        .unwrap_or(0.0)
        .clamp(-500.0, 500.0);
    let memory_graph_pan_x_level = format!("{:.2}", memory_graph_pan_x_value);
    let memory_graph_pan_y_level = format!("{:.2}", memory_graph_pan_y_value);
    let memory_graph_pan_step_value = 25.0f32;
    let memory_graph_pan_step = format!("{:.2}", memory_graph_pan_step_value);
    let memory_graph_pan_left_x_level = format!(
        "{:.2}",
        (memory_graph_pan_x_value - memory_graph_pan_step_value).max(-500.0)
    );
    let memory_graph_pan_right_x_level = format!(
        "{:.2}",
        (memory_graph_pan_x_value + memory_graph_pan_step_value).min(500.0)
    );
    let memory_graph_pan_up_y_level = format!(
        "{:.2}",
        (memory_graph_pan_y_value - memory_graph_pan_step_value).max(-500.0)
    );
    let memory_graph_pan_down_y_level = format!(
        "{:.2}",
        (memory_graph_pan_y_value + memory_graph_pan_step_value).min(500.0)
    );
    let memory_graph_filter_memory_type = {
        let value = context.chat.memory_graph_filter_memory_type.trim();
        if value.is_empty() {
            "all".to_string()
        } else {
            value.to_string()
        }
    };
    let memory_graph_filter_relation_type = {
        let value = context.chat.memory_graph_filter_relation_type.trim();
        if value.is_empty() {
            "all".to_string()
        } else {
            value.to_string()
        }
    };
    let filtered_memory_graph_node_rows = if memory_graph_filter_memory_type == "all" {
        memory_graph_node_rows.clone()
    } else {
        memory_graph_node_rows
            .iter()
            .filter(|row| row.memory_type.as_str() == memory_graph_filter_memory_type.as_str())
            .cloned()
            .collect::<Vec<_>>()
    };
    let filtered_memory_graph_node_ids = filtered_memory_graph_node_rows
        .iter()
        .map(|row| row.memory_id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let scope_edges_to_filtered_nodes = memory_graph_filter_memory_type != "all";
    let filtered_memory_graph_edge_rows = memory_graph_edge_rows
        .iter()
        .filter(|row| {
            let within_node_scope = if scope_edges_to_filtered_nodes {
                filtered_memory_graph_node_ids.contains(&row.source_memory_id)
                    && filtered_memory_graph_node_ids.contains(&row.target_memory_id)
            } else {
                true
            };
            (memory_graph_filter_relation_type == "all"
                || row.relation_type.as_str() == memory_graph_filter_relation_type.as_str())
                && within_node_scope
        })
        .cloned()
        .collect::<Vec<_>>();
    let memory_graph_route_href_base = format!(
        "/ops/memory-graph?theme={theme_attr}&sidebar={sidebar_state_attr}&session={chat_session_key}&workspace_id={memory_search_workspace_id}&channel_id={memory_search_channel_id}&actor_id={memory_search_actor_id}&memory_type={memory_search_memory_type}"
    );
    let memory_scope_graph_href = memory_graph_route_href_base.clone();
    let memory_scope_session_href = format!(
        "/ops/sessions/{chat_session_key}?theme={theme_attr}&sidebar={sidebar_state_attr}&session={chat_session_key}"
    );
    let memory_scope_query_label = if memory_search_query.trim().is_empty() {
        "all entries".to_string()
    } else {
        memory_search_query.clone()
    };
    let memory_scope_workspace_label = if memory_search_workspace_id.trim().is_empty() {
        "all workspaces".to_string()
    } else {
        memory_search_workspace_id.clone()
    };
    let memory_scope_type_label = if memory_search_memory_type.trim().is_empty() {
        "all types".to_string()
    } else {
        memory_search_memory_type.clone()
    };
    let memory_graph_zoom_in_href = format!(
        "{memory_graph_route_href_base}&graph_zoom={memory_graph_zoom_in_level}&graph_pan_x={memory_graph_pan_x_level}&graph_pan_y={memory_graph_pan_y_level}&graph_filter_memory_type={memory_graph_filter_memory_type}&graph_filter_relation_type={memory_graph_filter_relation_type}"
    );
    let memory_graph_zoom_out_href = format!(
        "{memory_graph_route_href_base}&graph_zoom={memory_graph_zoom_out_level}&graph_pan_x={memory_graph_pan_x_level}&graph_pan_y={memory_graph_pan_y_level}&graph_filter_memory_type={memory_graph_filter_memory_type}&graph_filter_relation_type={memory_graph_filter_relation_type}"
    );
    let memory_graph_pan_left_href = format!(
        "{memory_graph_route_href_base}&graph_zoom={memory_graph_zoom_level}&graph_pan_x={memory_graph_pan_left_x_level}&graph_pan_y={memory_graph_pan_y_level}&graph_filter_memory_type={memory_graph_filter_memory_type}&graph_filter_relation_type={memory_graph_filter_relation_type}"
    );
    let memory_graph_pan_right_href = format!(
        "{memory_graph_route_href_base}&graph_zoom={memory_graph_zoom_level}&graph_pan_x={memory_graph_pan_right_x_level}&graph_pan_y={memory_graph_pan_y_level}&graph_filter_memory_type={memory_graph_filter_memory_type}&graph_filter_relation_type={memory_graph_filter_relation_type}"
    );
    let memory_graph_pan_up_href = format!(
        "{memory_graph_route_href_base}&graph_zoom={memory_graph_zoom_level}&graph_pan_x={memory_graph_pan_x_level}&graph_pan_y={memory_graph_pan_up_y_level}&graph_filter_memory_type={memory_graph_filter_memory_type}&graph_filter_relation_type={memory_graph_filter_relation_type}"
    );
    let memory_graph_pan_down_href = format!(
        "{memory_graph_route_href_base}&graph_zoom={memory_graph_zoom_level}&graph_pan_x={memory_graph_pan_x_level}&graph_pan_y={memory_graph_pan_down_y_level}&graph_filter_memory_type={memory_graph_filter_memory_type}&graph_filter_relation_type={memory_graph_filter_relation_type}"
    );
    let memory_graph_filter_memory_type_all_href = format!(
        "{memory_graph_route_href_base}&graph_zoom={memory_graph_zoom_level}&graph_pan_x={memory_graph_pan_x_level}&graph_pan_y={memory_graph_pan_y_level}&graph_filter_memory_type=all&graph_filter_relation_type={memory_graph_filter_relation_type}"
    );
    let memory_graph_filter_memory_type_goal_href = format!(
        "{memory_graph_route_href_base}&graph_zoom={memory_graph_zoom_level}&graph_pan_x={memory_graph_pan_x_level}&graph_pan_y={memory_graph_pan_y_level}&graph_filter_memory_type=goal&graph_filter_relation_type={memory_graph_filter_relation_type}"
    );
    let memory_graph_filter_relation_type_all_href = format!(
        "{memory_graph_route_href_base}&graph_zoom={memory_graph_zoom_level}&graph_pan_x={memory_graph_pan_x_level}&graph_pan_y={memory_graph_pan_y_level}&graph_filter_memory_type={memory_graph_filter_memory_type}&graph_filter_relation_type=all"
    );
    let memory_graph_filter_relation_type_related_to_href = format!(
        "{memory_graph_route_href_base}&graph_zoom={memory_graph_zoom_level}&graph_pan_x={memory_graph_pan_x_level}&graph_pan_y={memory_graph_pan_y_level}&graph_filter_memory_type={memory_graph_filter_memory_type}&graph_filter_relation_type=related_to"
    );
    let memory_graph_node_count = filtered_memory_graph_node_rows.len().to_string();
    let memory_graph_edge_count = filtered_memory_graph_edge_rows.len().to_string();
    let memory_graph_node_count_panel_attr = memory_graph_node_count.clone();
    let memory_graph_edge_count_panel_attr = memory_graph_edge_count.clone();
    let memory_graph_scope_state_label = if filtered_memory_graph_node_rows.is_empty()
        && filtered_memory_graph_edge_rows.is_empty()
    {
        "empty graph"
    } else {
        "graph available"
    };
    let memory_graph_scope_memory_href = format!(
        "/ops/memory?theme={theme_attr}&sidebar={sidebar_state_attr}&session={chat_session_key}&workspace_id={memory_search_workspace_id}&channel_id={memory_search_channel_id}&actor_id={memory_search_actor_id}&memory_type={memory_search_memory_type}"
    );
    let memory_graph_scope_session_href = format!(
        "/ops/sessions/{chat_session_key}?theme={theme_attr}&sidebar={sidebar_state_attr}&session={chat_session_key}"
    );
    let selected_memory_graph_detail_id = memory_detail_selected_entry_id.clone();
    let memory_graph_node_detail_href_prefix = format!(
        "/ops/memory-graph?theme={theme_attr}&sidebar={sidebar_state_attr}&session={chat_session_key}&workspace_id={memory_search_workspace_id}&channel_id={memory_search_channel_id}&actor_id={memory_search_actor_id}&memory_type={memory_search_memory_type}&detail_memory_id="
    );
    let memory_graph_detail_visible =
        if matches!(context.active_route, TauOpsDashboardRoute::MemoryGraph)
            && context.chat.memory_detail_visible
        {
            "true"
        } else {
            "false"
        };
    let memory_graph_detail_summary = memory_detail_summary.clone();
    let memory_graph_detail_selected_entry_id = memory_detail_selected_entry_id.clone();
    let memory_graph_detail_memory_type = memory_detail_memory_type.clone();
    let memory_graph_detail_relation_count_panel_attr =
        memory_detail_relation_count_panel_attr.clone();
    let memory_graph_detail_open_memory_href = format!(
        "/ops/memory?theme={theme_attr}&sidebar={sidebar_state_attr}&session={chat_session_key}&workspace_id={memory_search_workspace_id}&channel_id={memory_search_channel_id}&actor_id={memory_search_actor_id}&memory_type={memory_search_memory_type}&detail_memory_id={memory_graph_detail_selected_entry_id}"
    );
    let memory_graph_scope_summary_view = if matches!(
        context.active_route,
        TauOpsDashboardRoute::MemoryGraph
    ) {
        leptos::either::Either::Left(view! {
            <article
                id="tau-ops-memory-graph-scope-summary"
                data-session-key=chat_session_key.clone()
                data-node-count=memory_graph_node_count.clone()
                data-edge-count=memory_graph_edge_count.clone()
                data-scope-memory-type=memory_search_memory_type.clone()
                data-filter-memory-type=memory_graph_filter_memory_type.clone()
                data-filter-relation-type=memory_graph_filter_relation_type.clone()
                data-graph-state=memory_graph_scope_state_label
            >
                <h3>Graph Scope</h3>
                <p>
                    {format!(
                        "session {chat_session_key} | nodes {memory_graph_node_count} | edges {memory_graph_edge_count} | scope type {memory_scope_type_label} | graph type filter {memory_graph_filter_memory_type} | relation {memory_graph_filter_relation_type} | {memory_graph_scope_state_label}"
                    )}
                </p>
                <nav
                    id="tau-ops-memory-graph-scope-actions"
                    aria-label="Memory graph scope actions"
                >
                    <a
                        id="tau-ops-memory-graph-open-memory"
                        href=memory_graph_scope_memory_href
                    >
                        Open Memory Explorer
                    </a>
                    <a
                        id="tau-ops-memory-graph-open-session"
                        href=memory_graph_scope_session_href
                    >
                        Open Session
                    </a>
                </nav>
            </article>
        })
    } else {
        leptos::either::Either::Right(())
    };
    let focused_memory_graph_detail_id =
        if memory_graph_detail_visible == "true" && !selected_memory_graph_detail_id.is_empty() {
            Some(selected_memory_graph_detail_id.clone())
        } else {
            None
        };
    let memory_graph_nodes_view = if filtered_memory_graph_node_rows.is_empty() {
        leptos::either::Either::Left(view! {
            <li id="tau-ops-memory-graph-empty-state" data-empty-state="true">
                No memory graph nodes available.
            </li>
        })
    } else {
        leptos::either::Either::Right(
            filtered_memory_graph_node_rows
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    let row_id = format!("tau-ops-memory-graph-node-{index}");
                    let (node_size_bucket, node_size_px) =
                        derive_memory_graph_node_size_contracts(row.importance.as_str());
                    let (node_color_token, node_color_hex) =
                        derive_memory_graph_node_color_contracts(row.memory_type.as_str());
                    let node_selected = if memory_graph_detail_visible == "true"
                        && row.memory_id.as_str() == selected_memory_graph_detail_id.as_str()
                    {
                        "true"
                    } else {
                        "false"
                    };
                    let node_hover_neighbor = if let Some(focused_memory_id) =
                        focused_memory_graph_detail_id.as_deref()
                    {
                        let is_connected_neighbor = row.memory_id.as_str() == focused_memory_id
                            || filtered_memory_graph_edge_rows.iter().any(|edge| {
                                (edge.source_memory_id.as_str() == focused_memory_id
                                    && edge.target_memory_id.as_str() == row.memory_id.as_str())
                                    || (edge.target_memory_id.as_str() == focused_memory_id
                                        && edge.source_memory_id.as_str() == row.memory_id.as_str())
                            });
                        if is_connected_neighbor {
                            "true"
                        } else {
                            "false"
                        }
                    } else {
                        "false"
                    };
                    let node_detail_href =
                        format!("{memory_graph_node_detail_href_prefix}{}", row.memory_id);
                    view! {
                        <li
                            id=row_id
                            data-memory-id=row.memory_id.clone()
                            data-memory-type=row.memory_type.clone()
                            data-importance=row.importance.clone()
                            data-node-size-bucket=node_size_bucket
                            data-node-size-px=node_size_px
                            data-node-color-token=node_color_token
                            data-node-color-hex=node_color_hex
                            data-node-selected=node_selected
                            data-node-hover-neighbor=node_hover_neighbor
                            data-node-detail-target="tau-ops-memory-graph-detail-panel"
                            data-node-detail-href=node_detail_href
                        ></li>
                    }
                })
                .collect_view(),
        )
    };
    let memory_graph_edges_view = filtered_memory_graph_edge_rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let row_id = format!("tau-ops-memory-graph-edge-{index}");
            let (edge_style_token, edge_stroke_dasharray) =
                derive_memory_graph_edge_style_contracts(row.relation_type.as_str());
            let edge_hover_highlighted =
                if let Some(focused_memory_id) = focused_memory_graph_detail_id.as_deref() {
                    if row.source_memory_id.as_str() == focused_memory_id
                        || row.target_memory_id.as_str() == focused_memory_id
                    {
                        "true"
                    } else {
                        "false"
                    }
                } else {
                    "false"
                };
            view! {
                <li
                    id=row_id
                    data-source-memory-id=row.source_memory_id.clone()
                    data-target-memory-id=row.target_memory_id.clone()
                    data-relation-type=row.relation_type.clone()
                    data-relation-weight=row.effective_weight.clone()
                    data-edge-style-token=edge_style_token
                    data-edge-stroke-dasharray=edge_stroke_dasharray
                    data-edge-hover-highlighted=edge_hover_highlighted
                ></li>
            }
        })
        .collect_view();
    let mut tools_inventory_rows = context.chat.tools_inventory_rows.clone();
    tools_inventory_rows.sort_by(|left, right| left.tool_name.cmp(&right.tool_name));
    let tools_total_count_value = tools_inventory_rows.len().to_string();
    let tools_total_count_panel_attr = tools_total_count_value.clone();
    let tools_total_count_summary_attr = tools_total_count_value.clone();
    let tools_row_count_table_attr = tools_total_count_value.clone();
    let tools_row_count_body_attr = tools_total_count_value;
    let tools_jobs_route_active = matches!(context.active_route, TauOpsDashboardRoute::ToolsJobs);
    let tools_inventory_rows_view = if tools_inventory_rows.is_empty() {
        leptos::either::Either::Left(view! {
            <tr id="tau-ops-tools-inventory-empty-state" data-empty-state="true">
                <td colspan="7">No tools registered.</td>
            </tr>
        })
    } else {
        leptos::either::Either::Right(
            tools_inventory_rows
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    let row_id = format!("tau-ops-tools-inventory-row-{index}");
                    let usage_count = row.usage_count.to_string();
                    let last_used_unix_ms = row.last_used_unix_ms.to_string();
                    let usage_count_attr = usage_count.clone();
                    let last_used_unix_ms_attr = last_used_unix_ms.clone();
                    view! {
                        <tr
                            id=row_id
                            data-tool-name=row.tool_name.clone()
                            data-tool-category=row.category.clone()
                            data-tool-policy=row.policy.clone()
                            data-usage-count=usage_count_attr
                            data-error-rate=row.error_rate.clone()
                            data-avg-latency-ms=row.avg_latency_ms.clone()
                            data-last-used-unix-ms=last_used_unix_ms_attr
                        >
                            <td>{row.tool_name.clone()}</td>
                            <td>{row.category.clone()}</td>
                            <td>{row.policy.clone()}</td>
                            <td>{usage_count}</td>
                            <td>{row.error_rate.clone()}</td>
                            <td>{row.avg_latency_ms.clone()}</td>
                            <td>{last_used_unix_ms}</td>
                        </tr>
                    }
                })
                .collect_view(),
        )
    };
    let tool_detail_selected_tool_name = {
        let selected = context.chat.tool_detail_selected_tool_name.trim();
        if !selected.is_empty() {
            selected.to_string()
        } else {
            tools_inventory_rows
                .first()
                .map(|row| row.tool_name.clone())
                .unwrap_or_default()
        }
    };
    let tool_detail_visible =
        if tools_jobs_route_active && !tool_detail_selected_tool_name.is_empty() {
            "true"
        } else {
            "false"
        };
    let tool_detail_description = if context.chat.tool_detail_description.trim().is_empty() {
        "No tool selected.".to_string()
    } else {
        context.chat.tool_detail_description.clone()
    };
    let tool_detail_parameter_schema =
        if context.chat.tool_detail_parameter_schema.trim().is_empty() {
            "{}".to_string()
        } else {
            context.chat.tool_detail_parameter_schema.clone()
        };
    let tool_detail_policy_timeout_ms = context.chat.tool_detail_policy_timeout_ms.to_string();
    let tool_detail_policy_max_output_chars =
        context.chat.tool_detail_policy_max_output_chars.to_string();
    let tool_detail_policy_sandbox_mode = if context.chat.tool_detail_policy_sandbox_mode.is_empty()
    {
        "default".to_string()
    } else {
        context.chat.tool_detail_policy_sandbox_mode.clone()
    };
    let tool_detail_usage_histogram_rows = context.chat.tool_detail_usage_histogram_rows.clone();
    let tool_detail_usage_bucket_count = tool_detail_usage_histogram_rows.len().to_string();
    let tool_detail_usage_histogram_view = if tool_detail_usage_histogram_rows.is_empty() {
        leptos::either::Either::Left(view! {
            <li id="tau-ops-tool-detail-usage-empty-state" data-empty-state="true">
                No usage histogram buckets available.
            </li>
        })
    } else {
        leptos::either::Either::Right(
            tool_detail_usage_histogram_rows
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    let row_id = format!("tau-ops-tool-detail-usage-bucket-{index}");
                    let hour_offset = row.hour_offset.to_string();
                    let call_count = row.call_count.to_string();
                    view! {
                        <li id=row_id data-hour-offset=hour_offset data-call-count=call_count></li>
                    }
                })
                .collect_view(),
        )
    };
    let tool_detail_recent_invocation_rows =
        context.chat.tool_detail_recent_invocation_rows.clone();
    let tool_detail_invocation_count = tool_detail_recent_invocation_rows.len().to_string();
    let tool_detail_invocations_view = if tool_detail_recent_invocation_rows.is_empty() {
        leptos::either::Either::Left(view! {
            <tr id="tau-ops-tool-detail-invocation-empty-state" data-empty-state="true">
                <td colspan="5">No recent invocations recorded.</td>
            </tr>
        })
    } else {
        leptos::either::Either::Right(
            tool_detail_recent_invocation_rows
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    let row_id = format!("tau-ops-tool-detail-invocation-row-{index}");
                    let timestamp_unix_ms = row.timestamp_unix_ms.to_string();
                    let duration_ms = row.duration_ms.to_string();
                    let timestamp_unix_ms_attr = timestamp_unix_ms.clone();
                    let duration_ms_attr = duration_ms.clone();
                    view! {
                        <tr
                            id=row_id
                            data-timestamp-unix-ms=timestamp_unix_ms_attr
                            data-args-summary=row.args_summary.clone()
                            data-result-summary=row.result_summary.clone()
                            data-duration-ms=duration_ms_attr
                            data-status=row.status.clone()
                        >
                            <td>{timestamp_unix_ms}</td>
                            <td>{row.args_summary.clone()}</td>
                            <td>{row.result_summary.clone()}</td>
                            <td>{duration_ms}</td>
                            <td>{row.status.clone()}</td>
                        </tr>
                    }
                })
                .collect_view(),
        )
    };
    let jobs_rows = if tools_jobs_route_active {
        context.chat.jobs_rows.clone()
    } else {
        Vec::new()
    };
    let jobs_total_count_value = jobs_rows.len().to_string();
    let jobs_total_count_panel_attr = jobs_total_count_value.clone();
    let jobs_row_count_table_attr = jobs_total_count_value.clone();
    let jobs_row_count_body_attr = jobs_total_count_value;
    let jobs_running_count = jobs_rows
        .iter()
        .filter(|row| row.job_status.as_str() == "running")
        .count()
        .to_string();
    let jobs_completed_count = jobs_rows
        .iter()
        .filter(|row| row.job_status.as_str() == "completed")
        .count()
        .to_string();
    let jobs_failed_count = jobs_rows
        .iter()
        .filter(|row| row.job_status.as_str() == "failed")
        .count()
        .to_string();
    let jobs_panel_visible = if tools_jobs_route_active {
        "true"
    } else {
        "false"
    };
    let jobs_rows_view = if jobs_rows.is_empty() {
        leptos::either::Either::Left(view! {
            <tr id="tau-ops-jobs-empty-state" data-empty-state="true">
                <td colspan="6">No jobs recorded.</td>
            </tr>
        })
    } else {
        leptos::either::Either::Right(
            jobs_rows
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    let row_id = format!("tau-ops-jobs-row-{index}");
                    let view_output_id = format!("tau-ops-jobs-view-output-{index}");
                    let cancel_id = format!("tau-ops-jobs-cancel-{index}");
                    let job_id = row.job_id.clone();
                    let started_unix_ms = row.started_unix_ms.to_string();
                    let finished_unix_ms = row.finished_unix_ms.to_string();
                    let started_unix_ms_attr = started_unix_ms.clone();
                    let finished_unix_ms_attr = finished_unix_ms.clone();
                    let view_output_href = format!(
                        "{active_shell_path}?theme={theme_attr}&sidebar={sidebar_state_attr}&session={chat_session_key}&job={job_id}"
                    );
                    let cancel_href = format!(
                        "{active_shell_path}?theme={theme_attr}&sidebar={sidebar_state_attr}&session={chat_session_key}&job={job_id}&cancel_job={job_id}"
                    );
                    let cancel_enabled = if row.job_status.as_str() == "running" {
                        "true"
                    } else {
                        "false"
                    };
                    view! {
                        <tr
                            id=row_id
                            data-job-id=row.job_id.clone()
                            data-job-name=row.job_name.clone()
                            data-job-status=row.job_status.clone()
                            data-started-unix-ms=started_unix_ms_attr
                            data-finished-unix-ms=finished_unix_ms_attr
                        >
                            <td>{row.job_id.clone()}</td>
                            <td>{row.job_name.clone()}</td>
                            <td>{row.job_status.clone()}</td>
                            <td>{started_unix_ms}</td>
                            <td>{finished_unix_ms}</td>
                            <td>
                                <a
                                    id=view_output_id
                                    data-action="view-job-output"
                                    data-job-id=row.job_id.clone()
                                    href=view_output_href
                                >
                                    View Output
                                </a>
                                <a
                                    id=cancel_id
                                    data-action="cancel-job"
                                    data-job-id=row.job_id.clone()
                                    data-cancel-enabled=cancel_enabled
                                    href=cancel_href
                                >
                                    Cancel
                                </a>
                            </td>
                        </tr>
                    }
                })
                .collect_view(),
        )
    };
    let job_detail_selected_job_id = {
        let selected = context.chat.job_detail_selected_job_id.trim();
        if !selected.is_empty() {
            selected.to_string()
        } else {
            jobs_rows
                .first()
                .map(|row| row.job_id.clone())
                .unwrap_or_default()
        }
    };
    let selected_job_row = jobs_rows
        .iter()
        .find(|row| row.job_id.as_str() == job_detail_selected_job_id.as_str());
    let job_detail_status = if context.chat.job_detail_status.trim().is_empty() {
        selected_job_row
            .map(|row| row.job_status.clone())
            .unwrap_or_default()
    } else {
        context.chat.job_detail_status.clone()
    };
    let job_detail_duration_ms_value = if context.chat.job_detail_duration_ms == 0 {
        selected_job_row
            .map(|row| row.finished_unix_ms.saturating_sub(row.started_unix_ms))
            .unwrap_or(0)
    } else {
        context.chat.job_detail_duration_ms
    };
    let job_detail_duration_ms = job_detail_duration_ms_value.to_string();
    let job_detail_stdout = context.chat.job_detail_stdout.clone();
    let job_detail_stdout_bytes = job_detail_stdout.len().to_string();
    let job_detail_stderr = context.chat.job_detail_stderr.clone();
    let job_detail_stderr_bytes = job_detail_stderr.len().to_string();
    let job_detail_visible = if tools_jobs_route_active && !job_detail_selected_job_id.is_empty() {
        "true"
    } else {
        "false"
    };
    let job_cancel_status = if job_detail_status.as_str() == "cancelled" {
        "cancelled"
    } else {
        "idle"
    };
    let job_cancel_panel_visible = if tools_jobs_route_active {
        "true"
    } else {
        "false"
    };
    let job_cancel_enabled = if tools_jobs_route_active
        && selected_job_row
            .map(|row| row.job_status.as_str() == "running")
            .unwrap_or(false)
    {
        "true"
    } else {
        "false"
    };
    let job_cancel_submit_href = format!(
        "{active_shell_path}?theme={theme_attr}&sidebar={sidebar_state_attr}&session={chat_session_key}&job={job_detail_selected_job_id}&cancel_job={job_detail_selected_job_id}"
    );
    let session_detail_panel_active = sessions_route_active && context.chat.session_detail_visible;
    let session_detail_panel_hidden = if session_detail_panel_active {
        "false"
    } else {
        "true"
    };
    let session_detail_route = context.chat.session_detail_route.clone();
    let session_graph_route = session_detail_route.clone();
    let session_reset_form_action = session_detail_route.clone();
    let session_detail_validation_entries =
        context.chat.session_detail_validation_entries.to_string();
    let session_detail_validation_duplicates = context
        .chat
        .session_detail_validation_duplicates
        .to_string();
    let session_detail_validation_invalid_parent = context
        .chat
        .session_detail_validation_invalid_parent
        .to_string();
    let session_detail_validation_cycles =
        context.chat.session_detail_validation_cycles.to_string();
    let session_detail_validation_is_valid = if context.chat.session_detail_validation_is_valid {
        "true"
    } else {
        "false"
    };
    let session_detail_usage_input_tokens =
        context.chat.session_detail_usage_input_tokens.to_string();
    let session_detail_usage_output_tokens =
        context.chat.session_detail_usage_output_tokens.to_string();
    let session_detail_usage_total_tokens =
        context.chat.session_detail_usage_total_tokens.to_string();
    let session_detail_usage_estimated_cost_usd =
        context.chat.session_detail_usage_estimated_cost_usd.clone();
    let session_detail_timeline_rows = if session_detail_panel_active {
        context.chat.session_detail_timeline_rows.clone()
    } else {
        Vec::new()
    };
    let session_detail_timeline_count = session_detail_timeline_rows.len().to_string();
    let session_detail_timeline_view = if session_detail_timeline_rows.is_empty() {
        leptos::either::Either::Left(view! {
            <li id="tau-ops-session-message-empty-state" data-empty-state="true">
                No session timeline entries yet.
            </li>
        })
    } else {
        leptos::either::Either::Right(
            session_detail_timeline_rows
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    let row_id = format!("tau-ops-session-message-row-{index}");
                    let entry_id = row.entry_id.to_string();
                    let row_entry_id_value = entry_id.clone();
                    let row_content_attr = row.content.clone();
                    let row_content_body = row.content.clone();
                    let form_entry_id_value = entry_id.clone();
                    let hidden_entry_id_value = entry_id.clone();
                    let branch_form_id = format!("tau-ops-session-branch-form-{index}");
                    let branch_source_id =
                        format!("tau-ops-session-branch-source-session-key-{index}");
                    let branch_entry_id = format!("tau-ops-session-branch-entry-id-{index}");
                    let branch_target_id =
                        format!("tau-ops-session-branch-target-session-key-{index}");
                    let branch_theme_id = format!("tau-ops-session-branch-theme-{index}");
                    let branch_sidebar_id = format!("tau-ops-session-branch-sidebar-{index}");
                    let branch_submit_id = format!("tau-ops-session-branch-submit-{index}");
                    view! {
                        <li
                            id=row_id
                            data-entry-id=row_entry_id_value
                            data-message-role=row.role.clone()
                            data-message-content=row_content_attr
                        >
                            {row_content_body}
                            <form
                                id=branch_form_id
                                action="/ops/sessions/branch"
                                method="post"
                                data-source-session-key=chat_session_key.clone()
                                data-entry-id=form_entry_id_value
                            >
                                <input
                                    id=branch_source_id
                                    type="hidden"
                                    name="source_session_key"
                                    value=chat_session_key.clone()
                                />
                                <input
                                    id=branch_entry_id
                                    type="hidden"
                                    name="entry_id"
                                    value=hidden_entry_id_value
                                />
                                <label for=branch_target_id.clone()>Branch Session Key</label>
                                <input
                                    id=branch_target_id
                                    type="text"
                                    name="target_session_key"
                                    value=""
                                />
                                <input id=branch_theme_id type="hidden" name="theme" value=theme_attr />
                                <input
                                    id=branch_sidebar_id
                                    type="hidden"
                                    name="sidebar"
                                    value=sidebar_state_attr
                                />
                                <button
                                    id=branch_submit_id
                                    type="submit"
                                    data-confirmation-required="true"
                                >
                                    Branch Session
                                </button>
                            </form>
                        </li>
                    }
                })
                .collect_view(),
        )
    };
    let session_graph_node_rows = if session_detail_panel_active {
        context.chat.session_graph_node_rows.clone()
    } else {
        Vec::new()
    };
    let session_graph_edge_rows = if session_detail_panel_active {
        context.chat.session_graph_edge_rows.clone()
    } else {
        Vec::new()
    };
    let session_graph_node_count = session_graph_node_rows.len().to_string();
    let session_graph_edge_count = session_graph_edge_rows.len().to_string();
    let session_graph_view = if session_graph_node_rows.is_empty() {
        leptos::either::Either::Left(view! {
            <li id="tau-ops-session-graph-empty-state" data-empty-state="true">
                No session graph nodes yet.
            </li>
        })
    } else {
        leptos::either::Either::Right(
            session_graph_node_rows
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    let row_id = format!("tau-ops-session-graph-node-{index}");
                    let entry_id = row.entry_id.to_string();
                    view! {
                        <li id=row_id data-entry-id=entry_id data-message-role=row.role.clone()></li>
                    }
                })
                .collect_view(),
        )
    };
    let session_graph_edges_view = session_graph_edge_rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let row_id = format!("tau-ops-session-graph-edge-{index}");
            let source_entry_id = row.source_entry_id.to_string();
            let target_entry_id = row.target_entry_id.to_string();
            view! {
                <li
                    id=row_id
                    data-source-entry-id=source_entry_id
                    data-target-entry-id=target_entry_id
                ></li>
            }
        })
        .collect_view();
    let chat_session_option_count_value = chat_session_options.len().to_string();
    let chat_message_count_value = chat_message_rows.len().to_string();
    let active_chat_session_option = chat_session_options
        .iter()
        .find(|option| option.session_key == chat_session_key);
    let active_session_entry_count_value = active_chat_session_option
        .map(|option| option.entry_count.max(rendered_chat_entry_count))
        .unwrap_or(rendered_chat_entry_count)
        .to_string();
    let active_session_total_tokens_value = active_chat_session_option
        .map(|option| option.usage_total_tokens)
        .unwrap_or(context.chat.session_detail_usage_total_tokens)
        .to_string();
    let active_session_validation_state = if active_chat_session_option
        .map(|option| option.validation_is_valid)
        .unwrap_or(context.chat.session_detail_validation_is_valid)
    {
        "valid"
    } else {
        "invalid"
    };
    let active_session_updated_unix_ms = active_chat_session_option
        .map(|option| option.updated_unix_ms)
        .unwrap_or(0);
    let active_session_updated_unix_ms_value = active_session_updated_unix_ms.to_string();
    let active_session_updated_label =
        format_chat_session_updated_label_at(active_session_updated_unix_ms, current_unix_ms());
    let latest_user_row = chat_message_rows
        .iter()
        .enumerate()
        .rev()
        .find(|(_, row)| row.role == "user");
    let latest_assistant_row = chat_message_rows
        .iter()
        .enumerate()
        .rev()
        .find(|(_, row)| row.role == "assistant");
    let chat_latest_turn_visible_bool = latest_user_row.is_some() || latest_assistant_row.is_some();
    let chat_latest_turn_visible = if chat_latest_turn_visible_bool {
        "true"
    } else {
        "false"
    };
    let chat_latest_turn_hidden = if chat_latest_turn_visible_bool {
        "false"
    } else {
        "true"
    };
    let chat_latest_user_index = latest_user_row
        .map(|(index, _)| index.to_string())
        .unwrap_or_else(|| "none".to_string());
    let chat_latest_assistant_index = latest_assistant_row
        .map(|(index, _)| index.to_string())
        .unwrap_or_else(|| "none".to_string());
    let chat_latest_message_index = match (latest_user_row, latest_assistant_row) {
        (Some((user_index, _)), Some((assistant_index, _))) => {
            user_index.max(assistant_index).to_string()
        }
        (Some((index, _)), None) | (None, Some((index, _))) => index.to_string(),
        (None, None) => "none".to_string(),
    };
    let chat_latest_message_href = if chat_latest_message_index == "none" {
        "#tau-ops-chat-transcript".to_string()
    } else {
        format!("#tau-ops-chat-message-row-{chat_latest_message_index}")
    };
    let chat_session_detail_href = format!(
        "{session_detail_route}?theme={theme_attr}&sidebar={sidebar_state_attr}&session={chat_session_key}"
    );
    let chat_latest_user_content = latest_user_row
        .map(|(_, row)| row.content.clone())
        .unwrap_or_default();
    let chat_latest_assistant_content = latest_assistant_row
        .map(|(_, row)| row.content.clone())
        .unwrap_or_default();
    let chat_new_session_form_action = context.chat.new_session_form_action.clone();
    let chat_new_session_form_method = context.chat.new_session_form_method.clone();
    let chat_send_form_action = context.chat.send_form_action.clone();
    let chat_send_form_method = context.chat.send_form_method.clone();
    let control_action_status = context.chat.control_action_status.clone();
    let control_action = context.chat.control_action.clone();
    let control_action_reason = context.chat.control_action_reason.clone();
    let control_action_status_message = match control_action_status.as_str() {
        "applied" => format!("Applied {control_action} action."),
        "missing" => "No control action was submitted.".to_string(),
        "failed" => format!("Failed to apply {control_action} action ({control_action_reason})."),
        _ => "No control action submitted yet.".to_string(),
    };
    let channel_action_status = context.command_center.channel_action_status.clone();
    let channel_action = context.command_center.channel_action.clone();
    let channel_action_channel = context.command_center.channel_action_channel.clone();
    let channel_action_reason = context.command_center.channel_action_reason.clone();
    let channel_action_status_message = match channel_action_status.as_str() {
        "applied" => format!("Applied {channel_action} for {channel_action_channel}."),
        "missing" => "No channel action was submitted.".to_string(),
        "failed" => format!(
            "Failed to apply {channel_action} for {channel_action_channel} ({channel_action_reason})."
        ),
        _ => "No channel action submitted yet.".to_string(),
    };
    let health_state = context.command_center.health_state.clone();
    let health_reason = context.command_center.health_reason.clone();
    let rollout_gate = context.command_center.rollout_gate.clone();
    let control_mode = context.command_center.control_mode.clone();
    let control_paused_value = if context.command_center.control_paused {
        "true"
    } else {
        "false"
    };
    let action_pause_enabled_value = if context.command_center.action_pause_enabled {
        "true"
    } else {
        "false"
    };
    let action_resume_enabled_value = if context.command_center.action_resume_enabled {
        "true"
    } else {
        "false"
    };
    let action_refresh_enabled_value = if context.command_center.action_refresh_enabled {
        "true"
    } else {
        "false"
    };
    let last_action_request_id = context.command_center.last_action_request_id.clone();
    let last_action_name = context.command_center.last_action_name.clone();
    let last_action_actor = context.command_center.last_action_actor.clone();
    let last_action_reason = context.command_center.last_action_reason.clone();
    let last_action_timestamp_value = context
        .command_center
        .last_action_timestamp_unix_ms
        .to_string();
    let timeline_range = context.command_center.timeline_range.clone();
    let timeline_point_count_value = context.command_center.timeline_point_count.to_string();
    let timeline_last_timestamp_value = context
        .command_center
        .timeline_last_timestamp_unix_ms
        .to_string();
    let range_1h_selected = if timeline_range == "1h" {
        "true"
    } else {
        "false"
    };
    let range_6h_selected = if timeline_range == "6h" {
        "true"
    } else {
        "false"
    };
    let range_24h_selected = if timeline_range == "24h" {
        "true"
    } else {
        "false"
    };
    let range_1h_href =
        format!("{active_shell_path}?theme={theme_attr}&sidebar={sidebar_state_attr}&range=1h");
    let range_6h_href =
        format!("{active_shell_path}?theme={theme_attr}&sidebar={sidebar_state_attr}&range=6h");
    let range_24h_href =
        format!("{active_shell_path}?theme={theme_attr}&sidebar={sidebar_state_attr}&range=24h");
    let queue_depth_value = context.command_center.queue_depth.to_string();
    let failure_streak_value = context.command_center.failure_streak.to_string();
    let processed_cases_value = context.command_center.processed_case_count.to_string();
    let alert_count_value = context.command_center.alert_count.to_string();
    let alert_count_feed_value = alert_count_value.clone();
    let alert_feed_rows = if context.command_center.alert_feed_rows.is_empty() {
        vec![TauOpsDashboardAlertFeedRow {
            code: context.command_center.primary_alert_code.clone(),
            severity: context.command_center.primary_alert_severity.clone(),
            message: context.command_center.primary_alert_message.clone(),
        }]
    } else {
        context.command_center.alert_feed_rows.clone()
    };
    let alert_row_count_value = alert_feed_rows.len().to_string();
    let alert_row_count_section_value = alert_row_count_value.clone();
    let alert_row_count_list_value = alert_row_count_value;
    let connector_health_rows = if context.command_center.connector_health_rows.is_empty() {
        vec![TauOpsDashboardConnectorHealthRow {
            channel: "none".to_string(),
            mode: "unknown".to_string(),
            liveness: "unknown".to_string(),
            events_ingested: 0,
            provider_failures: 0,
        }]
    } else {
        context.command_center.connector_health_rows.clone()
    };
    let connector_row_count_value = connector_health_rows.len().to_string();
    let connector_row_count_table_value = connector_row_count_value.clone();
    let connector_row_count_body_value = connector_row_count_value;
    let channels_panel_hidden = if matches!(context.active_route, TauOpsDashboardRoute::Channels) {
        "false"
    } else {
        "true"
    };
    let channels_panel_visible = if matches!(context.active_route, TauOpsDashboardRoute::Channels) {
        "true"
    } else {
        "false"
    };
    let channels_online_count = connector_health_rows
        .iter()
        .filter(|row| matches!(row.liveness.as_str(), "open" | "online"))
        .count()
        .to_string();
    let channels_offline_count = connector_health_rows
        .iter()
        .filter(|row| matches!(row.liveness.as_str(), "offline" | "unknown"))
        .count()
        .to_string();
    let channels_degraded_count = connector_health_rows
        .iter()
        .filter(|row| {
            !matches!(
                row.liveness.as_str(),
                "open" | "online" | "offline" | "unknown"
            )
        })
        .count()
        .to_string();
    let channels_online_count_summary = channels_online_count.clone();
    let channels_online_count_card = channels_online_count.clone();
    let channels_offline_count_summary = channels_offline_count.clone();
    let channels_offline_count_card = channels_offline_count.clone();
    let channels_degraded_count_summary = channels_degraded_count.clone();
    let channels_degraded_count_card = channels_degraded_count.clone();
    let channels_row_count_value = connector_health_rows.len().to_string();
    let channels_row_count_table_value = channels_row_count_value.clone();
    let channels_row_count_body_value = channels_row_count_value.clone();
    let channels_row_count_panel_value = channels_row_count_value;
    let channels_rows_view = connector_health_rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let row_id = format!("tau-ops-channels-row-{index}");
            let login_id = format!("tau-ops-channels-login-{index}");
            let logout_id = format!("tau-ops-channels-logout-{index}");
            let probe_id = format!("tau-ops-channels-probe-{index}");
            let action_form_id = format!("tau-ops-channels-action-form-{index}");
            let channel_input_id = format!("tau-ops-channels-action-{index}-channel");
            let theme_input_id = format!("tau-ops-channels-action-{index}-theme");
            let sidebar_input_id = format!("tau-ops-channels-action-{index}-sidebar");
            let session_input_id = format!("tau-ops-channels-action-{index}-session");
            let channel = row.channel.clone();
            let liveness = row.liveness.clone();
            let events_ingested = row.events_ingested.to_string();
            let provider_failures = row.provider_failures.to_string();
            let events_ingested_attr = events_ingested.clone();
            let provider_failures_attr = provider_failures.clone();
            let login_enabled = if matches!(liveness.as_str(), "offline" | "unknown") {
                "true"
            } else {
                "false"
            };
            let logout_enabled = if matches!(liveness.as_str(), "open" | "online") {
                "true"
            } else {
                "false"
            };
            let probe_enabled = "true";
            let login_aria_disabled = if login_enabled == "true" {
                "false"
            } else {
                "true"
            };
            let logout_aria_disabled = if logout_enabled == "true" {
                "false"
            } else {
                "true"
            };
            let probe_aria_disabled = "false";
            view! {
                <tr
                    id=row_id
                    data-channel=row.channel.clone()
                    data-mode=row.mode.clone()
                    data-liveness=row.liveness.clone()
                    data-events-ingested=events_ingested_attr
                    data-provider-failures=provider_failures_attr
                >
                    <td><span class="tau-ops-channel-name">{row.channel.clone()}</span></td>
                    <td><span class="tau-ops-channel-mode">{row.mode.clone()}</span></td>
                    <td>
                        <span
                            class="tau-ops-channel-liveness"
                            data-liveness=row.liveness.clone()
                        >
                            {row.liveness.clone()}
                        </span>
                    </td>
                    <td>{events_ingested}</td>
                    <td>{provider_failures}</td>
                    <td class="tau-ops-channel-action-cell" data-column="actions">
                        <div
                            class="tau-ops-channel-actions"
                            data-action-count="3"
                            data-hit-target-contract="separate-action-buttons"
                        >
                            <form
                                id=action_form_id
                                action="/ops/channels/action"
                                method="post"
                                data-action="channel-lifecycle"
                                data-channel=row.channel.clone()
                                data-action-enabled="true"
                                data-submit-contract="clicked-button-action"
                            >
                                <input id=channel_input_id type="hidden" name="channel" value=channel.clone() />
                                <input id=theme_input_id type="hidden" name="theme" value=theme_attr />
                                <input id=sidebar_input_id type="hidden" name="sidebar" value=sidebar_state_attr />
                                <input id=session_input_id type="hidden" name="session" value=chat_session_key.clone() />
                                <button
                                    id=login_id
                                    data-action="channel-login"
                                    data-channel=row.channel.clone()
                                    data-action-enabled=login_enabled
                                    type="submit"
                                    name="action"
                                    value="login"
                                    aria-disabled=login_aria_disabled
                                >
                                    Login
                                </button>
                                <button
                                    id=logout_id
                                    data-action="channel-logout"
                                    data-channel=row.channel.clone()
                                    data-action-enabled=logout_enabled
                                    type="submit"
                                    name="action"
                                    value="logout"
                                    aria-disabled=logout_aria_disabled
                                >
                                    Logout
                                </button>
                                <button
                                    id=probe_id
                                    data-action="channel-probe"
                                    data-channel=row.channel.clone()
                                    data-action-enabled=probe_enabled
                                    type="submit"
                                    name="action"
                                    value="probe"
                                    aria-disabled=probe_aria_disabled
                                >
                                    Probe
                                </button>
                            </form>
                        </div>
                    </td>
                </tr>
            }
        })
        .collect_view();
    let widget_count_value = context.command_center.widget_count.to_string();
    let timeline_cycle_count_value = context.command_center.timeline_cycle_count.to_string();
    let timeline_cycle_count_table_value = timeline_cycle_count_value.clone();
    let timeline_cycle_count_summary_value = timeline_cycle_count_table_value.clone();
    let timeline_point_count_table_value = timeline_point_count_value.clone();
    let timeline_last_timestamp_table_value = timeline_last_timestamp_value.clone();
    let timeline_invalid_cycle_count_value = context
        .command_center
        .timeline_invalid_cycle_count
        .to_string();
    let timeline_invalid_cycle_count_summary_value = timeline_invalid_cycle_count_value.clone();
    let timeline_empty_row = if context.command_center.timeline_point_count == 0 {
        Some(view! {
            <tr id="tau-ops-timeline-empty-row" data-empty-state="true">
                <td colspan="4">No timeline points yet.</td>
            </tr>
        })
    } else {
        None
    };
    let primary_alert_code = context.command_center.primary_alert_code.clone();
    let primary_alert_severity = context.command_center.primary_alert_severity.clone();
    let primary_alert_message = context.command_center.primary_alert_message.clone();

    let shell = view! {
        <div
            id="tau-ops-shell"
            data-app="tau-ops-dashboard"
            data-theme=theme_attr
            data-sidebar-state=sidebar_state_attr
            data-sidebar-mobile-default="collapsed"
            data-active-route=active_route_attr
            data-shell-quality="operator-route-parity"
        >
            <style id="tau-ops-dashboard-base-style">
                r#"
                #tau-ops-shell {
                    min-height: 100vh;
                    background: #08141c;
                    color: #dbe8ef;
                    font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
                    width: 100%;
                    max-width: 100%;
                    overflow-x: hidden;
                }
                #tau-ops-shell,
                #tau-ops-shell * {
                    box-sizing: border-box;
                }
                #tau-ops-shell a {
                    color: #8bbcff;
                }
                #tau-ops-shell [aria-hidden="true"] {
                    display: none !important;
                }
                #tau-ops-header {
                    display: grid;
                    grid-template-columns: minmax(0, 1fr) auto;
                    gap: 8px 16px;
                    align-items: center;
                    padding: 12px 16px;
                    border-bottom: 1px solid #203847;
                    background: #0a1a24;
                }
                #tau-ops-header h1 {
                    margin: 0;
                    color: #edf8fb;
                    font-size: 1.05rem;
                    font-weight: 720;
                    letter-spacing: 0;
                }
                #tau-ops-header p {
                    margin: 2px 0 0;
                    color: #8fa8b3;
                    font-size: .76rem;
                }
                #tau-ops-skip-to-main {
                    position: absolute;
                    left: 12px;
                    top: 12px;
                    transform: translateY(-150%);
                    padding: 6px 10px;
                    border-radius: 6px;
                    background: #e8f4ff;
                    color: #051018;
                    font-size: .76rem;
                    font-weight: 700;
                    text-decoration: none;
                    z-index: 3;
                }
                #tau-ops-skip-to-main:focus-visible {
                    transform: translateY(0);
                    outline: 2px solid #66b8ff;
                    outline-offset: 2px;
                }
                #tau-ops-shell-controls {
                    display: flex;
                    align-items: center;
                    gap: 8px;
                    grid-column: 2;
                    grid-row: 1 / span 2;
                }
                #tau-ops-sidebar-hamburger,
                #tau-ops-theme-controls a {
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    min-height: 30px;
                    border: 1px solid #2b4b5d;
                    border-radius: 6px;
                    padding: 5px 9px;
                    background: #102838;
                    color: #d9edf6;
                    font-size: .72rem;
                    font-weight: 700;
                    text-decoration: none;
                }
                #tau-ops-theme-controls {
                    display: flex;
                    gap: 5px;
                    padding: 3px;
                    border: 1px solid #263f4e;
                    border-radius: 7px;
                    background: #07151d;
                }
                #tau-ops-theme-controls a[aria-pressed="true"] {
                    background: #1b5fbf;
                    border-color: #3d8fff;
                    color: #fff;
                }
                #tau-ops-breadcrumbs {
                    grid-column: 1 / -1;
                    color: #8fa8b3;
                    font-size: .72rem;
                }
                #tau-ops-breadcrumbs ol {
                    display: flex;
                    gap: 6px;
                    margin: 0;
                    padding: 0;
                    list-style: none;
                }
                #tau-ops-breadcrumb-home::after {
                    content: "/";
                    margin-left: 6px;
                    color: #547181;
                }
                #tau-ops-layout {
                    display: grid;
                    grid-template-columns: 176px minmax(0, 1fr);
                    min-height: calc(100vh - 82px);
                    width: 100%;
                    max-width: 100vw;
                    overflow-x: hidden;
                }
                #tau-ops-sidebar {
                    padding: 10px 6px;
                    border-right: 1px solid #203847;
                    background: #0a1a24;
                }
                #tau-ops-sidebar ul {
                    display: grid;
                    gap: 6px;
                    margin: 0;
                    padding: 0;
                    list-style: none;
                }
                #tau-ops-sidebar a {
                    display: flex;
                    align-items: center;
                    justify-content: flex-start;
                    min-height: 32px;
                    border: 1px solid transparent;
                    border-radius: 6px;
                    padding: 7px 10px;
                    color: #bdd5df;
                    font-size: .74rem;
                    font-weight: 700;
                    line-height: 1.1;
                    text-align: left;
                    text-decoration: none;
                    overflow-wrap: normal;
                    word-break: normal;
                }
                #tau-ops-sidebar a:hover,
                #tau-ops-sidebar a:focus-visible {
                    border-color: #31596e;
                    background: #122d3d;
                    outline: none;
                }
                #tau-ops-sidebar a[aria-current="page"] {
                    border-color: #3d8fff;
                    background: #1b5fbf;
                    color: #fff;
                }
                #tau-ops-auth-shell,
                #tau-ops-protected-shell {
                    min-width: 0;
                    max-width: 100%;
                    overflow-x: hidden;
                }
                #tau-ops-protected-shell {
                    display: block;
                    padding: 14px;
                    width: calc(100vw - 208px);
                    max-width: calc(100vw - 208px);
                }
                #tau-ops-protected-shell > section[data-panel-visible="true"],
                #tau-ops-command-center[aria-hidden="false"] {
                    min-width: 0;
                    max-width: 100%;
                    overflow-x: hidden;
                    border: 1px solid #243e4d;
                    border-radius: 8px;
                    padding: 14px;
                    background: #0b1d28;
                    box-shadow: inset 0 1px 0 rgba(255, 255, 255, .04);
                }
                #tau-ops-protected-shell > section[data-panel-visible="true"] > h2,
                #tau-ops-command-center[aria-hidden="false"] > h2,
                #tau-ops-protected-shell > section[data-panel-visible="true"] h2,
                #tau-ops-command-center[aria-hidden="false"] h2 {
                    margin-top: 0;
                    color: #edf8fb;
                    font-size: .95rem;
                    letter-spacing: 0;
                }
                #tau-ops-protected-shell section section,
                #tau-ops-protected-shell article {
                    min-width: 0;
                }
                #tau-ops-protected-shell form,
                #tau-ops-deploy-model-selection {
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
                    gap: 8px 10px;
                    align-items: end;
                    margin: 10px 0;
                }
                #tau-ops-protected-shell label {
                    display: grid;
                    gap: 5px;
                    color: #9bb6c2;
                    font-size: .72rem;
                    font-weight: 700;
                }
                #tau-ops-protected-shell input,
                #tau-ops-protected-shell textarea,
                #tau-ops-protected-shell select {
                    width: 100%;
                    min-height: 32px;
                    border: 1px solid #2c4b5d;
                    border-radius: 6px;
                    padding: 6px 8px;
                    background: #07151d;
                    color: #edf8fb;
                    font: inherit;
                    font-size: .78rem;
                }
                #tau-ops-protected-shell button,
                #tau-ops-protected-shell a[role="button"] {
                    min-height: 32px;
                    border: 1px solid #31596e;
                    border-radius: 6px;
                    padding: 6px 10px;
                    background: #123149;
                    color: #edf8fb;
                    font: inherit;
                    font-size: .76rem;
                    font-weight: 700;
                }
                #tau-ops-protected-shell table {
                    width: 100%;
                    border-collapse: collapse;
                    table-layout: fixed;
                    font-size: .74rem;
                }
                #tau-ops-protected-shell th,
                #tau-ops-protected-shell td {
                    border-bottom: 1px solid #203847;
                    padding: 7px 8px;
                    text-align: left;
                    vertical-align: top;
                    overflow-wrap: anywhere;
                }
                #tau-ops-protected-shell th {
                    color: #8fa8b3;
                    font-weight: 700;
                }
                #tau-ops-protected-shell ul,
                #tau-ops-protected-shell ol {
                    padding-left: 1rem;
                }
                #tau-ops-chat-panel[aria-hidden="false"] {
                    display: grid;
                    grid-template-columns: minmax(0, 1fr);
                    gap: 12px;
                    align-items: start;
                    width: 100%;
                    max-width: 720px;
                }
                #tau-ops-chat-panel[aria-hidden="false"] > h2 {
                    grid-column: 1 / -1;
                    margin-bottom: 0;
                }
                #tau-ops-chat-session-summary,
                #tau-ops-chat-session-selector,
                #tau-ops-chat-new-session-form {
                    grid-column: 1;
                    width: 100%;
                    max-width: 420px;
                }
                #tau-ops-chat-send-form,
                #tau-ops-chat-latest-turn,
                #tau-ops-chat-transcript,
                #tau-ops-chat-token-counter {
                    grid-column: 1;
                    min-width: 0;
                    width: 100%;
                    max-width: 720px;
                }
                #tau-ops-chat-session-summary {
                    display: grid;
                    gap: 8px;
                    border: 1px solid #203847;
                    border-radius: 7px;
                    padding: 10px;
                    background: #091923;
                }
                #tau-ops-chat-session-summary h3 {
                    margin: 0;
                    color: #edf8fb;
                    font-size: .82rem;
                    letter-spacing: 0;
                }
                #tau-ops-chat-session-summary dl {
                    display: grid;
                    grid-template-columns: repeat(2, minmax(0, 1fr));
                    gap: 8px;
                    margin: 0;
                }
                #tau-ops-chat-session-summary div {
                    min-width: 0;
                    border: 1px solid #263f4e;
                    border-radius: 6px;
                    padding: 7px 8px;
                    background: #0d2331;
                }
                #tau-ops-chat-session-summary dt {
                    color: #8fa8b3;
                    font-size: .64rem;
                    font-weight: 800;
                    letter-spacing: .02em;
                    text-transform: uppercase;
                }
                #tau-ops-chat-session-summary dd {
                    margin: 2px 0 0;
                    color: #edf8fb;
                    font-size: .8rem;
                    font-weight: 750;
                    overflow-wrap: anywhere;
                }
                #tau-ops-chat-session-actions {
                    display: grid;
                    grid-template-columns: repeat(2, minmax(0, 1fr));
                    gap: 6px;
                    margin-top: 2px;
                }
                #tau-ops-chat-session-actions a {
                    display: flex;
                    min-width: 0;
                    min-height: 30px;
                    align-items: center;
                    justify-content: center;
                    border: 1px solid #2f5368;
                    border-radius: 6px;
                    padding: 6px 8px;
                    background: #102b3a;
                    color: #dbe8ef;
                    font-size: .72rem;
                    font-weight: 800;
                    text-align: center;
                    text-decoration: none;
                    overflow-wrap: anywhere;
                }
                #tau-ops-chat-session-selector {
                    border: 1px solid #203847;
                    border-radius: 7px;
                    padding: 10px;
                    background: #091923;
                }
                #tau-ops-chat-session-selector h3 {
                    margin: 0 0 8px;
                    color: #9bb6c2;
                    font-size: .74rem;
                    letter-spacing: 0;
                }
                #tau-ops-chat-session-options {
                    display: grid;
                    gap: 6px;
                    margin: 0;
                    padding: 0;
                    list-style: none;
                }
                #tau-ops-chat-session-options a {
                    display: flex;
                    min-width: 0;
                    min-height: 30px;
                    align-items: center;
                    border: 1px solid #263f4e;
                    border-radius: 6px;
                    padding: 6px 8px;
                    background: #0d2331;
                    color: #dbe8ef;
                    font-size: .76rem;
                    font-weight: 700;
                    text-decoration: none;
                    overflow-wrap: anywhere;
                }
                #tau-ops-chat-session-options [data-selected="true"] a {
                    border-color: #3d8fff;
                    background: #123b61;
                    color: #fff;
                }
                #tau-ops-protected-shell #tau-ops-chat-new-session-form {
                    display: grid;
                    grid-template-columns: minmax(0, 1fr);
                    gap: 8px;
                    align-items: start;
                    margin: 0;
                    border: 1px solid #203847;
                    border-radius: 7px;
                    padding: 10px;
                    background: #091923;
                }
                #tau-ops-protected-shell #tau-ops-chat-send-form {
                    display: grid;
                    grid-template-columns: minmax(0, 1fr);
                    gap: 8px;
                    align-items: stretch;
                    margin: 0;
                }
                #tau-ops-chat-send-form label {
                    grid-column: 1 / -1;
                }
                #tau-ops-chat-input-shortcut-hint {
                    grid-column: 1 / -1;
                    margin: 0;
                    color: #8fa8b3;
                    font-size: .72rem;
                }
                #tau-ops-chat-input {
                    min-height: 92px;
                    resize: vertical;
                }
                #tau-ops-chat-send-button {
                    width: 100%;
                    min-height: 42px;
                }
                #tau-ops-chat-latest-turn {
                    display: grid;
                    gap: 8px;
                    margin: 0;
                    border: 1px solid #2c4b5d;
                    border-radius: 7px;
                    padding: 10px;
                    background: #091923;
                }
                #tau-ops-chat-latest-turn h3 {
                    margin: 0;
                    color: #edf8fb;
                    font-size: .82rem;
                    letter-spacing: 0;
                }
                #tau-ops-chat-latest-turn section {
                    display: grid;
                    gap: 5px;
                    min-width: 0;
                    border: 1px solid #24475a;
                    border-radius: 7px;
                    padding: 9px 10px;
                    background: #0d2331;
                    overflow-wrap: anywhere;
                    word-break: break-word;
                }
                #tau-ops-chat-latest-turn section[data-message-role="user"] {
                    border-color: #2d6ead;
                    background: #123b61;
                }
                #tau-ops-chat-latest-turn section[data-message-role="assistant"] {
                    border-color: #255845;
                    background: #102c24;
                }
                #tau-ops-chat-latest-turn h4,
                #tau-ops-chat-latest-turn p {
                    margin: 0;
                }
                #tau-ops-chat-latest-turn h4 {
                    color: #8fa8b3;
                    font-size: .64rem;
                    font-weight: 800;
                    letter-spacing: .02em;
                    text-transform: uppercase;
                }
                #tau-ops-chat-latest-turn p {
                    color: #dbe8ef;
                    font-size: .78rem;
                    line-height: 1.45;
                    white-space: pre-wrap;
                }
                #tau-ops-chat-transcript {
                    display: grid;
                    grid-template-columns: minmax(0, 1fr);
                    gap: 10px;
                    max-height: 58vh;
                    min-height: 280px;
                    margin: 0;
                    padding: 10px;
                    overflow: auto;
                    overflow-x: hidden;
                    border: 1px solid #203847;
                    border-radius: 7px;
                    background: #07151d;
                    list-style: none;
                }
                #tau-ops-chat-transcript > li {
                    display: grid;
                    gap: 6px;
                    width: min(760px, 100%);
                    max-width: min(760px, 100%);
                    min-width: 0;
                    margin: 0;
                    border: 1px solid #24475a;
                    border-radius: 7px;
                    padding: 9px 10px;
                    background: #0d2331;
                    color: #dbe8ef;
                    font-size: .78rem;
                    line-height: 1.45;
                    white-space: pre-wrap;
                    overflow-wrap: anywhere;
                    word-break: break-word;
                }
                #tau-ops-chat-transcript > li::before {
                    content: attr(data-message-role);
                    color: #8fa8b3;
                    font-size: .64rem;
                    font-weight: 800;
                    letter-spacing: .02em;
                    text-transform: uppercase;
                }
                #tau-ops-chat-transcript > li[data-message-role="user"] {
                    justify-self: end;
                    border-color: #2d6ead;
                    background: #123b61;
                }
                #tau-ops-chat-transcript > li[data-message-role="assistant"] {
                    justify-self: start;
                    border-color: #255845;
                    background: #102c24;
                }
                #tau-ops-chat-transcript > li[data-message-role="tool"] {
                    justify-self: stretch;
                    max-width: 100%;
                    border-color: #5a4a1e;
                    background: #211f13;
                }
                #tau-ops-chat-transcript [data-token-stream="assistant"] {
                    display: none;
                }
                #tau-ops-chat-token-counter {
                    margin: 0;
                    border: 1px solid #203847;
                    border-radius: 7px;
                    padding: 8px 10px;
                    background: #091923;
                    color: #9bb6c2;
                    font-size: .72rem;
                }
                #tau-ops-memory-scope-summary,
                #tau-ops-memory-graph-scope-summary {
                    display: grid;
                    gap: 8px;
                    width: min(720px, 100%);
                    margin: 0 0 12px;
                    border: 1px solid #203847;
                    border-radius: 7px;
                    padding: 10px;
                    background: #091923;
                }
                #tau-ops-memory-scope-summary h3,
                #tau-ops-memory-graph-scope-summary h3 {
                    margin: 0;
                    color: #edf8fb;
                    font-size: .82rem;
                    letter-spacing: 0;
                }
                #tau-ops-memory-scope-summary dl,
                #tau-ops-memory-graph-scope-summary dl {
                    display: grid;
                    grid-template-columns: repeat(3, minmax(0, 1fr));
                    gap: 8px;
                    margin: 0;
                }
                #tau-ops-memory-scope-summary div,
                #tau-ops-memory-graph-scope-summary div {
                    min-width: 0;
                    border: 1px solid #263f4e;
                    border-radius: 6px;
                    padding: 7px 8px;
                    background: #0d2331;
                }
                #tau-ops-memory-scope-summary dt,
                #tau-ops-memory-graph-scope-summary dt {
                    color: #8fa8b3;
                    font-size: .64rem;
                    font-weight: 800;
                    letter-spacing: .02em;
                    text-transform: uppercase;
                }
                #tau-ops-memory-scope-summary dd,
                #tau-ops-memory-graph-scope-summary dd {
                    margin: 2px 0 0;
                    color: #edf8fb;
                    font-size: .8rem;
                    font-weight: 750;
                    overflow-wrap: anywhere;
                }
                #tau-ops-memory-scope-actions,
                #tau-ops-memory-graph-scope-actions {
                    display: grid;
                    grid-template-columns: repeat(2, minmax(0, 1fr));
                    gap: 6px;
                }
                #tau-ops-memory-scope-actions a,
                #tau-ops-memory-graph-scope-actions a {
                    display: flex;
                    min-width: 0;
                    min-height: 30px;
                    align-items: center;
                    justify-content: center;
                    border: 1px solid #2f5368;
                    border-radius: 6px;
                    padding: 6px 8px;
                    background: #102b3a;
                    color: #dbe8ef;
                    font-size: .72rem;
                    font-weight: 800;
                    text-align: center;
                    text-decoration: none;
                    overflow-wrap: anywhere;
                }
                #tau-ops-kpi-grid {
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
                    gap: 10px;
                    margin-bottom: 12px;
                }
                #tau-ops-kpi-grid article {
                    border: 1px solid #243e4d;
                    border-radius: 7px;
                    padding: 10px;
                    background: #102838;
                }
                #tau-ops-kpi-grid article p {
                    margin: 4px 0 0;
                }
                #tau-ops-deploy-wizard-steps ol {
                    display: grid;
                    grid-template-columns: minmax(0, 640px);
                    gap: 8px;
                    margin: 0 0 12px;
                    padding: 0;
                    list-style: none;
                }
                #tau-ops-deploy-wizard-steps button {
                    min-width: 0;
                    width: 100%;
                    overflow-wrap: anywhere;
                }
                #tau-ops-deploy-model-selection {
                    grid-template-columns: minmax(0, 420px);
                    align-items: start;
                }
                #tau-ops-deploy-model-selection select {
                    max-width: 100%;
                }
                #tau-ops-channels-panel {
                    display: grid;
                    gap: 12px;
                }
                #tau-ops-channels-header {
                    display: grid;
                    gap: 4px;
                }
                #tau-ops-channels-header h2,
                #tau-ops-channels-header p {
                    margin: 0;
                }
                #tau-ops-channels-kpi-grid {
                    display: grid;
                    grid-template-columns: repeat(3, minmax(0, 1fr));
                    gap: 10px;
                }
                #tau-ops-channels-kpi-grid article {
                    border: 1px solid #243e4d;
                    border-radius: 7px;
                    padding: 10px;
                    background: #102838;
                }
                #tau-ops-channels-kpi-grid h3 {
                    margin: 0;
                    color: #9bb6c2;
                    font-size: .72rem;
                    letter-spacing: 0;
                }
                #tau-ops-channels-kpi-grid p {
                    margin: 4px 0 0;
                    color: #edf8fb;
                    font-size: 1.12rem;
                    font-weight: 760;
                }
                #tau-ops-channels-action-status {
                    border: 1px solid #24475a;
                    border-radius: 7px;
                    padding: 10px;
                    background: #0d2331;
                }
                #tau-ops-channels-action-status h3,
                #tau-ops-channels-action-status p {
                    margin: 0;
                }
                #tau-ops-channels-action-status p {
                    margin-top: 4px;
                    color: #abc0c9;
                    font-size: .76rem;
                }
                #tau-ops-channels-table-wrap {
                    max-width: 100%;
                    overflow-x: auto;
                }
                #tau-ops-channels-table {
                    min-width: 760px;
                }
                .tau-ops-channel-name,
                .tau-ops-channel-mode,
                .tau-ops-channel-liveness {
                    display: inline-flex;
                    align-items: center;
                    min-height: 22px;
                    border-radius: 999px;
                    padding: 3px 7px;
                    background: #102838;
                    color: #edf8fb;
                    font-size: .72rem;
                    font-weight: 700;
                    overflow-wrap: anywhere;
                }
                .tau-ops-channel-liveness[data-liveness="open"],
                .tau-ops-channel-liveness[data-liveness="online"] {
                    background: #123c2f;
                    color: #9cf0bd;
                }
                .tau-ops-channel-liveness[data-liveness="offline"],
                .tau-ops-channel-liveness[data-liveness="unknown"] {
                    background: #3d2f12;
                    color: #f7d77b;
                }
                #tau-ops-channels-table th[data-column="actions"],
                #tau-ops-channels-table td[data-column="actions"] {
                    min-width: 270px;
                    width: 270px;
                }
                .tau-ops-channel-actions {
                    display: grid;
                    grid-template-columns: repeat(3, minmax(78px, 1fr));
                    gap: 6px;
                    min-width: 252px;
                }
                .tau-ops-channel-actions form {
                    display: block;
                    margin: 0;
                    min-width: 0;
                }
                #tau-ops-channels-panel button[data-action^="channel-"] {
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    box-sizing: border-box;
                    position: relative;
                    z-index: 1;
                    width: 100%;
                    min-height: 30px;
                    border: 1px solid #31596e;
                    border-radius: 6px;
                    padding: 5px 8px;
                    background: #123149;
                    color: #edf8fb;
                    font-size: .72rem;
                    font-weight: 700;
                    text-decoration: none;
                    white-space: nowrap;
                    cursor: pointer;
                }
                #tau-ops-channels-panel button[data-action^="channel-"][data-action-enabled="false"] {
                    border-color: #263f4e;
                    background: #0b1d28;
                    color: #6f8996;
                    cursor: not-allowed;
                    pointer-events: none;
                }
                @media (max-width: 900px) {
                    #tau-ops-header {
                        grid-template-columns: 1fr;
                    }
                    #tau-ops-shell-controls {
                        grid-column: 1;
                        grid-row: auto;
                        flex-wrap: wrap;
                    }
                    #tau-ops-layout {
                        grid-template-columns: 1fr;
                    }
                    #tau-ops-sidebar {
                        border-right: 0;
                        border-bottom: 1px solid #203847;
                    }
                    #tau-ops-sidebar ul {
                        grid-template-columns: repeat(3, minmax(0, 1fr));
                    }
                    #tau-ops-sidebar a {
                        justify-content: center;
                        text-align: center;
                    }
                    #tau-ops-protected-shell {
                        padding: 10px;
                        width: 100%;
                        max-width: 100%;
                    }
                    #tau-ops-channels-kpi-grid {
                        grid-template-columns: 1fr;
                    }
                }
                @media (max-width: 640px) {
                    #tau-ops-sidebar ul {
                        grid-template-columns: repeat(2, minmax(0, 1fr));
                    }
                }
                #tau-ops-accessibility-contract,
                #tau-ops-stream-contract,
                #tau-ops-performance-contract {
                    position: absolute;
                    width: 1px;
                    height: 1px;
                    overflow: hidden;
                    clip-path: inset(50%);
                    white-space: nowrap;
                }
                #tau-ops-shell[data-active-route="harness"] {
                    background:
                        radial-gradient(circle at top left, rgba(36, 87, 112, .24), transparent 28rem),
                        #06121a;
                    color: #dbe8ef;
                    width: 100%;
                    max-width: 100%;
                    overflow-x: hidden;
                }
                #tau-ops-shell[data-active-route="harness"] #tau-ops-header {
                    display: none;
                }
                #tau-ops-shell[data-active-route="harness"] #tau-ops-layout {
                    display: grid;
                    grid-template-columns: 76px minmax(0, 1fr);
                    min-height: 100vh;
                    width: 100%;
                    max-width: 100vw;
                    gap: 0;
                    overflow-x: hidden;
                }
                #tau-ops-shell[data-active-route="harness"] #tau-ops-sidebar {
                    padding: 10px 5px;
                    border-right: 1px solid #243e4d;
                    background: linear-gradient(180deg, #0e2433, #07131b);
                }
                #tau-ops-shell[data-active-route="harness"] #tau-ops-sidebar ul {
                    display: grid;
                    gap: 6px;
                    margin: 0;
                    padding: 0;
                    list-style: none;
                }
                #tau-ops-shell[data-active-route="harness"] #tau-ops-sidebar a {
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    min-height: 30px;
                    border-radius: 5px;
                    padding: 6px 4px;
                    color: #b8cfda;
                    font-size: 0;
                    line-height: 1;
                    text-decoration: none;
                    white-space: nowrap;
                    overflow: hidden;
                }
                #tau-ops-shell[data-active-route="harness"] #tau-ops-sidebar a::before {
                    content: attr(data-harness-rail-label);
                    color: inherit;
                    font-size: .68rem;
                    font-weight: 650;
                    line-height: 1;
                    text-align: center;
                }
                #tau-ops-shell[data-active-route="harness"] #tau-ops-nav-harness a {
                    background: #1b5fbf;
                    color: white;
                }
                #tau-ops-shell[data-active-route="harness"] #tau-ops-auth-shell,
                #tau-ops-shell[data-active-route="harness"] #tau-ops-protected-shell {
                    min-width: 0;
                    max-width: 100%;
                    overflow-x: hidden;
                }
                #tau-ops-shell[data-active-route="harness"] #tau-ops-protected-shell {
                    padding: 8px;
                    width: 100%;
                    max-width: 100%;
                }
                @media (max-width: 900px) {
                    #tau-ops-shell[data-active-route="harness"] #tau-ops-layout {
                        grid-template-columns: 1fr;
                    }
                    #tau-ops-shell[data-active-route="harness"] #tau-ops-sidebar {
                        border-right: 0;
                        border-bottom: 1px solid #243e4d;
                        overflow-x: hidden;
                    }
                    #tau-ops-shell[data-active-route="harness"] #tau-ops-protected-shell {
                        width: 100%;
                        max-width: 100%;
                    }
                    #tau-ops-shell[data-active-route="harness"] #tau-ops-sidebar ul {
                        grid-template-columns: repeat(3, minmax(0, 1fr));
                    }
                }
                @media (max-width: 760px) {
                    #tau-ops-shell[data-active-route="harness"] #tau-ops-sidebar ul {
                        grid-template-columns: repeat(2, minmax(0, 1fr));
                    }
                }
                "#
            </style>
            <p
                id="tau-ops-static-preview-status"
                data-preview-route-status="idle"
                hidden
                aria-live="polite"
            >
                "Preview route idle."
            </p>
            <header id="tau-ops-header">
                <h1>Tau Ops Dashboard</h1>
                <p>{shell_subtitle}</p>
                <a
                    id="tau-ops-skip-to-main"
                    href="#tau-ops-protected-shell"
                    data-keyboard-navigation="true"
                >
                    Skip to main content
                </a>
                <div id="tau-ops-shell-controls">
                    <input
                        id="tau-ops-sidebar-toggle"
                        type="checkbox"
                        data-sidebar-state=sidebar_state_attr
                        aria-hidden="true"
                    />
                    <a
                        id="tau-ops-sidebar-hamburger"
                        data-sidebar-toggle="true"
                        data-sidebar-target-state=sidebar_toggle_target_state
                        aria-controls="tau-ops-sidebar"
                        aria-expanded=context.sidebar_state.aria_expanded()
                        href=sidebar_toggle_href
                    >
                        Toggle Navigation
                    </a>
                    <div id="tau-ops-theme-controls" role="group" aria-label="Theme controls">
                        <a
                            id="tau-ops-theme-toggle-dark"
                            data-theme-option="dark"
                            aria-pressed=dark_theme_pressed
                            href=dark_theme_href
                        >
                            Dark
                        </a>
                        <a
                            id="tau-ops-theme-toggle-light"
                            data-theme-option="light"
                            aria-pressed=light_theme_pressed
                            href=light_theme_href
                        >
                            Light
                        </a>
                    </div>
                </div>
                <nav
                    id="tau-ops-breadcrumbs"
                    aria-label="Tau Ops breadcrumbs"
                    data-breadcrumb-current=breadcrumb_current
                >
                    <ol>
                        <li id="tau-ops-breadcrumb-home">
                            <a href="/ops">Home</a>
                        </li>
                        <li id="tau-ops-breadcrumb-current">{breadcrumb_label}</li>
                    </ol>
                </nav>
            </header>
            <div id="tau-ops-layout">
                <aside id="tau-ops-sidebar" data-harness-rail="compact">
                    <nav aria-label="Tau Ops navigation">
                        <ul>
                            <li id="tau-ops-nav-command-center"><a data-nav-item="command-center" href="/ops" data-harness-rail-label="Command" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::Ops)>Command Center</a></li>
                            <li id="tau-ops-nav-agent-fleet"><a data-nav-item="agent-fleet" href="/ops/agents" data-harness-rail-label="Fleet" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::Agents)>Agent Fleet</a></li>
                            <li id="tau-ops-nav-agent-detail"><a data-nav-item="agent-detail" href="/ops/agents/default" data-harness-rail-label="Agent" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::AgentDetail)>Agent Detail</a></li>
                            <li id="tau-ops-nav-chat"><a data-nav-item="chat" href="/ops/chat" data-harness-rail-label="Chat" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::Chat)>Conversation / Chat</a></li>
                            <li id="tau-ops-nav-sessions"><a data-nav-item="sessions" href="/ops/sessions" data-harness-rail-label="Sessions" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::Sessions)>Sessions Explorer</a></li>
                            <li id="tau-ops-nav-memory"><a data-nav-item="memory" href="/ops/memory" data-harness-rail-label="Memory" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::Memory)>Memory Explorer</a></li>
                            <li id="tau-ops-nav-memory-graph"><a data-nav-item="memory-graph" href="/ops/memory-graph" data-harness-rail-label="Graph" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::MemoryGraph)>Memory Graph</a></li>
                            <li id="tau-ops-nav-tools-jobs"><a data-nav-item="tools-jobs" href="/ops/tools-jobs" data-harness-rail-label="Tools" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::ToolsJobs)>Tools & Jobs</a></li>
                            <li id="tau-ops-nav-channels"><a data-nav-item="channels" href="/ops/channels" data-harness-rail-label="Channels" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::Channels)>Channels</a></li>
                            <li id="tau-ops-nav-harness"><a data-nav-item="mission-harness" href="/ops/harness" data-harness-rail-label="Missions" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::Harness)>Mission Harness</a></li>
                            <li id="tau-ops-nav-config"><a data-nav-item="config" href="/ops/config" data-harness-rail-label="Config" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::Config)>Configuration</a></li>
                            <li id="tau-ops-nav-training"><a data-nav-item="training" href="/ops/training" data-harness-rail-label="Training" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::Training)>Training & RL</a></li>
                            <li id="tau-ops-nav-safety"><a data-nav-item="safety" href="/ops/safety" data-harness-rail-label="Safety" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::Safety)>Safety & Security</a></li>
                            <li id="tau-ops-nav-diagnostics"><a data-nav-item="diagnostics" href="/ops/diagnostics" data-harness-rail-label="Audit" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::Diagnostics)>Diagnostics & Audit</a></li>
                            <li id="tau-ops-nav-deploy"><a data-nav-item="deploy" href="/ops/deploy" data-harness-rail-label="Deploy" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::Deploy)>Deploy Agent</a></li>
                            <li id="tau-ops-nav-login"><a href="/ops/login" data-harness-rail-label="Login" aria-current=aria_current_for(context.active_route, TauOpsDashboardRoute::Login)>Operator Login</a></li>
                            <li id="tau-ops-nav-legacy-dashboard"><a href="/dashboard" data-harness-rail-label="Legacy">Legacy Dashboard</a></li>
                            <li id="tau-ops-nav-webchat"><a href="/webchat" data-harness-rail-label="Webchat">Webchat</a></li>
                        </ul>
                    </nav>
                </aside>
                <div
                    id="tau-ops-auth-shell"
                    data-auth-mode=auth_mode_attr
                    data-login-required=if login_required { "true" } else { "false" }
                    data-active-route=active_route_attr
                >
                    <section id="tau-ops-login-shell" data-route="/ops/login" aria-hidden=login_hidden>
                        <h2>Operator Authentication</h2>
                        <p>
                            Use configured gateway auth mode to continue to protected operations views.
                        </p>
                        <form id="tau-ops-login-form">
                            <label for="tau-ops-auth-input">{auth_mode.auth_input_label()}</label>
                            <input
                                id="tau-ops-auth-input"
                                type="password"
                                autocomplete="off"
                                placeholder=auth_mode.auth_input_placeholder()
                            />
                            <button id="tau-ops-login-submit" type="button">Continue</button>
                        </form>
                    </section>
                    <main id="tau-ops-protected-shell" data-route="/ops" aria-hidden=protected_hidden>
                        <section
                            id="tau-ops-accessibility-contract"
                            data-component="AccessibilityContract"
                            data-operator-visible="false"
                            data-axe-contract="required"
                            data-keyboard-navigation="true"
                            data-focus-visible-contract="true"
                            data-focus-ring-token="tau-focus-ring"
                            data-reduced-motion-contract="prefers-reduced-motion"
                            data-reduced-motion-behavior="suppress-nonessential-animation"
                            hidden
                        >
                            <h2>Accessibility Contracts</h2>
                            <p id="tau-ops-live-announcer" aria-live="polite" aria-atomic="true">
                                Accessibility live region ready.
                            </p>
                        </section>
                        <section
                            id="tau-ops-stream-contract"
                            data-component="RealtimeStreamContract"
                            data-operator-visible="false"
                            data-stream-transport="websocket"
                            data-stream-connect-on-load="true"
                            data-heartbeat-target="tau-ops-kpi-grid"
                            data-alert-feed-target="tau-ops-alert-feed-list"
                            data-chat-stream-mode="websocket"
                            data-chat-polling="disabled"
                            data-connector-health-target="tau-ops-connector-table-body"
                            data-reconnect-strategy="exponential-backoff"
                            data-reconnect-base-ms="250"
                            data-reconnect-max-ms="8000"
                            hidden
                        >
                            <h2>Real-Time Stream Contracts</h2>
                        </section>
                        <section
                            id="tau-ops-performance-contract"
                            data-component="PerformanceBudgetContract"
                            data-operator-visible="false"
                            data-wasm-budget-gzip-kb="500"
                            data-lcp-budget-ms="1500"
                            data-layout-shift-budget="0.00"
                            data-layout-shift-mitigation="skeletons"
                            data-websocket-process-budget-ms="50"
                            hidden
                        >
                            <h2>Performance Budgets</h2>
                        </section>
                        <section
                            id="tau-ops-chat-panel"
                            data-route="/ops/chat"
                            aria-hidden=chat_panel_hidden
                            data-active-session-key=chat_session_key.clone()
                            data-panel-visible=chat_panel_visible
                        >
                            <h2>Conversation / Chat</h2>
                            <article
                                id="tau-ops-chat-session-summary"
                                data-active-session-key=chat_session_key.clone()
                                data-entry-count=active_session_entry_count_value.clone()
                                data-total-tokens=active_session_total_tokens_value.clone()
                                data-validation-state=active_session_validation_state
                                data-updated-unix-ms=active_session_updated_unix_ms_value.clone()
                                data-latest-message-index=chat_latest_message_index.clone()
                            >
                                <h3>Session Summary</h3>
                                <dl>
                                    <div>
                                        <dt>Session</dt>
                                        <dd>{chat_session_key.clone()}</dd>
                                    </div>
                                    <div>
                                        <dt>Entries</dt>
                                        <dd>{active_session_entry_count_value.clone()}</dd>
                                    </div>
                                    <div>
                                        <dt>Tokens</dt>
                                        <dd>{active_session_total_tokens_value.clone()}</dd>
                                    </div>
                                    <div>
                                        <dt>Validation</dt>
                                        <dd>{active_session_validation_state}</dd>
                                    </div>
                                    <div>
                                        <dt>Last Updated</dt>
                                        <dd
                                            id="tau-ops-chat-session-updated-label"
                                            data-updated-unix-ms=active_session_updated_unix_ms_value.clone()
                                        >
                                            {active_session_updated_label}
                                        </dd>
                                    </div>
                                </dl>
                                <nav
                                    id="tau-ops-chat-session-actions"
                                    aria-label="Chat session actions"
                                >
                                    <a
                                        id="tau-ops-chat-open-session-detail"
                                        href=chat_session_detail_href
                                    >
                                        Open Session Detail
                                    </a>
                                    <a id="tau-ops-chat-jump-latest" href=chat_latest_message_href>
                                        Jump To Latest
                                    </a>
                                </nav>
                            </article>
                            <section
                                id="tau-ops-chat-session-selector"
                                data-active-session-key=chat_session_key.clone()
                                data-option-count=chat_session_option_count_value
                            >
                                <ul id="tau-ops-chat-session-options">
                                    {chat_session_options
                                        .iter()
                                        .enumerate()
                                        .map(|(index, session_option)| {
                                            let session_option_row_id =
                                                format!("tau-ops-chat-session-option-{index}");
                                            let selected_attr = if session_option.selected {
                                                "true"
                                            } else {
                                                "false"
                                            };
                                            let session_href = format!(
                                                "/ops/chat?theme={theme_attr}&sidebar={sidebar_state_attr}&session={}",
                                                session_option.session_key
                                            );
                                            view! {
                                                <li
                                                    id=session_option_row_id
                                                    data-session-key=session_option.session_key.clone()
                                                    data-selected=selected_attr
                                                >
                                                    <a
                                                        data-session-link=session_option.session_key.clone()
                                                        href=session_href
                                                    >
                                                        {session_option.session_key.clone()}
                                                    </a>
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                </ul>
                            </section>
                            <form
                                id="tau-ops-chat-new-session-form"
                                action=chat_new_session_form_action
                                method=chat_new_session_form_method
                                data-active-session-key=chat_session_key.clone()
                            >
                                <label for="tau-ops-chat-new-session-key">New Session</label>
                                <input
                                    id="tau-ops-chat-new-session-key"
                                    type="text"
                                    name="session_key"
                                    value=""
                                    autocomplete="off"
                                />
                                <input
                                    id="tau-ops-chat-new-theme"
                                    type="hidden"
                                    name="theme"
                                    value=theme_attr
                                />
                                <input
                                    id="tau-ops-chat-new-sidebar"
                                    type="hidden"
                                    name="sidebar"
                                    value=sidebar_state_attr
                                />
                                <button id="tau-ops-chat-new-session-button" type="submit">
                                    Create Session
                                </button>
                            </form>
                            <form
                                id="tau-ops-chat-send-form"
                                action=chat_send_form_action
                                method=chat_send_form_method
                                data-session-key=chat_session_key.clone()
                            >
                                <input
                                    id="tau-ops-chat-session-key"
                                    type="hidden"
                                    name="session_key"
                                    value=chat_session_key.clone()
                                />
                                <input id="tau-ops-chat-theme" type="hidden" name="theme" value=theme_attr />
                                <input
                                    id="tau-ops-chat-sidebar"
                                    type="hidden"
                                    name="sidebar"
                                    value=sidebar_state_attr
                                />
                                <label for="tau-ops-chat-input">Message</label>
                                <p
                                    id="tau-ops-chat-input-shortcut-hint"
                                    data-shortcut-contract="shift-enter"
                                >
                                    Shift+Enter inserts a newline in the message editor.
                                </p>
                                <textarea
                                    id="tau-ops-chat-input"
                                    name="message"
                                    placeholder="Type a message for the active session"
                                    rows="4"
                                    data-multiline-enabled="true"
                                    data-newline-shortcut="shift-enter"
                                ></textarea>
                                <button id="tau-ops-chat-send-button" type="submit">Send</button>
                            </form>
                            <script
                                id="tau-ops-chat-compose-shortcuts"
                                data-submit-shortcut="enter"
                                data-newline-shortcut="shift-enter"
                            >
                                r#"
                                (function () {
                                    function installChatComposeShortcuts() {
                                        var input = document.getElementById("tau-ops-chat-input");
                                        var form = document.getElementById("tau-ops-chat-send-form");
                                        if (!input || !form || input.getAttribute("data-compose-shortcuts-bound") === "true") {
                                            return;
                                        }

                                        input.setAttribute("data-compose-shortcuts-bound", "true");
                                        input.addEventListener("keydown", function (event) {
                                            if (event.key !== "Enter" || event.shiftKey) {
                                                return;
                                            }

                                            event.preventDefault();
                                            if (!input.value || !input.value.trim()) {
                                                input.setAttribute("data-submit-blocked", "empty-message");
                                                return;
                                            }

                                            input.removeAttribute("data-submit-blocked");
                                            if (typeof form.requestSubmit === "function") {
                                                form.requestSubmit();
                                            } else {
                                                form.submit();
                                            }
                                        });
                                    }

                                    if (document.readyState === "loading") {
                                        document.addEventListener("DOMContentLoaded", installChatComposeShortcuts);
                                    } else {
                                        installChatComposeShortcuts();
                                    }
                                })();
                                "#
                            </script>
                            <article
                                id="tau-ops-chat-latest-turn"
                                data-latest-turn-visible=chat_latest_turn_visible
                                aria-hidden=chat_latest_turn_hidden
                                data-latest-user-index=chat_latest_user_index
                                data-latest-assistant-index=chat_latest_assistant_index
                            >
                                <h3>Latest Turn</h3>
                                <section id="tau-ops-chat-latest-user" data-message-role="user">
                                    <h4>User</h4>
                                    <p>{chat_latest_user_content}</p>
                                </section>
                                <section
                                    id="tau-ops-chat-latest-assistant"
                                    data-message-role="assistant"
                                >
                                    <h4>Assistant</h4>
                                    <p>{chat_latest_assistant_content}</p>
                                </section>
                            </article>
                            <ul id="tau-ops-chat-transcript" data-message-count=chat_message_count_value>
                                {chat_message_rows
                                    .iter()
                                    .enumerate()
                                    .map(|(index, message_row)| {
                                        let row_id = format!("tau-ops-chat-message-row-{index}");
                                        if message_row.role == "tool" {
                                            let tool_card_id =
                                                format!("tau-ops-chat-tool-card-{index}");
                                            view! {
                                                <li id=row_id data-message-role=message_row.role.clone()>
                                                    <article
                                                        id=tool_card_id
                                                        data-tool-card="true"
                                                        data-inline-result="true"
                                                    >
                                                        {message_row.content.clone()}
                                                    </article>
                                                </li>
                                            }
                                            .into_any()
                                        } else if message_row.role == "assistant" {
                                            let assistant_tokens =
                                                extract_assistant_stream_tokens(&message_row.content);
                                            let assistant_token_count =
                                                assistant_tokens.len().to_string();
                                            let assistant_token_stream_id =
                                                format!("tau-ops-chat-token-stream-{index}");
                                            let render_assistant_tokens = || {
                                                assistant_tokens
                                                    .iter()
                                                    .enumerate()
                                                    .map(|(token_index, token)| {
                                                        let token_id = format!(
                                                            "tau-ops-chat-token-{index}-{token_index}"
                                                        );
                                                        let token_index_attr = token_index.to_string();
                                                        view! {
                                                            <li
                                                                id=token_id
                                                                data-token-index=token_index_attr
                                                                data-token-value=token.clone()
                                                            >
                                                                {token.clone()}
                                                            </li>
                                                        }
                                                    })
                                                    .collect_view()
                                            };
                                            let markdown_contract =
                                                contains_markdown_contract_syntax(&message_row.content);
                                            let code_block_contract =
                                                extract_first_fenced_code_block(&message_row.content);
                                            if markdown_contract || code_block_contract.is_some() {
                                                let token_count_row_attr =
                                                    assistant_token_count.clone();
                                                let token_count_stream_attr =
                                                    assistant_token_count.clone();
                                                let content_view = if markdown_contract {
                                                    let markdown_card_id =
                                                        format!("tau-ops-chat-markdown-{index}");
                                                    view! {
                                                        <article
                                                            id=markdown_card_id
                                                            data-markdown-rendered="true"
                                                        >
                                                            {message_row.content.clone()}
                                                        </article>
                                                    }
                                                    .into_any()
                                                } else {
                                                    view! { {message_row.content.clone()} }.into_any()
                                                };
                                                let code_view = code_block_contract.map(
                                                    |(language, code)| {
                                                        let code_block_id =
                                                            format!("tau-ops-chat-code-block-{index}");
                                                        let code_attribute = code.clone();
                                                        view! {
                                                            <pre
                                                                id=code_block_id
                                                                data-code-block="true"
                                                                data-language=language.clone()
                                                                data-code=code_attribute
                                                            >
                                                                {code}
                                                            </pre>
                                                        }
                                                    },
                                                );
                                                view! {
                                                    <li
                                                        id=row_id
                                                        data-message-role=message_row.role.clone()
                                                        data-assistant-token-stream="true"
                                                        data-token-count=token_count_row_attr
                                                    >
                                                        {content_view}
                                                        {code_view}
                                                        <ol
                                                            id=assistant_token_stream_id.clone()
                                                            data-token-stream="assistant"
                                                            data-token-count=token_count_stream_attr
                                                        >
                                                            {render_assistant_tokens()}
                                                        </ol>
                                                    </li>
                                                }
                                                .into_any()
                                            } else {
                                                let token_count_row_attr =
                                                    assistant_token_count.clone();
                                                let token_count_stream_attr =
                                                    assistant_token_count.clone();
                                                view! {
                                                    <li
                                                        id=row_id
                                                        data-message-role=message_row.role.clone()
                                                        data-assistant-token-stream="true"
                                                        data-token-count=token_count_row_attr
                                                    >
                                                        {message_row.content.clone()}
                                                        <ol
                                                            id=assistant_token_stream_id
                                                            data-token-stream="assistant"
                                                            data-token-count=token_count_stream_attr
                                                        >
                                                            {render_assistant_tokens()}
                                                        </ol>
                                                    </li>
                                                }
                                                .into_any()
                                            }
                                        } else {
                                            view! {
                                                <li id=row_id data-message-role=message_row.role.clone()>
                                                    {message_row.content.clone()}
                                                </li>
                                            }
                                            .into_any()
                                        }
                                    })
                                    .collect_view()}
                            </ul>
                            <article
                                id="tau-ops-chat-token-counter"
                                data-session-key=chat_session_key.clone()
                                data-input-tokens=session_detail_usage_input_tokens.clone()
                                data-output-tokens=session_detail_usage_output_tokens.clone()
                                data-total-tokens=session_detail_usage_total_tokens.clone()
                            >
                                Token Counter
                            </article>
                        </section>
                        <section
                            id="tau-ops-sessions-panel"
                            data-route="/ops/sessions"
                            aria-hidden=sessions_panel_hidden
                            data-panel-visible=sessions_panel_visible
                        >
                            <h2>Sessions Explorer</h2>
                            <ul id="tau-ops-sessions-list" data-session-count=sessions_row_count_value>
                                {sessions_rows_view}
                            </ul>
                        </section>
                        <section
                            id="tau-ops-session-detail-panel"
                            data-route=session_detail_route
                            data-session-key=chat_session_key.clone()
                            aria-hidden=session_detail_panel_hidden
                        >
                            <h2>Session Detail</h2>
                            <form
                                id="tau-ops-session-reset-form"
                                action=session_reset_form_action
                                method="post"
                                data-session-key=chat_session_key.clone()
                                data-confirmation-required="true"
                            >
                                <input
                                    id="tau-ops-session-reset-session-key"
                                    type="hidden"
                                    name="session_key"
                                    value=chat_session_key.clone()
                                />
                                <input id="tau-ops-session-reset-theme" type="hidden" name="theme" value=theme_attr />
                                <input
                                    id="tau-ops-session-reset-sidebar"
                                    type="hidden"
                                    name="sidebar"
                                    value=sidebar_state_attr
                                />
                                <input
                                    id="tau-ops-session-reset-confirm"
                                    type="hidden"
                                    name="confirm_reset"
                                    value="true"
                                />
                                <button
                                    id="tau-ops-session-reset-submit"
                                    type="submit"
                                    data-confirmation-required="true"
                                >
                                    Reset Session
                                </button>
                            </form>
                            <article
                                id="tau-ops-session-validation-report"
                                data-entries=session_detail_validation_entries
                                data-duplicates=session_detail_validation_duplicates
                                data-invalid-parent=session_detail_validation_invalid_parent
                                data-cycles=session_detail_validation_cycles
                                data-is-valid=session_detail_validation_is_valid
                            >
                                Validation Summary
                            </article>
                            <article
                                id="tau-ops-session-usage-summary"
                                data-input-tokens=session_detail_usage_input_tokens
                                data-output-tokens=session_detail_usage_output_tokens
                                data-total-tokens=session_detail_usage_total_tokens
                                data-estimated-cost-usd=session_detail_usage_estimated_cost_usd
                            >
                                Usage Summary
                            </article>
                            <ul
                                id="tau-ops-session-message-timeline"
                                data-entry-count=session_detail_timeline_count
                            >
                                {session_detail_timeline_view}
                            </ul>
                        </section>
                        <section
                            id="tau-ops-session-graph-panel"
                            data-route=session_graph_route
                            data-session-key=chat_session_key.clone()
                            aria-hidden=session_detail_panel_hidden
                        >
                            <h2>Session Graph</h2>
                            <ul id="tau-ops-session-graph-nodes" data-node-count=session_graph_node_count>
                                {session_graph_view}
                            </ul>
                            <ul id="tau-ops-session-graph-edges" data-edge-count=session_graph_edge_count>
                                {session_graph_edges_view}
                            </ul>
                        </section>
                        <section
                            id="tau-ops-memory-panel"
                            data-route="/ops/memory"
                            aria-hidden=memory_panel_hidden
                            data-panel-visible=memory_panel_visible
                            data-query=memory_query_panel_attr
                            data-result-count=memory_result_count_panel_attr
                            data-workspace-id=memory_workspace_id_panel_attr
                            data-channel-id=memory_channel_id_panel_attr
                            data-actor-id=memory_actor_id_panel_attr
                            data-memory-type=memory_type_panel_attr
                            data-create-status=memory_create_status_panel_attr
                            data-created-memory-id=memory_create_created_entry_id_panel_attr
                            data-edit-status=memory_edit_status_panel_attr
                            data-edited-memory-id=memory_edit_edited_memory_id_panel_attr
                            data-delete-status=memory_delete_status_panel_attr
                            data-deleted-memory-id=memory_delete_deleted_entry_id_panel_attr
                        >
                            <h2>Memory Explorer</h2>
                            <article
                                id="tau-ops-memory-scope-summary"
                                data-session-key=chat_session_key.clone()
                                data-result-count=memory_result_count_value.clone()
                                data-query=memory_query_panel_attr.clone()
                                data-workspace-id=memory_workspace_id_panel_attr.clone()
                                data-channel-id=memory_channel_id_panel_attr.clone()
                                data-actor-id=memory_actor_id_panel_attr.clone()
                                data-memory-type=memory_type_panel_attr.clone()
                                data-create-status=memory_create_status_panel_attr.clone()
                                data-created-memory-id=memory_create_created_entry_id_panel_attr.clone()
                            >
                                <h3>Memory Scope</h3>
                                <dl>
                                    <div>
                                        <dt>Session</dt>
                                        <dd>{chat_session_key.clone()}</dd>
                                    </div>
                                    <div>
                                        <dt>Results</dt>
                                        <dd>{memory_result_count_value.clone()}</dd>
                                    </div>
                                    <div>
                                        <dt>Query</dt>
                                        <dd>{memory_scope_query_label}</dd>
                                    </div>
                                    <div>
                                        <dt>Workspace</dt>
                                        <dd>{memory_scope_workspace_label}</dd>
                                    </div>
                                    <div>
                                        <dt>Type</dt>
                                        <dd>{memory_scope_type_label}</dd>
                                    </div>
                                    <div>
                                        <dt>Write Status</dt>
                                        <dd>{memory_create_status_panel_attr.clone()}</dd>
                                    </div>
                                </dl>
                                <nav
                                    id="tau-ops-memory-scope-actions"
                                    aria-label="Memory scope actions"
                                >
                                    <a id="tau-ops-memory-open-graph" href=memory_scope_graph_href>
                                        Open Memory Graph
                                    </a>
                                    <a id="tau-ops-memory-open-session" href=memory_scope_session_href>
                                        Open Session
                                    </a>
                                </nav>
                            </article>
                            <form
                                id="tau-ops-memory-search-form"
                                action=memory_search_form_action
                                method=memory_search_form_method
                            >
                                <input id="tau-ops-memory-theme" type="hidden" name="theme" value=theme_attr />
                                <input
                                    id="tau-ops-memory-sidebar"
                                    type="hidden"
                                    name="sidebar"
                                    value=sidebar_state_attr
                                />
                                <input
                                    id="tau-ops-memory-session"
                                    type="hidden"
                                    name="session"
                                    value=chat_session_key.clone()
                                />
                                <label for="tau-ops-memory-query">Search Memory</label>
                                <input
                                    id="tau-ops-memory-query"
                                    type="search"
                                    name="query"
                                    value=memory_query_input_value
                                />
                                <label for="tau-ops-memory-workspace-filter">Workspace</label>
                                <input
                                    id="tau-ops-memory-workspace-filter"
                                    type="text"
                                    name="workspace_id"
                                    value=memory_workspace_id_input_value
                                />
                                <label for="tau-ops-memory-channel-filter">Channel</label>
                                <input
                                    id="tau-ops-memory-channel-filter"
                                    type="text"
                                    name="channel_id"
                                    value=memory_channel_id_input_value
                                />
                                <label for="tau-ops-memory-actor-filter">Actor</label>
                                <input
                                    id="tau-ops-memory-actor-filter"
                                    type="text"
                                    name="actor_id"
                                    value=memory_actor_id_input_value
                                />
                                <label for="tau-ops-memory-type-filter">Memory Type</label>
                                <input
                                    id="tau-ops-memory-type-filter"
                                    type="text"
                                    name="memory_type"
                                    value=memory_type_input_value
                                />
                                <button id="tau-ops-memory-search-button" type="submit">
                                    Search
                                </button>
                            </form>
                            <p
                                id="tau-ops-memory-create-status"
                                data-create-status=memory_create_status_marker_attr
                                data-created-memory-id=memory_create_created_entry_id_marker_attr
                            >
                                {memory_create_status_message}
                            </p>
                            <form
                                id="tau-ops-memory-create-form"
                                action=memory_create_form_action
                                method=memory_create_form_method
                            >
                                <input id="tau-ops-memory-create-theme" type="hidden" name="theme" value=theme_attr />
                                <input
                                    id="tau-ops-memory-create-sidebar"
                                    type="hidden"
                                    name="sidebar"
                                    value=sidebar_state_attr
                                />
                                <input
                                    id="tau-ops-memory-create-session"
                                    type="hidden"
                                    name="session"
                                    value=chat_session_key.clone()
                                />
                                <input
                                    id="tau-ops-memory-create-operation"
                                    type="hidden"
                                    name="operation"
                                    value="create"
                                />
                                <label for="tau-ops-memory-create-entry-id">Entry ID</label>
                                <input
                                    id="tau-ops-memory-create-entry-id"
                                    type="text"
                                    name="entry_id"
                                    value=memory_create_entry_id
                                />
                                <label for="tau-ops-memory-create-summary">Summary</label>
                                <input
                                    id="tau-ops-memory-create-summary"
                                    type="text"
                                    name="summary"
                                    value=memory_create_summary
                                />
                                <label for="tau-ops-memory-create-tags">Tags</label>
                                <input
                                    id="tau-ops-memory-create-tags"
                                    type="text"
                                    name="tags"
                                    value=memory_create_tags
                                />
                                <label for="tau-ops-memory-create-facts">Facts</label>
                                <input
                                    id="tau-ops-memory-create-facts"
                                    type="text"
                                    name="facts"
                                    value=memory_create_facts
                                />
                                <label for="tau-ops-memory-create-source-event-key">Source Event Key</label>
                                <input
                                    id="tau-ops-memory-create-source-event-key"
                                    type="text"
                                    name="source_event_key"
                                    value=memory_create_source_event_key
                                />
                                <label for="tau-ops-memory-create-workspace-id">Workspace</label>
                                <input
                                    id="tau-ops-memory-create-workspace-id"
                                    type="text"
                                    name="workspace_id"
                                    value=memory_create_workspace_id
                                />
                                <label for="tau-ops-memory-create-channel-id">Channel</label>
                                <input
                                    id="tau-ops-memory-create-channel-id"
                                    type="text"
                                    name="channel_id"
                                    value=memory_create_channel_id
                                />
                                <label for="tau-ops-memory-create-actor-id">Actor</label>
                                <input
                                    id="tau-ops-memory-create-actor-id"
                                    type="text"
                                    name="actor_id"
                                    value=memory_create_actor_id
                                />
                                <label for="tau-ops-memory-create-memory-type">Memory Type</label>
                                <input
                                    id="tau-ops-memory-create-memory-type"
                                    type="text"
                                    name="memory_type"
                                    value=memory_create_memory_type
                                />
                                <label for="tau-ops-memory-create-importance">Importance</label>
                                <input
                                    id="tau-ops-memory-create-importance"
                                    type="number"
                                    step="0.01"
                                    name="importance"
                                    value=memory_create_importance
                                />
                                <label for="tau-ops-memory-create-relation-target-id">Relation Target</label>
                                <input
                                    id="tau-ops-memory-create-relation-target-id"
                                    type="text"
                                    name="relation_target_id"
                                    value=memory_create_relation_target_id
                                />
                                <label for="tau-ops-memory-create-relation-type">Relation Type</label>
                                <input
                                    id="tau-ops-memory-create-relation-type"
                                    type="text"
                                    name="relation_type"
                                    value=memory_create_relation_type
                                />
                                <label for="tau-ops-memory-create-relation-weight">Relation Weight</label>
                                <input
                                    id="tau-ops-memory-create-relation-weight"
                                    type="number"
                                    step="0.01"
                                    name="relation_weight"
                                    value=memory_create_relation_weight
                                />
                                <button id="tau-ops-memory-create-button" type="submit">
                                    Create Entry
                                </button>
                            </form>
                            <p
                                id="tau-ops-memory-edit-status"
                                data-edit-status=memory_edit_status_marker_attr
                                data-edited-memory-id=memory_edit_edited_memory_id_marker_attr
                            >
                                {memory_edit_status_message}
                            </p>
                            <form
                                id="tau-ops-memory-edit-form"
                                action=memory_edit_form_action
                                method=memory_edit_form_method
                            >
                                <input id="tau-ops-memory-edit-theme" type="hidden" name="theme" value=theme_attr />
                                <input
                                    id="tau-ops-memory-edit-sidebar"
                                    type="hidden"
                                    name="sidebar"
                                    value=sidebar_state_attr
                                />
                                <input
                                    id="tau-ops-memory-edit-session"
                                    type="hidden"
                                    name="session"
                                    value=chat_session_key.clone()
                                />
                                <input
                                    id="tau-ops-memory-edit-operation"
                                    type="hidden"
                                    name="operation"
                                    value="edit"
                                />
                                <label for="tau-ops-memory-edit-entry-id">Entry ID</label>
                                <input
                                    id="tau-ops-memory-edit-entry-id"
                                    type="text"
                                    name="entry_id"
                                    value=memory_edit_entry_id
                                />
                                <label for="tau-ops-memory-edit-summary">Summary</label>
                                <input
                                    id="tau-ops-memory-edit-summary"
                                    type="text"
                                    name="summary"
                                    value=memory_edit_summary
                                />
                                <label for="tau-ops-memory-edit-tags">Tags</label>
                                <input
                                    id="tau-ops-memory-edit-tags"
                                    type="text"
                                    name="tags"
                                    value=memory_edit_tags
                                />
                                <label for="tau-ops-memory-edit-facts">Facts</label>
                                <input
                                    id="tau-ops-memory-edit-facts"
                                    type="text"
                                    name="facts"
                                    value=memory_edit_facts
                                />
                                <label for="tau-ops-memory-edit-source-event-key">Source Event Key</label>
                                <input
                                    id="tau-ops-memory-edit-source-event-key"
                                    type="text"
                                    name="source_event_key"
                                    value=memory_edit_source_event_key
                                />
                                <label for="tau-ops-memory-edit-workspace-id">Workspace</label>
                                <input
                                    id="tau-ops-memory-edit-workspace-id"
                                    type="text"
                                    name="workspace_id"
                                    value=memory_edit_workspace_id
                                />
                                <label for="tau-ops-memory-edit-channel-id">Channel</label>
                                <input
                                    id="tau-ops-memory-edit-channel-id"
                                    type="text"
                                    name="channel_id"
                                    value=memory_edit_channel_id
                                />
                                <label for="tau-ops-memory-edit-actor-id">Actor</label>
                                <input
                                    id="tau-ops-memory-edit-actor-id"
                                    type="text"
                                    name="actor_id"
                                    value=memory_edit_actor_id
                                />
                                <label for="tau-ops-memory-edit-memory-type">Memory Type</label>
                                <input
                                    id="tau-ops-memory-edit-memory-type"
                                    type="text"
                                    name="memory_type"
                                    value=memory_edit_memory_type
                                />
                                <label for="tau-ops-memory-edit-importance">Importance</label>
                                <input
                                    id="tau-ops-memory-edit-importance"
                                    type="number"
                                    step="0.01"
                                    name="importance"
                                    value=memory_edit_importance
                                />
                                <label for="tau-ops-memory-edit-relation-target-id">Relation Target</label>
                                <input
                                    id="tau-ops-memory-edit-relation-target-id"
                                    type="text"
                                    name="relation_target_id"
                                    value=memory_edit_relation_target_id
                                />
                                <label for="tau-ops-memory-edit-relation-type">Relation Type</label>
                                <input
                                    id="tau-ops-memory-edit-relation-type"
                                    type="text"
                                    name="relation_type"
                                    value=memory_edit_relation_type
                                />
                                <label for="tau-ops-memory-edit-relation-weight">Relation Weight</label>
                                <input
                                    id="tau-ops-memory-edit-relation-weight"
                                    type="number"
                                    step="0.01"
                                    name="relation_weight"
                                    value=memory_edit_relation_weight
                                />
                                <button id="tau-ops-memory-edit-button" type="submit">
                                    Update Entry
                                </button>
                            </form>
                            <p
                                id="tau-ops-memory-delete-status"
                                data-delete-status=memory_delete_status_marker_attr
                                data-deleted-memory-id=memory_delete_deleted_entry_id_marker_attr
                            >
                                {memory_delete_status_message}
                            </p>
                            <form
                                id="tau-ops-memory-delete-form"
                                action=memory_delete_form_action
                                method=memory_delete_form_method
                            >
                                <input id="tau-ops-memory-delete-theme" type="hidden" name="theme" value=theme_attr />
                                <input
                                    id="tau-ops-memory-delete-sidebar"
                                    type="hidden"
                                    name="sidebar"
                                    value=sidebar_state_attr
                                />
                                <input
                                    id="tau-ops-memory-delete-session"
                                    type="hidden"
                                    name="session"
                                    value=chat_session_key.clone()
                                />
                                <input
                                    id="tau-ops-memory-delete-operation"
                                    type="hidden"
                                    name="operation"
                                    value="delete"
                                />
                                <label for="tau-ops-memory-delete-entry-id">Entry ID</label>
                                <input
                                    id="tau-ops-memory-delete-entry-id"
                                    type="text"
                                    name="entry_id"
                                    value=memory_delete_entry_id
                                />
                                <label for="tau-ops-memory-delete-confirm">Confirm Delete</label>
                                <input
                                    id="tau-ops-memory-delete-confirm"
                                    type="checkbox"
                                    name="confirm_delete"
                                    value="true"
                                />
                                <button id="tau-ops-memory-delete-button" type="submit">
                                    Delete Entry
                                </button>
                            </form>
                            <section
                                id="tau-ops-memory-detail-panel"
                                data-detail-visible=memory_detail_visible
                                data-memory-id=memory_detail_selected_entry_id.clone()
                                data-memory-type=memory_detail_memory_type.clone()
                                data-embedding-source=memory_detail_embedding_source_panel_attr
                                data-embedding-model=memory_detail_embedding_model_panel_attr
                                data-embedding-reason-code=memory_detail_embedding_reason_code_panel_attr
                                data-embedding-dimensions=memory_detail_embedding_dimensions_panel_attr
                                data-relation-count=memory_detail_relation_count_panel_attr
                            >
                                <p
                                    id="tau-ops-memory-detail-embedding"
                                    data-embedding-source=memory_detail_embedding_source
                                    data-embedding-model=memory_detail_embedding_model
                                    data-embedding-reason-code=memory_detail_embedding_reason_code
                                    data-embedding-dimensions=memory_detail_embedding_dimensions
                                >
                                    {memory_detail_summary.clone()}
                                </p>
                                <ul id="tau-ops-memory-relations" data-relation-count=memory_detail_relation_count>
                                    {memory_detail_relations_view}
                                </ul>
                            </section>
                            <ul id="tau-ops-memory-results" data-result-count=memory_result_count_list_attr>
                                {memory_results_view}
                            </ul>
                        </section>
                        <section
                            id="tau-ops-memory-graph-panel"
                            data-route="/ops/memory-graph"
                            aria-hidden=memory_graph_panel_hidden
                            data-panel-visible=memory_graph_panel_visible
                            data-node-count=memory_graph_node_count_panel_attr
                            data-edge-count=memory_graph_edge_count_panel_attr
                        >
                            <h2>Memory Graph</h2>
                            {memory_graph_scope_summary_view}
                            <ul id="tau-ops-memory-graph-nodes" data-node-count=memory_graph_node_count>
                                {memory_graph_nodes_view}
                            </ul>
                            <ul id="tau-ops-memory-graph-edges" data-edge-count=memory_graph_edge_count>
                                {memory_graph_edges_view}
                            </ul>
                            <div
                                id="tau-ops-memory-graph-zoom-controls"
                                data-zoom-level=memory_graph_zoom_level
                                data-zoom-min=memory_graph_zoom_min
                                data-zoom-max=memory_graph_zoom_max
                                data-zoom-step=memory_graph_zoom_step
                            >
                                <a
                                    id="tau-ops-memory-graph-zoom-in"
                                    data-zoom-action="in"
                                    href=memory_graph_zoom_in_href
                                >
                                    Zoom +
                                </a>
                                <a
                                    id="tau-ops-memory-graph-zoom-out"
                                    data-zoom-action="out"
                                    href=memory_graph_zoom_out_href
                                >
                                    Zoom -
                                </a>
                            </div>
                            <div
                                id="tau-ops-memory-graph-pan-controls"
                                data-pan-x=memory_graph_pan_x_level
                                data-pan-y=memory_graph_pan_y_level
                                data-pan-step=memory_graph_pan_step
                            >
                                <a
                                    id="tau-ops-memory-graph-pan-left"
                                    data-pan-action="left"
                                    href=memory_graph_pan_left_href
                                >
                                    Pan Left
                                </a>
                                <a
                                    id="tau-ops-memory-graph-pan-right"
                                    data-pan-action="right"
                                    href=memory_graph_pan_right_href
                                >
                                    Pan Right
                                </a>
                                <a
                                    id="tau-ops-memory-graph-pan-up"
                                    data-pan-action="up"
                                    href=memory_graph_pan_up_href
                                >
                                    Pan Up
                                </a>
                                <a
                                    id="tau-ops-memory-graph-pan-down"
                                    data-pan-action="down"
                                    href=memory_graph_pan_down_href
                                >
                                    Pan Down
                                </a>
                            </div>
                            <div
                                id="tau-ops-memory-graph-filter-controls"
                                data-filter-memory-type=memory_graph_filter_memory_type
                                data-filter-relation-type=memory_graph_filter_relation_type
                            >
                                <a
                                    id="tau-ops-memory-graph-filter-memory-type-all"
                                    data-filter-target="memory-type"
                                    href=memory_graph_filter_memory_type_all_href
                                >
                                    Memory Type: All
                                </a>
                                <a
                                    id="tau-ops-memory-graph-filter-memory-type-goal"
                                    data-filter-target="memory-type"
                                    href=memory_graph_filter_memory_type_goal_href
                                >
                                    Memory Type: Goal
                                </a>
                                <a
                                    id="tau-ops-memory-graph-filter-relation-type-all"
                                    data-filter-target="relation-type"
                                    href=memory_graph_filter_relation_type_all_href
                                >
                                    Relation: All
                                </a>
                                <a
                                    id="tau-ops-memory-graph-filter-relation-type-related-to"
                                    data-filter-target="relation-type"
                                    href=memory_graph_filter_relation_type_related_to_href
                                >
                                    Relation: related_to
                                </a>
                            </div>
                            <section
                                id="tau-ops-memory-graph-detail-panel"
                                data-detail-visible=memory_graph_detail_visible
                                data-memory-id=memory_graph_detail_selected_entry_id.clone()
                                data-memory-type=memory_graph_detail_memory_type
                                data-relation-count=memory_graph_detail_relation_count_panel_attr
                            >
                                <p
                                    id="tau-ops-memory-graph-detail-summary"
                                    data-memory-id=memory_graph_detail_selected_entry_id.clone()
                                >
                                    {memory_graph_detail_summary}
                                </p>
                                <a
                                    id="tau-ops-memory-graph-detail-open-memory"
                                    href=memory_graph_detail_open_memory_href
                                    data-detail-memory-id=memory_graph_detail_selected_entry_id.clone()
                                >
                                    Open in Memory Explorer
                                </a>
                            </section>
                        </section>
                        <section
                            id="tau-ops-tools-panel"
                            data-route="/ops/tools-jobs"
                            aria-hidden=tools_panel_hidden
                            data-panel-visible=tools_panel_visible
                            data-total-tools=tools_total_count_panel_attr
                        >
                            <h2>Tools & Jobs</h2>
                            <p
                                id="tau-ops-tools-inventory-summary"
                                data-total-tools=tools_total_count_summary_attr
                            >
                                Registered tools visible in the current runtime.
                            </p>
                            <table
                                id="tau-ops-tools-inventory-table"
                                data-row-count=tools_row_count_table_attr
                                data-column-count="7"
                            >
                                <thead>
                                    <tr>
                                        <th scope="col">Tool Name</th>
                                        <th scope="col">Category</th>
                                        <th scope="col">Policy</th>
                                        <th scope="col">Usage Count</th>
                                        <th scope="col">Error Rate</th>
                                        <th scope="col">Avg Latency (ms)</th>
                                        <th scope="col">Last Used (unix ms)</th>
                                    </tr>
                                </thead>
                                <tbody
                                    id="tau-ops-tools-inventory-body"
                                    data-row-count=tools_row_count_body_attr
                                >
                                    {tools_inventory_rows_view}
                                </tbody>
                            </table>
                            <section
                                id="tau-ops-tool-detail-panel"
                                data-selected-tool=tool_detail_selected_tool_name.clone()
                                data-detail-visible=tool_detail_visible
                            >
                                <section
                                    id="tau-ops-tool-detail-metadata"
                                    data-tool-name=tool_detail_selected_tool_name.clone()
                                    data-parameter-schema=tool_detail_parameter_schema.clone()
                                >
                                    <p id="tau-ops-tool-detail-description">
                                        {tool_detail_description}
                                    </p>
                                </section>
                                <section
                                    id="tau-ops-tool-detail-policy"
                                    data-timeout-ms=tool_detail_policy_timeout_ms
                                    data-max-output-chars=tool_detail_policy_max_output_chars
                                    data-sandbox-mode=tool_detail_policy_sandbox_mode
                                >
                                    <h3>Policy</h3>
                                </section>
                                <section
                                    id="tau-ops-tool-detail-usage-histogram"
                                    data-bucket-count=tool_detail_usage_bucket_count
                                >
                                    <h3>Usage (24h)</h3>
                                    <ul>{tool_detail_usage_histogram_view}</ul>
                                </section>
                                <section
                                    id="tau-ops-tool-detail-invocations"
                                    data-row-count=tool_detail_invocation_count
                                >
                                    <h3>Recent Invocations</h3>
                                    <table>
                                        <thead>
                                            <tr>
                                                <th scope="col">Timestamp</th>
                                                <th scope="col">Args</th>
                                                <th scope="col">Result</th>
                                                <th scope="col">Duration (ms)</th>
                                                <th scope="col">Status</th>
                                            </tr>
                                        </thead>
                                        <tbody>{tool_detail_invocations_view}</tbody>
                                    </table>
                                </section>
                            </section>
                            <section
                                id="tau-ops-jobs-panel"
                                data-panel-visible=jobs_panel_visible
                                data-total-jobs=jobs_total_count_panel_attr
                            >
                                <h3>Jobs</h3>
                                <p
                                    id="tau-ops-jobs-summary"
                                    data-running-count=jobs_running_count
                                    data-completed-count=jobs_completed_count
                                    data-failed-count=jobs_failed_count
                                >
                                    Running/completed/failed job counts.
                                </p>
                                <table id="tau-ops-jobs-table" data-row-count=jobs_row_count_table_attr>
                                    <thead>
                                        <tr>
                                            <th scope="col">Job ID</th>
                                            <th scope="col">Job Name</th>
                                            <th scope="col">Status</th>
                                            <th scope="col">Started (unix ms)</th>
                                            <th scope="col">Finished (unix ms)</th>
                                            <th scope="col">Actions</th>
                                        </tr>
                                    </thead>
                                    <tbody id="tau-ops-jobs-body" data-row-count=jobs_row_count_body_attr>
                                        {jobs_rows_view}
                                    </tbody>
                                </table>
                            </section>
                            <section
                                id="tau-ops-job-detail-panel"
                                data-selected-job-id=job_detail_selected_job_id.clone()
                                data-detail-visible=job_detail_visible
                            >
                                <section
                                    id="tau-ops-job-detail-metadata"
                                    data-job-id=job_detail_selected_job_id.clone()
                                    data-job-status=job_detail_status.clone()
                                    data-duration-ms=job_detail_duration_ms
                                >
                                    <h4>Selected Job Output</h4>
                                </section>
                                <pre
                                    id="tau-ops-job-detail-stdout"
                                    data-output-bytes=job_detail_stdout_bytes
                                >
                                    {job_detail_stdout}
                                </pre>
                                <pre
                                    id="tau-ops-job-detail-stderr"
                                    data-output-bytes=job_detail_stderr_bytes
                                >
                                    {job_detail_stderr}
                                </pre>
                            </section>
                            <section
                                id="tau-ops-job-cancel-panel"
                                data-requested-job-id=job_detail_selected_job_id.clone()
                                data-cancel-status=job_cancel_status
                                data-panel-visible=job_cancel_panel_visible
                                data-cancel-endpoint-template="/gateway/jobs/{job_id}/cancel"
                            >
                                <a
                                    id="tau-ops-job-cancel-submit"
                                    data-action="cancel-job"
                                    data-job-id=job_detail_selected_job_id.clone()
                                    data-cancel-enabled=job_cancel_enabled
                                    href=job_cancel_submit_href
                                >
                                    Cancel Selected Job
                                </a>
                            </section>
                        </section>
                        <section
                            id="tau-ops-channels-panel"
                            data-route="/ops/channels"
                            aria-hidden=channels_panel_hidden
                            data-panel-visible=channels_panel_visible
                            data-channel-count=channels_row_count_panel_value
                            data-visual-contract="channel-operator-console"
                        >
                            <section id="tau-ops-channels-header" data-layout="summary-with-kpis">
                                <h2>Channels</h2>
                                <p
                                    id="tau-ops-channels-summary"
                                    data-online-count=channels_online_count_summary
                                    data-offline-count=channels_offline_count_summary
                                    data-degraded-count=channels_degraded_count_summary
                                >
                                    Connector health, liveness, and operator action state for configured channels.
                                </p>
                            </section>
                            <section
                                id="tau-ops-channels-action-status"
                                data-channel-action-status=channel_action_status
                                data-channel-action=channel_action
                                data-channel-action-channel=channel_action_channel
                                data-channel-action-reason=channel_action_reason
                            >
                                <h3>Lifecycle Action</h3>
                                <p>{channel_action_status_message}</p>
                            </section>
                            <section id="tau-ops-channels-kpi-grid" data-card-count="3" aria-label="Channel health summary">
                                <article id="tau-ops-channels-online-card" data-kpi="online" data-count=channels_online_count_card>
                                    <h3>Online</h3>
                                    <p>{channels_online_count}</p>
                                </article>
                                <article id="tau-ops-channels-offline-card" data-kpi="offline" data-count=channels_offline_count_card>
                                    <h3>Offline</h3>
                                    <p>{channels_offline_count}</p>
                                </article>
                                <article id="tau-ops-channels-degraded-card" data-kpi="degraded" data-count=channels_degraded_count_card>
                                    <h3>Degraded</h3>
                                    <p>{channels_degraded_count}</p>
                                </article>
                            </section>
                            <section id="tau-ops-channels-table-section" data-layout="connector-action-matrix">
                                <h3>Connector Matrix</h3>
                                <div id="tau-ops-channels-table-wrap" class="tau-ops-table-wrap" data-horizontal-overflow="contained">
                                    <table id="tau-ops-channels-table" data-row-count=channels_row_count_table_value>
                                        <thead>
                                            <tr>
                                                <th scope="col">Channel</th>
                                                <th scope="col">Mode</th>
                                                <th scope="col">Liveness</th>
                                                <th scope="col">Events Ingested</th>
                                                <th scope="col">Provider Failures</th>
                                                <th scope="col" data-column="actions">Actions</th>
                                            </tr>
                                        </thead>
                                        <tbody
                                            id="tau-ops-channels-body"
                                            data-row-count=channels_row_count_body_value
                                        >
                                            {channels_rows_view}
                                        </tbody>
                                    </table>
                                </div>
                            </section>
                        </section>
                        <section
                            id="tau-ops-config-panel"
                            data-route="/ops/config"
                            aria-hidden=config_panel_hidden
                            data-panel-visible=config_panel_visible
                        >
                            <h2>Configuration</h2>
                            <p>Gateway runtime configuration profile and policy contracts.</p>
                            <section
                                id="tau-ops-config-endpoints"
                                data-config-get-endpoint="/gateway/config"
                                data-config-patch-endpoint="/gateway/config"
                            >
                                <h3>Config Endpoints</h3>
                            </section>
                            <section
                                id="tau-ops-config-profile-controls"
                                data-model-ref="gpt-4.1-mini"
                                data-fallback-model-count="2"
                                data-system-prompt-chars="0"
                                data-max-turns="64"
                            >
                                <h3>Profile</h3>
                                <label for="tau-ops-config-profile-model-ref">Model</label>
                                <select
                                    id="tau-ops-config-profile-model-ref"
                                    name="model_ref"
                                    data-control="select"
                                >
                                    <option value="gpt-4.1-mini">gpt-4.1-mini</option>
                                    <option value="gpt-4.1">gpt-4.1</option>
                                </select>
                                <section
                                    id="tau-ops-config-profile-fallback-models"
                                    data-control="ordered-list"
                                >
                                    <h4>Fallback Models</h4>
                                    <ol>
                                        <li data-model-ref="gpt-4.1">gpt-4.1</li>
                                        <li data-model-ref="gpt-5.2">gpt-5.2</li>
                                    </ol>
                                </section>
                                <label for="tau-ops-config-profile-system-prompt">System Prompt</label>
                                <textarea
                                    id="tau-ops-config-profile-system-prompt"
                                    name="system_prompt"
                                    data-control="textarea"
                                ></textarea>
                                <label for="tau-ops-config-profile-max-turns">Max Turns</label>
                                <input
                                    id="tau-ops-config-profile-max-turns"
                                    name="max_turns"
                                    data-control="number"
                                    type="number"
                                    value="64"
                                />
                            </section>
                            <section
                                id="tau-ops-config-policy-controls"
                                data-tool-policy-preset="balanced"
                                data-bash-profile="balanced"
                                data-os-sandbox-mode="auto"
                            >
                                <h3>Policy</h3>
                                <section
                                    id="tau-ops-config-policy-limits"
                                    data-bash-timeout-ms="120000"
                                    data-max-command-length="8192"
                                    data-max-tool-output-bytes="32768"
                                    data-max-file-read-bytes="262144"
                                    data-max-file-write-bytes="262144"
                                >
                                    <h4>Limits</h4>
                                </section>
                                <section
                                    id="tau-ops-config-policy-heartbeat"
                                    data-runtime-heartbeat-enabled="true"
                                    data-runtime-heartbeat-interval-ms="5000"
                                    data-runtime-self-repair-enabled="true"
                                >
                                    <h4>Heartbeat</h4>
                                </section>
                                <section
                                    id="tau-ops-config-policy-compaction"
                                    data-warn-threshold="70"
                                    data-aggressive-threshold="85"
                                    data-emergency-threshold="95"
                                >
                                    <h4>Compaction Thresholds</h4>
                                </section>
                            </section>
                        </section>
                        <section
                            id="tau-ops-training-panel"
                            data-route="/ops/training"
                            aria-hidden=training_panel_hidden
                            data-panel-visible=training_panel_visible
                        >
                            <h2>Training & RL</h2>
                            <p>Training status, rollout history, optimizer, and controls.</p>
                            <section
                                id="tau-ops-training-status"
                                data-status="running"
                                data-gate=context.command_center.rollout_gate.clone()
                                data-store-path=".tau/training/rl.sqlite"
                                data-update-interval-rollouts="8"
                                data-max-rollouts-per-update="64"
                                data-failure-streak="0/3"
                            >
                                <h3>Status</h3>
                            </section>
                            <section
                                id="tau-ops-training-rollouts"
                                data-rollout-count="3"
                                data-last-rollout-id="142"
                            >
                                <h3>Rollout History</h3>
                                <ol>
                                    <li data-rollout-id="142" data-steps="12" data-reward="+0.8" data-outcome="completed">#142</li>
                                    <li data-rollout-id="141" data-steps="8" data-reward="+0.5" data-outcome="completed">#141</li>
                                    <li data-rollout-id="140" data-steps="15" data-reward="-0.2" data-outcome="failed">#140</li>
                                </ol>
                            </section>
                            <section
                                id="tau-ops-training-optimizer"
                                data-mean-total-loss="0.023"
                                data-approx-kl="0.0012"
                                data-early-stop="false"
                            >
                                <h3>Optimizer Report</h3>
                            </section>
                            <section
                                id="tau-ops-training-endpoints"
                                data-training-status-endpoint="/gateway/training/status"
                                data-training-rollouts-endpoint="/gateway/training/rollouts"
                                data-training-config-endpoint="/gateway/training/config"
                            >
                                <h3>Training Endpoints</h3>
                            </section>
                            <section
                                id="tau-ops-training-actions"
                                data-pause-endpoint="/gateway/training/config"
                                data-reset-endpoint="/gateway/training/config"
                                data-export-endpoint="/gateway/training/rollouts"
                            >
                                <a
                                    id="tau-ops-training-action-pause"
                                    data-action="pause-training"
                                    data-action-enabled="true"
                                    href="/ops/training?action=pause"
                                >
                                    Pause Training
                                </a>
                                <a
                                    id="tau-ops-training-action-reset"
                                    data-action="reset-store"
                                    data-action-enabled="true"
                                    href="/ops/training?action=reset"
                                >
                                    Reset Store
                                </a>
                                <a
                                    id="tau-ops-training-action-export"
                                    data-action="export-data"
                                    data-action-enabled="true"
                                    href="/ops/training?action=export"
                                >
                                    Export Data
                                </a>
                            </section>
                        </section>
                        <section
                            id="tau-ops-safety-panel"
                            data-route="/ops/safety"
                            aria-hidden=safety_panel_hidden
                            data-panel-visible=safety_panel_visible
                        >
                            <h2>Safety & Security</h2>
                            <p>Safety policy/rules contract endpoints.</p>
                            <section
                                id="tau-ops-safety-endpoints"
                                data-safety-policy-get-endpoint="/gateway/safety/policy"
                                data-safety-policy-put-endpoint="/gateway/safety/policy"
                                data-safety-rules-get-endpoint="/gateway/safety/rules"
                                data-safety-rules-put-endpoint="/gateway/safety/rules"
                                data-safety-test-endpoint="/gateway/safety/test"
                            >
                                <h3>Safety Endpoints</h3>
                            </section>
                        </section>
                        <section
                            id="tau-ops-diagnostics-panel"
                            data-route="/ops/diagnostics"
                            aria-hidden=diagnostics_panel_hidden
                            data-panel-visible=diagnostics_panel_visible
                        >
                            <h2>Diagnostics & Audit</h2>
                            <p>Audit and telemetry contract endpoints.</p>
                            <section
                                id="tau-ops-diagnostics-endpoints"
                                data-audit-summary-endpoint="/gateway/audit/summary"
                                data-audit-log-endpoint="/gateway/audit/log"
                                data-ui-telemetry-endpoint="/gateway/ui/telemetry"
                            >
                                <h3>Diagnostics Endpoints</h3>
                            </section>
                        </section>
                        <style id="tau-ops-harness-template-style">
                            r#"
                            #tau-ops-harness-panel[aria-hidden="true"] {
                                display: none;
                            }
                            #tau-ops-harness-panel {
                                --tau-harness-bg: #071118;
                                --tau-harness-rail: #0b1a23;
                                --tau-harness-panel: #0f202a;
                                --tau-harness-panel-2: #142935;
                                --tau-harness-panel-3: #182f3c;
                                --tau-harness-line: #2f4652;
                                --tau-harness-line-soft: #1e3440;
                                --tau-harness-text: #e3eef3;
                                --tau-harness-muted: #8fa8b3;
                                --tau-harness-dim: #68818d;
                                --tau-harness-blue: #4d8dff;
                                --tau-harness-green: #5fca7a;
                                --tau-harness-yellow: #d8ad45;
                                --tau-harness-red: #df635b;
                                --tau-harness-cyan: #7bd6e8;
                                display: grid;
                                grid-template-columns: minmax(0, .82fr) minmax(0, 1.04fr) minmax(0, .86fr);
                                grid-template-areas:
                                    "topbar topbar topbar"
                                    "dashboard proof review"
                                    "dashboard proof tui";
                                gap: 8px;
                                align-items: start;
                                min-height: calc(100vh - 16px);
                                padding: 10px;
                                border: 1px solid var(--tau-harness-line);
                                border-radius: 6px;
                                background:
                                    linear-gradient(180deg, #091821 0%, var(--tau-harness-bg) 58%, #050c11 100%);
                                color: var(--tau-harness-text);
                                font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
                                font-size: 13px;
                                line-height: 1.35;
                                width: min(100%, calc(100vw - 108px));
                                min-width: 0;
                                max-width: calc(100vw - 108px);
                                overflow: hidden;
                            }
                            #tau-ops-harness-panel h2,
                            #tau-ops-harness-panel h3,
                            #tau-ops-harness-panel h4,
                            #tau-ops-harness-panel h5,
                            #tau-ops-harness-panel p,
                            #tau-ops-harness-panel dd,
                            #tau-ops-harness-panel li {
                                margin: 0;
                                min-width: 0;
                                overflow-wrap: anywhere;
                            }
                            #tau-ops-harness-panel h2,
                            #tau-ops-harness-panel h3 {
                                letter-spacing: 0;
                            }
                            #tau-ops-harness-panel h2 {
                                font-size: 1.05rem;
                                font-weight: 700;
                            }
                            #tau-ops-harness-panel h3 {
                                font-size: .98rem;
                                font-weight: 700;
                            }
                            #tau-ops-harness-panel h4 {
                                color: #f2f8fb;
                                font-size: .82rem;
                                font-weight: 700;
                            }
                            #tau-ops-harness-panel h5 {
                                font-size: .78rem;
                                font-weight: 700;
                            }
                            #tau-ops-harness-topbar,
                            #tau-ops-harness-dashboard-window,
                            #tau-ops-harness-proof-window,
                            #tau-ops-harness-self-improvement-window,
                            #tau-ops-harness-tui-companion {
                                border: 1px solid var(--tau-harness-line);
                                border-radius: 6px;
                                background:
                                    linear-gradient(180deg, rgba(24, 47, 60, .94), rgba(13, 29, 39, .96));
                                box-shadow: 0 16px 34px rgba(0, 0, 0, .34);
                                box-sizing: border-box;
                                min-width: 0;
                                overflow: hidden;
                            }
                            #tau-ops-harness-topbar {
                                grid-area: topbar;
                                display: flex;
                                justify-content: space-between;
                                gap: 12px;
                                align-items: center;
                                min-height: 46px;
                                padding: 9px 12px;
                                background:
                                    linear-gradient(180deg, #132833, #0a1821);
                            }
                            #tau-ops-harness-dashboard-window {
                                grid-area: dashboard;
                            }
                            #tau-ops-harness-proof-window {
                                grid-area: proof;
                            }
                            #tau-ops-harness-self-improvement-window {
                                grid-area: review;
                            }
                            #tau-ops-harness-tui-companion {
                                grid-area: tui;
                            }
                            #tau-ops-harness-dashboard-window,
                            #tau-ops-harness-proof-window {
                                max-height: calc(100vh - 88px);
                                overflow: auto;
                            }
                            #tau-ops-harness-self-improvement-window {
                                max-height: calc(100vh - 354px);
                                overflow: auto;
                            }
                            #tau-ops-harness-tui-companion {
                                max-height: 200px;
                                overflow: auto;
                            }
                            #tau-ops-harness-topbar nav {
                                display: flex;
                                gap: 8px;
                                flex-wrap: wrap;
                            }
                            #tau-ops-harness-topbar nav form {
                                margin: 0;
                            }
                            #tau-ops-harness-route-action {
                                border-left: 2px solid var(--tau-harness-cyan);
                                color: var(--tau-harness-muted);
                                display: grid;
                                gap: 2px;
                                margin-top: 6px;
                                max-width: 760px;
                                padding-left: 8px;
                            }
                            #tau-ops-harness-route-action[hidden] {
                                display: none;
                            }
                            #tau-ops-harness-route-action strong {
                                color: var(--tau-harness-text);
                                font-size: .72rem;
                            }
                            #tau-ops-harness-route-action span {
                                font-size: .66rem;
                            }
                            #tau-ops-harness-history-view {
                                border: 1px solid var(--tau-harness-line-soft);
                                border-radius: 6px;
                                background: rgba(123, 214, 232, .07);
                                display: grid;
                                gap: 6px;
                                grid-column: 1 / -1;
                                margin-bottom: 7px;
                                padding: 8px;
                            }
                            #tau-ops-harness-history-view header {
                                display: flex;
                                align-items: flex-start;
                                justify-content: space-between;
                                gap: 8px;
                            }
                            #tau-ops-harness-history-view p {
                                color: var(--tau-harness-muted);
                                font-size: .62rem;
                                line-height: 1.18;
                            }
                            #tau-ops-harness-history-view nav {
                                display: flex;
                                flex-wrap: wrap;
                                gap: 4px;
                            }
                            #tau-ops-harness-history-view nav a {
                                border: 1px solid var(--tau-harness-line);
                                border-radius: 4px;
                                color: var(--tau-harness-muted);
                                line-height: 1;
                                padding: 4px 5px;
                            }
                            #tau-ops-harness-history-view nav a[aria-current="page"] {
                                border-color: rgba(123, 214, 232, .76);
                                color: var(--tau-harness-text);
                                background: rgba(123, 214, 232, .12);
                            }
                            #tau-ops-harness-history-view dl {
                                display: grid;
                                grid-template-columns: repeat(5, minmax(0, 1fr));
                                gap: 5px;
                            }
                            #tau-ops-harness-history-view dt {
                                color: var(--tau-harness-muted);
                                font-size: .54rem;
                                text-transform: uppercase;
                            }
                            #tau-ops-harness-history-view dd {
                                color: var(--tau-harness-text);
                                font-size: .68rem;
                                font-weight: 700;
                                white-space: nowrap;
                                overflow: hidden;
                                text-overflow: ellipsis;
                            }
                            #tau-ops-harness-history-detail {
                                border: 1px solid var(--tau-harness-line);
                                border-radius: 5px;
                                background: rgba(12, 20, 28, .28);
                                display: grid;
                                gap: 5px;
                                padding: 6px;
                            }
                            #tau-ops-harness-history-detail header {
                                align-items: flex-start;
                                display: flex;
                                gap: 8px;
                                justify-content: space-between;
                            }
                            #tau-ops-harness-history-detail header a,
                            #tau-ops-harness-history-detail header span {
                                color: var(--tau-harness-blue);
                                font-family: var(--tau-harness-mono);
                                font-size: .56rem;
                                max-width: 45%;
                                overflow-wrap: anywhere;
                                text-align: right;
                            }
                            #tau-ops-harness-history-detail dl {
                                display: grid;
                                grid-template-columns: repeat(4, minmax(0, 1fr));
                                gap: 5px;
                            }
                            #tau-ops-harness-history-detail dd {
                                overflow-wrap: anywhere;
                                white-space: normal;
                            }
                            #tau-ops-harness-history-detail .tau-harness-history-preview {
                                border-top: 1px solid var(--tau-harness-line-soft);
                                display: grid;
                                gap: 4px;
                                padding-top: 5px;
                            }
                            #tau-ops-harness-history-detail .tau-harness-history-preview header {
                                align-items: center;
                                display: flex;
                                justify-content: space-between;
                            }
                            #tau-ops-harness-history-detail .tau-harness-history-preview h5 {
                                color: var(--tau-harness-text);
                                font-size: .62rem;
                                margin: 0;
                            }
                            #tau-ops-harness-history-detail .tau-harness-history-preview pre {
                                background: rgba(2, 8, 13, .48);
                                border: 1px solid var(--tau-harness-line-soft);
                                border-radius: 4px;
                                color: var(--tau-harness-text);
                                font-family: var(--tau-harness-mono);
                                font-size: .55rem;
                                line-height: 1.22;
                                margin: 0;
                                max-height: 94px;
                                overflow: auto;
                                padding: 5px;
                                white-space: pre-wrap;
                                overflow-wrap: anywhere;
                            }
                            #tau-ops-harness-history-view a {
                                color: var(--tau-harness-blue);
                                font-size: .62rem;
                                text-decoration: none;
                            }
                            #tau-ops-harness-history-view a:hover {
                                text-decoration: underline;
                            }
                            .tau-harness-topbar-meta {
                                display: flex;
                                flex-wrap: wrap;
                                gap: 5px;
                                margin-top: 3px;
                                max-width: min(78vw, 760px);
                            }
                            .tau-harness-topbar-meta span {
                                border: 1px solid var(--tau-harness-line);
                                border-radius: 4px;
                                color: var(--tau-harness-muted);
                                font-size: .62rem;
                                line-height: 1;
                                max-width: 260px;
                                min-width: 0;
                                overflow: hidden;
                                padding: 3px 5px;
                                text-overflow: ellipsis;
                                white-space: nowrap;
                            }
                            #tau-ops-harness-panel a,
                            #tau-ops-harness-panel button {
                                border: 1px solid var(--tau-harness-line);
                                border-radius: 4px;
                                background: #132a38;
                                color: var(--tau-harness-text);
                                min-height: 30px;
                                padding: 6px 10px;
                                text-decoration: none;
                                transition: border-color .14s ease, background .14s ease, transform .14s ease;
                                cursor: pointer;
                                max-width: 100%;
                                white-space: normal;
                            }
                            #tau-ops-harness-panel a:hover,
                            #tau-ops-harness-panel button:hover {
                                border-color: #557181;
                                background: #193343;
                            }
                            #tau-ops-harness-panel :is(a, button):focus-visible {
                                outline: 2px solid var(--tau-harness-cyan);
                                outline-offset: 2px;
                            }
                            #tau-ops-harness-panel button[type="submit"] {
                                background: #132a38;
                            }
                            #tau-ops-harness-panel #tau-ops-harness-run-benchmark,
                            #tau-ops-harness-panel #tau-ops-harness-action-dry-run {
                                background: linear-gradient(180deg, #1d4f68, #173b4e);
                                border-color: rgba(123, 214, 232, .42);
                            }
                            #tau-ops-harness-panel #tau-ops-harness-action-approve[data-action-tone="approve"] {
                                background: linear-gradient(180deg, #2d7446, #1e5934);
                                border-color: rgba(95, 202, 122, .58);
                            }
                            #tau-ops-harness-panel #tau-ops-harness-action-reject[data-action-tone="reject"] {
                                background: linear-gradient(180deg, #7a342c, #5f261f);
                                border-color: rgba(223, 99, 91, .58);
                            }
                            #tau-ops-harness-action-apply[aria-disabled="true"] {
                                color: var(--tau-harness-muted);
                                opacity: .72;
                                cursor: not-allowed;
                            }
                            #tau-ops-harness-dashboard-window,
                            #tau-ops-harness-proof-window,
                            #tau-ops-harness-self-improvement-window,
                            #tau-ops-harness-tui-companion {
                                padding: 10px;
                            }
                            #tau-ops-harness-dashboard-window,
                            #tau-ops-harness-proof-window,
                            #tau-ops-harness-self-improvement-window {
                                display: grid;
                                gap: 10px;
                            }
                            .tau-harness-window-titlebar {
                                display: flex;
                                align-items: center;
                                justify-content: space-between;
                                gap: 10px;
                                min-height: 28px;
                                padding-bottom: 8px;
                                border-bottom: 1px solid var(--tau-harness-line-soft);
                            }
                            .tau-harness-window-titlebar > div {
                                min-width: 0;
                            }
                            .tau-harness-window-titlebar small {
                                color: var(--tau-harness-dim);
                                text-transform: uppercase;
                                font-size: .66rem;
                                font-weight: 700;
                            }
                            .tau-harness-window-grid {
                                display: grid;
                                grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
                                gap: 10px;
                                min-width: 0;
                            }
                            #tau-ops-harness-kpi-grid {
                                display: grid;
                                grid-template-columns: repeat(4, minmax(0, 1fr));
                                gap: 7px;
                                min-width: 0;
                                max-width: 100%;
                            }
                            #tau-ops-harness-active-missions,
                            #tau-ops-harness-benchmark-panel {
                                min-width: 0;
                                max-width: 100%;
                            }
                            #tau-ops-harness-active-missions {
                                max-height: 480px;
                                overflow: hidden;
                            }
                            #tau-ops-harness-active-missions .tau-harness-table-wrap {
                                max-height: 432px;
                                overflow: auto;
                            }
                            #tau-ops-harness-active-missions[data-active-mission-scroll-boundary="whole-row"] .tau-harness-table-wrap {
                                max-height: 388px;
                                overflow: auto;
                                overscroll-behavior: contain;
                                scrollbar-gutter: stable;
                                scroll-snap-type: y proximity;
                            }
                            #tau-ops-harness-active-missions[data-left-table-fit="compact-no-overflow"] #tau-ops-harness-missions-table,
                            #tau-ops-harness-benchmark-panel[data-left-table-fit="compact-no-overflow"] #tau-ops-harness-benchmark-table {
                                min-width: 0;
                                table-layout: fixed;
                            }
                            #tau-ops-harness-benchmark-proof {
                                margin-top: 8px;
                                padding-top: 8px;
                                border-top: 1px solid var(--tau-harness-line-soft);
                            }
                            #tau-ops-harness-benchmark-proof h5 {
                                margin: 0 0 6px;
                                color: var(--tau-harness-text);
                                font-size: .68rem;
                                letter-spacing: 0;
                                text-transform: none;
                            }
                            #tau-ops-harness-benchmark-proof dl {
                                display: grid;
                                grid-template-columns: repeat(2, minmax(0, 1fr));
                                gap: 6px 8px;
                                margin: 0;
                            }
                            #tau-ops-harness-benchmark-proof dt,
                            #tau-ops-harness-benchmark-proof dd {
                                margin: 0;
                            }
                            #tau-ops-harness-benchmark-proof dt {
                                color: var(--tau-harness-muted);
                                font-size: .55rem;
                            }
                            #tau-ops-harness-benchmark-proof dd {
                                color: var(--tau-harness-text);
                                font-size: .62rem;
                                line-height: 1.22;
                                overflow-wrap: anywhere;
                            }
                            #tau-ops-harness-benchmark-proof code {
                                color: var(--tau-harness-green);
                                font-family: var(--tau-harness-mono);
                                font-size: .58rem;
                                white-space: normal;
                            }
                            #tau-ops-harness-kpi-grid article,
                            #tau-ops-harness-proof-window section,
                            #tau-ops-harness-self-improvement-window section {
                                border: 1px solid var(--tau-harness-line-soft);
                                border-radius: 5px;
                                background: rgba(7, 18, 25, .68);
                                padding: 9px;
                                min-width: 0;
                            }
                            #tau-ops-harness-kpi-grid article {
                                min-height: 82px;
                                overflow: hidden;
                            }
                            #tau-ops-harness-kpi-grid[data-kpi-label-fit="word-boundary"] h4 {
                                overflow-wrap: normal;
                                word-break: normal;
                                hyphens: none;
                                font-size: .7rem;
                                line-height: 1.05;
                                letter-spacing: 0;
                            }
                            #tau-ops-harness-kpi-grid[data-kpi-label-fit="word-boundary"] h4 span {
                                display: block;
                                white-space: nowrap;
                                max-width: 100%;
                            }
                            #tau-ops-harness-tool-evidence {
                                grid-column: 1 / -1;
                            }
                            #tau-ops-harness-kpi-grid p {
                                margin-top: 6px;
                                color: white;
                                font-size: 1.36rem;
                                font-weight: 750;
                            }
                            .tau-harness-table-wrap {
                                min-width: 0;
                                width: 100%;
                                max-width: 100%;
                                overflow-x: auto;
                                border: 1px solid var(--tau-harness-line-soft);
                                border-radius: 5px;
                                background: rgba(5, 14, 20, .44);
                            }
                            #tau-ops-harness-panel table {
                                width: 100%;
                                border-collapse: collapse;
                                min-width: 520px;
                                font-size: .78rem;
                            }
                            #tau-ops-harness-panel th,
                            #tau-ops-harness-panel td {
                                border-bottom: 1px solid var(--tau-harness-line-soft);
                                padding: 7px 8px;
                                text-align: left;
                                vertical-align: middle;
                                white-space: nowrap;
                            }
                            #tau-ops-harness-panel td {
                                color: var(--tau-harness-text);
                            }
                            #tau-ops-harness-panel td:first-child {
                                color: #cfe1e8;
                                font-weight: 560;
                            }
                            #tau-ops-harness-panel tbody tr {
                                background: rgba(7, 18, 25, .52);
                            }
                            #tau-ops-harness-panel tbody tr:hover {
                                background: rgba(77, 141, 255, .08);
                            }
                            #tau-ops-harness-panel small,
                            #tau-ops-harness-panel dt,
                            #tau-ops-harness-panel th {
                                color: var(--tau-harness-muted);
                            }
                            #tau-ops-harness-panel meter {
                                width: 82px;
                                height: 8px;
                                vertical-align: middle;
                            }
                            #tau-ops-harness-missions-table td[data-mission-summary="inline-status"] {
                                white-space: normal;
                            }
                            #tau-ops-harness-missions-table .tau-harness-mission-title {
                                display: block;
                                line-height: 1.24;
                                color: var(--tau-harness-text);
                                text-decoration: none;
                            }
                            #tau-ops-harness-missions-table .tau-harness-mission-title:hover,
                            #tau-ops-harness-missions-table .tau-harness-mission-title:focus-visible {
                                color: var(--tau-harness-blue);
                                text-decoration: underline;
                            }
                            #tau-ops-harness-missions-table .tau-harness-mission-identity {
                                margin-top: 4px;
                                color: var(--tau-harness-muted);
                                font-family: var(--tau-harness-mono);
                                font-size: .59rem;
                                line-height: 1.2;
                                overflow-wrap: anywhere;
                            }
                            #tau-ops-harness-missions-table .tau-harness-mission-identity[data-selected-proof="true"] {
                                color: var(--tau-harness-green);
                            }
                            #tau-ops-harness-missions-table tr[data-selected-proof="true"] {
                                background: rgba(87, 225, 161, .08);
                                box-shadow: inset 2px 0 0 rgba(87, 225, 161, .86);
                            }
                            #tau-ops-harness-missions-table .tau-harness-mission-meta {
                                display: flex;
                                align-items: center;
                                flex-wrap: wrap;
                                gap: 5px;
                                margin-top: 6px;
                            }
                            #tau-ops-harness-missions-table .tau-harness-mission-meta form {
                                margin: 0;
                            }
                            #tau-ops-harness-missions-table [data-action="start-mission"] {
                                min-height: 22px;
                                padding: 2px 8px;
                                background: linear-gradient(180deg, #1d4f68, #173b4e);
                                border-color: rgba(123, 214, 232, .42);
                                font-size: .68rem;
                            }
                            #tau-ops-harness-missions-table .tau-harness-row-action-status {
                                color: var(--tau-harness-dim);
                                font-size: .65rem;
                            }
                            #tau-ops-harness-active-missions[data-left-table-fit="compact-no-overflow"] #tau-ops-harness-missions-table th:nth-child(n+4),
                            #tau-ops-harness-active-missions[data-left-table-fit="compact-no-overflow"] #tau-ops-harness-missions-table td:nth-child(n+4) {
                                display: none;
                            }
                            #tau-ops-harness-active-missions[data-left-table-fit="compact-no-overflow"] #tau-ops-harness-missions-table th,
                            #tau-ops-harness-active-missions[data-left-table-fit="compact-no-overflow"] #tau-ops-harness-missions-table td,
                            #tau-ops-harness-benchmark-panel[data-left-table-fit="compact-no-overflow"] #tau-ops-harness-benchmark-table td {
                                white-space: normal;
                            }
                            #tau-ops-harness-benchmark-panel[data-category-label-fit="operator-readable"] #tau-ops-harness-benchmark-table td:first-child {
                                white-space: normal;
                                overflow-wrap: normal;
                                word-break: normal;
                            }
                            #tau-ops-harness-benchmark-panel[data-category-label-fit="operator-readable"] .tau-harness-benchmark-category-label {
                                display: block;
                                line-height: 1.16;
                                max-width: 100%;
                            }
                            #tau-ops-harness-active-missions[data-left-table-fit="compact-no-overflow"] #tau-ops-harness-missions-table th {
                                font-size: .66rem;
                                padding-inline: 6px;
                                white-space: nowrap;
                            }
                            #tau-ops-harness-active-missions[data-left-table-fit="compact-no-overflow"] #tau-ops-harness-missions-table th:nth-child(2),
                            #tau-ops-harness-active-missions[data-left-table-fit="compact-no-overflow"] #tau-ops-harness-missions-table td:nth-child(2) {
                                width: 76px;
                            }
                            #tau-ops-harness-active-missions[data-left-table-fit="compact-no-overflow"] #tau-ops-harness-missions-table th:nth-child(3),
                            #tau-ops-harness-active-missions[data-left-table-fit="compact-no-overflow"] #tau-ops-harness-missions-table td:nth-child(3) {
                                width: 70px;
                            }
                            #tau-ops-harness-active-missions[data-left-table-fit="compact-no-overflow"] #tau-ops-harness-missions-table meter {
                                width: 58px;
                            }
                            #tau-ops-harness-active-missions[data-active-mission-scroll-boundary="whole-row"] #tau-ops-harness-missions-table tbody tr {
                                scroll-snap-align: start;
                            }
                            #tau-ops-harness-active-missions[data-active-mission-scroll-boundary="whole-row"] #tau-ops-harness-missions-table td {
                                vertical-align: top;
                            }
                            .tau-harness-status-chip,
                            #tau-ops-harness-verification-gates li,
                            #tau-ops-harness-acceptance li,
                            #tau-ops-harness-learning-queue li {
                                display: inline-flex;
                                align-items: center;
                                gap: 6px;
                                border: 1px solid var(--tau-harness-line);
                                border-radius: 999px;
                                background: rgba(20, 43, 59, .9);
                                padding: 4px 8px;
                                color: var(--tau-harness-text);
                                font-size: .74rem;
                            }
                            .tau-harness-status-chip::before,
                            #tau-ops-harness-verification-gates li::before,
                            #tau-ops-harness-acceptance li::before,
                            #tau-ops-harness-learning-queue li::before {
                                content: "";
                                width: 7px;
                                height: 7px;
                                border-radius: 999px;
                                background: var(--tau-harness-muted);
                            }
                            [data-status="running"] .tau-harness-status-chip::before,
                            [data-status="verifying"] .tau-harness-status-chip::before,
                            #tau-ops-harness-verification-gates [data-gate-status="running"]::before {
                                background: var(--tau-harness-blue);
                            }
                            [data-status="completed"] .tau-harness-status-chip::before,
                            #tau-ops-harness-verification-gates [data-gate-status="passed"]::before,
                            #tau-ops-harness-acceptance [data-ac-status="met"]::before,
                            #tau-ops-harness-learning-queue [data-status="applied"]::before,
                            #tau-ops-harness-learning-queue [data-status="completed"]::before,
                            #tau-ops-harness-learning-queue [data-status="proposal"]::before {
                                background: var(--tau-harness-green);
                            }
                            [data-status="blocked"] .tau-harness-status-chip::before,
                            #tau-ops-harness-verification-gates [data-gate-status="failed"]::before {
                                background: var(--tau-harness-red);
                            }
                            [data-verification-state="needs-review"] .tau-harness-status-chip {
                                border-color: rgba(216, 173, 69, .78);
                                color: #f2d590;
                            }
                            #tau-ops-harness-missions-table .tau-harness-mission-meta .tau-harness-status-chip {
                                padding: 2px 6px;
                                gap: 4px;
                                font-size: .64rem;
                                color: var(--tau-harness-text);
                                border-color: var(--tau-harness-line);
                            }
                            #tau-ops-harness-missions-table .tau-harness-mission-meta .tau-harness-status-chip::before {
                                width: 6px;
                                height: 6px;
                            }
                            .tau-harness-status-chip[data-mission-state-chip="running"]::before,
                            .tau-harness-status-chip[data-mission-state-chip="verifying"]::before,
                            .tau-harness-status-chip[data-mission-gate-chip="running"]::before {
                                background: var(--tau-harness-blue);
                            }
                            .tau-harness-status-chip[data-mission-state-chip="completed"]::before,
                            .tau-harness-status-chip[data-mission-gate-chip="passed"]::before {
                                background: var(--tau-harness-green);
                            }
                            .tau-harness-status-chip[data-mission-state-chip="blocked"]::before,
                            .tau-harness-status-chip[data-mission-gate-chip="failed"]::before {
                                background: var(--tau-harness-red);
                            }
                            .tau-harness-status-chip[data-mission-gate-chip="needs-review"] {
                                border-color: rgba(216, 173, 69, .78);
                                color: #f2d590;
                            }
                            .tau-harness-status-chip[data-mission-gate-chip="needs-review"]::before {
                                background: #d8ad45;
                            }
                            #tau-ops-harness-plan-dag {
                                display: grid;
                                grid-template-columns: repeat(5, minmax(0, 1fr));
                                gap: 6px;
                                margin: 0;
                                padding: 0;
                                list-style: none;
                            }
                            #tau-ops-harness-plan-dag li {
                                border: 1px solid var(--tau-harness-line);
                                border-radius: 999px;
                                min-height: 30px;
                                padding: 5px 4px;
                                text-align: center;
                                background: #142b3b;
                                min-width: 0;
                                overflow: hidden;
                                text-overflow: ellipsis;
                                white-space: nowrap;
                                font-size: .66rem;
                            }
                            #tau-ops-harness-plan-dag [data-node-status="passed"],
                            #tau-ops-harness-verification-gates [data-gate-status="passed"],
                            #tau-ops-harness-policy-allowed {
                                border-color: var(--tau-harness-green);
                            }
                            #tau-ops-harness-verification-gates [data-gate-status="failed"],
                            #tau-ops-harness-policy-blocked {
                                border-color: var(--tau-harness-red);
                            }
                            #tau-ops-harness-plan-dag [data-node-status="running"],
                            #tau-ops-harness-verification-gates [data-gate-status="running"] {
                                border-color: var(--tau-harness-blue);
                            }
                            #tau-ops-harness-proof-header dl,
                            #tau-ops-harness-proposal-detail dl {
                                display: grid;
                                grid-template-columns: minmax(7rem, max-content) minmax(0, 1fr);
                                gap: 6px 12px;
                                margin: 8px 0 0;
                            }
                            #tau-ops-harness-proof-header[data-metadata-fit="no-wrap"] dl {
                                grid-template-columns: minmax(5.5rem, max-content) minmax(4.75rem, max-content);
                                gap: 4px 8px;
                                margin-left: auto;
                                font-size: .78rem;
                            }
                            #tau-ops-harness-proof-header[data-metadata-fit="no-wrap"] dt,
                            #tau-ops-harness-proof-header[data-metadata-fit="no-wrap"] dd {
                                white-space: nowrap;
                                overflow-wrap: normal;
                            }
                            #tau-ops-harness-proof-header[data-metadata-fit="no-wrap"] dd {
                                overflow: hidden;
                                text-overflow: clip;
                            }
                            #tau-ops-harness-selected-mission-actions {
                                display: grid;
                                grid-template-columns: minmax(0, 1fr) auto;
                                gap: 8px;
                                align-items: center;
                                border: 1px solid rgba(87, 225, 161, .22);
                                border-radius: 6px;
                                padding: 8px 10px;
                                background: rgba(87, 225, 161, .06);
                            }
                            #tau-ops-harness-selected-mission-actions h4,
                            #tau-ops-harness-selected-mission-actions p,
                            #tau-ops-harness-selected-mission-actions form {
                                margin: 0;
                            }
                            #tau-ops-harness-selected-mission-actions h4 {
                                color: var(--tau-harness-text);
                                font-size: .72rem;
                                letter-spacing: 0;
                            }
                            #tau-ops-harness-selected-mission-actions p {
                                color: var(--tau-harness-muted);
                                font-size: .66rem;
                            }
                            .tau-harness-selected-mission-proof {
                                display: grid;
                                grid-template-columns: repeat(5, minmax(74px, 1fr));
                                gap: 6px;
                                margin: 7px 0 0;
                            }
                            .tau-harness-selected-mission-proof div {
                                min-width: 0;
                                border: 1px solid rgba(148, 163, 184, .18);
                                border-radius: 5px;
                                padding: 5px 6px;
                                background: rgba(15, 23, 42, .28);
                            }
                            .tau-harness-selected-mission-proof dt,
                            .tau-harness-selected-mission-proof dd {
                                margin: 0;
                                white-space: nowrap;
                                overflow: hidden;
                                text-overflow: ellipsis;
                            }
                            .tau-harness-selected-mission-proof dt {
                                color: var(--tau-harness-muted);
                                font-size: .55rem;
                            }
                            .tau-harness-selected-mission-proof dd {
                                color: var(--tau-harness-text);
                                font-size: .66rem;
                                font-weight: 700;
                            }
                            #tau-ops-harness-start-selected-mission {
                                min-height: 28px;
                                padding: 4px 10px;
                                background: linear-gradient(180deg, #1d5f42, #164530);
                                border-color: rgba(87, 225, 161, .48);
                                font-size: .7rem;
                            }
                            #tau-ops-harness-proposal-detail {
                                max-height: 128px;
                                overflow: auto;
                            }
                            #tau-ops-harness-proposal-detail[data-proposal-detail-overflow-budget="contained"] {
                                overflow: hidden;
                            }
                            #tau-ops-harness-proposal-detail[data-proposal-detail-vertical-overflow-budget="none"] {
                                max-height: 132px;
                            }
                            #tau-ops-harness-proposal-detail[data-proposal-detail-overflow-budget="contained"] h4 {
                                overflow: hidden;
                                text-overflow: ellipsis;
                                white-space: nowrap;
                                font-size: .74rem;
                                line-height: 1.12;
                            }
                            #tau-ops-harness-proposal-detail[data-proposal-detail-overflow-budget="contained"] dl {
                                gap: 3px 10px;
                                margin-top: 5px;
                                grid-template-columns: minmax(6.7rem, max-content) minmax(0, 1fr);
                                font-size: .64rem;
                                line-height: 1.06;
                            }
                            #tau-ops-harness-proposal-detail[data-proposal-summary-fit="full-text"] dl {
                                gap: 2px 10px;
                                font-size: .62rem;
                                line-height: 1.04;
                            }
                            #tau-ops-harness-proposal-detail[data-proposal-detail-overflow-budget="contained"] dt,
                            #tau-ops-harness-proposal-detail[data-proposal-detail-overflow-budget="contained"] dd {
                                min-width: 0;
                                margin: 0;
                                overflow: hidden;
                                text-overflow: ellipsis;
                                white-space: nowrap;
                            }
                            #tau-ops-harness-proposal-detail[data-proposal-summary-fit="full-text"] dd[data-summary-fit="full-text"] {
                                white-space: normal;
                                overflow-wrap: normal;
                                overflow: visible;
                                text-overflow: clip;
                            }
                            #tau-ops-harness-proposal-detail[data-proposal-detail-overflow-budget="contained"] a {
                                min-height: 0;
                                padding: 0;
                                border: 0;
                                background: transparent;
                                display: inline;
                                font: inherit;
                            }
                            #tau-ops-harness-self-improvement-proof {
                                max-height: 178px;
                                overflow: auto;
                            }
                            #tau-ops-harness-self-improvement-proof p {
                                margin: 5px 0 0;
                                color: var(--tau-harness-text);
                                font-size: .64rem;
                                line-height: 1.18;
                            }
                            .tau-harness-self-improvement-proof-grid {
                                display: grid;
                                grid-template-columns: repeat(3, minmax(0, 1fr));
                                gap: 6px;
                                margin-top: 6px;
                            }
                            .tau-harness-self-improvement-proof-grid section {
                                padding: 0;
                                border: 0;
                                background: transparent;
                            }
                            .tau-harness-self-improvement-proof-grid h5 {
                                color: var(--tau-harness-muted);
                                font-size: .62rem;
                                text-transform: uppercase;
                            }
                            #tau-ops-harness-self-improvement-proof li {
                                width: 100%;
                                min-width: 0;
                                justify-content: flex-start;
                                overflow: hidden;
                                text-overflow: ellipsis;
                                white-space: nowrap;
                                border-radius: 6px;
                                padding: 2px 5px;
                                font-size: .58rem;
                                line-height: 1.04;
                            }
                            #tau-ops-harness-self-improvement-proof [data-proof-status="completed"]::before,
                            #tau-ops-harness-self-improvement-proof [data-proof-status="passed"]::before,
                            #tau-ops-harness-self-improvement-proof [data-proof-status="skill"]::before {
                                background: var(--tau-harness-green);
                            }
                            #tau-ops-harness-self-improvement-window[data-review-audit-priority="first-viewport-recent-history"] {
                                gap: 8px;
                            }
                            #tau-ops-harness-self-improvement-window[data-review-audit-priority="first-viewport-recent-history"] section {
                                padding: 8px;
                            }
                            #tau-ops-harness-self-improvement-window[data-review-audit-priority="first-viewport-recent-history"] h4,
                            #tau-ops-harness-self-improvement-window[data-review-audit-priority="first-viewport-recent-history"] h5 {
                                margin: 0;
                            }
                            #tau-ops-harness-learning-queue {
                                max-height: 124px;
                                overflow: auto;
                            }
                            #tau-ops-harness-learning-queue ul {
                                gap: 5px;
                            }
                            #tau-ops-harness-learning-queue[data-queue-density="all-items-visible"] ul {
                                display: grid;
                                grid-template-columns: repeat(2, minmax(0, 1fr));
                                gap: 5px;
                            }
                            #tau-ops-harness-learning-queue li {
                                padding: 3px 7px;
                                font-size: .70rem;
                            }
                            #tau-ops-harness-learning-queue[data-queue-density="all-items-visible"] li {
                                min-width: 0;
                                overflow: hidden;
                                text-overflow: ellipsis;
                                white-space: nowrap;
                                padding: 2px 6px;
                                font-size: .66rem;
                            }
                            #tau-ops-harness-learning-queue[data-queue-readability="full-labels"] {
                                max-height: 112px;
                                overflow: hidden;
                            }
                            #tau-ops-harness-learning-queue[data-queue-readability="full-labels"] ul {
                                display: grid;
                                grid-template-columns: minmax(0, 1fr);
                                gap: 3px;
                                margin-top: 6px;
                            }
                            #tau-ops-harness-learning-queue[data-queue-readability="full-labels"] li {
                                width: 100%;
                                min-width: 0;
                                overflow: hidden;
                                white-space: nowrap;
                                text-overflow: clip;
                                justify-content: flex-start;
                                border-radius: 6px;
                                padding: 1px 6px;
                                font-size: .62rem;
                                line-height: 1.05;
                            }
                            #tau-ops-harness-learning-queue li a,
                            #tau-ops-harness-learning-queue .tau-harness-queue-static {
                                display: flex;
                                align-items: center;
                                justify-content: space-between;
                                gap: 8px;
                                min-width: 0;
                                width: 100%;
                                color: inherit;
                                text-decoration: none;
                            }
                            #tau-ops-harness-learning-queue li[data-selected="true"] {
                                border-color: rgba(89, 151, 255, .82);
                                background: rgba(33, 92, 158, .34);
                            }
                            #tau-ops-harness-learning-queue li[data-actionable="true"] {
                                cursor: pointer;
                            }
                            #tau-ops-harness-learning-queue .tau-harness-queue-label {
                                min-width: 0;
                                overflow: hidden;
                                text-overflow: ellipsis;
                                white-space: nowrap;
                            }
                            #tau-ops-harness-learning-queue .tau-harness-queue-status {
                                flex: 0 0 auto;
                                color: var(--tau-harness-muted);
                                font-size: .54rem;
                                text-transform: uppercase;
                            }
                            #tau-ops-harness-learning-queue[data-queue-readability="full-labels"] li::before {
                                flex: 0 0 6px;
                                width: 6px;
                                height: 6px;
                            }
                            #tau-ops-harness-operator-actions {
                                gap: 6px;
                            }
                            #tau-ops-harness-operator-actions button,
                            #tau-ops-harness-operator-actions a {
                                min-height: 26px;
                                padding: 4px 8px;
                                font-size: .76rem;
                            }
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] table {
                                min-width: 0;
                                width: 100%;
                                table-layout: fixed;
                                font-size: .68rem;
                            }
                            #tau-ops-harness-tool-evidence[data-tool-label-fit="full-memory-tool-names"] table {
                                font-size: .66rem;
                            }
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] th,
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] td {
                                overflow: hidden;
                                text-overflow: ellipsis;
                                white-space: nowrap;
                                padding: 5px 6px;
                            }
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] th:nth-child(1),
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] td:nth-child(1) {
                                width: 82px;
                            }
                            #tau-ops-harness-tool-evidence[data-tool-label-fit="full-memory-tool-names"] th:nth-child(1),
                            #tau-ops-harness-tool-evidence[data-tool-label-fit="full-memory-tool-names"] td:nth-child(1) {
                                width: 96px;
                            }
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] th:nth-child(2),
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] td:nth-child(2) {
                                display: none;
                            }
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] th:nth-child(3),
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] td:nth-child(3) {
                                width: 86px;
                            }
                            #tau-ops-harness-tool-evidence[data-tool-label-fit="full-memory-tool-names"] th:nth-child(3),
                            #tau-ops-harness-tool-evidence[data-tool-label-fit="full-memory-tool-names"] td:nth-child(3) {
                                width: 86px;
                            }
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] th:nth-child(4),
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] td:nth-child(4),
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] th:nth-child(5),
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] td:nth-child(5) {
                                width: 66px;
                            }
                            #tau-ops-harness-tool-evidence[data-tool-label-fit="full-memory-tool-names"] th:nth-child(4),
                            #tau-ops-harness-tool-evidence[data-tool-label-fit="full-memory-tool-names"] td:nth-child(4) {
                                width: 60px;
                            }
                            #tau-ops-harness-tool-evidence[data-tool-label-fit="full-memory-tool-names"] th:nth-child(5),
                            #tau-ops-harness-tool-evidence[data-tool-label-fit="full-memory-tool-names"] td:nth-child(5) {
                                width: 56px;
                            }
                            #tau-ops-harness-conservative-policy {
                                gap: 6px;
                            }
                            #tau-ops-harness-conservative-policy ul {
                                gap: 5px;
                                margin-top: 5px;
                            }
                            #tau-ops-harness-audit-log {
                                max-height: 104px;
                                overflow: hidden;
                            }
                            #tau-ops-harness-audit-log .tau-harness-table-wrap {
                                max-height: 64px;
                                overflow: auto;
                            }
                            #tau-ops-harness-audit-log table {
                                min-width: 0;
                                font-size: .68rem;
                            }
                            #tau-ops-harness-audit-log td {
                                padding: 4px 5px;
                            }
                            #tau-ops-harness-audit-log[data-audit-overflow-budget="all-rows-visible"] td {
                                padding: 2px 5px;
                                font-size: .62rem;
                                line-height: 1.05;
                            }
                            #tau-ops-harness-audit-log td:nth-child(2),
                            #tau-ops-harness-audit-log td:nth-child(4) {
                                display: none;
                            }
                            #tau-ops-harness-audit-log td[data-audit-item-cell="item-proof"] > span,
                            #tau-ops-harness-audit-log td[data-audit-item-cell="item-proof"] > a {
                                display: block;
                            }
                            #tau-ops-harness-audit-log .tau-harness-audit-detail,
                            #tau-ops-harness-audit-log .tau-harness-audit-proof {
                                margin-top: 2px;
                                color: var(--tau-harness-muted);
                                font-family: var(--tau-harness-mono);
                                font-size: .55rem;
                                line-height: 1.1;
                                overflow-wrap: anywhere;
                            }
                            #tau-ops-harness-audit-log a.tau-harness-audit-proof {
                                color: var(--tau-harness-blue);
                                text-decoration: none;
                            }
                            #tau-ops-harness-audit-log a.tau-harness-audit-inspect {
                                color: var(--tau-harness-blue);
                                font-size: .54rem;
                                margin-top: 2px;
                                text-decoration: none;
                            }
                            #tau-ops-harness-audit-log a.tau-harness-audit-inspect[aria-current="page"] {
                                color: var(--tau-harness-green);
                                font-weight: 700;
                            }
                            #tau-ops-harness-audit-log a.tau-harness-audit-proof:hover {
                                text-decoration: underline;
                            }
                            #tau-ops-harness-audit-log a.tau-harness-audit-inspect:hover {
                                text-decoration: underline;
                            }
                            #tau-ops-harness-audit-log .tau-harness-audit-detail[data-audit-detail-visible="true"] {
                                color: var(--tau-harness-green);
                            }
                            #tau-ops-harness-acceptance ul,
                            #tau-ops-harness-verification-gates ul,
                            #tau-ops-harness-artifacts ul,
                            #tau-ops-harness-learning-queue ul,
                            #tau-ops-harness-self-improvement-proof ul,
                            #tau-ops-harness-conservative-policy ul {
                                display: flex;
                                flex-wrap: wrap;
                                gap: 7px;
                                margin: 8px 0 0;
                                padding: 0;
                                list-style: none;
                            }
                            #tau-ops-harness-acceptance ul,
                            #tau-ops-harness-verification-gates ul {
                                gap: 5px;
                            }
                            #tau-ops-harness-acceptance,
                            #tau-ops-harness-verification-gates {
                                max-height: 160px;
                                overflow: auto;
                            }
                            #tau-ops-harness-memory-learning,
                            #tau-ops-harness-artifacts {
                                min-height: 0;
                            }
                            #tau-ops-harness-acceptance li,
                            #tau-ops-harness-verification-gates li,
                            #tau-ops-harness-self-improvement-proof li {
                                padding: 3px 7px;
                            }
                            #tau-ops-harness-acceptance[data-acceptance-overflow-budget="all-criteria-visible"] {
                                overflow: hidden;
                            }
                            #tau-ops-harness-acceptance[data-acceptance-overflow-budget="all-criteria-visible"] ul {
                                display: grid;
                                grid-template-columns: minmax(0, 1fr);
                                gap: 4px;
                                margin-top: 6px;
                            }
                            #tau-ops-harness-acceptance[data-acceptance-overflow-budget="all-criteria-visible"] li {
                                min-width: 0;
                                width: 100%;
                                justify-content: flex-start;
                                overflow: hidden;
                                text-overflow: ellipsis;
                                white-space: nowrap;
                                padding: 2px 6px;
                                font-size: .64rem;
                                line-height: 1.05;
                            }
                            #tau-ops-harness-verification-gates[data-gate-visibility="all-gates-first-viewport"] {
                                overflow: hidden;
                            }
                            #tau-ops-harness-verification-gates[data-gate-visibility="all-gates-first-viewport"] ul {
                                display: grid;
                                grid-template-columns: repeat(2, minmax(0, 1fr));
                                gap: 5px;
                            }
                            #tau-ops-harness-verification-gates[data-gate-visibility="all-gates-first-viewport"] li {
                                min-width: 0;
                                justify-content: flex-start;
                                padding: 3px 6px;
                                font-size: .68rem;
                                line-height: 1.15;
                            }
                            #tau-ops-harness-conservative-policy {
                                display: grid;
                                grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
                                gap: 8px;
                            }
                            #tau-ops-harness-conservative-policy h4 {
                                grid-column: 1 / -1;
                            }
                            #tau-ops-harness-operator-actions {
                                display: grid;
                                grid-template-columns: repeat(2, minmax(0, 1fr));
                                align-items: stretch;
                                gap: 8px;
                            }
                            #tau-ops-harness-operator-actions h4 {
                                grid-column: 1 / -1;
                            }
                            #tau-ops-harness-operator-action-state {
                                grid-column: 1 / -1;
                                border: 1px solid rgba(148, 163, 184, .18);
                                border-radius: 5px;
                                padding: 7px 8px;
                                background: rgba(15, 23, 42, .35);
                            }
                            #tau-ops-harness-operator-action-state h5,
                            #tau-ops-harness-operator-action-state p,
                            #tau-ops-harness-operator-action-state small {
                                margin: 0;
                            }
                            #tau-ops-harness-operator-action-state h5 {
                                color: var(--tau-harness-muted);
                                font-size: .55rem;
                            }
                            #tau-ops-harness-operator-action-state p {
                                color: var(--tau-harness-text);
                                font-size: .7rem;
                                font-weight: 700;
                            }
                            #tau-ops-harness-operator-action-state small {
                                color: var(--tau-harness-muted);
                                font-size: .6rem;
                                line-height: 1.25;
                            }
                            #tau-ops-harness-operator-actions form {
                                margin: 0;
                                min-width: 0;
                            }
                            #tau-ops-harness-operator-actions button,
                            #tau-ops-harness-operator-actions a {
                                width: 100%;
                                justify-content: center;
                                text-align: center;
                            }
                            #tau-ops-harness-apply-form,
                            #tau-ops-harness-action-apply {
                                grid-column: 1 / -1;
                            }
                            #tau-ops-harness-operator-log {
                                grid-column: 1 / -1;
                            }
                            #tau-ops-harness-panel pre {
                                overflow: auto;
                                border: 1px solid #1f3644;
                                border-radius: 5px;
                                background: #061016;
                                color: #c3dce7;
                                padding: 10px;
                                font-family: ui-monospace, "SFMono-Regular", Menlo, Consolas, monospace;
                                font-size: .76rem;
                                line-height: 1.5;
                                max-height: 250px;
                            }
                            #tau-ops-harness-operator-log pre {
                                max-height: 118px;
                            }
                            #tau-ops-harness-tui-companion pre {
                                max-height: 126px;
                                margin-top: 6px;
                                padding: 8px;
                                font-size: .72rem;
                                line-height: 1.35;
                            }
                            @media (max-width: 1400px) {
                                #tau-ops-harness-kpi-grid {
                                    grid-template-columns: repeat(2, minmax(0, 1fr));
                                }
                                #tau-ops-harness-panel #tau-ops-harness-missions-table,
                                #tau-ops-harness-panel #tau-ops-harness-benchmark-table {
                                    min-width: 0;
                                    table-layout: fixed;
                                }
                                #tau-ops-harness-missions-table th:nth-child(n+4),
                                #tau-ops-harness-missions-table td:nth-child(n+4) {
                                    display: none;
                                }
                                #tau-ops-harness-missions-table th,
                                #tau-ops-harness-missions-table td,
                                #tau-ops-harness-benchmark-table td {
                                    white-space: normal;
                                }
                                #tau-ops-harness-missions-table th {
                                    font-size: .66rem;
                                    padding-inline: 6px;
                                    white-space: nowrap;
                                }
                                #tau-ops-harness-missions-table th:nth-child(2),
                                #tau-ops-harness-missions-table td:nth-child(2) {
                                    width: 76px;
                                }
                                #tau-ops-harness-missions-table th:nth-child(3),
                                #tau-ops-harness-missions-table td:nth-child(3) {
                                    width: 70px;
                                }
                                #tau-ops-harness-missions-table meter {
                                    width: 58px;
                                }
                                #tau-ops-harness-proof-header {
                                    align-items: stretch;
                                    flex-direction: column;
                                }
                                #tau-ops-harness-proof-header dl {
                                    grid-template-columns: minmax(6rem, max-content) minmax(0, 1fr);
                                    width: 100%;
                                }
                                #tau-ops-harness-tool-evidence table {
                                    min-width: 0;
                                    table-layout: fixed;
                                }
                                #tau-ops-harness-tool-evidence th,
                                #tau-ops-harness-tool-evidence td {
                                    white-space: normal;
                                    overflow-wrap: anywhere;
                                }
                                #tau-ops-harness-tool-evidence th:nth-child(6),
                                #tau-ops-harness-tool-evidence td:nth-child(6),
                                #tau-ops-harness-tool-evidence th:nth-child(2),
                                #tau-ops-harness-tool-evidence td:nth-child(2) {
                                    display: none;
                                }
                                #tau-ops-harness-tool-evidence th:nth-child(4),
                                #tau-ops-harness-tool-evidence td:nth-child(4),
                                #tau-ops-harness-tool-evidence th:nth-child(5),
                                #tau-ops-harness-tool-evidence td:nth-child(5) {
                                    width: 82px;
                                    white-space: nowrap;
                                    overflow-wrap: normal;
                                }
                                #tau-ops-harness-operator-log pre,
                                #tau-ops-harness-tui-companion pre {
                                    overflow-x: hidden;
                                    overflow-wrap: anywhere;
                                    white-space: pre-wrap;
                                }
                                #tau-ops-harness-acceptance[data-narrow-label-fit="full-labels-at-1400px"][data-acceptance-overflow-budget="all-criteria-visible"] li {
                                    font-size: .59rem;
                                    padding-inline: 5px;
                                }
                                #tau-ops-harness-verification-gates[data-narrow-height-budget="no-hidden-overflow"][data-gate-visibility="all-gates-first-viewport"] {
                                    max-height: 168px;
                                }
                            }
                            @media (max-width: 1450px) {
                                #tau-ops-harness-panel {
                                    grid-template-columns: minmax(0, .94fr) minmax(0, 1.06fr);
                                    grid-template-areas:
                                        "topbar topbar"
                                        "dashboard proof"
                                        "review tui";
                                }
                                #tau-ops-harness-self-improvement-window,
                                #tau-ops-harness-tui-companion {
                                    max-height: 260px;
                                }
                            }
                            @media (max-width: 1180px) {
                                #tau-ops-harness-panel {
                                    grid-template-columns: minmax(0, 1fr);
                                    grid-template-areas:
                                        "topbar"
                                        "dashboard"
                                        "proof"
                                        "review"
                                        "tui";
                                }
                                .tau-harness-window-grid,
                                #tau-ops-harness-conservative-policy {
                                    grid-template-columns: minmax(0, 1fr);
                                }
                                #tau-ops-harness-kpi-grid {
                                    grid-template-columns: repeat(2, minmax(0, 1fr));
                                }
                                #tau-ops-harness-plan-dag {
                                    grid-template-columns: repeat(2, minmax(0, 1fr));
                                }
                                #tau-ops-harness-dashboard-window,
                                #tau-ops-harness-proof-window,
                                #tau-ops-harness-self-improvement-window,
                                #tau-ops-harness-tui-companion {
                                    max-height: none;
                                }
                            }
                            @media (max-width: 900px) {
                                #tau-ops-harness-panel {
                                    width: 100%;
                                    max-width: 100%;
                                }
                            }
                            @media (max-width: 760px) {
                                #tau-ops-harness-panel {
                                    padding: 8px;
                                    gap: 8px;
                                }
                                #tau-ops-harness-topbar {
                                    align-items: flex-start;
                                    flex-direction: column;
                                }
                                .tau-harness-window-titlebar {
                                    align-items: flex-start;
                                    flex-direction: column;
                                }
                                #tau-ops-harness-proof-header dl,
                                #tau-ops-harness-proposal-detail dl {
                                    grid-template-columns: max-content minmax(0, 1fr);
                                    width: 100%;
                                }
                                #tau-ops-harness-kpi-grid {
                                    grid-template-columns: minmax(0, 1fr);
                                }
                            }
                            "#
                        </style>
                        <section
                            id="tau-ops-harness-panel"
                            data-route="/ops/harness"
                            data-component="MissionHarnessWorkspace"
                            data-design-template="three-window-agent-harness"
                            aria-hidden=harness_panel_hidden
                            data-panel-visible=harness_panel_visible
                            data-layout-density="operator-console"
                            data-visual-contract="mission-control"
                            data-desktop-preview-layout="three-window"
                            data-responsive-collapse-width="1450px"
                            data-in-app-browser-fit="no-right-rail-clipping"
                        >
                            <p
                                id="tau-ops-harness-preview-status"
                                data-preview-status="idle"
                                hidden
                                aria-live="polite"
                            >
                                "Preview action idle."
                            </p>
                            <header
                                id="tau-ops-harness-topbar"
                                data-workspace=harness_runtime_workspace_label.clone()
                                data-model=harness_runtime_model_label.clone()
                                data-transport=harness_runtime_transport_label.clone()
                                data-health=harness_runtime_health_key.clone()
                                data-window-chrome="compact"
                            >
                                <div>
                                    <small>"Tau"</small>
                                    <h2>"Tau Agent Harness"</h2>
                                    <div class="tau-harness-topbar-meta" aria-label="Harness runtime context">
                                        <span data-topbar-field="workspace">{harness_runtime_workspace_label.clone()}</span>
                                        <span data-topbar-field="model">{harness_runtime_model_label.clone()}</span>
                                        <span data-topbar-field="transport">{harness_runtime_transport_label.clone()}</span>
                                        <span data-topbar-field="health">{harness_runtime_health_label}</span>
                                    </div>
                                    <section
                                        id="tau-ops-harness-route-action"
                                        data-route-action-key=context.harness.route_action_key.clone()
                                        data-route-action-label=context.harness.route_action_label.clone()
                                        data-route-action-count=context.harness.route_action_count
                                        data-route-action-visible=harness_route_action_visible
                                        hidden=harness_route_action_hidden
                                        aria-live="polite"
                                    >
                                        <strong>{context.harness.route_action_label.clone()}</strong>
                                        <span>{context.harness.route_action_detail.clone()}</span>
                                    </section>
                                </div>
                                <nav aria-label="Mission harness actions">
                                    <form
                                        id="tau-ops-harness-new-mission-form"
                                        action=harness_new_mission_action
                                        method="post"
                                        data-action-contract="durable-mission-draft"
                                        data-preserves-shell-context="true"
                                    >
                                    <button
                                        id="tau-ops-harness-new-mission"
                                        data-action="new-mission"
                                        data-action-contract="durable-mission-draft"
                                        data-preserves-session="true"
                                        data-preserves-proposal="true"
                                        type="submit"
                                    >
                                        "New Mission"
                                    </button>
                                    </form>
                                    <a
                                        id="tau-ops-harness-history"
                                        data-action="history"
                                        data-action-contract="context-preserving"
                                        data-preserves-session="true"
                                        data-preserves-proposal="true"
                                        href=harness_history_href
                                    >
                                        "History"
                                    </a>
                                </nav>
                            </header>
                            {harness_history_view}
                            <section
                                id="tau-ops-harness-dashboard-window"
                                data-window="mission-harness-dashboard"
                                data-window-order="1"
                                data-window-chrome="compact"
                                data-compact-dashboard-breakpoint="1400px"
                            >
                                <header class="tau-harness-window-titlebar">
                                    <div>
                                        <small>"Dashboard"</small>
                                        <h3>"Mission Harness Dashboard"</h3>
                                    </div>
                                    <span class="tau-harness-status-chip">"Healthy"</span>
                                </header>
                                <section id="tau-ops-harness-kpi-grid" data-kpi-card-count="4" data-kpi-label-fit="word-boundary" data-kpi-label-overflow-budget="none">
                                    <article id="tau-ops-harness-kpi-active" data-harness-kpi-card="active-missions" data-kpi-value=harness_kpi_missions_count.clone() data-kpi-heading-fit="word-boundary">
                                        <h4>{context.harness.kpi_missions_title.clone()}</h4>
                                        <p>{context.harness.kpi_missions_count}</p>
                                        <small>{context.harness.kpi_missions_detail.clone()}</small>
                                    </article>
                                    <article id="tau-ops-harness-kpi-verifications" data-harness-kpi-card="pending-verifications" data-kpi-value=harness_kpi_pending_verification_count data-kpi-heading-fit="word-boundary">
                                        <h4 aria-label="Pending Verifications"><span>"Pending"</span><span>"Verifications"</span></h4>
                                        <p>{context.harness.kpi_pending_verification_count}</p>
                                        <small>{context.harness.kpi_pending_verification_detail.clone()}</small>
                                    </article>
                                    <article id="tau-ops-harness-kpi-memory" data-harness-kpi-card="memory-writes" data-kpi-value=harness_kpi_memory_write_count data-kpi-heading-fit="word-boundary">
                                        <h4>"Memory Writes"</h4>
                                        <p>{context.harness.kpi_memory_write_count}</p>
                                        <small>{context.harness.kpi_memory_write_detail.clone()}</small>
                                    </article>
                                    <article id="tau-ops-harness-kpi-cost" data-harness-kpi-card="runtime-cost-today" data-kpi-value=context.harness.kpi_runtime_cost_today.clone() data-kpi-heading-fit="word-boundary">
                                        <h4>"Runtime Cost Today"</h4>
                                        <p>{context.harness.kpi_runtime_cost_today.clone()}</p>
                                        <small>{context.harness.kpi_runtime_cost_detail.clone()}</small>
                                    </article>
                                </section>
                                <section
                                    id="tau-ops-harness-active-missions"
                                    data-active-count=harness_kpi_missions_count
                                    data-running-count=context.harness.mission_rows.iter().filter(|row| matches!(row.status_key.as_str(), "running" | "verifying")).count().to_string()
                                    data-blocked-count=context.harness.mission_rows.iter().filter(|row| row.status_key == "blocked").count().to_string()
                                    data-compact-table-breakpoint="1400px"
                                    data-compact-mission-summary="status-and-gates"
                                    data-first-viewport-budget="benchmark-visible"
                                    data-active-mission-scroll-boundary="whole-row"
                                    data-active-mission-visible-rows="3"
                                    data-left-table-fit="compact-no-overflow"
                                    data-horizontal-overflow-budget="none"
                                >
                                    <h4>{context.harness.mission_table_title.clone()}</h4>
                                    <div class="tau-harness-table-wrap" data-scroll-region="active-missions" data-scroll-boundary="whole-row">
                                        <table id="tau-ops-harness-missions-table">
                                            <thead>
                                                <tr>
                                                    <th scope="col">"Goal"</th>
                                                    <th scope="col">"Acceptance"</th>
                                                    <th scope="col">"Plan"</th>
                                                    <th scope="col">"Tool Budget"</th>
                                                    <th scope="col">"Memory Hits"</th>
                                                    <th scope="col">"Verification"</th>
                                                    <th scope="col">"Last Checkpoint"</th>
                                                    <th scope="col">"Artifacts"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {harness_mission_rows}
                                            </tbody>
                                        </table>
                                    </div>
                                </section>
                                <section
                                    id="tau-ops-harness-benchmark-panel"
                                    data-benchmark-id=context.harness.benchmark_id.clone()
                                    data-proof-artifact=context.harness.proof_artifact.clone()
                                    data-task-count=harness_task_count
                                    data-pass-count=harness_pass_count
                                    data-failed-gates=context.harness.failed_gate_label.clone()
                                    data-proof-source=context.harness.proof_source.clone()
                                    data-first-viewport-anchor="canonical-benchmark"
                                    data-left-table-fit="compact-no-overflow"
                                    data-horizontal-overflow-budget="none"
                                    data-category-label-fit="operator-readable"
                                    data-category-overflow-budget="none"
                                >
                                    <h4>"M334 Autonomy Benchmark"</h4>
                                    <div class="tau-harness-table-wrap">
                                        <table id="tau-ops-harness-benchmark-table">
                                            <tbody>
                                                {harness_benchmark_rows}
                                            </tbody>
                                        </table>
                                    </div>
                                    <p
                                        id="tau-ops-harness-benchmark-latest"
                                        data-latest-result=context.harness.latest_result.clone()
                                        data-runtime=context.harness.latest_runtime.clone()
                                        data-cost=context.harness.latest_cost.clone()
                                    >
                                        {context.harness.latest_summary.clone()}
                                    </p>
                                    <section
                                        id="tau-ops-harness-benchmark-proof"
                                        data-benchmark-id=context.harness.benchmark_id.clone()
                                        data-proof-artifact=context.harness.proof_artifact.clone()
                                        data-proof-source=context.harness.proof_source.clone()
                                        data-latest-result=context.harness.latest_result.clone()
                                        data-task-count=harness_task_count.clone()
                                        data-pass-count=harness_pass_count.clone()
                                        data-failed-gates=context.harness.failed_gate_label.clone()
                                        data-proof-visible="true"
                                    >
                                        <h5>"Benchmark Proof"</h5>
                                        <dl>
                                            <div data-proof-metric="benchmark-id">
                                                <dt>"Benchmark"</dt>
                                                <dd>{context.harness.benchmark_id.clone()}</dd>
                                            </div>
                                            <div data-proof-metric="proof-artifact">
                                                <dt>"Proof artifact"</dt>
                                                <dd><code>{context.harness.proof_artifact.clone()}</code></dd>
                                            </div>
                                            <div data-proof-metric="latest-result">
                                                <dt>"Result"</dt>
                                                <dd>{context.harness.latest_result.clone()}</dd>
                                            </div>
                                            <div data-proof-metric="failed-gates">
                                                <dt>"Failed gates"</dt>
                                                <dd>{context.harness.failed_gate_label.clone()}</dd>
                                            </div>
                                        </dl>
                                    </section>
                                    <form
                                        id="tau-ops-harness-run-benchmark-form"
                                        action=harness_benchmark_action
                                        method="post"
                                        data-command="tau_agent_harness"
                                        data-preserves-shell-context="true"
                                    >
                                        <button id="tau-ops-harness-run-benchmark" type="submit" data-action-tone="benchmark">"Run Benchmark"</button>
                                    </form>
                                </section>
                            </section>
                            <section
                                id="tau-ops-harness-proof-window"
                                data-window="mission-detail-proof-view"
                                data-window-order="2"
                                data-run-id=context.harness.detail_run_id.clone()
                                data-mission-status=context.harness.detail_status.clone()
                                data-tool-budget=context.harness.detail_tool_budget.clone()
                                data-detail-proof-artifact=context.harness.detail_proof_artifact.clone()
                                data-window-chrome="compact"
                                data-narrow-proof-fit="no-hidden-overflow"
                            >
                                <header
                                    id="tau-ops-harness-proof-header"
                                    class="tau-harness-window-titlebar"
                                    data-compact-metadata-breakpoint="1400px"
                                    data-metadata-fit="no-wrap"
                                    data-run-id-wrap="single-line"
                                    data-metadata-value-overflow-budget="none"
                                >
                                    <div>
                                        <p>"Goal"</p>
                                        <h3>{context.harness.detail_goal.clone()}</h3>
                                    </div>
                                    <dl>
                                        <dt>"Run ID"</dt><dd>{context.harness.detail_run_id.clone()}</dd>
                                        <dt>"Elapsed"</dt><dd>{context.harness.detail_elapsed.clone()}</dd>
                                        <dt>"Tool Budget"</dt><dd>{context.harness.detail_tool_budget.clone()}</dd>
                                        <dt>"Cost"</dt><dd>{context.harness.detail_cost.clone()}</dd>
                                        <dt>"Retry Count"</dt><dd>{context.harness.detail_retry_count.clone()}</dd>
                                    </dl>
                                </header>
                                {harness_selected_mission_actions}
                                <ol
                                    id="tau-ops-harness-plan-dag"
                                    data-dag-node-count=context.harness.detail_plan_rows.len().to_string()
                                    data-current-node=context.harness.detail_plan_current_node.clone()
                                    data-proof-dag-density="single-row"
                                >
                                    {harness_detail_plan_rows}
                                </ol>
                                <div class="tau-harness-window-grid" data-proof-grid-priority="evidence-log-gates-first">
                                    <section
                                        id="tau-ops-harness-tool-evidence"
                                        data-tool-call-count=harness_detail_tool_call_count
                                        data-compact-evidence-breakpoint="1400px"
                                        data-compact-call-id-visibility="hidden-at-1400px"
                                        data-proof-evidence-priority="first-screen"
                                        data-tool-evidence-fit="compact-no-overflow"
                                        data-tool-evidence-overflow-budget="none"
                                        data-tool-evidence-visible-columns="tool,plan-node,runtime,status,artifact"
                                        data-tool-label-fit="full-memory-tool-names"
                                        data-tool-column-overflow-budget="none"
                                    >
                                        <h4>"Tool Execution Evidence"</h4>
                                        <div class="tau-harness-table-wrap">
                                            <table>
                                                <thead><tr><th scope="col">"Tool"</th><th scope="col">"Call ID"</th><th scope="col">"Plan Node"</th><th scope="col">"Runtime"</th><th scope="col">"Status"</th><th scope="col">"Artifact"</th></tr></thead>
                                                <tbody>
                                                    {harness_detail_tool_rows}
                                                </tbody>
                                            </table>
                                        </div>
                                    </section>
                                    <section
                                        id="tau-ops-harness-operator-log"
                                        data-log-follow="true"
                                        data-log-wrap="pre-wrap"
                                        data-log-priority="first-screen"
                                    >
                                        <h4>"Operator Log"</h4>
                                        <pre>{context.harness.detail_operator_log.clone()}</pre>
                                    </section>
                                    <section
                                        id="tau-ops-harness-acceptance"
                                        data-acceptance-met=harness_detail_acceptance_met
                                        data-acceptance-total=harness_detail_acceptance_total
                                        data-proof-detail-budget="compact-scroll"
                                        data-acceptance-overflow-budget="all-criteria-visible"
                                        data-acceptance-layout="compact-contained"
                                        data-narrow-label-fit="full-labels-at-1400px"
                                    >
                                        <h4>"Acceptance Criteria"</h4>
                                        <ul>
                                            {harness_detail_acceptance_rows}
                                        </ul>
                                    </section>
                                    <section
                                        id="tau-ops-harness-verification-gates"
                                        data-gate-count=harness_detail_gate_count
                                        data-failed-gate-count=harness_detail_failed_gate_count
                                        data-proof-secondary-priority="first-screen"
                                        data-proof-detail-budget="compact-scroll"
                                        data-gate-visibility="all-gates-first-viewport"
                                        data-gate-layout="two-column-compact"
                                        data-narrow-height-budget="no-hidden-overflow"
                                    >
                                        <h4>"Verification Gates"</h4>
                                        <ul>
                                            {harness_detail_gate_rows}
                                        </ul>
                                    </section>
                                    <section
                                        id="tau-ops-harness-memory-learning"
                                        data-memory-hits=harness_detail_memory_hits
                                        data-learning-records=harness_detail_learning_records
                                        data-last-memory-write=context.harness.detail_last_memory_write.clone()
                                        data-proof-footer-priority="first-viewport"
                                    >
                                        <h4>"Memory / Learning"</h4>
                                        <p>{format!("Memory hits: {}", context.harness.detail_memory_hit_count)}</p>
                                        <p>{format!("No-memory evidence: {}", context.harness.detail_memory_evidence_label)}</p>
                                        <p>{format!("Learning records: {}", context.harness.detail_learning_record_count)}</p>
                                    </section>
                                    <section
                                        id="tau-ops-harness-artifacts"
                                        data-artifact-count=harness_detail_artifact_count
                                        data-proof-footer-priority="first-viewport"
                                    >
                                        <h4>"Artifacts"</h4>
                                        <ul>
                                            {harness_detail_artifact_rows}
                                        </ul>
                                    </section>
                                </div>
                            </section>
                            <section
                                id="tau-ops-harness-self-improvement-window"
                                data-window="self-improvement-review-apply-flow"
                                data-window-order="3"
                                data-selected-proposal=context.harness.selected_proposal_id.clone()
                                data-approval-gated="true"
                                data-window-chrome="compact"
                                data-review-action-placement="actions-before-detail"
                                data-review-audit-priority="first-viewport-recent-history"
                                data-review-density="audit-visible"
                                data-review-overflow-contract="contained-proof-rows"
                            >
                                <header class="tau-harness-window-titlebar">
                                    <div>
                                        <small>"Review"</small>
                                        <h3>"Self-Improvement Review / Apply Flow"</h3>
                                    </div>
                                    <span class="tau-harness-status-chip">"Approval gated"</span>
                                </header>
                                <section
                                    id="tau-ops-harness-learning-queue"
                                    data-queue-count=harness_proposal_queue_count
                                    data-queue-density="all-items-visible"
                                    data-queue-overflow-budget="none"
                                    data-queue-readability="full-labels"
                                    data-queue-layout="single-column-readable"
                                    data-queue-truncation-budget="none"
                                    data-queue-navigation="proposal-links"
                                    data-queue-source=context.harness.proposal_queue_source.clone()
                                >
                                    <h4>"Learning & Proposals"</h4>
                                    <ul>
                                        {harness_proposal_queue_rows}
                                    </ul>
                                </section>
                                <section
                                    id="tau-ops-harness-operator-actions"
                                    data-apply-requires-approval="true"
                                    data-action-row-priority="approval-flow"
                                    data-action-grid="two-column-priority"
                                    data-action-first-viewport="all-controls"
                                    data-review-state=harness_selected_apply_state
                                    data-terminal-state=harness_selected_review_terminal_attr
                                    data-selected-proposal=harness_selected_proposal_id.clone()
                                >
                                    <h4>"Operator Actions"</h4>
                                    <section
                                        id="tau-ops-harness-operator-action-state"
                                        data-review-state=harness_selected_apply_state
                                        data-terminal-state=harness_selected_review_terminal_attr
                                        data-selected-proposal=harness_selected_proposal_id.clone()
                                        data-action-state-visible="true"
                                    >
                                        <h5>"Decision State"</h5>
                                        <p>{harness_selected_decision_label}</p>
                                        <small>{harness_selected_decision_detail}</small>
                                    </section>
                                    <form id="tau-ops-harness-approve-form" action=harness_selected_approve_action method="post" data-preserves-shell-context="true">
                                        <input id="tau-ops-harness-approve-theme" type="hidden" name="theme" value=theme_attr />
                                        <input id="tau-ops-harness-approve-sidebar" type="hidden" name="sidebar" value=sidebar_state_attr />
                                        <input id="tau-ops-harness-approve-session" type="hidden" name="session" value=context.chat.active_session_key.clone() />
                                        <button
                                            id="tau-ops-harness-action-approve"
                                            type="submit"
                                            data-action="approve"
                                            data-action-tone="approve"
                                            data-action-state=harness_selected_apply_state
                                            data-disabled=harness_selected_decision_control_disabled
                                            aria-disabled=harness_selected_decision_control_disabled
                                            disabled=harness_selected_review_terminal
                                        >
                                            "Approve"
                                        </button>
                                    </form>
                                    <form id="tau-ops-harness-reject-form" action=harness_selected_reject_action method="post" data-preserves-shell-context="true">
                                        <input id="tau-ops-harness-reject-theme" type="hidden" name="theme" value=theme_attr />
                                        <input id="tau-ops-harness-reject-sidebar" type="hidden" name="sidebar" value=sidebar_state_attr />
                                        <input id="tau-ops-harness-reject-session" type="hidden" name="session" value=context.chat.active_session_key.clone() />
                                        <button
                                            id="tau-ops-harness-action-reject"
                                            type="submit"
                                            data-action="reject"
                                            data-action-tone="reject"
                                            data-action-state=harness_selected_apply_state
                                            data-disabled=harness_selected_decision_control_disabled
                                            aria-disabled=harness_selected_decision_control_disabled
                                            disabled=harness_selected_review_terminal
                                        >
                                            "Reject"
                                        </button>
                                    </form>
                                    <form id="tau-ops-harness-dry-run-form" action=harness_selected_dry_run_action method="post" data-preserves-shell-context="true">
                                        <input id="tau-ops-harness-dry-run-theme" type="hidden" name="theme" value=theme_attr />
                                        <input id="tau-ops-harness-dry-run-sidebar" type="hidden" name="sidebar" value=sidebar_state_attr />
                                        <input id="tau-ops-harness-dry-run-session" type="hidden" name="session" value=context.chat.active_session_key.clone() />
                                        <button id="tau-ops-harness-action-dry-run" type="submit" data-action="dry-run" data-action-tone="secondary">"Dry Run Again"</button>
                                    </form>
                                    <a id="tau-ops-harness-action-view-diff" data-action="view-diff" data-action-tone="secondary" href=harness_selected_diff_href>"View Diff"</a>
                                    <form id="tau-ops-harness-apply-form" action=harness_selected_apply_action method="post" data-approval-state=harness_selected_apply_state data-preserves-shell-context="true">
                                        <input id="tau-ops-harness-apply-theme" type="hidden" name="theme" value=theme_attr />
                                        <input id="tau-ops-harness-apply-sidebar" type="hidden" name="sidebar" value=sidebar_state_attr />
                                        <input id="tau-ops-harness-apply-session" type="hidden" name="session" value=context.chat.active_session_key.clone() />
                                        <button
                                            id="tau-ops-harness-action-apply"
                                            type="submit"
                                            data-action="apply"
                                            data-action-tone="disabled"
                                            data-disabled=harness_selected_apply_disabled
                                            aria-disabled=harness_selected_apply_aria_disabled
                                            data-approval-required="true"
                                            disabled=!harness_selected_apply_enabled
                                        >
                                            {harness_selected_apply_label}
                                        </button>
                                    </form>
                                </section>
                                <section
                                    id="tau-ops-harness-conservative-policy"
                                    data-policy="conservative-self-improvement"
                                    data-allowed-targets="skill,config,prompt"
                                    data-blocked-targets="source-code,safety-policy"
                                    data-review-policy-priority="first-viewport"
                                >
                                    <h4>"Conservative Change Policy"</h4>
                                    <div id="tau-ops-harness-policy-allowed" data-policy-side="allowed">
                                        <h5>"Allowed"</h5>
                                        <ul><li>"Skill"</li><li>"Config"</li><li>"Prompt"</li></ul>
                                    </div>
                                    <div id="tau-ops-harness-policy-blocked" data-policy-side="blocked">
                                        <h5>"Blocked"</h5>
                                        <ul><li>"Source Code"</li><li>"Safety Policy"</li></ul>
                                    </div>
                                </section>
                                <section
                                    id="tau-ops-harness-proposal-detail"
                                    data-proposal-id=context.harness.selected_proposal.proposal_id.clone()
                                    data-learning-record=context.harness.selected_proposal.learning_record_id.clone()
                                    data-target-type=context.harness.selected_proposal.target_type.clone()
                                    data-target-path=context.harness.selected_proposal.target_path.clone()
                                    data-proposal-detail-priority="first-viewport-summary"
                                    data-proposal-detail-density="compact-scroll"
                                    data-proposal-detail-overflow-budget="contained"
                                    data-proposal-visible-rows="7"
                                    data-proposal-summary-fit="full-text"
                                    data-proposal-summary-overflow-budget="none"
                                    data-proposal-detail-vertical-overflow-budget="none"
                                    data-proposal-detail-max-height="132px"
                                >
                                    <h4>{format!("{} {}", context.harness.selected_proposal.proposal_id, context.harness.selected_proposal.title)}</h4>
                                    <dl>
                                        <dt>"Dry-run Result"</dt><dd data-result=context.harness.selected_proposal.dry_run_result_key.clone()>{context.harness.selected_proposal.dry_run_result_label.clone()}</dd>
                                        <dt>"Safety Check"</dt><dd data-result=context.harness.selected_proposal.safety_check_key.clone()>{context.harness.selected_proposal.safety_check_label.clone()}</dd>
                                        <dt>"Rollback Plan"</dt><dd>{context.harness.selected_proposal.rollback_plan.clone()}</dd>
                                        <dt>"Patch Summary"</dt><dd data-proposal-row="patch-summary" data-summary-fit="full-text">{context.harness.selected_proposal.patch_summary.clone()}</dd>
                                        <dt>"Failure Observed"</dt><dd>{context.harness.selected_proposal.failure_observed.clone()}</dd>
                                        <dt>"Root Cause"</dt><dd>{context.harness.selected_proposal.root_cause.clone()}</dd>
                                        <dt>"Test Evidence"</dt><dd><a href=context.harness.selected_proposal.test_evidence_href.clone()>{context.harness.selected_proposal.test_evidence_label.clone()}</a></dd>
                                    </dl>
                                </section>
                                {harness_self_improvement_proof}
                                <section
                                    id="tau-ops-harness-audit-log"
                                    data-audit-row-count=harness_audit_row_count
                                    data-audit-source=context.harness.audit_source.clone()
                                    data-audit-priority="first-viewport-recent-proof"
                                    data-audit-density="compact-scroll"
                                    data-audit-visible-columns="time,action,item,result"
                                    data-audit-overflow-budget="all-rows-visible"
                                >
                                    <h4>"Audit Log (Recent)"</h4>
                                    <div class="tau-harness-table-wrap">
                                        <table>
                                            <tbody>
                                                {harness_audit_rows}
                                            </tbody>
                                        </table>
                                    </div>
                                </section>
                            </section>
                            <aside
                                id="tau-ops-harness-tui-companion"
                                data-component="TuiCompanion"
                                data-command="tau status"
                                data-window-chrome="compact"
                                data-log-wrap="pre-wrap"
                                data-tui-priority="first-viewport-summary"
                            >
                                <header class="tau-harness-window-titlebar">
                                    <div>
                                        <small>"Terminal"</small>
                                        <h3>"TUI Companion"</h3>
                                    </div>
                                    <span class="tau-harness-status-chip">"Connected"</span>
                                </header>
                                <pre>{harness_tui_summary}</pre>
                            </aside>
                            <script
                                id="tau-ops-harness-preview-guard"
                                data-preview-submit-guard="file-protocol-post-forms"
                            >
                                r#"
                                (function () {
                                    if (window.location.protocol !== "file:") {
                                        return;
                                    }

                                    document.addEventListener("submit", function (event) {
                                        var panel = document.getElementById("tau-ops-harness-panel");
                                        var form = event.target;
                                        if (!panel || !(form instanceof HTMLFormElement) || !panel.contains(form)) {
                                            return;
                                        }
                                        if ((form.method || "").toLowerCase() !== "post") {
                                            return;
                                        }

                                        event.preventDefault();
                                        form.setAttribute("data-preview-submit-blocked", "true");
                                        var status = document.getElementById("tau-ops-harness-preview-status");
                                        if (status) {
                                            status.setAttribute("data-preview-status", "blocked-post");
                                            status.textContent = "Preview submission blocked for local file review.";
                                        }
                                    });
                                })();
                                "#
                            </script>
                        </section>
                        <section
                            id="tau-ops-command-center"
                            data-route="/ops"
                            aria-hidden=command_center_panel_hidden
                            aria-live="polite"
                        >
                            <section id="tau-ops-kpi-grid" data-kpi-card-count="6">
                                <article
                                    id="tau-ops-kpi-health"
                                    data-component="HealthBadge"
                                    data-health-state=health_state
                                    data-health-reason=health_reason
                                >
                                    <h2>System Health</h2>
                                    <p id="tau-ops-health-state-value">{context.command_center.health_state.clone()}</p>
                                    <p id="tau-ops-health-reason-value">{context.command_center.health_reason.clone()}</p>
                                </article>
                                <article id="tau-ops-kpi-queue-depth" data-component="StatCard" data-kpi-card="queue-depth" data-kpi-value=queue_depth_value>
                                    <h2>Queue Depth</h2>
                                    <p>{context.command_center.queue_depth}</p>
                                </article>
                                <article id="tau-ops-kpi-failure-streak" data-component="StatCard" data-kpi-card="failure-streak" data-kpi-value=failure_streak_value>
                                    <h2>Failure Streak</h2>
                                    <p>{context.command_center.failure_streak}</p>
                                </article>
                                <article id="tau-ops-kpi-processed-cases" data-component="StatCard" data-kpi-card="processed-cases" data-kpi-value=processed_cases_value>
                                    <h2>Processed Cases</h2>
                                    <p>{context.command_center.processed_case_count}</p>
                                </article>
                                <article id="tau-ops-kpi-alert-count" data-component="StatCard" data-kpi-card="alert-count" data-kpi-value=alert_count_value>
                                    <h2>Alert Count</h2>
                                    <p>{context.command_center.alert_count}</p>
                                </article>
                                <article id="tau-ops-kpi-widget-count" data-component="StatCard" data-kpi-card="widget-count" data-kpi-value=widget_count_value>
                                    <h2>Widget Count</h2>
                                    <p>{context.command_center.widget_count}</p>
                                </article>
                                <article id="tau-ops-kpi-timeline-cycles" data-component="StatCard" data-kpi-card="timeline-cycles" data-kpi-value=timeline_cycle_count_value>
                                    <h2>Timeline Cycles</h2>
                                    <p>{context.command_center.timeline_cycle_count}</p>
                                </article>
                            </section>
                            <section
                                id="tau-ops-queue-timeline-chart"
                                data-component="TimelineChart"
                                data-timeline-range=timeline_range.clone()
                                data-timeline-point-count=timeline_point_count_value
                                data-timeline-last-timestamp=timeline_last_timestamp_value
                            >
                                <h2>Queue Timeline</h2>
                                <section
                                    id="tau-ops-timeline-range-controls"
                                    role="group"
                                    aria-label="Timeline range"
                                >
                                    <a
                                        id="tau-ops-timeline-range-1h"
                                        data-range-option="1h"
                                        data-range-selected=range_1h_selected
                                        href=range_1h_href
                                    >
                                        1h
                                    </a>
                                    <a
                                        id="tau-ops-timeline-range-6h"
                                        data-range-option="6h"
                                        data-range-selected=range_6h_selected
                                        href=range_6h_href
                                    >
                                        6h
                                    </a>
                                    <a
                                        id="tau-ops-timeline-range-24h"
                                        data-range-option="24h"
                                        data-range-selected=range_24h_selected
                                        href=range_24h_href
                                    >
                                        24h
                                    </a>
                                </section>
                            </section>
                            <section
                                id="tau-ops-control-panel"
                                data-component="ControlPanel"
                                data-control-mode=control_mode
                                data-rollout-gate=rollout_gate
                                data-control-paused=control_paused_value
                            >
                                <h2>Control State</h2>
                                <section
                                    id="tau-ops-control-actions"
                                    data-action-count="3"
                                    data-action-endpoint="/ops/control-action"
                                >
                                    <form
                                        id="tau-ops-control-action-form-pause"
                                        action="/ops/control-action"
                                        method="post"
                                        data-action="pause"
                                    >
                                        <input
                                            id="tau-ops-control-action-pause-value"
                                            type="hidden"
                                            name="action"
                                            value="pause"
                                        />
                                        <input
                                            id="tau-ops-control-action-pause-reason"
                                            type="hidden"
                                            name="reason"
                                            value="ops-shell-control-panel"
                                        />
                                        <input
                                            id="tau-ops-control-action-pause-theme"
                                            type="hidden"
                                            name="theme"
                                            value=theme_attr
                                        />
                                        <input
                                            id="tau-ops-control-action-pause-sidebar"
                                            type="hidden"
                                            name="sidebar"
                                            value=sidebar_state_attr
                                        />
                                        <button
                                            id="tau-ops-control-action-pause"
                                            data-action-enabled=action_pause_enabled_value
                                            data-action="pause"
                                            data-confirm-required="true"
                                            data-confirm-title="Confirm pause action"
                                            data-confirm-body="Pause command-center processing until resumed."
                                            data-confirm-verb="pause"
                                            type="submit"
                                        >
                                            Pause
                                        </button>
                                    </form>
                                    <form
                                        id="tau-ops-control-action-form-resume"
                                        action="/ops/control-action"
                                        method="post"
                                        data-action="resume"
                                    >
                                        <input
                                            id="tau-ops-control-action-resume-value"
                                            type="hidden"
                                            name="action"
                                            value="resume"
                                        />
                                        <input
                                            id="tau-ops-control-action-resume-reason"
                                            type="hidden"
                                            name="reason"
                                            value="ops-shell-control-panel"
                                        />
                                        <input
                                            id="tau-ops-control-action-resume-theme"
                                            type="hidden"
                                            name="theme"
                                            value=theme_attr
                                        />
                                        <input
                                            id="tau-ops-control-action-resume-sidebar"
                                            type="hidden"
                                            name="sidebar"
                                            value=sidebar_state_attr
                                        />
                                        <button
                                            id="tau-ops-control-action-resume"
                                            data-action-enabled=action_resume_enabled_value
                                            data-action="resume"
                                            data-confirm-required="true"
                                            data-confirm-title="Confirm resume action"
                                            data-confirm-body="Resume command-center processing."
                                            data-confirm-verb="resume"
                                            type="submit"
                                        >
                                            Resume
                                        </button>
                                    </form>
                                    <form
                                        id="tau-ops-control-action-form-refresh"
                                        action="/ops/control-action"
                                        method="post"
                                        data-action="refresh"
                                    >
                                        <input
                                            id="tau-ops-control-action-refresh-value"
                                            type="hidden"
                                            name="action"
                                            value="refresh"
                                        />
                                        <input
                                            id="tau-ops-control-action-refresh-reason"
                                            type="hidden"
                                            name="reason"
                                            value="ops-shell-control-panel"
                                        />
                                        <input
                                            id="tau-ops-control-action-refresh-theme"
                                            type="hidden"
                                            name="theme"
                                            value=theme_attr
                                        />
                                        <input
                                            id="tau-ops-control-action-refresh-sidebar"
                                            type="hidden"
                                            name="sidebar"
                                            value=sidebar_state_attr
                                        />
                                        <button
                                            id="tau-ops-control-action-refresh"
                                            data-action-enabled=action_refresh_enabled_value
                                            data-action="refresh"
                                            data-confirm-required="true"
                                            data-confirm-title="Confirm refresh action"
                                            data-confirm-body="Refresh command-center state from latest runtime artifacts."
                                            data-confirm-verb="refresh"
                                            type="submit"
                                        >
                                            Refresh
                                        </button>
                                    </form>
                                </section>
                                <section
                                    id="tau-ops-control-action-status"
                                    data-control-action-status=control_action_status
                                    data-control-action=control_action
                                    data-control-action-reason=control_action_reason
                                >
                                    <h3>Action Submit Status</h3>
                                    <p id="tau-ops-control-action-status-message">
                                        {control_action_status_message}
                                    </p>
                                </section>
                                <section
                                    id="tau-ops-control-last-action"
                                    data-last-action-request-id=last_action_request_id
                                    data-last-action-name=last_action_name
                                    data-last-action-actor=last_action_actor
                                    data-last-action-reason=last_action_reason
                                    data-last-action-timestamp=last_action_timestamp_value
                                >
                                    <h3>Last Action</h3>
                                    <p id="tau-ops-last-action-request-id">{format!("request.id: {last_action_request_id}")}</p>
                                    <p id="tau-ops-last-action-name">{format!("action: {last_action_name}")}</p>
                                    <p id="tau-ops-last-action-actor">{format!("actor: {last_action_actor}")}</p>
                                    <p id="tau-ops-last-action-reason">{format!("reason: {last_action_reason}")}</p>
                                    <p id="tau-ops-last-action-timestamp">{format!("timestamp: {last_action_timestamp_value}")}</p>
                                </section>
                            </section>
                            <section
                                id="tau-ops-alert-feed"
                                data-component="AlertFeed"
                                data-alert-count=alert_count_feed_value
                                data-primary-alert-code=primary_alert_code
                                data-primary-alert-severity=primary_alert_severity
                                data-alert-row-count=alert_row_count_section_value
                            >
                                <h2>Alerts</h2>
                                <p id="tau-ops-primary-alert-message">{primary_alert_message}</p>
                                <ul id="tau-ops-alert-feed-list" data-alert-row-count=alert_row_count_list_value>
                                    {alert_feed_rows
                                        .iter()
                                        .enumerate()
                                        .map(|(index, alert_row)| {
                                            let alert_row_id = format!("tau-ops-alert-row-{index}");
                                            view! {
                                                <li
                                                    id=alert_row_id
                                                    data-alert-code=alert_row.code.clone()
                                                    data-alert-severity=alert_row.severity.clone()
                                                >
                                                    {alert_row.message.clone()}
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                </ul>
                            </section>
                            <section
                                id="tau-ops-connector-health-table"
                                data-component="ConnectorHealthTable"
                                data-connector-row-count=connector_row_count_table_value
                            >
                                <h2>Connector Health</h2>
                                <table>
                                    <thead>
                                        <tr>
                                            <th scope="col">Channel</th>
                                            <th scope="col">Mode</th>
                                            <th scope="col">Liveness</th>
                                            <th scope="col">Events Ingested</th>
                                            <th scope="col">Provider Failures</th>
                                        </tr>
                                    </thead>
                                    <tbody
                                        id="tau-ops-connector-table-body"
                                        data-connector-row-count=connector_row_count_body_value
                                    >
                                        {connector_health_rows
                                            .iter()
                                            .enumerate()
                                            .map(|(index, connector_row)| {
                                                let connector_row_id =
                                                    format!("tau-ops-connector-row-{index}");
                                                let events_ingested_value =
                                                    connector_row.events_ingested.to_string();
                                                let provider_failures_value =
                                                    connector_row.provider_failures.to_string();
                                                view! {
                                                    <tr
                                                        id=connector_row_id
                                                        data-channel=connector_row.channel.clone()
                                                        data-mode=connector_row.mode.clone()
                                                        data-liveness=connector_row.liveness.clone()
                                                        data-events-ingested=events_ingested_value
                                                        data-provider-failures=provider_failures_value
                                                    >
                                                        <td>{connector_row.channel.clone()}</td>
                                                        <td>{connector_row.mode.clone()}</td>
                                                        <td>{connector_row.liveness.clone()}</td>
                                                        <td>{connector_row.events_ingested}</td>
                                                        <td>{connector_row.provider_failures}</td>
                                                    </tr>
                                                }
                                            })
                                            .collect_view()}
                                    </tbody>
                                </table>
                            </section>
                        <section
                            id="tau-ops-data-table"
                                data-route="/ops"
                                data-timeline-range=timeline_range
                                data-component="DataTable"
                                data-timeline-cycle-count=timeline_cycle_count_table_value
                                data-timeline-invalid-cycle-count=timeline_invalid_cycle_count_value
                            >
                                <h2>Recent Cycles</h2>
                                <table>
                                    <thead>
                                        <tr>
                                            <th scope="col">Last Timestamp</th>
                                            <th scope="col">Point Count</th>
                                            <th scope="col">Cycle Reports</th>
                                            <th scope="col">Invalid Reports</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <tr
                                            id="tau-ops-timeline-summary-row"
                                            data-row-kind="summary"
                                            data-last-timestamp=timeline_last_timestamp_table_value
                                            data-point-count=timeline_point_count_table_value
                                            data-cycle-count=timeline_cycle_count_summary_value
                                            data-invalid-cycle-count=timeline_invalid_cycle_count_summary_value
                                        >
                                            <td>{context.command_center.timeline_last_timestamp_unix_ms}</td>
                                            <td>{context.command_center.timeline_point_count}</td>
                                            <td>{context.command_center.timeline_cycle_count}</td>
                                            <td>{context.command_center.timeline_invalid_cycle_count}</td>
                                        </tr>
                                        {timeline_empty_row}
                                    </tbody>
                                </table>
                            </section>
                        </section>
                        <section
                            id="tau-ops-deploy-panel"
                            data-route="/ops/deploy"
                            data-component="DeployWizard"
                            aria-hidden=deploy_panel_hidden
                            data-panel-visible=deploy_panel_visible
                        >
                            <h2>Deploy Agent</h2>
                            <nav
                                id="tau-ops-deploy-wizard-steps"
                                data-component="DeployWizardSteps"
                                aria-label="Deploy wizard steps"
                            >
                                <ol>
                                    <li>
                                        <button
                                            type="button"
                                            data-wizard-step="model"
                                            data-step-index="1"
                                        >
                                            "1. Model"
                                        </button>
                                    </li>
                                    <li>
                                        <button
                                            type="button"
                                            data-wizard-step="configuration"
                                            data-step-index="2"
                                        >
                                            "2. Configuration"
                                        </button>
                                    </li>
                                    <li>
                                        <button
                                            type="button"
                                            data-wizard-step="validation"
                                            data-step-index="3"
                                        >
                                            "3. Validation"
                                        </button>
                                    </li>
                                    <li>
                                        <button
                                            type="button"
                                            data-wizard-step="review"
                                            data-step-index="4"
                                        >
                                            "4. Review"
                                        </button>
                                    </li>
                                </ol>
                            </nav>
                            <section id="tau-ops-deploy-model-selection">
                                <label for="tau-ops-deploy-model-catalog">Model Catalog</label>
                                <select
                                    id="tau-ops-deploy-model-catalog"
                                    data-component="ModelCatalogDropdown"
                                >
                                    <option value="gpt-4.1-mini">gpt-4.1-mini</option>
                                    <option value="gpt-4.1">gpt-4.1</option>
                                </select>
                            </section>
                            <section
                                id="tau-ops-deploy-validation"
                                data-component="StepValidation"
                                data-validation-state="pending"
                            >
                                <h3>Validation</h3>
                                <p>Configuration validates on each wizard step.</p>
                            </section>
                            <section id="tau-ops-deploy-review" data-component="DeployReviewSummary">
                                <h3>Review</h3>
                                <p data-field="summary">Pending full configuration summary.</p>
                            </section>
                            <div id="tau-ops-deploy-actions">
                                <button
                                    id="tau-ops-deploy-submit"
                                    type="button"
                                    data-action="deploy-agent"
                                    data-success-redirect-template="/ops/agents/{agent_id}"
                                >
                                    Deploy Agent
                                </button>
                            </div>
                        </section>
                    </main>
                </div>
            </div>
            <script
                id="tau-ops-static-preview-route-guard"
                data-preview-link-guard="file-protocol-absolute-routes"
            >
                r#"
                (function () {
                    if (window.location.protocol !== "file:") {
                        return;
                    }

                    document.addEventListener("click", function (event) {
                        var shell = document.getElementById("tau-ops-shell");
                        var target = event.target;
                        var anchor = target && target.closest ? target.closest("a[href]") : null;
                        if (!shell || !anchor || !shell.contains(anchor)) {
                            return;
                        }

                        var rawHref = anchor.getAttribute("href") || "";
                        if (rawHref.charAt(0) !== "/" || rawHref.charAt(1) === "/") {
                            return;
                        }

                        event.preventDefault();
                        anchor.setAttribute("data-preview-link-blocked", "true");
                        var status = document.getElementById("tau-ops-static-preview-status");
                        if (status) {
                            status.setAttribute("data-preview-route-status", "blocked-link");
                            status.textContent = "Preview navigation blocked for local file review.";
                        }
                    });
                })();
                "#
            </script>
        </div>
    };
    shell.to_html()
}

#[cfg(test)]
mod tests;
