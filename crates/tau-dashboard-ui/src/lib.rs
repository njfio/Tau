//! Leptos SSR shell foundations for Tau Ops Dashboard.

use leptos::prelude::*;

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
            Self::Channels => "Multi-Channel",
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

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardHarnessAuditRow` in `tau-dashboard-ui`.
pub struct TauOpsDashboardHarnessAuditRow {
    pub timestamp_label: String,
    pub actor: String,
    pub action_label: String,
    pub action_key: String,
    pub scope: String,
    pub item: String,
    pub result_label: String,
    pub result_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TauOpsDashboardHarnessSnapshot` in `tau-dashboard-ui`.
pub struct TauOpsDashboardHarnessSnapshot {
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
    pub audit_source: String,
    pub audit_rows: Vec<TauOpsDashboardHarnessAuditRow>,
}

impl Default for TauOpsDashboardHarnessSnapshot {
    fn default() -> Self {
        Self {
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
            audit_source: "fallback".to_string(),
            audit_rows: vec![
                TauOpsDashboardHarnessAuditRow {
                    timestamp_label: "May 15, 10:11".to_string(),
                    actor: "Operator".to_string(),
                    action_label: "Dry Run".to_string(),
                    action_key: "dry-run".to_string(),
                    scope: "Prompt".to_string(),
                    item: "PR-044".to_string(),
                    result_label: "Passed".to_string(),
                    result_key: "passed".to_string(),
                },
                TauOpsDashboardHarnessAuditRow {
                    timestamp_label: "May 15, 09:42".to_string(),
                    actor: "Operator".to_string(),
                    action_label: "Apply".to_string(),
                    action_key: "apply".to_string(),
                    scope: "Config".to_string(),
                    item: "CL-031".to_string(),
                    result_label: "Applied".to_string(),
                    result_key: "applied".to_string(),
                },
                TauOpsDashboardHarnessAuditRow {
                    timestamp_label: "May 15, 09:12".to_string(),
                    actor: "Curator".to_string(),
                    action_label: "Apply".to_string(),
                    action_key: "apply".to_string(),
                    scope: "Skill".to_string(),
                    item: "SK-102".to_string(),
                    result_label: "Applied".to_string(),
                    result_key: "applied".to_string(),
                },
                TauOpsDashboardHarnessAuditRow {
                    timestamp_label: "May 15, 08:33".to_string(),
                    actor: "Operator".to_string(),
                    action_label: "Reject".to_string(),
                    action_key: "reject".to_string(),
                    scope: "Prompt".to_string(),
                    item: "PR-029".to_string(),
                    result_label: "Rejected".to_string(),
                    result_key: "rejected".to_string(),
                },
            ],
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
    let chat_panel_hidden = if matches!(context.active_route, TauOpsDashboardRoute::Chat) {
        "false"
    } else {
        "true"
    };
    let chat_panel_visible = if matches!(context.active_route, TauOpsDashboardRoute::Chat) {
        "true"
    } else {
        "false"
    };
    let sessions_panel_hidden = if matches!(context.active_route, TauOpsDashboardRoute::Sessions) {
        "false"
    } else {
        "true"
    };
    let sessions_panel_visible = if matches!(context.active_route, TauOpsDashboardRoute::Sessions) {
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
    let harness_task_count = context.harness.task_count.to_string();
    let harness_pass_count = context.harness.pass_count.to_string();
    let harness_audit_row_count = context.harness.audit_rows.len().to_string();
    let harness_benchmark_rows = context
        .harness
        .benchmark_rows
        .iter()
        .map(|row| {
            let task_count = row.task_count.to_string();
            let last_run = format!("{}/{} pass", row.pass_count, row.total_count);
            let last_run_attr = last_run.clone();
            view! {
                <tr
                    data-category=row.category.clone()
                    data-task-count=task_count
                    data-last-run=last_run_attr
                    data-pass-rate=row.pass_rate.clone()
                >
                    <td>{row.category.clone()}</td>
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
        .map(|row| {
            view! {
                <tr data-action=row.action_key.clone() data-result=row.result_key.clone()>
                    <td>{row.timestamp_label.clone()}</td>
                    <td>{row.actor.clone()}</td>
                    <td>{row.action_label.clone()}</td>
                    <td>{row.scope.clone()}</td>
                    <td>{row.item.clone()}</td>
                    <td>{row.result_label.clone()}</td>
                </tr>
            }
        })
        .collect_view();
    let harness_tui_summary = format!(
        "tau@harness:~$ tau status\nmission=run_8f3a2\ntransport=gateway\nskill=repo_patch\nstatus=verifying\ncalls: repo.read, repo.edit, test.run, report.write\nbench: {} pass; proof {}\n\nBenchmark M334\nPassed: {}\nFailed Gates:\n  {}\nProof: {}",
        context.harness.latest_result.clone(),
        context.harness.proof_artifact.clone(),
        context.harness.latest_result.clone(),
        context.harness.failed_gate_label.clone(),
        context.harness.proof_artifact.clone()
    );
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
    let chat_message_rows = if context.chat.message_rows.is_empty() {
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
            entry_count: 0,
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
    let session_detail_panel_hidden =
        if matches!(context.active_route, TauOpsDashboardRoute::Sessions)
            && context.chat.session_detail_visible
        {
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
    let session_detail_timeline_rows = context.chat.session_detail_timeline_rows.clone();
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
    let session_graph_node_rows = context.chat.session_graph_node_rows.clone();
    let session_graph_edge_rows = context.chat.session_graph_edge_rows.clone();
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
            let login_href = format!(
                "{active_shell_path}?theme={theme_attr}&sidebar={sidebar_state_attr}&session={chat_session_key}&channel={channel}&channel_action=login"
            );
            let logout_href = format!(
                "{active_shell_path}?theme={theme_attr}&sidebar={sidebar_state_attr}&session={chat_session_key}&channel={channel}&channel_action=logout"
            );
            let probe_href = format!(
                "{active_shell_path}?theme={theme_attr}&sidebar={sidebar_state_attr}&session={chat_session_key}&channel={channel}&channel_action=probe"
            );
            view! {
                <tr
                    id=row_id
                    data-channel=row.channel.clone()
                    data-mode=row.mode.clone()
                    data-liveness=row.liveness.clone()
                    data-events-ingested=events_ingested_attr
                    data-provider-failures=provider_failures_attr
                >
                    <td>{row.channel.clone()}</td>
                    <td>{row.mode.clone()}</td>
                    <td>{row.liveness.clone()}</td>
                    <td>{events_ingested}</td>
                    <td>{provider_failures}</td>
                    <td>
                        <a
                            id=login_id
                            data-action="channel-login"
                            data-channel=row.channel.clone()
                            data-action-enabled=login_enabled
                            href=login_href
                        >
                            Login
                        </a>
                        <a
                            id=logout_id
                            data-action="channel-logout"
                            data-channel=row.channel.clone()
                            data-action-enabled=logout_enabled
                            href=logout_href
                        >
                            Logout
                        </a>
                        <a
                            id=probe_id
                            data-action="channel-probe"
                            data-channel=row.channel.clone()
                            data-action-enabled=probe_enabled
                            href=probe_href
                        >
                            Probe
                        </a>
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
        >
            <style id="tau-ops-dashboard-base-style">
                r#"
                #tau-ops-shell {
                    min-height: 100vh;
                    background: #08141c;
                    color: #dbe8ef;
                    font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
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
                <p>Leptos SSR foundation shell</p>
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
                            <li id="tau-ops-nav-command-center"><a data-nav-item="command-center" href="/ops" data-harness-rail-label="Command">Command Center</a></li>
                            <li id="tau-ops-nav-agent-fleet"><a data-nav-item="agent-fleet" href="/ops/agents" data-harness-rail-label="Fleet">Agent Fleet</a></li>
                            <li id="tau-ops-nav-agent-detail"><a data-nav-item="agent-detail" href="/ops/agents/default" data-harness-rail-label="Agent">Agent Detail</a></li>
                            <li id="tau-ops-nav-chat"><a data-nav-item="chat" href="/ops/chat" data-harness-rail-label="Chat">Conversation / Chat</a></li>
                            <li id="tau-ops-nav-sessions"><a data-nav-item="sessions" href="/ops/sessions" data-harness-rail-label="Sessions">Sessions Explorer</a></li>
                            <li id="tau-ops-nav-memory"><a data-nav-item="memory" href="/ops/memory" data-harness-rail-label="Memory">Memory Explorer</a></li>
                            <li id="tau-ops-nav-memory-graph"><a data-nav-item="memory-graph" href="/ops/memory-graph" data-harness-rail-label="Graph">Memory Graph</a></li>
                            <li id="tau-ops-nav-tools-jobs"><a data-nav-item="tools-jobs" href="/ops/tools-jobs" data-harness-rail-label="Tools">Tools & Jobs</a></li>
                            <li id="tau-ops-nav-channels"><a data-nav-item="channels" href="/ops/channels" data-harness-rail-label="Channels">Multi-Channel</a></li>
                            <li id="tau-ops-nav-harness"><a data-nav-item="mission-harness" href="/ops/harness" data-harness-rail-label="Missions">Mission Harness</a></li>
                            <li id="tau-ops-nav-config"><a data-nav-item="config" href="/ops/config" data-harness-rail-label="Config">Configuration</a></li>
                            <li id="tau-ops-nav-training"><a data-nav-item="training" href="/ops/training" data-harness-rail-label="Training">Training & RL</a></li>
                            <li id="tau-ops-nav-safety"><a data-nav-item="safety" href="/ops/safety" data-harness-rail-label="Safety">Safety & Security</a></li>
                            <li id="tau-ops-nav-diagnostics"><a data-nav-item="diagnostics" href="/ops/diagnostics" data-harness-rail-label="Audit">Diagnostics & Audit</a></li>
                            <li id="tau-ops-nav-deploy"><a data-nav-item="deploy" href="/ops/deploy" data-harness-rail-label="Deploy">Deploy Agent</a></li>
                            <li id="tau-ops-nav-login"><a href="/ops/login" data-harness-rail-label="Login">Operator Login</a></li>
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
                            data-axe-contract="required"
                            data-keyboard-navigation="true"
                            data-focus-visible-contract="true"
                            data-focus-ring-token="tau-focus-ring"
                            data-reduced-motion-contract="prefers-reduced-motion"
                            data-reduced-motion-behavior="suppress-nonessential-animation"
                        >
                            <h2>Accessibility Contracts</h2>
                            <p id="tau-ops-live-announcer" aria-live="polite" aria-atomic="true">
                                Accessibility live region ready.
                            </p>
                        </section>
                        <section
                            id="tau-ops-stream-contract"
                            data-component="RealtimeStreamContract"
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
                        >
                            <h2>Real-Time Stream Contracts</h2>
                        </section>
                        <section
                            id="tau-ops-performance-contract"
                            data-component="PerformanceBudgetContract"
                            data-wasm-budget-gzip-kb="500"
                            data-lcp-budget-ms="1500"
                            data-layout-shift-budget="0.00"
                            data-layout-shift-mitigation="skeletons"
                            data-websocket-process-budget-ms="50"
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
                        >
                            <h2>Multi-Channel</h2>
                            <p
                                id="tau-ops-channels-summary"
                                data-online-count=channels_online_count
                                data-offline-count=channels_offline_count
                                data-degraded-count=channels_degraded_count
                            >
                                Channel health summary for all configured connectors.
                            </p>
                            <table id="tau-ops-channels-table" data-row-count=channels_row_count_table_value>
                                <thead>
                                    <tr>
                                        <th scope="col">Channel</th>
                                        <th scope="col">Mode</th>
                                        <th scope="col">Liveness</th>
                                        <th scope="col">Events Ingested</th>
                                        <th scope="col">Provider Failures</th>
                                        <th scope="col">Actions</th>
                                    </tr>
                                </thead>
                                <tbody
                                    id="tau-ops-channels-body"
                                    data-row-count=channels_row_count_body_value
                                >
                                    {channels_rows_view}
                                </tbody>
                            </table>
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
                            }
                            #tau-ops-harness-missions-table .tau-harness-mission-meta {
                                display: flex;
                                flex-wrap: wrap;
                                gap: 5px;
                                margin-top: 6px;
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
                            #tau-ops-harness-proposal-detail {
                                max-height: 128px;
                                overflow: auto;
                            }
                            #tau-ops-harness-proposal-detail[data-proposal-detail-overflow-budget="contained"] {
                                overflow: hidden;
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
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] th:nth-child(2),
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] td:nth-child(2) {
                                display: none;
                            }
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] th:nth-child(3),
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] td:nth-child(3) {
                                width: 86px;
                            }
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] th:nth-child(4),
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] td:nth-child(4),
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] th:nth-child(5),
                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit="compact-no-overflow"] td:nth-child(5) {
                                width: 66px;
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
                            #tau-ops-harness-acceptance ul,
                            #tau-ops-harness-verification-gates ul,
                            #tau-ops-harness-artifacts ul,
                            #tau-ops-harness-learning-queue ul,
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
                            #tau-ops-harness-verification-gates li {
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
                            data-responsive-collapse-width="1180px"
                        >
                            <p
                                id="tau-ops-harness-preview-status"
                                data-preview-status="idle"
                                hidden
                                aria-live="polite"
                            >
                                "Preview action idle."
                            </p>
                            <header id="tau-ops-harness-topbar" data-workspace="/workspace/tau" data-model="gpt-5.4" data-transport="gateway" data-health="healthy" data-window-chrome="compact">
                                <div>
                                    <small>"Tau"</small>
                                    <h2>"Tau Agent Harness"</h2>
                                </div>
                                <nav aria-label="Mission harness actions">
                                    <a id="tau-ops-harness-new-mission" data-action="new-mission" href="/ops/harness?intent=new-mission">"New Mission"</a>
                                    <a id="tau-ops-harness-history" data-action="history" href="/ops/harness?view=history">"History"</a>
                                </nav>
                            </header>
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
                                    <article id="tau-ops-harness-kpi-active" data-harness-kpi-card="active-missions" data-kpi-value="5" data-kpi-heading-fit="word-boundary">
                                        <h4>"Active Missions"</h4>
                                        <p>"5"</p>
                                        <small>"3 running"</small>
                                    </article>
                                    <article id="tau-ops-harness-kpi-verifications" data-harness-kpi-card="pending-verifications" data-kpi-value="3" data-kpi-heading-fit="word-boundary">
                                        <h4 aria-label="Pending Verifications"><span>"Pending"</span><span>"Verifications"</span></h4>
                                        <p>"3"</p>
                                        <small>"2 need review"</small>
                                    </article>
                                    <article id="tau-ops-harness-kpi-memory" data-harness-kpi-card="memory-writes" data-kpi-value="12" data-kpi-heading-fit="word-boundary">
                                        <h4>"Memory Writes"</h4>
                                        <p>"12"</p>
                                        <small>"Today"</small>
                                    </article>
                                    <article id="tau-ops-harness-kpi-cost" data-harness-kpi-card="runtime-cost-today" data-kpi-value="18.74" data-kpi-heading-fit="word-boundary">
                                        <h4>"Runtime Cost Today"</h4>
                                        <p>"$18.74"</p>
                                        <small>"Across 5 runs"</small>
                                    </article>
                                </section>
                                <section
                                    id="tau-ops-harness-active-missions"
                                    data-active-count="5"
                                    data-running-count="3"
                                    data-blocked-count="1"
                                    data-compact-table-breakpoint="1400px"
                                    data-compact-mission-summary="status-and-gates"
                                    data-first-viewport-budget="benchmark-visible"
                                    data-active-mission-scroll-boundary="whole-row"
                                    data-active-mission-visible-rows="3"
                                    data-left-table-fit="compact-no-overflow"
                                    data-horizontal-overflow-budget="none"
                                >
                                    <h4>"Active Missions"</h4>
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
                                                <tr id="tau-ops-harness-mission-row-0" data-mission-id="run_linux_ci" data-status="running" data-plan-progress="68" data-verification-state="needs-review">
                                                    <td data-mission-summary="inline-status">
                                                        <span class="tau-harness-mission-title">"Port repo test harness to Linux CI"</span>
                                                        <span class="tau-harness-mission-meta" data-compact-mission-meta="status-gates">
                                                            <span class="tau-harness-status-chip" data-mission-state-chip="running">"Running"</span>
                                                            <span class="tau-harness-status-chip" data-mission-gate-chip="needs-review">"3/5 gates"</span>
                                                        </span>
                                                    </td>
                                                    <td>"4/5"</td>
                                                    <td><meter min="0" max="100" value="68">"68%"</meter></td>
                                                    <td>"34/60"</td>
                                                    <td>"12"</td>
                                                    <td><span class="tau-harness-status-chip" data-gate-status="needs-review">"3/5 gates"</span></td>
                                                    <td>"10:22:31 May 15"</td>
                                                    <td>"5"</td>
                                                </tr>
                                                <tr id="tau-ops-harness-mission-row-1" data-mission-id="run_m334_flaky" data-status="verifying" data-plan-progress="72" data-verification-state="needs-review">
                                                    <td data-mission-summary="inline-status">
                                                        <span class="tau-harness-mission-title">"Investigate flaky benchmark on M334"</span>
                                                        <span class="tau-harness-mission-meta" data-compact-mission-meta="status-gates">
                                                            <span class="tau-harness-status-chip" data-mission-state-chip="verifying">"Verifying"</span>
                                                            <span class="tau-harness-status-chip" data-mission-gate-chip="needs-review">"2/5 gates"</span>
                                                        </span>
                                                    </td>
                                                    <td>"3/5"</td>
                                                    <td><meter min="0" max="100" value="72">"72%"</meter></td>
                                                    <td>"28/60"</td>
                                                    <td>"8"</td>
                                                    <td><span class="tau-harness-status-chip" data-gate-status="needs-review">"2/5 gates"</span></td>
                                                    <td>"10:25:52 May 15"</td>
                                                    <td>"4"</td>
                                                </tr>
                                                <tr id="tau-ops-harness-mission-row-2" data-mission-id="run_research_brief" data-status="completed" data-plan-progress="100" data-verification-state="passed">
                                                    <td data-mission-summary="inline-status">
                                                        <span class="tau-harness-mission-title">"Generate weekly research brief on model routing"</span>
                                                        <span class="tau-harness-mission-meta" data-compact-mission-meta="status-gates">
                                                            <span class="tau-harness-status-chip" data-mission-state-chip="completed">"Completed"</span>
                                                            <span class="tau-harness-status-chip" data-mission-gate-chip="passed">"5/5 gates"</span>
                                                        </span>
                                                    </td>
                                                    <td>"5/5"</td>
                                                    <td><meter min="0" max="100" value="100">"100%"</meter></td>
                                                    <td>"18/60"</td>
                                                    <td>"15"</td>
                                                    <td><span class="tau-harness-status-chip" data-gate-status="passed">"5/5 gates"</span></td>
                                                    <td>"09:55:11 May 15"</td>
                                                    <td>"7"</td>
                                                </tr>
                                                <tr id="tau-ops-harness-mission-row-3" data-mission-id="run_receipts" data-status="blocked" data-plan-progress="36" data-verification-state="failed">
                                                    <td data-mission-summary="inline-status">
                                                        <span class="tau-harness-mission-title">"Automate receipt classification pipeline"</span>
                                                        <span class="tau-harness-mission-meta" data-compact-mission-meta="status-gates">
                                                            <span class="tau-harness-status-chip" data-mission-state-chip="blocked">"Blocked"</span>
                                                            <span class="tau-harness-status-chip" data-mission-gate-chip="failed">"1/5 gates"</span>
                                                        </span>
                                                    </td>
                                                    <td>"2/5"</td>
                                                    <td><meter min="0" max="100" value="36">"36%"</meter></td>
                                                    <td>"16/60"</td>
                                                    <td>"6"</td>
                                                    <td><span class="tau-harness-status-chip" data-gate-status="failed">"1/5 gates"</span></td>
                                                    <td>"09:48:03 May 15"</td>
                                                    <td>"3"</td>
                                                </tr>
                                                <tr id="tau-ops-harness-mission-row-4" data-mission-id="run_8f3a2" data-status="running" data-plan-progress="55" data-verification-state="running">
                                                    <td data-mission-summary="inline-status">
                                                        <span class="tau-harness-mission-title">"Refactor plugin registry for safer hot reload"</span>
                                                        <span class="tau-harness-mission-meta" data-compact-mission-meta="status-gates">
                                                            <span class="tau-harness-status-chip" data-mission-state-chip="running">"Running"</span>
                                                            <span class="tau-harness-status-chip" data-mission-gate-chip="running">"2/5 gates"</span>
                                                        </span>
                                                    </td>
                                                    <td>"3/5"</td>
                                                    <td><meter min="0" max="100" value="55">"55%"</meter></td>
                                                    <td>"42/60"</td>
                                                    <td>"12"</td>
                                                    <td><span class="tau-harness-status-chip" data-gate-status="running">"2/5 gates"</span></td>
                                                    <td>"10:24:18 May 15"</td>
                                                    <td>"6"</td>
                                                </tr>
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
                                    <form
                                        id="tau-ops-harness-run-benchmark-form"
                                        action="/ops/harness/run-benchmark"
                                        method="post"
                                        data-command="tau_agent_harness"
                                    >
                                        <button id="tau-ops-harness-run-benchmark" type="submit" data-action-tone="benchmark">"Run Benchmark"</button>
                                    </form>
                                </section>
                            </section>
                            <section
                                id="tau-ops-harness-proof-window"
                                data-window="mission-detail-proof-view"
                                data-window-order="2"
                                data-run-id="run_8f3a2"
                                data-mission-status="running"
                                data-tool-budget="42/60"
                                data-window-chrome="compact"
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
                                        <h3>"Refactor plugin registry for safer hot reload"</h3>
                                    </div>
                                    <dl>
                                        <dt>"Run ID"</dt><dd>"run_8f3a2"</dd>
                                        <dt>"Elapsed"</dt><dd>"01:42:18"</dd>
                                        <dt>"Tool Budget"</dt><dd>"42/60"</dd>
                                        <dt>"Cost"</dt><dd>"$3.82"</dd>
                                        <dt>"Retry Count"</dt><dd>"1"</dd>
                                    </dl>
                                </header>
                                <ol id="tau-ops-harness-plan-dag" data-dag-node-count="5" data-current-node="verify" data-proof-dag-density="single-row">
                                    <li id="tau-ops-harness-dag-plan" data-plan-node="Plan" data-node-status="passed">"Plan"</li>
                                    <li id="tau-ops-harness-dag-execute" data-plan-node="Execute" data-node-status="passed">"Execute"</li>
                                    <li id="tau-ops-harness-dag-memory-write" data-plan-node="Memory Write" data-node-status="passed">"Memory Write"</li>
                                    <li id="tau-ops-harness-dag-verify" data-plan-node="Verify" data-node-status="running">"Verify"</li>
                                    <li id="tau-ops-harness-dag-learn" data-plan-node="Learn" data-node-status="pending">"Learn"</li>
                                </ol>
                                <div class="tau-harness-window-grid" data-proof-grid-priority="evidence-log-gates-first">
                                    <section
                                        id="tau-ops-harness-tool-evidence"
                                        data-tool-call-count="8"
                                        data-compact-evidence-breakpoint="1400px"
                                        data-compact-call-id-visibility="hidden-at-1400px"
                                        data-proof-evidence-priority="first-screen"
                                        data-tool-evidence-fit="compact-no-overflow"
                                        data-tool-evidence-overflow-budget="none"
                                        data-tool-evidence-visible-columns="tool,plan-node,runtime,status,artifact"
                                    >
                                        <h4>"Tool Execution Evidence"</h4>
                                        <div class="tau-harness-table-wrap">
                                            <table>
                                                <thead><tr><th scope="col">"Tool"</th><th scope="col">"Call ID"</th><th scope="col">"Plan Node"</th><th scope="col">"Runtime"</th><th scope="col">"Status"</th><th scope="col">"Artifact"</th></tr></thead>
                                                <tbody>
                                                    <tr data-tool="repo.read" data-status="passed"><td>"repo.read"</td><td>"c1a2bf3"</td><td>"Execute"</td><td>"00:01:12"</td><td>"passed"</td><td>"/artifacts/repo-read.json"</td></tr>
                                                    <tr data-tool="repo.edit" data-status="passed"><td>"repo.edit"</td><td>"c1a2b4c"</td><td>"Execute"</td><td>"00:02:34"</td><td>"passed"</td><td>"/artifacts/edit.patch"</td></tr>
                                                    <tr data-tool="test.run" data-status="passed"><td>"test.run"</td><td>"c1a2b6e"</td><td>"Execute"</td><td>"00:08:42"</td><td>"passed"</td><td>"/artifacts/tests.json"</td></tr>
                                                    <tr data-tool="memory.search" data-status="passed"><td>"memory.search"</td><td>"c1a2b7f"</td><td>"Memory Write"</td><td>"00:00:48"</td><td>"passed"</td><td>"/artifacts/memory.json"</td></tr>
                                                    <tr data-tool="memory.write" data-status="passed"><td>"memory.write"</td><td>"c1a2b8e"</td><td>"Memory Write"</td><td>"00:00:36"</td><td>"passed"</td><td>"/artifacts/learning.json"</td></tr>
                                                    <tr data-tool="report.write" data-status="running"><td>"report.write"</td><td>"c1a2b9"</td><td>"Verify"</td><td>"00:01:21"</td><td>"running"</td><td>"/artifacts/report.md"</td></tr>
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
                                        <pre>"10:18:22  Plan accepted
10:18:23  Executing plan with tool budget 42/60
10:18:25  repo.read call_id=c1a2b3 path=src/registry/**
10:18:37  repo.edit completed (42 files)
10:20:55  memory.write call_id=c1a2b8 record=learning
10:24:18  Verification started
10:25:52  verification gate VG-03 pending (collecting no-memory evidence)"</pre>
                                    </section>
                                    <section
                                        id="tau-ops-harness-acceptance"
                                        data-acceptance-met="3"
                                        data-acceptance-total="5"
                                        data-proof-detail-budget="compact-scroll"
                                        data-acceptance-overflow-budget="all-criteria-visible"
                                        data-acceptance-layout="compact-contained"
                                    >
                                        <h4>"Acceptance Criteria"</h4>
                                        <ul>
                                            <li data-ac-id="VG-01" data-ac-status="met">"Registry loads plugins deterministically"</li>
                                            <li data-ac-id="VG-02" data-ac-status="met">"Hot reload preserves active sessions"</li>
                                            <li data-ac-id="VG-03" data-ac-status="met">"Added regression tests"</li>
                                            <li data-ac-id="VG-04" data-ac-status="pending">"Docs updated"</li>
                                            <li data-ac-id="VG-05" data-ac-status="pending">"Benchmark proof emitted"</li>
                                        </ul>
                                    </section>
                                    <section
                                        id="tau-ops-harness-verification-gates"
                                        data-gate-count="5"
                                        data-failed-gate-count="1"
                                        data-proof-secondary-priority="first-screen"
                                        data-proof-detail-budget="compact-scroll"
                                        data-gate-visibility="all-gates-first-viewport"
                                        data-gate-layout="two-column-compact"
                                    >
                                        <h4>"Verification Gates"</h4>
                                        <ul>
                                            <li id="tau-ops-harness-gate-planning" data-gate-id="VG-01" data-gate-status="passed">"Planning proof"</li>
                                            <li id="tau-ops-harness-gate-tool-exec" data-gate-id="VG-02" data-gate-status="passed">"Tool execution proof"</li>
                                            <li id="tau-ops-harness-gate-memory" data-gate-id="VG-03" data-gate-status="failed">"Memory proof"</li>
                                            <li id="tau-ops-harness-gate-verification" data-gate-id="VG-04" data-gate-status="running">"Verification proof"</li>
                                            <li id="tau-ops-harness-gate-learning" data-gate-id="VG-05" data-gate-status="pending">"Learning proof"</li>
                                        </ul>
                                    </section>
                                    <section id="tau-ops-harness-memory-learning" data-memory-hits="12" data-learning-records="2" data-last-memory-write="10:20:55" data-proof-footer-priority="first-viewport">
                                        <h4>"Memory / Learning"</h4>
                                        <p>"Memory hits: 12"</p>
                                        <p>"No-memory evidence: Collected"</p>
                                        <p>"Learning records: 2"</p>
                                    </section>
                                    <section id="tau-ops-harness-artifacts" data-artifact-count="3" data-proof-footer-priority="first-viewport">
                                        <h4>"Artifacts"</h4>
                                        <ul>
                                            <li><a href="/artifacts/code.diff">"Code changes"</a></li>
                                            <li><a href="/artifacts/docs.md">"Docs"</a></li>
                                            <li><a href="/artifacts/proof.json">"Benchmark proof"</a></li>
                                        </ul>
                                    </section>
                                </div>
                            </section>
                            <section
                                id="tau-ops-harness-self-improvement-window"
                                data-window="self-improvement-review-apply-flow"
                                data-window-order="3"
                                data-selected-proposal="PR-044"
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
                                    data-queue-count="4"
                                    data-queue-density="all-items-visible"
                                    data-queue-overflow-budget="none"
                                    data-queue-readability="full-labels"
                                    data-queue-layout="single-column-readable"
                                    data-queue-truncation-budget="none"
                                >
                                    <h4>"Learning & Proposals"</h4>
                                    <ul>
                                        <li data-learning-id="LR-219" data-status="needs-review">"Retry storm in document synthesis"</li>
                                        <li data-learning-id="LR-220" data-status="needs-review">"Missing memory write after verification"</li>
                                        <li data-learning-id="PR-044" data-status="proposal">"Prompt compression for research tasks"</li>
                                        <li data-learning-id="PR-045" data-status="proposal">"Skill patch for benchmark artifact naming"</li>
                                    </ul>
                                </section>
                                <section
                                    id="tau-ops-harness-operator-actions"
                                    data-apply-requires-approval="true"
                                    data-action-row-priority="approval-flow"
                                    data-action-grid="two-column-priority"
                                    data-action-first-viewport="all-controls"
                                >
                                    <h4>"Operator Actions"</h4>
                                    <form id="tau-ops-harness-approve-form" action="/ops/harness/proposals/PR-044/approve" method="post"><button id="tau-ops-harness-action-approve" type="submit" data-action="approve" data-action-tone="approve">"Approve"</button></form>
                                    <form id="tau-ops-harness-reject-form" action="/ops/harness/proposals/PR-044/reject" method="post"><button id="tau-ops-harness-action-reject" type="submit" data-action="reject" data-action-tone="reject">"Reject"</button></form>
                                    <form id="tau-ops-harness-dry-run-form" action="/ops/harness/proposals/PR-044/dry-run" method="post"><button id="tau-ops-harness-action-dry-run" type="submit" data-action="dry-run" data-action-tone="secondary">"Dry Run Again"</button></form>
                                    <a id="tau-ops-harness-action-view-diff" data-action="view-diff" data-action-tone="secondary" href="/ops/harness/proposals/PR-044/diff">"View Diff"</a>
                                    <button id="tau-ops-harness-action-apply" type="button" data-action="apply" data-action-tone="disabled" data-disabled="true" aria-disabled="true" data-approval-required="true">"Apply (Approval Required)"</button>
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
                                    data-proposal-id="PR-044"
                                    data-learning-record="LR-044"
                                    data-target-type="Prompt"
                                    data-target-path="prompts/research_to_doc/system.md"
                                    data-proposal-detail-priority="first-viewport-summary"
                                    data-proposal-detail-density="compact-scroll"
                                    data-proposal-detail-overflow-budget="contained"
                                    data-proposal-visible-rows="7"
                                    data-proposal-summary-fit="full-text"
                                    data-proposal-summary-overflow-budget="none"
                                >
                                    <h4>"PR-044 Prompt compression for research tasks"</h4>
                                    <dl>
                                        <dt>"Dry-run Result"</dt><dd data-result="passed">"Tests passed (18/18)"</dd>
                                        <dt>"Safety Check"</dt><dd data-result="passed">"Passed"</dd>
                                        <dt>"Rollback Plan"</dt><dd>"Revert to previous prompt version"</dd>
                                        <dt>"Patch Summary"</dt><dd data-proposal-row="patch-summary" data-summary-fit="full-text">"Compress system prompt by removing redundant instructions and examples."</dd>
                                        <dt>"Failure Observed"</dt><dd>"Token overrun during research-to-doc tasks"</dd>
                                        <dt>"Root Cause"</dt><dd>"Verbose prompts with redundant context"</dd>
                                        <dt>"Test Evidence"</dt><dd><a href="/evidence/pr-044-dryrun.json">"evidence/pr-044-dryrun.json"</a></dd>
                                    </dl>
                                </section>
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
