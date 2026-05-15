use serde::Deserialize;
use tau_dashboard_ui::{TauOpsDashboardSidebarState, TauOpsDashboardTheme};
use tau_memory::runtime::MemoryType;

fn normalize_memory_graph_relation_filter(value: &str) -> String {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty()
        || !normalized
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        "all".to_string()
    } else {
        normalized
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(super) struct OpsShellControlsQuery {
    #[serde(default)]
    theme: String,
    #[serde(default)]
    sidebar: String,
    #[serde(default)]
    range: String,
    #[serde(default)]
    session_key: String,
    #[serde(default)]
    session: String,
    #[serde(default)]
    query: String,
    #[serde(default)]
    workspace_id: String,
    #[serde(default)]
    channel_id: String,
    #[serde(default)]
    actor_id: String,
    #[serde(default)]
    limit: String,
    #[serde(default)]
    memory_type: String,
    #[serde(default)]
    create_status: String,
    #[serde(default)]
    created_memory_id: String,
    #[serde(default)]
    delete_status: String,
    #[serde(default)]
    deleted_memory_id: String,
    #[serde(default)]
    detail_memory_id: String,
    #[serde(default)]
    preview_memory_id: String,
    #[serde(default)]
    graph_zoom: String,
    #[serde(default)]
    graph_pan_x: String,
    #[serde(default)]
    graph_pan_y: String,
    #[serde(default)]
    graph_filter_memory_type: String,
    #[serde(default)]
    graph_filter_relation_type: String,
    #[serde(default)]
    tool: String,
    #[serde(default)]
    job: String,
    #[serde(default)]
    cancel_job: String,
    #[serde(default)]
    control_action_status: String,
    #[serde(default)]
    control_action: String,
    #[serde(default)]
    control_action_reason: String,
    #[serde(default)]
    chat_status: String,
    #[serde(default)]
    new_session_status: String,
    #[serde(default)]
    channel_action_status: String,
    #[serde(default)]
    channel_action: String,
    #[serde(default)]
    channel_action_channel: String,
    #[serde(default)]
    channel_action_reason: String,
    #[serde(default)]
    proposal_id: String,
    #[serde(default)]
    intent: String,
    #[serde(default)]
    view: String,
    #[serde(default)]
    mission_id: String,
    #[serde(default)]
    mission_status: String,
    #[serde(default)]
    proposal_status: String,
    #[serde(default)]
    audit_action: String,
    #[serde(default)]
    audit_ref: String,
}

impl OpsShellControlsQuery {
    pub(super) fn theme(&self) -> TauOpsDashboardTheme {
        match self.theme.as_str() {
            "light" => TauOpsDashboardTheme::Light,
            _ => TauOpsDashboardTheme::Dark,
        }
    }

    pub(super) fn sidebar_state(&self) -> TauOpsDashboardSidebarState {
        match self.sidebar.as_str() {
            "collapsed" => TauOpsDashboardSidebarState::Collapsed,
            _ => TauOpsDashboardSidebarState::Expanded,
        }
    }

    pub(super) fn timeline_range(&self) -> &'static str {
        match self.range.as_str() {
            "6h" => "6h",
            "24h" => "24h",
            _ => "1h",
        }
    }

    pub(super) fn requested_session_key(&self) -> Option<&str> {
        let session_key = if self.session_key.trim().is_empty() {
            self.session.trim()
        } else {
            self.session_key.trim()
        };
        if session_key.is_empty() {
            None
        } else {
            Some(session_key)
        }
    }

    pub(super) fn requested_memory_query(&self) -> Option<&str> {
        let query = self.query.trim();
        if query.is_empty() {
            None
        } else {
            Some(query)
        }
    }

    pub(super) fn requested_harness_proposal_id(&self) -> Option<&str> {
        let value = self.proposal_id.trim();
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    }

    pub(super) fn requested_harness_intent(&self) -> Option<&'static str> {
        match self.intent.trim() {
            "new-mission" => Some("new-mission"),
            _ => None,
        }
    }

    pub(super) fn requested_harness_view(&self) -> Option<&'static str> {
        match self.view.trim() {
            "history" => Some("history"),
            _ => None,
        }
    }

    pub(super) fn requested_harness_mission_id(&self) -> Option<&str> {
        let value = self.mission_id.trim();
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    }

    pub(super) fn requested_harness_mission_status(&self) -> Option<&'static str> {
        match self.mission_status.trim() {
            "draft_created" => Some("draft_created"),
            "mission_blocked" => Some("mission_blocked"),
            "mission_completed" => Some("mission_completed"),
            "mission_started" => Some("mission_started"),
            "start_failed" => Some("start_failed"),
            "write_failed" => Some("write_failed"),
            _ => None,
        }
    }

    pub(super) fn requested_harness_proposal_status(&self) -> Option<&'static str> {
        match self.proposal_status.trim() {
            "applied" => Some("applied"),
            "apply_failed" => Some("apply_failed"),
            "approved" => Some("approved"),
            "dry_run_failed" => Some("dry_run_failed"),
            "dry_run_passed" => Some("dry_run_passed"),
            "rejected" => Some("rejected"),
            _ => None,
        }
    }

    pub(super) fn requested_harness_audit_action(&self) -> Option<&'static str> {
        match self.audit_action.trim() {
            "apply" => Some("apply"),
            "approve" => Some("approve"),
            "dry-run" => Some("dry-run"),
            "reject" => Some("reject"),
            "run-benchmark" => Some("run-benchmark"),
            "start-mission" => Some("start-mission"),
            _ => None,
        }
    }

    pub(super) fn requested_harness_audit_ref(&self) -> Option<String> {
        let sanitized = self
            .audit_ref
            .trim()
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
            .collect::<String>();
        if sanitized.is_empty() {
            None
        } else {
            Some(sanitized)
        }
    }

    pub(super) fn requested_memory_workspace_id(&self) -> Option<String> {
        let value = self.workspace_id.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }

    pub(super) fn requested_memory_channel_id(&self) -> Option<String> {
        let value = self.channel_id.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }

    pub(super) fn requested_memory_actor_id(&self) -> Option<String> {
        let value = self.actor_id.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }

    pub(super) fn requested_memory_limit(&self) -> usize {
        self.limit
            .trim()
            .parse::<usize>()
            .ok()
            .map(|value| value.clamp(1, 25))
            .unwrap_or(25)
    }

    pub(super) fn requested_memory_type(&self) -> Option<String> {
        let value = self.memory_type.trim();
        if value.is_empty() {
            return None;
        }
        MemoryType::parse(value).map(|memory_type| memory_type.as_str().to_string())
    }

    pub(super) fn requested_memory_create_status(&self) -> &'static str {
        match self.create_status.trim() {
            "created" => "created",
            "updated" => "updated",
            _ => "idle",
        }
    }

    pub(super) fn requested_memory_created_entry_id(&self) -> Option<String> {
        let value = self.created_memory_id.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }

    pub(super) fn requested_memory_delete_status(&self) -> &'static str {
        match self.delete_status.trim() {
            "deleted" => "deleted",
            _ => "idle",
        }
    }

    pub(super) fn requested_memory_deleted_entry_id(&self) -> Option<String> {
        let value = self.deleted_memory_id.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }

    pub(super) fn requested_memory_detail_entry_id(&self) -> Option<String> {
        let value = self.detail_memory_id.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }

    pub(super) fn requested_memory_preview_entry_id(&self) -> Option<String> {
        let value = self.preview_memory_id.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }

    pub(super) fn requested_memory_graph_zoom_level(&self) -> f32 {
        self.graph_zoom
            .trim()
            .parse::<f32>()
            .ok()
            .map(|value| value.clamp(0.25, 2.0))
            .unwrap_or(1.0)
    }

    pub(super) fn requested_memory_graph_pan_x_level(&self) -> f32 {
        self.graph_pan_x
            .trim()
            .parse::<f32>()
            .ok()
            .map(|value| value.clamp(-500.0, 500.0))
            .unwrap_or(0.0)
    }

    pub(super) fn requested_memory_graph_pan_y_level(&self) -> f32 {
        self.graph_pan_y
            .trim()
            .parse::<f32>()
            .ok()
            .map(|value| value.clamp(-500.0, 500.0))
            .unwrap_or(0.0)
    }

    pub(super) fn requested_memory_graph_filter_memory_type(&self) -> String {
        let value = self.graph_filter_memory_type.trim();
        if value.is_empty() {
            return "all".to_string();
        }
        MemoryType::parse(value)
            .map(|memory_type| memory_type.as_str().to_string())
            .unwrap_or_else(|| "all".to_string())
    }

    pub(super) fn requested_memory_graph_filter_relation_type(&self) -> String {
        normalize_memory_graph_relation_filter(self.graph_filter_relation_type.as_str())
    }

    pub(super) fn requested_tool_name(&self) -> Option<String> {
        let value = self.tool.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }

    pub(super) fn requested_job_id(&self) -> Option<String> {
        let value = self.job.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }

    pub(super) fn requested_cancel_job_id(&self) -> Option<String> {
        let value = self.cancel_job.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }

    pub(super) fn requested_control_action_status(&self) -> &'static str {
        match self.control_action_status.trim() {
            "applied" => "applied",
            "missing" => "missing",
            "failed" => "failed",
            _ => "idle",
        }
    }

    pub(super) fn requested_chat_send_status(&self) -> &'static str {
        match self.chat_status.trim() {
            "empty-message" => "empty-message",
            _ => "idle",
        }
    }

    pub(super) fn requested_chat_new_session_status(&self) -> &'static str {
        match self.new_session_status.trim() {
            "empty-key" => "empty-key",
            "created" => "created",
            _ => "idle",
        }
    }

    pub(super) fn requested_control_action(&self) -> &'static str {
        match self.control_action.trim() {
            "pause" => "pause",
            "resume" => "resume",
            "refresh" => "refresh",
            _ => "none",
        }
    }

    pub(super) fn requested_control_action_reason(&self) -> &'static str {
        match self.control_action_reason.trim() {
            "control_action_applied" => "control_action_applied",
            "control_action_form_missing_action" => "missing_action",
            "missing_action" => "missing_action",
            "invalid_dashboard_action" => "invalid_dashboard_action",
            "unauthorized" => "unauthorized",
            "internal_error" => "internal_error",
            _ => "none",
        }
    }

    pub(super) fn requested_channel_action_status(&self) -> &'static str {
        match self.channel_action_status.trim() {
            "applied" => "applied",
            "missing" => "missing",
            "failed" => "failed",
            _ => "idle",
        }
    }

    pub(super) fn requested_channel_action(&self) -> &'static str {
        match self.channel_action.trim() {
            "login" => "login",
            "logout" => "logout",
            "probe" => "probe",
            "status" => "status",
            _ => "none",
        }
    }

    pub(super) fn requested_channel_action_channel(&self) -> &'static str {
        match self.channel_action_channel.trim() {
            "telegram" => "telegram",
            "discord" => "discord",
            "whatsapp" => "whatsapp",
            _ => "none",
        }
    }

    pub(super) fn requested_channel_action_reason(&self) -> &'static str {
        match self.channel_action_reason.trim() {
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
}

#[cfg(test)]
mod tests {
    use super::OpsShellControlsQuery;

    #[test]
    fn unit_timeline_range_returns_selected_supported_values() {
        let six_hours = OpsShellControlsQuery {
            range: "6h".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(six_hours.timeline_range(), "6h");

        let twenty_four_hours = OpsShellControlsQuery {
            range: "24h".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(twenty_four_hours.timeline_range(), "24h");
    }

    #[test]
    fn unit_timeline_range_defaults_to_one_hour_for_invalid_values() {
        let invalid = OpsShellControlsQuery {
            range: "quarter".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(invalid.timeline_range(), "1h");

        let empty = OpsShellControlsQuery::default();
        assert_eq!(empty.timeline_range(), "1h");
    }

    #[test]
    fn unit_requested_session_key_prefers_explicit_session_key_over_session_alias() {
        let controls = OpsShellControlsQuery {
            session_key: "priority-key".to_string(),
            session: "fallback-key".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(controls.requested_session_key(), Some("priority-key"));
    }

    #[test]
    fn unit_requested_session_key_returns_none_when_both_inputs_empty() {
        let controls = OpsShellControlsQuery::default();
        assert_eq!(controls.requested_session_key(), None);
    }

    #[test]
    fn unit_requested_memory_query_returns_trimmed_query_when_present() {
        let controls = OpsShellControlsQuery {
            query: " ArcSwap ".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(controls.requested_memory_query(), Some("ArcSwap"));
    }

    #[test]
    fn unit_requested_memory_workspace_id_trims_and_normalizes_empty_values() {
        let controls = OpsShellControlsQuery {
            workspace_id: " workspace-a ".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(
            controls.requested_memory_workspace_id().as_deref(),
            Some("workspace-a")
        );

        let empty = OpsShellControlsQuery::default();
        assert_eq!(empty.requested_memory_workspace_id(), None);
    }

    #[test]
    fn unit_requested_memory_channel_id_trims_and_normalizes_empty_values() {
        let controls = OpsShellControlsQuery {
            channel_id: " channel-a ".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(
            controls.requested_memory_channel_id().as_deref(),
            Some("channel-a")
        );

        let empty = OpsShellControlsQuery::default();
        assert_eq!(empty.requested_memory_channel_id(), None);
    }

    #[test]
    fn unit_requested_memory_actor_id_trims_and_normalizes_empty_values() {
        let controls = OpsShellControlsQuery {
            actor_id: " operator ".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(
            controls.requested_memory_actor_id().as_deref(),
            Some("operator")
        );

        let empty = OpsShellControlsQuery::default();
        assert_eq!(empty.requested_memory_actor_id(), None);
    }

    #[test]
    fn unit_requested_memory_limit_parses_and_clamps_supported_values() {
        let valid = OpsShellControlsQuery {
            limit: "7".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(valid.requested_memory_limit(), 7);

        let too_large = OpsShellControlsQuery {
            limit: "250".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(too_large.requested_memory_limit(), 25);

        let invalid = OpsShellControlsQuery {
            limit: "not-a-number".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(invalid.requested_memory_limit(), 25);
    }

    #[test]
    fn unit_requested_memory_type_normalizes_supported_values() {
        let valid = OpsShellControlsQuery {
            memory_type: " Goal ".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(valid.requested_memory_type().as_deref(), Some("goal"));

        let invalid = OpsShellControlsQuery {
            memory_type: "unsupported".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(invalid.requested_memory_type(), None);
    }

    #[test]
    fn unit_requested_memory_create_status_defaults_to_idle_and_accepts_known_states() {
        let idle = OpsShellControlsQuery::default();
        assert_eq!(idle.requested_memory_create_status(), "idle");

        let created = OpsShellControlsQuery {
            create_status: "created".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(created.requested_memory_create_status(), "created");

        let updated = OpsShellControlsQuery {
            create_status: "updated".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(updated.requested_memory_create_status(), "updated");

        let invalid = OpsShellControlsQuery {
            create_status: "invalid".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(invalid.requested_memory_create_status(), "idle");
    }

    #[test]
    fn unit_requested_memory_created_entry_id_trims_and_normalizes_empty_values() {
        let valid = OpsShellControlsQuery {
            created_memory_id: " mem-create-1 ".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(
            valid.requested_memory_created_entry_id().as_deref(),
            Some("mem-create-1")
        );

        let empty = OpsShellControlsQuery::default();
        assert_eq!(empty.requested_memory_created_entry_id(), None);
    }

    #[test]
    fn unit_requested_memory_delete_status_defaults_to_idle_and_accepts_deleted() {
        let idle = OpsShellControlsQuery::default();
        assert_eq!(idle.requested_memory_delete_status(), "idle");

        let deleted = OpsShellControlsQuery {
            delete_status: "deleted".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(deleted.requested_memory_delete_status(), "deleted");

        let invalid = OpsShellControlsQuery {
            delete_status: "invalid".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(invalid.requested_memory_delete_status(), "idle");
    }

    #[test]
    fn unit_requested_memory_deleted_entry_id_trims_and_normalizes_empty_values() {
        let valid = OpsShellControlsQuery {
            deleted_memory_id: " mem-delete-1 ".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(
            valid.requested_memory_deleted_entry_id().as_deref(),
            Some("mem-delete-1")
        );

        let empty = OpsShellControlsQuery::default();
        assert_eq!(empty.requested_memory_deleted_entry_id(), None);
    }

    #[test]
    fn unit_requested_memory_detail_entry_id_trims_and_normalizes_empty_values() {
        let valid = OpsShellControlsQuery {
            detail_memory_id: " mem-detail-1 ".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(
            valid.requested_memory_detail_entry_id().as_deref(),
            Some("mem-detail-1")
        );

        let empty = OpsShellControlsQuery::default();
        assert_eq!(empty.requested_memory_detail_entry_id(), None);
    }

    #[test]
    fn unit_requested_memory_preview_entry_id_trims_and_normalizes_empty_values() {
        let valid = OpsShellControlsQuery {
            preview_memory_id: " mem-preview-1 ".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(
            valid.requested_memory_preview_entry_id().as_deref(),
            Some("mem-preview-1")
        );

        let empty = OpsShellControlsQuery::default();
        assert_eq!(empty.requested_memory_preview_entry_id(), None);
    }

    #[test]
    fn unit_requested_memory_graph_zoom_level_defaults_and_clamps_values() {
        let default_zoom = OpsShellControlsQuery::default();
        assert_eq!(default_zoom.requested_memory_graph_zoom_level(), 1.0);

        let valid = OpsShellControlsQuery {
            graph_zoom: "1.55".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(valid.requested_memory_graph_zoom_level(), 1.55);

        let too_low = OpsShellControlsQuery {
            graph_zoom: "0.01".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(too_low.requested_memory_graph_zoom_level(), 0.25);

        let too_high = OpsShellControlsQuery {
            graph_zoom: "9.99".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(too_high.requested_memory_graph_zoom_level(), 2.0);
    }

    #[test]
    fn unit_requested_memory_graph_pan_levels_default_and_clamp_values() {
        let default_pan = OpsShellControlsQuery::default();
        assert_eq!(default_pan.requested_memory_graph_pan_x_level(), 0.0);
        assert_eq!(default_pan.requested_memory_graph_pan_y_level(), 0.0);

        let valid = OpsShellControlsQuery {
            graph_pan_x: "325.5".to_string(),
            graph_pan_y: "-124.25".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(valid.requested_memory_graph_pan_x_level(), 325.5);
        assert_eq!(valid.requested_memory_graph_pan_y_level(), -124.25);

        let clamped = OpsShellControlsQuery {
            graph_pan_x: "9999".to_string(),
            graph_pan_y: "-9999".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(clamped.requested_memory_graph_pan_x_level(), 500.0);
        assert_eq!(clamped.requested_memory_graph_pan_y_level(), -500.0);
    }

    #[test]
    fn unit_requested_memory_graph_filter_memory_type_defaults_and_normalizes_values() {
        let default_filters = OpsShellControlsQuery::default();
        assert_eq!(
            default_filters.requested_memory_graph_filter_memory_type(),
            "all"
        );

        let valid = OpsShellControlsQuery {
            graph_filter_memory_type: "goal".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(valid.requested_memory_graph_filter_memory_type(), "goal");

        let invalid = OpsShellControlsQuery {
            graph_filter_memory_type: "unknown".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(invalid.requested_memory_graph_filter_memory_type(), "all");
    }

    #[test]
    fn unit_requested_memory_graph_filter_relation_type_defaults_and_normalizes_values() {
        let default_filters = OpsShellControlsQuery::default();
        assert_eq!(
            default_filters.requested_memory_graph_filter_relation_type(),
            "all"
        );

        let valid = OpsShellControlsQuery {
            graph_filter_relation_type: "related_to".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(
            valid.requested_memory_graph_filter_relation_type(),
            "related_to"
        );

        let alias = OpsShellControlsQuery {
            graph_filter_relation_type: "supports".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(
            alias.requested_memory_graph_filter_relation_type(),
            "supports"
        );

        let raw_lineage = OpsShellControlsQuery {
            graph_filter_relation_type: "contains".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(
            raw_lineage.requested_memory_graph_filter_relation_type(),
            "contains"
        );

        let invalid = OpsShellControlsQuery {
            graph_filter_relation_type: "<unknown>".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(invalid.requested_memory_graph_filter_relation_type(), "all");
    }

    #[test]
    fn unit_requested_harness_route_action_normalizes_supported_values() {
        let intent = OpsShellControlsQuery {
            intent: "new-mission".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(intent.requested_harness_intent(), Some("new-mission"));

        let view = OpsShellControlsQuery {
            view: "history".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(view.requested_harness_view(), Some("history"));

        let audit = OpsShellControlsQuery {
            audit_action: "run-benchmark".to_string(),
            audit_ref: "1778419944988".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(
            audit.requested_harness_audit_action(),
            Some("run-benchmark")
        );
        assert_eq!(
            audit.requested_harness_audit_ref(),
            Some("1778419944988".to_string())
        );

        let invalid = OpsShellControlsQuery {
            intent: "delete-all".to_string(),
            view: "unknown".to_string(),
            mission_status: "unknown".to_string(),
            proposal_status: "unknown".to_string(),
            audit_action: "wipe".to_string(),
            audit_ref: "../../wipe".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(invalid.requested_harness_intent(), None);
        assert_eq!(invalid.requested_harness_view(), None);
        assert_eq!(invalid.requested_harness_mission_status(), None);
        assert_eq!(invalid.requested_harness_proposal_status(), None);
        assert_eq!(invalid.requested_harness_audit_action(), None);
        assert_eq!(
            invalid.requested_harness_audit_ref(),
            Some("wipe".to_string())
        );
    }

    #[test]
    fn unit_requested_harness_mission_status_normalizes_supported_values() {
        let created = OpsShellControlsQuery {
            mission_id: "mission-draft-123".to_string(),
            mission_status: "draft_created".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(
            created.requested_harness_mission_id(),
            Some("mission-draft-123")
        );
        assert_eq!(
            created.requested_harness_mission_status(),
            Some("draft_created")
        );

        for status in [
            "mission_started",
            "mission_completed",
            "mission_blocked",
            "start_failed",
        ] {
            let controls = OpsShellControlsQuery {
                mission_status: status.to_string(),
                ..OpsShellControlsQuery::default()
            };
            assert_eq!(
                controls.requested_harness_mission_status(),
                Some(status),
                "status `{status}` should be supported"
            );
        }
    }

    #[test]
    fn unit_requested_harness_proposal_status_normalizes_supported_values() {
        for status in [
            "applied",
            "apply_failed",
            "approved",
            "dry_run_failed",
            "dry_run_passed",
            "rejected",
        ] {
            let controls = OpsShellControlsQuery {
                proposal_status: status.to_string(),
                ..OpsShellControlsQuery::default()
            };
            assert_eq!(
                controls.requested_harness_proposal_status(),
                Some(status),
                "proposal status `{status}` should be supported"
            );
        }
    }

    #[test]
    fn unit_requested_tool_name_returns_trimmed_name_or_none() {
        let controls = OpsShellControlsQuery {
            tool: " bash ".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(controls.requested_tool_name(), Some("bash".to_string()));

        let empty = OpsShellControlsQuery::default();
        assert_eq!(empty.requested_tool_name(), None);
    }

    #[test]
    fn unit_requested_job_id_returns_trimmed_value_or_none() {
        let controls = OpsShellControlsQuery {
            job: " job-002 ".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(controls.requested_job_id(), Some("job-002".to_string()));

        let empty = OpsShellControlsQuery::default();
        assert_eq!(empty.requested_job_id(), None);
    }

    #[test]
    fn unit_requested_cancel_job_id_returns_trimmed_value_or_none() {
        let controls = OpsShellControlsQuery {
            cancel_job: " job-001 ".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(
            controls.requested_cancel_job_id(),
            Some("job-001".to_string())
        );

        let empty = OpsShellControlsQuery::default();
        assert_eq!(empty.requested_cancel_job_id(), None);
    }

    #[test]
    fn unit_requested_control_action_status_defaults_to_idle_and_normalizes_values() {
        let idle = OpsShellControlsQuery::default();
        assert_eq!(idle.requested_control_action_status(), "idle");

        let applied = OpsShellControlsQuery {
            control_action_status: "applied".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(applied.requested_control_action_status(), "applied");

        let failed = OpsShellControlsQuery {
            control_action_status: "failed".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(failed.requested_control_action_status(), "failed");

        let invalid = OpsShellControlsQuery {
            control_action_status: "unsupported".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(invalid.requested_control_action_status(), "idle");
    }

    #[test]
    fn unit_requested_control_action_defaults_to_none_and_normalizes_values() {
        let empty = OpsShellControlsQuery::default();
        assert_eq!(empty.requested_control_action(), "none");

        let pause = OpsShellControlsQuery {
            control_action: "pause".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(pause.requested_control_action(), "pause");

        let invalid = OpsShellControlsQuery {
            control_action: "explode".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(invalid.requested_control_action(), "none");
    }

    #[test]
    fn unit_requested_control_action_reason_defaults_and_normalizes_values() {
        let empty = OpsShellControlsQuery::default();
        assert_eq!(empty.requested_control_action_reason(), "none");

        let applied = OpsShellControlsQuery {
            control_action_reason: "control_action_applied".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(
            applied.requested_control_action_reason(),
            "control_action_applied"
        );

        let alias = OpsShellControlsQuery {
            control_action_reason: "control_action_form_missing_action".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(alias.requested_control_action_reason(), "missing_action");

        let invalid = OpsShellControlsQuery {
            control_action_reason: "custom".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(invalid.requested_control_action_reason(), "none");
    }
}
