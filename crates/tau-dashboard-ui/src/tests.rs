use super::{
    contains_markdown_contract_syntax, extract_assistant_stream_tokens,
    extract_first_fenced_code_block, format_chat_session_updated_label_at,
    render_tau_ops_dashboard_shell, render_tau_ops_dashboard_shell_for_route,
    render_tau_ops_dashboard_shell_with_context, TauOpsDashboardAlertFeedRow,
    TauOpsDashboardAuthMode, TauOpsDashboardChatMessageRow, TauOpsDashboardChatSessionOptionRow,
    TauOpsDashboardChatSnapshot, TauOpsDashboardCommandCenterSnapshot,
    TauOpsDashboardConnectorHealthRow, TauOpsDashboardHarnessAuditRow,
    TauOpsDashboardHarnessBenchmarkCategoryRow, TauOpsDashboardHarnessMissionRow,
    TauOpsDashboardHarnessProofRow, TauOpsDashboardHarnessSelfImprovementProof,
    TauOpsDashboardHarnessSnapshot, TauOpsDashboardJobRow, TauOpsDashboardMemoryGraphEdgeRow,
    TauOpsDashboardMemoryGraphNodeRow, TauOpsDashboardRoute, TauOpsDashboardSessionGraphEdgeRow,
    TauOpsDashboardSessionGraphNodeRow, TauOpsDashboardSessionTimelineRow,
    TauOpsDashboardShellContext, TauOpsDashboardSidebarState, TauOpsDashboardTheme,
    TauOpsDashboardToolInventoryRow, TauOpsDashboardToolInvocationRow,
    TauOpsDashboardToolUsageHistogramRow,
};
use tau_tui::{render_operator_shell_frame, OperatorShellFrame};

#[test]
fn unit_contains_markdown_contract_syntax_rejects_plain_text() {
    assert!(!contains_markdown_contract_syntax("plain response"));
}

#[test]
fn unit_contains_markdown_contract_syntax_accepts_fenced_code_only() {
    assert!(contains_markdown_contract_syntax(
        "```rust\nfn main() {}\n```"
    ));
}

#[test]
fn unit_contains_markdown_contract_syntax_rejects_pipe_without_table_delimiter() {
    assert!(!contains_markdown_contract_syntax("left|right"));
}

#[test]
fn unit_contains_markdown_contract_syntax_accepts_each_non_table_marker_path() {
    assert!(contains_markdown_contract_syntax("# heading"));
    assert!(contains_markdown_contract_syntax("intro\n# heading"));
    assert!(contains_markdown_contract_syntax("- item"));
    assert!(contains_markdown_contract_syntax("intro\n- item"));
    assert!(contains_markdown_contract_syntax(
        "[docs](https://example.com)"
    ));
}

#[test]
fn unit_extract_first_fenced_code_block_parses_language_and_code_payload() {
    assert_eq!(
        extract_first_fenced_code_block("prefix ```rust\nfn main() {}\n``` suffix"),
        Some(("rust".to_string(), "fn main() {}".to_string()))
    );
}

#[test]
fn unit_extract_assistant_stream_tokens_normalizes_whitespace() {
    assert_eq!(
        extract_assistant_stream_tokens("stream   one\ntwo"),
        vec!["stream".to_string(), "one".to_string(), "two".to_string()]
    );
}

#[test]
fn unit_extract_assistant_stream_tokens_ignores_blank_content() {
    assert!(extract_assistant_stream_tokens("   \n\t  ").is_empty());
}

#[test]
fn unit_format_chat_session_updated_label_renders_operator_age_labels() {
    assert_eq!(format_chat_session_updated_label_at(0, 100_000), "never");
    assert_eq!(
        format_chat_session_updated_label_at(100_000, 100_000),
        "just now"
    );
    assert_eq!(
        format_chat_session_updated_label_at(100_000, 400_000),
        "5m ago"
    );
    assert_eq!(
        format_chat_session_updated_label_at(100_000, 10_900_000),
        "3h ago"
    );
    assert_eq!(
        format_chat_session_updated_label_at(100_000, 172_900_000),
        "2d ago"
    );
}

#[test]
fn functional_render_shell_includes_foundation_markers() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("id=\"tau-ops-shell\""));
    assert!(html.contains("id=\"tau-ops-header\""));
    assert!(html.contains("id=\"tau-ops-sidebar\""));
    assert!(html.contains("id=\"tau-ops-command-center\""));
}

#[test]
fn regression_render_shell_includes_prd_component_contract_markers() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("data-component=\"HealthBadge\""));
    assert!(html.contains("data-component=\"StatCard\""));
    assert!(html.contains("data-component=\"AlertFeed\""));
    assert!(html.contains("data-component=\"DataTable\""));
}

#[test]
fn spec_c01_deploy_route_renders_wizard_root_and_steps() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/deploy");
    assert!(html.contains("id=\"tau-ops-deploy-panel\""));
    assert!(html.contains("id=\"tau-ops-deploy-wizard-steps\""));
    assert!(html.contains("data-wizard-step=\"model\""));
    assert!(html.contains("data-wizard-step=\"review\""));
    assert!(html.contains("aria-hidden=\"false\""));
}

#[test]
fn spec_c02_deploy_route_renders_model_catalog_marker() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/deploy");
    assert!(html.contains("id=\"tau-ops-deploy-model-catalog\""));
    assert!(html.contains("data-component=\"ModelCatalogDropdown\""));
}

#[test]
fn spec_c03_deploy_route_renders_validation_and_review_markers() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/deploy");
    assert!(html.contains("id=\"tau-ops-deploy-validation\""));
    assert!(html.contains("data-component=\"StepValidation\""));
    assert!(html.contains("id=\"tau-ops-deploy-review\""));
    assert!(html.contains("data-component=\"DeployReviewSummary\""));
}

#[test]
fn spec_c04_deploy_route_renders_deploy_action_marker() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/deploy");
    assert!(html.contains("id=\"tau-ops-deploy-submit\""));
    assert!(html.contains("data-action=\"deploy-agent\""));
    assert!(html.contains("data-success-redirect-template=\"/ops/agents/{agent_id}\""));
}

#[test]
fn spec_c05_non_deploy_route_hides_deploy_panel_markers() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops");
    assert!(html.contains("id=\"tau-ops-deploy-panel\""));
    assert!(html.contains("aria-hidden=\"true\""));
    assert!(html.contains("data-panel-visible=\"false\""));
}

#[test]
fn spec_c01_stream_contract_declares_websocket_connect_on_load() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("id=\"tau-ops-stream-contract\""));
    assert!(html.contains("data-stream-transport=\"websocket\""));
    assert!(html.contains("data-stream-connect-on-load=\"true\""));
}

#[test]
fn spec_c02_stream_contract_declares_heartbeat_target() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("data-heartbeat-target=\"tau-ops-kpi-grid\""));
}

#[test]
fn spec_c03_stream_contract_declares_alert_feed_target() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("data-alert-feed-target=\"tau-ops-alert-feed-list\""));
}

#[test]
fn spec_c04_stream_contract_declares_chat_token_stream_without_polling() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("data-chat-stream-mode=\"websocket\""));
    assert!(html.contains("data-chat-polling=\"disabled\""));
}

#[test]
fn spec_c05_stream_contract_declares_connector_health_target() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("data-connector-health-target=\"tau-ops-connector-table-body\""));
}

#[test]
fn spec_c06_stream_contract_declares_reconnect_backoff_strategy() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("data-reconnect-strategy=\"exponential-backoff\""));
    assert!(html.contains("data-reconnect-base-ms=\"250\""));
    assert!(html.contains("data-reconnect-max-ms=\"8000\""));
}

#[test]
fn spec_c01_accessibility_contract_section_marker_exists() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("id=\"tau-ops-accessibility-contract\""));
    assert!(html.contains("data-axe-contract=\"required\""));
}

#[test]
fn spec_c02_accessibility_keyboard_navigation_markers_exist() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("id=\"tau-ops-skip-to-main\""));
    assert!(html.contains("data-keyboard-navigation=\"true\""));
}

#[test]
fn spec_c03_accessibility_live_region_markers_exist() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("id=\"tau-ops-live-announcer\""));
    assert!(html.contains("aria-live=\"polite\""));
    assert!(html.contains("aria-atomic=\"true\""));
}

#[test]
fn spec_c04_accessibility_focus_indicator_markers_exist() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("data-focus-visible-contract=\"true\""));
    assert!(html.contains("data-focus-ring-token=\"tau-focus-ring\""));
}

#[test]
fn spec_c05_accessibility_reduced_motion_marker_exists() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("data-reduced-motion-contract=\"prefers-reduced-motion\""));
    assert!(html.contains("data-reduced-motion-behavior=\"suppress-nonessential-animation\""));
}

#[test]
fn spec_c01_performance_contract_declares_wasm_budget_marker() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("id=\"tau-ops-performance-contract\""));
    assert!(html.contains("data-wasm-budget-gzip-kb=\"500\""));
}

#[test]
fn spec_c02_performance_contract_declares_lcp_budget_marker() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("data-lcp-budget-ms=\"1500\""));
}

#[test]
fn spec_c03_performance_contract_declares_layout_shift_skeleton_markers() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("data-layout-shift-budget=\"0.00\""));
    assert!(html.contains("data-layout-shift-mitigation=\"skeletons\""));
}

#[test]
fn spec_c04_performance_contract_declares_websocket_processing_budget_marker() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("data-websocket-process-budget-ms=\"50\""));
}

#[test]
fn functional_spec_2786_c03_shell_exposes_auth_bootstrap_markers() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("id=\"tau-ops-auth-shell\""));
    assert!(html.contains("data-auth-mode=\"token\""));
    assert!(html.contains("data-login-required=\"true\""));
    assert!(html.contains("id=\"tau-ops-login-shell\""));
    assert!(html.contains("id=\"tau-ops-protected-shell\""));
}

#[test]
fn conformance_spec_2786_c03_shell_login_route_marks_login_panel_visible() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::PasswordSession,
        active_route: TauOpsDashboardRoute::Login,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });
    assert!(html.contains("data-auth-mode=\"password-session\""));
    assert!(html.contains("data-active-route=\"login\""));
    assert!(html.contains("id=\"tau-ops-login-shell\""));
    assert!(html.contains("aria-hidden=\"false\""));
    assert!(html.contains("id=\"tau-ops-protected-shell\""));
}

#[test]
fn regression_spec_2786_c03_shell_none_mode_marks_auth_not_required() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::None,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });
    assert!(html.contains("data-auth-mode=\"none\""));
    assert!(html.contains("data-login-required=\"false\""));
}

#[test]
fn spec_c28_regression_dashboard_and_tui_require_shared_operator_flow_markers() {
    let context = TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::PasswordSession,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            health_state: "degraded".to_string(),
            health_reason: "queue backlog and connector retries observed".to_string(),
            queue_depth: 3,
            failure_streak: 2,
            primary_alert_code: "dashboard_queue_backlog".to_string(),
            primary_alert_severity: "warning".to_string(),
            primary_alert_message: "runtime backlog detected (queue_depth=3)".to_string(),
            ..TauOpsDashboardCommandCenterSnapshot::default()
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    };

    let dashboard_html = render_tau_ops_dashboard_shell_with_context(context);
    assert!(dashboard_html.contains("data-auth-mode=\"password-session\""));
    assert!(dashboard_html
        .contains("data-health-reason=\"queue backlog and connector retries observed\""));
    assert!(dashboard_html.contains("id=\"tau-ops-kpi-queue-depth\""));
    assert!(dashboard_html.contains("data-kpi-value=\"3\""));
    assert!(dashboard_html.contains("data-primary-alert-code=\"dashboard_queue_backlog\""));

    let mut shell_frame = OperatorShellFrame::deterministic_fixture("ops-west".to_string());
    shell_frame.heartbeat = "degraded".to_string();
    shell_frame.auth_mode = "password-session".to_string();
    shell_frame.auth_required = true;
    shell_frame.health_reason = "queue backlog and connector retries observed".to_string();
    shell_frame.queue_depth = 3;
    shell_frame.failure_streak = 2;
    shell_frame.primary_alert_code = "dashboard_queue_backlog".to_string();
    shell_frame.primary_alert_severity = "warning".to_string();
    shell_frame.primary_alert_message = "runtime backlog detected (queue_depth=3)".to_string();
    let tui_rendered = render_operator_shell_frame(&shell_frame, 80).join("\n");

    for marker in [
        "auth.mode     : password-session",
        "auth.required : true",
        "health.reason : queue backlog and connector retries observed",
        "queue.depth        : 3",
        "failure.streak     : 2",
        "primary_alert.code     : dashboard_queue_backlog",
    ] {
        assert!(
            tui_rendered.contains(marker),
            "missing shared operator-flow marker `{marker}` in TUI output:\n{tui_rendered}"
        );
    }
}

#[test]
fn functional_spec_2790_c01_sidebar_includes_15_ops_route_links() {
    let html = render_tau_ops_dashboard_shell();
    assert_eq!(html.matches("data-nav-item=").count(), 15);

    let expected_routes = [
        "/ops",
        "/ops/agents",
        "/ops/agents/default",
        "/ops/chat",
        "/ops/sessions",
        "/ops/memory",
        "/ops/memory-graph",
        "/ops/tools-jobs",
        "/ops/channels",
        "/ops/harness",
        "/ops/config",
        "/ops/training",
        "/ops/safety",
        "/ops/diagnostics",
        "/ops/deploy",
    ];

    for route in expected_routes {
        assert!(
            html.contains(&format!("href=\"{route}\"")),
            "missing nav route {route}"
        );
    }
}

#[test]
fn functional_spec_3796_c01_non_harness_routes_use_operator_shell_chrome() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/deploy");

    for marker in [
        "data-shell-quality=\"operator-route-parity\"",
        "#tau-ops-layout {\n                    display: grid;",
        "grid-template-columns: 176px minmax(0, 1fr);",
        "#tau-ops-protected-shell {\n                    display: block;\n                    padding: 14px;\n                    width: calc(100vw - 208px);\n                    max-width: calc(100vw - 208px);",
        "#tau-ops-protected-shell > section[data-panel-visible=\"true\"]",
        "#tau-ops-sidebar a[aria-current=\"page\"]",
        "#tau-ops-deploy-wizard-steps ol {\n                    display: grid;\n                    grid-template-columns: minmax(0, 640px);",
        "id=\"tau-ops-deploy-panel\" data-route=\"/ops/deploy\" data-component=\"DeployWizard\" aria-hidden=\"false\" data-panel-visible=\"true\"",
    ] {
        assert!(
            html.contains(marker),
            "missing non-harness operator shell marker `{marker}`"
        );
    }

    assert!(
        !html.contains("Leptos SSR foundation shell"),
        "non-harness routes should not expose the foundation-shell placeholder subtitle"
    );
}

#[test]
fn functional_spec_3796_c02_left_nav_marks_active_route() {
    let deploy_html = render_tau_ops_dashboard_shell_for_route("/ops/deploy");
    assert_eq!(deploy_html.matches("aria-current=\"page\">").count(), 1);
    assert!(deploy_html.contains(
        "id=\"tau-ops-nav-deploy\"><a data-nav-item=\"deploy\" href=\"/ops/deploy\" data-harness-rail-label=\"Deploy\" aria-current=\"page\""
    ));
    assert!(deploy_html.contains(
        "id=\"tau-ops-nav-chat\"><a data-nav-item=\"chat\" href=\"/ops/chat\" data-harness-rail-label=\"Chat\" aria-current=\"false\""
    ));

    let chat_html = render_tau_ops_dashboard_shell_for_route("/ops/chat");
    assert_eq!(chat_html.matches("aria-current=\"page\">").count(), 1);
    assert!(chat_html.contains(
        "id=\"tau-ops-nav-chat\"><a data-nav-item=\"chat\" href=\"/ops/chat\" data-harness-rail-label=\"Chat\" aria-current=\"page\""
    ));
    assert!(chat_html.contains(
        "id=\"tau-ops-nav-deploy\"><a data-nav-item=\"deploy\" href=\"/ops/deploy\" data-harness-rail-label=\"Deploy\" aria-current=\"false\""
    ));
}

#[test]
fn functional_spec_3760_c02_c03_static_preview_link_guard_preserves_gateway_routes() {
    let html = render_tau_ops_dashboard_shell();

    for marker in [
        "data-nav-item=\"agent-fleet\" href=\"/ops/agents\"",
        "data-nav-item=\"mission-harness\" href=\"/ops/harness\"",
        "id=\"tau-ops-breadcrumb-home\"><a href=\"/ops\"",
        "id=\"tau-ops-static-preview-status\" data-preview-route-status=\"idle\" hidden",
        "id=\"tau-ops-static-preview-route-guard\" data-preview-link-guard=\"file-protocol-absolute-routes\"",
        "window.location.protocol !== \"file:\"",
        "anchor.setAttribute(\"data-preview-link-blocked\", \"true\")",
        "shell.contains(anchor)",
        "rawHref.charAt(0) !== \"/\" || rawHref.charAt(1) === \"/\"",
    ] {
        assert!(
            html.contains(marker),
            "missing static preview link guard marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_2790_c02_breadcrumb_markers_reflect_ops_route() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("id=\"tau-ops-breadcrumbs\""));
    assert!(html.contains("data-breadcrumb-current=\"command-center\""));
    assert!(html.contains("id=\"tau-ops-breadcrumb-current\""));
}

#[test]
fn functional_spec_2790_c03_breadcrumb_markers_reflect_login_route() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::PasswordSession,
        active_route: TauOpsDashboardRoute::Login,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });
    assert!(html.contains("id=\"tau-ops-breadcrumbs\""));
    assert!(html.contains("data-breadcrumb-current=\"login\""));
    assert!(html.contains("id=\"tau-ops-breadcrumb-current\""));
}

#[test]
fn functional_spec_2794_c02_c03_route_context_tokens_match_expected_values() {
    let route_cases = [
        (TauOpsDashboardRoute::Ops, "ops", "command-center"),
        (TauOpsDashboardRoute::Agents, "agents", "agent-fleet"),
        (
            TauOpsDashboardRoute::AgentDetail,
            "agent-detail",
            "agent-detail",
        ),
        (TauOpsDashboardRoute::Chat, "chat", "chat"),
        (TauOpsDashboardRoute::Sessions, "sessions", "sessions"),
        (TauOpsDashboardRoute::Memory, "memory", "memory"),
        (
            TauOpsDashboardRoute::MemoryGraph,
            "memory-graph",
            "memory-graph",
        ),
        (TauOpsDashboardRoute::ToolsJobs, "tools-jobs", "tools-jobs"),
        (TauOpsDashboardRoute::Channels, "channels", "channels"),
        (TauOpsDashboardRoute::Harness, "harness", "mission-harness"),
        (TauOpsDashboardRoute::Config, "config", "config"),
        (TauOpsDashboardRoute::Training, "training", "training"),
        (TauOpsDashboardRoute::Safety, "safety", "safety"),
        (
            TauOpsDashboardRoute::Diagnostics,
            "diagnostics",
            "diagnostics",
        ),
        (TauOpsDashboardRoute::Deploy, "deploy", "deploy"),
        (TauOpsDashboardRoute::Login, "login", "login"),
    ];

    for (route, expected_active_route, expected_breadcrumb) in route_cases {
        let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
            auth_mode: TauOpsDashboardAuthMode::Token,
            active_route: route,
            theme: TauOpsDashboardTheme::Dark,
            sidebar_state: TauOpsDashboardSidebarState::Expanded,
            command_center: TauOpsDashboardCommandCenterSnapshot::default(),
            chat: TauOpsDashboardChatSnapshot::default(),
            harness: TauOpsDashboardHarnessSnapshot::default(),
        });
        assert!(html.contains(&format!("data-active-route=\"{expected_active_route}\"")));
        assert!(html.contains(&format!(
            "data-breadcrumb-current=\"{expected_breadcrumb}\""
        )));
    }
}

#[test]
fn functional_spec_3756_c01_harness_route_renders_template_panels() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "id=\"tau-ops-nav-harness\"",
        "data-nav-item=\"mission-harness\" href=\"/ops/harness\"",
        "id=\"tau-ops-dashboard-base-style\"",
        "data-active-route=\"harness\"",
        "id=\"tau-ops-breadcrumbs\"",
        "data-breadcrumb-current=\"mission-harness\"",
        "id=\"tau-ops-harness-template-style\"",
        "id=\"tau-ops-harness-panel\" data-route=\"/ops/harness\" data-component=\"MissionHarnessWorkspace\" data-design-template=\"three-window-agent-harness\" aria-hidden=\"false\" data-panel-visible=\"true\"",
        "id=\"tau-ops-harness-topbar\" data-workspace=\"/workspace/tau\" data-model=\"gpt-5.4\" data-transport=\"gateway\" data-health=\"healthy\" data-window-chrome=\"compact\"",
        "<span data-topbar-field=\"workspace\">/workspace/tau</span>",
        "<span data-topbar-field=\"model\">gpt-5.4</span>",
        "<span data-topbar-field=\"transport\">gateway</span>",
        "<span data-topbar-field=\"health\">Healthy</span>",
        "id=\"tau-ops-harness-new-mission-form\" action=\"/ops/harness/missions/draft?theme=dark&amp;sidebar=expanded&amp;session=default&amp;proposal_id=PR-044\" method=\"post\" data-action-contract=\"durable-mission-draft\" data-preserves-shell-context=\"true\"",
        "id=\"tau-ops-harness-new-mission\" data-action=\"new-mission\" data-action-contract=\"durable-mission-draft\" data-preserves-session=\"true\" data-preserves-proposal=\"true\" type=\"submit\"",
        "id=\"tau-ops-harness-history\" data-action=\"history\" data-action-contract=\"context-preserving\" data-preserves-session=\"true\" data-preserves-proposal=\"true\" href=\"/ops/harness?theme=dark&amp;sidebar=expanded&amp;session=default&amp;proposal_id=PR-044&amp;view=history\"",
        "id=\"tau-ops-harness-route-action\" data-route-action-key=\"overview\" data-route-action-label=\"Overview\" data-route-action-count=\"0\" data-route-action-visible=\"false\" hidden",
        "id=\"tau-ops-harness-dashboard-window\"",
        "id=\"tau-ops-harness-proof-window\"",
        "id=\"tau-ops-harness-self-improvement-window\"",
        "id=\"tau-ops-harness-tui-companion\"",
    ] {
        assert!(
            html.contains(marker),
            "missing harness template marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3756_c02_harness_controls_expose_benchmark_and_policy_contracts() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "id=\"tau-ops-harness-kpi-grid\" data-kpi-card-count=\"4\"",
        "id=\"tau-ops-harness-benchmark-panel\" data-benchmark-id=\"m334-tranche-one-autonomy\" data-proof-artifact=\"/artifacts/bench/m334/latest.json\" data-task-count=\"4\" data-pass-count=\"4\" data-failed-gates=\"none\"",
        "id=\"tau-ops-harness-run-benchmark-form\" action=\"/ops/harness/run-benchmark?theme=dark&amp;sidebar=expanded&amp;session=default&amp;proposal_id=PR-044\" method=\"post\" data-command=\"tau_agent_harness\" data-preserves-shell-context=\"true\"",
        "id=\"tau-ops-harness-gate-memory\" data-gate-id=\"VG-03\" data-gate-status=\"failed\"",
        "id=\"tau-ops-harness-gate-learning\" data-gate-id=\"VG-05\" data-gate-status=\"pending\"",
        "id=\"tau-ops-harness-conservative-policy\" data-policy=\"conservative-self-improvement\" data-allowed-targets=\"skill,config,prompt\" data-blocked-targets=\"source-code,safety-policy\"",
        "id=\"tau-ops-harness-operator-actions\" data-apply-requires-approval=\"true\"",
        "id=\"tau-ops-harness-action-approve\" type=\"submit\" data-action=\"approve\"",
        "id=\"tau-ops-harness-action-reject\" type=\"submit\" data-action=\"reject\"",
        "id=\"tau-ops-harness-action-dry-run\" type=\"submit\" data-action=\"dry-run\"",
        "id=\"tau-ops-harness-action-view-diff\" data-action=\"view-diff\" data-action-tone=\"secondary\" href=\"/ops/harness/proposals/PR-044/diff\"",
        "id=\"tau-ops-harness-apply-form\" action=\"/ops/harness/proposals/PR-044/apply?theme=dark&amp;sidebar=expanded&amp;session=default\" method=\"post\" data-approval-state=\"approval-required\"",
        "id=\"tau-ops-harness-action-apply\" type=\"submit\" data-action=\"apply\" data-action-tone=\"disabled\" data-disabled=\"true\" aria-disabled=\"true\" data-approval-required=\"true\" disabled",
    ] {
        assert!(
            html.contains(marker),
            "missing harness control marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3757_c01_c02_harness_snapshot_drives_benchmark_and_audit_rows() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        active_route: TauOpsDashboardRoute::Harness,
        harness: TauOpsDashboardHarnessSnapshot {
            proof_source: "state".to_string(),
            benchmark_id: "m334-tranche-one-autonomy".to_string(),
            proof_artifact: "/state/ops-harness/m334/latest.json".to_string(),
            task_count: 7,
            pass_count: 6,
            failed_gate_count: 1,
            failed_gate_label: "1".to_string(),
            latest_result: "6/7".to_string(),
            latest_runtime: "state".to_string(),
            latest_cost: "0.00".to_string(),
            latest_summary: "Latest state-backed result: 6/7. Failed gates: 1.".to_string(),
            benchmark_rows: vec![TauOpsDashboardHarnessBenchmarkCategoryRow {
                category: "repo_build".to_string(),
                task_count: 3,
                pass_count: 2,
                total_count: 3,
                pass_rate: "67".to_string(),
            }],
            audit_source: "state".to_string(),
            audit_rows: vec![TauOpsDashboardHarnessAuditRow {
                timestamp_label: "ts:1777986484661".to_string(),
                timestamp_unix_ms: "1777986484661".to_string(),
                actor: "Gateway".to_string(),
                action_label: "Apply".to_string(),
                action_key: "apply".to_string(),
                scope: "Prompt".to_string(),
                item: "PR-044".to_string(),
                result_label: "Blocked Approval Required".to_string(),
                result_key: "blocked_approval_required".to_string(),
            }],
            ..TauOpsDashboardHarnessSnapshot::default()
        },
        ..TauOpsDashboardShellContext::default()
    });

    for marker in [
        "id=\"tau-ops-harness-benchmark-panel\" data-benchmark-id=\"m334-tranche-one-autonomy\" data-proof-artifact=\"/state/ops-harness/m334/latest.json\" data-task-count=\"7\" data-pass-count=\"6\" data-failed-gates=\"1\" data-proof-source=\"state\"",
        "data-category=\"repo_build\" data-task-count=\"3\" data-last-run=\"2/3 pass\" data-pass-rate=\"67\"",
        "id=\"tau-ops-harness-benchmark-latest\" data-latest-result=\"6/7\" data-runtime=\"state\" data-cost=\"0.00\"",
        "Latest state-backed result: 6/7. Failed gates: 1.",
        "id=\"tau-ops-harness-audit-log\" data-audit-row-count=\"1\" data-audit-source=\"state\"",
        "data-action=\"apply\" data-result=\"blocked_approval_required\" data-timestamp-unix-ms=\"1777986484661\"",
        "Blocked Approval Required",
    ] {
        assert!(
            html.contains(marker),
            "missing state-backed harness marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3758_c01_harness_uses_operator_console_visual_contract() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "data-layout-density=\"operator-console\"",
        "data-visual-contract=\"mission-control\"",
        "data-window-chrome=\"compact\"",
        "class=\"tau-harness-window-titlebar\"",
        "class=\"tau-harness-window-grid\"",
        "class=\"tau-harness-status-chip\"",
        "class=\"tau-harness-table-wrap\"",
        "grid-template-columns: 76px minmax(0, 1fr);",
    ] {
        assert!(
            html.contains(marker),
            "missing harness visual contract marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3758_c03_harness_style_guards_overflow_focus_and_responsive_layout() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "#tau-ops-harness-panel :is(a, button):focus-visible",
        ".tau-harness-table-wrap",
        "overflow-x: auto;",
        "grid-template-columns: minmax(0, .82fr) minmax(0, 1.04fr) minmax(0, .86fr);",
        "@media (max-width: 760px)",
        "minmax(0, 1fr)",
    ] {
        assert!(
            html.contains(marker),
            "missing harness responsive/focus style contract `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3761_c01_c02_c03_harness_keeps_desktop_three_window_layout() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "data-desktop-preview-layout=\"three-window\"",
        "data-responsive-collapse-width=\"1450px\"",
        "grid-template-columns: minmax(0, .82fr) minmax(0, 1.04fr) minmax(0, .86fr);",
        "#tau-ops-shell[data-active-route=\"harness\"] {\n                    background:",
        "max-width: 100vw;",
        "width: min(100%, calc(100vw - 108px));",
        "max-width: calc(100vw - 108px);",
        "@media (max-width: 1400px)",
        "@media (max-width: 1180px)",
        "max-height: calc(100vh - 88px);",
        "overflow: auto;",
        "overflow-wrap: anywhere;",
        "white-space: normal;",
        ".tau-harness-window-titlebar > div",
        "grid-template-columns: minmax(7rem, max-content) minmax(0, 1fr);",
        "#tau-ops-harness-self-improvement-window {\n                                max-height: calc(100vh - 354px);",
        "#tau-ops-harness-tui-companion {\n                                max-height: 200px;",
    ] {
        assert!(
            html.contains(marker),
            "missing harness desktop layout marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3795_c01_c02_harness_fits_in_app_browser_without_review_rail_clipping() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "data-responsive-collapse-width=\"1450px\"",
        "data-in-app-browser-fit=\"no-right-rail-clipping\"",
        "@media (max-width: 1450px)",
        "grid-template-areas:\n                                        \"topbar topbar\"\n                                        \"dashboard proof\"\n                                        \"review tui\";",
        "#tau-ops-harness-tui-companion {\n                                    max-height: 260px;",
    ] {
        assert!(
            html.contains(marker),
            "missing in-app browser fit marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3762_c01_c02_c03_harness_compacts_dashboard_without_clipping() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "data-compact-dashboard-breakpoint=\"1400px\"",
        "data-compact-table-breakpoint=\"1400px\"",
        "data-compact-metadata-breakpoint=\"1400px\"",
        "#tau-ops-harness-kpi-grid {\n                                    grid-template-columns: repeat(2, minmax(0, 1fr));",
        "#tau-ops-harness-active-missions,\n                            #tau-ops-harness-benchmark-panel {\n                                min-width: 0;",
        "#tau-ops-harness-panel #tau-ops-harness-missions-table,\n                                #tau-ops-harness-panel #tau-ops-harness-benchmark-table {\n                                    min-width: 0;",
        "table-layout: fixed;",
        "#tau-ops-harness-missions-table th:nth-child(n+4)",
        "white-space: normal;",
        "#tau-ops-harness-missions-table th {\n                                    font-size: .66rem;",
        "#tau-ops-harness-missions-table meter {\n                                    width: 58px;",
        "#tau-ops-harness-proof-header {\n                                    align-items: stretch;\n                                    flex-direction: column;",
        "#tau-ops-harness-proof-header dl {\n                                    grid-template-columns: minmax(6rem, max-content) minmax(0, 1fr);",
        "@media (max-width: 900px)",
    ] {
        assert!(
            html.contains(marker),
            "missing compact dashboard marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3763_c01_c02_c03_harness_wraps_proof_evidence_and_terminal_logs() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "data-compact-evidence-breakpoint=\"1400px\"",
        "data-log-wrap=\"pre-wrap\"",
        "#tau-ops-harness-tool-evidence table {\n                                    min-width: 0;",
        "#tau-ops-harness-tool-evidence th,\n                                #tau-ops-harness-tool-evidence td {\n                                    white-space: normal;",
        "#tau-ops-harness-tool-evidence th:nth-child(6)",
        "#tau-ops-harness-tool-evidence th:nth-child(2)",
        "#tau-ops-harness-operator-log pre,\n                                #tau-ops-harness-tui-companion pre {\n                                    overflow-x: hidden;",
        "white-space: pre-wrap;",
    ] {
        assert!(
            html.contains(marker),
            "missing compact proof/log marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3765_c01_c02_c03_harness_prioritizes_compact_evidence_columns() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "data-compact-call-id-visibility=\"hidden-at-1400px\"",
        "#tau-ops-harness-tool-evidence th:nth-child(6),\n                                #tau-ops-harness-tool-evidence td:nth-child(6),\n                                #tau-ops-harness-tool-evidence th:nth-child(2),\n                                #tau-ops-harness-tool-evidence td:nth-child(2) {\n                                    display: none;",
        "#tau-ops-harness-tool-evidence th:nth-child(4),\n                                #tau-ops-harness-tool-evidence td:nth-child(4),\n                                #tau-ops-harness-tool-evidence th:nth-child(5),\n                                #tau-ops-harness-tool-evidence td:nth-child(5) {\n                                    width: 82px;\n                                    white-space: nowrap;",
        "overflow-wrap: normal;",
    ] {
        assert!(
            html.contains(marker),
            "missing compact evidence column-priority marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3764_c01_c02_c03_harness_prioritizes_self_improvement_actions() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let action_index = html
        .find("id=\"tau-ops-harness-operator-actions\"")
        .expect("operator actions section should render");
    let policy_index = html
        .find("id=\"tau-ops-harness-conservative-policy\"")
        .expect("conservative policy section should render");
    let audit_index = html
        .find("id=\"tau-ops-harness-audit-log\"")
        .expect("audit log section should render");

    assert!(
        action_index < policy_index,
        "operator actions should be prioritized before conservative policy"
    );
    assert!(
        policy_index < audit_index,
        "conservative policy should remain before audit history"
    );

    for marker in [
        "data-review-action-placement=\"actions-before-detail\"",
        "id=\"tau-ops-harness-operator-actions\" data-apply-requires-approval=\"true\" data-action-row-priority=\"approval-flow\" data-action-grid=\"two-column-priority\" data-action-first-viewport=\"all-controls\"",
        "#tau-ops-harness-self-improvement-window {\n                                max-height: calc(100vh - 354px);",
        "#tau-ops-harness-tui-companion pre {\n                                max-height: 126px;",
        "#tau-ops-harness-operator-actions {\n                                display: grid;\n                                grid-template-columns: repeat(2, minmax(0, 1fr));",
        "#tau-ops-harness-operator-actions button,\n                            #tau-ops-harness-operator-actions a {\n                                width: 100%;",
        "#tau-ops-harness-action-apply {\n                                grid-column: 1 / -1;",
        "id=\"tau-ops-harness-approve-form\" action=\"/ops/harness/proposals/PR-044/approve?theme=dark&amp;sidebar=expanded&amp;session=default\" method=\"post\"",
        "id=\"tau-ops-harness-reject-form\" action=\"/ops/harness/proposals/PR-044/reject?theme=dark&amp;sidebar=expanded&amp;session=default\" method=\"post\"",
        "id=\"tau-ops-harness-dry-run-form\" action=\"/ops/harness/proposals/PR-044/dry-run?theme=dark&amp;sidebar=expanded&amp;session=default\" method=\"post\"",
        "id=\"tau-ops-harness-action-view-diff\" data-action=\"view-diff\" data-action-tone=\"secondary\" href=\"/ops/harness/proposals/PR-044/diff\"",
        "id=\"tau-ops-harness-action-apply\" type=\"submit\" data-action=\"apply\" data-action-tone=\"disabled\" data-disabled=\"true\" aria-disabled=\"true\" data-approval-required=\"true\" disabled",
    ] {
        assert!(
            html.contains(marker),
            "missing self-improvement action priority marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3766_c01_c02_c03_harness_uses_compact_navigation_rail() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "grid-template-columns: 76px minmax(0, 1fr);",
        "data-harness-rail=\"compact\"",
        "data-nav-item=\"command-center\" href=\"/ops\" data-harness-rail-label=\"Command\"",
        "data-nav-item=\"agent-fleet\" href=\"/ops/agents\" data-harness-rail-label=\"Fleet\"",
        "data-nav-item=\"agent-detail\" href=\"/ops/agents/default\" data-harness-rail-label=\"Agent\"",
        "data-nav-item=\"chat\" href=\"/ops/chat\" data-harness-rail-label=\"Chat\"",
        "data-nav-item=\"sessions\" href=\"/ops/sessions\" data-harness-rail-label=\"Sessions\"",
        "data-nav-item=\"memory\" href=\"/ops/memory\" data-harness-rail-label=\"Memory\"",
        "data-nav-item=\"memory-graph\" href=\"/ops/memory-graph\" data-harness-rail-label=\"Graph\"",
        "data-nav-item=\"tools-jobs\" href=\"/ops/tools-jobs\" data-harness-rail-label=\"Tools\"",
        "data-nav-item=\"channels\" href=\"/ops/channels\" data-harness-rail-label=\"Channels\"",
        "id=\"tau-ops-nav-harness\"><a data-nav-item=\"mission-harness\" href=\"/ops/harness\" data-harness-rail-label=\"Missions\" aria-current=\"page\">Mission Harness</a>",
        "data-nav-item=\"deploy\" href=\"/ops/deploy\" data-harness-rail-label=\"Deploy\"",
        "#tau-ops-shell[data-active-route=\"harness\"] #tau-ops-sidebar a {\n                    display: flex;",
        "font-size: 0;",
        "#tau-ops-shell[data-active-route=\"harness\"] #tau-ops-sidebar a::before {\n                    content: attr(data-harness-rail-label);",
        "width: min(100%, calc(100vw - 108px));",
        "max-width: calc(100vw - 108px);",
    ] {
        assert!(
            html.contains(marker),
            "missing compact harness rail marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3767_c01_c02_c03_harness_prioritizes_proof_evidence_in_primary_grid() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let evidence_index = html
        .find("id=\"tau-ops-harness-tool-evidence\"")
        .expect("tool evidence section should render");
    let acceptance_index = html
        .find("id=\"tau-ops-harness-acceptance\"")
        .expect("acceptance section should render");
    let gates_index = html
        .find("id=\"tau-ops-harness-verification-gates\"")
        .expect("verification gates section should render");
    let artifacts_index = html
        .find("id=\"tau-ops-harness-artifacts\"")
        .expect("artifacts section should render");

    assert!(
        evidence_index < acceptance_index,
        "tool evidence should precede acceptance criteria in the proof grid"
    );
    assert!(
        evidence_index < gates_index,
        "tool evidence should precede verification gates in the proof grid"
    );
    assert!(
        evidence_index < artifacts_index,
        "tool evidence should precede artifacts in the proof grid"
    );

    for marker in [
        "data-proof-grid-priority=\"evidence-log-gates-first\"",
        "id=\"tau-ops-harness-tool-evidence\"",
        "data-proof-evidence-priority=\"first-screen\"",
        "data-tool-call-count=\"8\"",
        "#tau-ops-harness-tool-evidence {\n                                grid-column: 1 / -1;",
        "data-compact-call-id-visibility=\"hidden-at-1400px\"",
    ] {
        assert!(
            html.contains(marker),
            "missing proof evidence priority marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3768_c01_c02_c03_harness_compacts_proof_dag_to_single_row() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "id=\"tau-ops-harness-plan-dag\" data-dag-node-count=\"5\" data-current-node=\"verify\" data-proof-dag-density=\"single-row\"",
        "grid-template-columns: repeat(5, minmax(0, 1fr));",
        "#tau-ops-harness-plan-dag li {\n                                border: 1px solid var(--tau-harness-line);\n                                border-radius: 999px;\n                                min-height: 30px;\n                                padding: 5px 4px;",
        "font-size: .66rem;",
        "white-space: nowrap;",
        "text-overflow: ellipsis;",
        "id=\"tau-ops-harness-dag-plan\" data-plan-node=\"Plan\" data-node-status=\"passed\"",
        "id=\"tau-ops-harness-dag-execute\" data-plan-node=\"Execute\" data-node-status=\"passed\"",
        "id=\"tau-ops-harness-dag-memory-write\" data-plan-node=\"Memory Write\" data-node-status=\"passed\"",
        "id=\"tau-ops-harness-dag-verify\" data-plan-node=\"Verify\" data-node-status=\"running\"",
        "id=\"tau-ops-harness-dag-learn\" data-plan-node=\"Learn\" data-node-status=\"pending\"",
    ] {
        assert!(
            html.contains(marker),
            "missing compact proof DAG marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3769_c01_c02_c03_harness_uses_distinct_operator_action_tones() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "id=\"tau-ops-harness-run-benchmark\" type=\"submit\" data-action-tone=\"benchmark\"",
        "id=\"tau-ops-harness-action-approve\" type=\"submit\" data-action=\"approve\" data-action-tone=\"approve\"",
        "id=\"tau-ops-harness-action-reject\" type=\"submit\" data-action=\"reject\" data-action-tone=\"reject\"",
        "id=\"tau-ops-harness-action-dry-run\" type=\"submit\" data-action=\"dry-run\" data-action-tone=\"secondary\"",
        "id=\"tau-ops-harness-action-view-diff\" data-action=\"view-diff\" data-action-tone=\"secondary\"",
        "id=\"tau-ops-harness-action-apply\" type=\"submit\" data-action=\"apply\" data-action-tone=\"disabled\" data-disabled=\"true\"",
        "#tau-ops-harness-panel button[type=\"submit\"] {\n                                background: #132a38;",
        "#tau-ops-harness-panel #tau-ops-harness-run-benchmark,\n                            #tau-ops-harness-panel #tau-ops-harness-action-dry-run {\n                                background: linear-gradient(180deg, #1d4f68, #173b4e);",
        "#tau-ops-harness-panel #tau-ops-harness-action-approve[data-action-tone=\"approve\"] {\n                                background: linear-gradient(180deg, #2d7446, #1e5934);",
        "#tau-ops-harness-panel #tau-ops-harness-action-reject[data-action-tone=\"reject\"] {\n                                background: linear-gradient(180deg, #7a342c, #5f261f);",
    ] {
        assert!(
            html.contains(marker),
            "missing distinct action tone marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3770_c01_c02_c03_harness_keeps_operator_log_in_first_proof_view() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let evidence_index = html
        .find("id=\"tau-ops-harness-tool-evidence\"")
        .expect("tool evidence section should render");
    let log_index = html
        .find("id=\"tau-ops-harness-operator-log\"")
        .expect("operator log section should render");
    let acceptance_index = html
        .find("id=\"tau-ops-harness-acceptance\"")
        .expect("acceptance section should render");
    let gates_index = html
        .find("id=\"tau-ops-harness-verification-gates\"")
        .expect("verification gates section should render");
    let artifacts_index = html
        .find("id=\"tau-ops-harness-artifacts\"")
        .expect("artifacts section should render");

    assert!(
        evidence_index < log_index,
        "operator log should remain after tool evidence"
    );
    assert!(
        log_index < acceptance_index,
        "operator log should be promoted before secondary acceptance detail"
    );
    assert!(
        log_index < gates_index,
        "operator log should be promoted before secondary gate detail"
    );
    assert!(
        log_index < artifacts_index,
        "operator log should be promoted before secondary artifact detail"
    );

    for marker in [
        "id=\"tau-ops-harness-operator-log\" data-log-follow=\"true\" data-log-wrap=\"pre-wrap\" data-log-priority=\"first-screen\"",
        "#tau-ops-harness-operator-log {\n                                grid-column: 1 / -1;",
        "#tau-ops-harness-operator-log pre {\n                                max-height: 118px;",
        "data-proof-grid-priority=\"evidence-log-gates-first\"",
    ] {
        assert!(
            html.contains(marker),
            "missing first-screen operator log marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3771_c01_c02_c03_harness_prioritizes_verification_gates_after_log() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let log_index = html
        .find("id=\"tau-ops-harness-operator-log\"")
        .expect("operator log section should render");
    let acceptance_index = html
        .find("id=\"tau-ops-harness-acceptance\"")
        .expect("acceptance section should render");
    let gates_index = html
        .find("id=\"tau-ops-harness-verification-gates\"")
        .expect("verification gates section should render");
    let memory_index = html
        .find("id=\"tau-ops-harness-memory-learning\"")
        .expect("memory learning section should render");
    let artifacts_index = html
        .find("id=\"tau-ops-harness-artifacts\"")
        .expect("artifacts section should render");

    assert!(
        log_index < acceptance_index,
        "acceptance should remain directly after the operator log"
    );
    assert!(
        acceptance_index < gates_index,
        "verification gates should be promoted after acceptance"
    );
    assert!(
        gates_index < memory_index,
        "verification gates should appear before memory summary"
    );
    assert!(
        gates_index < artifacts_index,
        "verification gates should appear before artifacts"
    );

    for marker in [
        "data-proof-grid-priority=\"evidence-log-gates-first\"",
        "id=\"tau-ops-harness-verification-gates\" data-gate-count=\"5\" data-failed-gate-count=\"1\" data-proof-secondary-priority=\"first-screen\"",
        "#tau-ops-harness-verification-gates li,\n                            #tau-ops-harness-self-improvement-proof li {\n                                padding: 3px 7px;",
        "#tau-ops-harness-acceptance ul,\n                            #tau-ops-harness-verification-gates ul {\n                                gap: 5px;",
    ] {
        assert!(
            html.contains(marker),
            "missing first-screen verification gate marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3772_c01_c02_c03_harness_keeps_mission_state_visible_in_compact_table() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "id=\"tau-ops-harness-active-missions\"",
        "data-active-count=\"5\"",
        "data-running-count=\"3\"",
        "data-blocked-count=\"1\"",
        "data-compact-table-breakpoint=\"1400px\"",
        "data-compact-mission-summary=\"status-and-gates\"",
        "data-mission-summary=\"inline-status\"",
        "class=\"tau-harness-mission-title\"",
        "class=\"tau-harness-mission-meta\"",
        "data-compact-mission-meta=\"status-gates\"",
        "data-mission-state-chip=\"running\"",
        "data-mission-state-chip=\"verifying\"",
        "data-mission-state-chip=\"completed\"",
        "data-mission-state-chip=\"blocked\"",
        "data-mission-gate-chip=\"needs-review\"",
        "data-mission-gate-chip=\"failed\"",
        "#tau-ops-harness-missions-table .tau-harness-mission-meta {\n                                display: flex;",
        "#tau-ops-harness-missions-table .tau-harness-mission-meta .tau-harness-status-chip {\n                                padding: 2px 6px;",
        "#tau-ops-harness-missions-table th:nth-child(n+4)",
    ] {
        assert!(
            html.contains(marker),
            "missing compact mission status marker `{marker}`"
        );
    }

    let first_row = html
        .find("id=\"tau-ops-harness-mission-row-0\"")
        .expect("first mission row should render");
    let first_title = html[first_row..]
        .find("class=\"tau-harness-mission-title\"")
        .expect("first mission title should render")
        + first_row;
    let first_state = html[first_row..]
        .find("data-mission-state-chip=\"running\"")
        .expect("first mission state chip should render")
        + first_row;
    let first_gate = html[first_row..]
        .find("data-mission-gate-chip=\"needs-review\"")
        .expect("first mission gate chip should render")
        + first_row;

    assert!(
        first_title < first_state,
        "mission state chip should follow the title inside the compact goal cell"
    );
    assert!(
        first_state < first_gate,
        "mission gate chip should follow the state chip inside the compact goal cell"
    );
}

#[test]
fn functional_spec_3773_c01_c02_c03_harness_keeps_tui_companion_in_first_viewport() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "id=\"tau-ops-harness-tui-companion\" data-component=\"TuiCompanion\" data-command=\"tau status\" data-window-chrome=\"compact\" data-log-wrap=\"pre-wrap\" data-tui-priority=\"first-viewport-summary\"",
        "#tau-ops-harness-self-improvement-window {\n                                max-height: calc(100vh - 354px);",
        "#tau-ops-harness-tui-companion {\n                                max-height: 200px;",
        "#tau-ops-harness-tui-companion pre {\n                                max-height: 126px;",
        "box-sizing: border-box;",
        "mission=run_8f3a2",
        "status=running",
        "tool_budget=42/60",
        "bench: ",
        "Proof: ",
    ] {
        assert!(
            html.contains(marker),
            "missing first-viewport TUI marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3774_c01_c02_c03_harness_keeps_all_operator_actions_visible_before_detail() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let queue_index = html
        .find("id=\"tau-ops-harness-learning-queue\"")
        .expect("learning queue should render");
    let action_index = html
        .find("id=\"tau-ops-harness-operator-actions\"")
        .expect("operator actions section should render");
    let detail_index = html
        .find("id=\"tau-ops-harness-proposal-detail\"")
        .expect("proposal detail should render");
    let policy_index = html
        .find("id=\"tau-ops-harness-conservative-policy\"")
        .expect("conservative policy should render");

    assert!(
        queue_index < action_index,
        "operator actions should follow the learning queue context"
    );
    assert!(
        action_index < detail_index,
        "operator actions should be visible before long proposal detail content"
    );
    assert!(
        action_index < policy_index,
        "operator actions should still be prioritized before conservative policy"
    );

    for marker in [
        "data-review-action-placement=\"actions-before-detail\"",
        "id=\"tau-ops-harness-operator-actions\" data-apply-requires-approval=\"true\" data-action-row-priority=\"approval-flow\" data-action-grid=\"two-column-priority\" data-action-first-viewport=\"all-controls\"",
        "id=\"tau-ops-harness-approve-form\" action=\"/ops/harness/proposals/PR-044/approve?theme=dark&amp;sidebar=expanded&amp;session=default\" method=\"post\"",
        "id=\"tau-ops-harness-reject-form\" action=\"/ops/harness/proposals/PR-044/reject?theme=dark&amp;sidebar=expanded&amp;session=default\" method=\"post\"",
        "id=\"tau-ops-harness-dry-run-form\" action=\"/ops/harness/proposals/PR-044/dry-run?theme=dark&amp;sidebar=expanded&amp;session=default\" method=\"post\"",
        "id=\"tau-ops-harness-action-view-diff\" data-action=\"view-diff\" data-action-tone=\"secondary\" href=\"/ops/harness/proposals/PR-044/diff\"",
        "id=\"tau-ops-harness-action-apply\" type=\"submit\" data-action=\"apply\" data-action-tone=\"disabled\" data-disabled=\"true\" aria-disabled=\"true\" data-approval-required=\"true\" disabled",
        "id=\"tau-ops-harness-conservative-policy\"",
    ] {
        assert!(
            html.contains(marker),
            "missing first-viewport operator action marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3777_c01_c02_c03_harness_keeps_conservative_policy_visible_before_detail() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let queue_index = html
        .find("id=\"tau-ops-harness-learning-queue\"")
        .expect("learning queue should render");
    let action_index = html
        .find("id=\"tau-ops-harness-operator-actions\"")
        .expect("operator actions should render");
    let policy_index = html
        .find("id=\"tau-ops-harness-conservative-policy\"")
        .expect("conservative policy should render");
    let detail_index = html
        .find("id=\"tau-ops-harness-proposal-detail\"")
        .expect("proposal detail should render");
    let audit_index = html
        .find("id=\"tau-ops-harness-audit-log\"")
        .expect("audit log should render");

    assert!(
        queue_index < action_index,
        "operator actions should remain after learning queue context"
    );
    assert!(
        action_index < policy_index,
        "conservative policy should remain after operator actions"
    );
    assert!(
        policy_index < detail_index,
        "conservative policy should be visible before long proposal detail"
    );
    assert!(
        detail_index < audit_index,
        "audit history should remain after proposal detail"
    );

    for marker in [
        "id=\"tau-ops-harness-conservative-policy\" data-policy=\"conservative-self-improvement\" data-allowed-targets=\"skill,config,prompt\" data-blocked-targets=\"source-code,safety-policy\" data-review-policy-priority=\"first-viewport\"",
        "id=\"tau-ops-harness-policy-allowed\" data-policy-side=\"allowed\"",
        "<ul><li>Skill</li><li>Config</li><li>Prompt</li></ul>",
        "id=\"tau-ops-harness-policy-blocked\" data-policy-side=\"blocked\"",
        "<ul><li>Source Code</li><li>Safety Policy</li></ul>",
        "id=\"tau-ops-harness-audit-log\"",
    ] {
        assert!(
            html.contains(marker),
            "missing first-viewport conservative policy marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3778_c01_c02_c03_harness_keeps_proposal_safety_summary_visible_after_policy() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let policy_index = html
        .find("id=\"tau-ops-harness-conservative-policy\"")
        .expect("conservative policy should render");
    let detail_index = html
        .find("id=\"tau-ops-harness-proposal-detail\"")
        .expect("proposal detail should render");
    let audit_index = html
        .find("id=\"tau-ops-harness-audit-log\"")
        .expect("audit log should render");

    let dry_run_index = html[detail_index..]
        .find("<dt>Dry-run Result</dt>")
        .expect("dry-run result row should render")
        + detail_index;
    let safety_index = html[detail_index..]
        .find("<dt>Safety Check</dt>")
        .expect("safety check row should render")
        + detail_index;
    let rollback_index = html[detail_index..]
        .find("<dt>Rollback Plan</dt>")
        .expect("rollback plan row should render")
        + detail_index;
    let patch_index = html[detail_index..]
        .find("<dt>Patch Summary</dt>")
        .expect("patch summary row should render")
        + detail_index;
    let failure_index = html[detail_index..]
        .find("<dt>Failure Observed</dt>")
        .expect("failure observed row should render")
        + detail_index;
    let root_cause_index = html[detail_index..]
        .find("<dt>Root Cause</dt>")
        .expect("root cause row should render")
        + detail_index;

    assert!(
        policy_index < detail_index,
        "proposal detail should remain directly after the conservative policy"
    );
    assert!(
        detail_index < dry_run_index
            && dry_run_index < safety_index
            && safety_index < rollback_index
            && rollback_index < patch_index
            && patch_index < failure_index
            && failure_index < root_cause_index,
        "proposal detail should prioritize safety summary before explanatory rows"
    );
    assert!(
        detail_index < audit_index,
        "audit history should remain after proposal detail"
    );

    for marker in [
        "id=\"tau-ops-harness-proposal-detail\" data-proposal-id=\"PR-044\" data-learning-record=\"LR-044\" data-target-type=\"Prompt\" data-target-path=\"prompts/research_to_doc/system.md\" data-proposal-detail-priority=\"first-viewport-summary\" data-proposal-detail-density=\"compact-scroll\"",
        "#tau-ops-harness-proposal-detail {\n                                max-height: 128px;\n                                overflow: auto;",
        "<dt>Dry-run Result</dt><dd data-result=\"passed\">Tests passed (18/18)</dd>",
        "<dt>Safety Check</dt><dd data-result=\"passed\">Passed</dd>",
        "<dt>Rollback Plan</dt><dd>Revert to previous prompt version</dd>",
        "<dt>Test Evidence</dt><dd><a href=\"/evidence/pr-044-dryrun.json\">evidence/pr-044-dryrun.json</a></dd>",
        "<dt>Failure Observed</dt><dd>Token overrun during research-to-doc tasks</dd>",
        "<dt>Root Cause</dt><dd>Verbose prompts with redundant context</dd>",
        "id=\"tau-ops-harness-audit-log\"",
    ] {
        assert!(
            html.contains(marker),
            "missing compact proposal detail marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3779_c01_c02_c03_harness_keeps_recent_audit_proof_visible() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let detail_index = html
        .find("id=\"tau-ops-harness-proposal-detail\"")
        .expect("proposal detail should render");
    let audit_index = html
        .find("id=\"tau-ops-harness-audit-log\"")
        .expect("audit log should render");
    let tui_index = html
        .find("id=\"tau-ops-harness-tui-companion\"")
        .expect("TUI companion should render");

    assert!(
        detail_index < audit_index,
        "audit proof should remain after proposal detail"
    );
    assert!(
        audit_index < tui_index,
        "audit proof should remain in the review pane before the TUI companion"
    );

    for marker in [
        "id=\"tau-ops-harness-self-improvement-window\" data-window=\"self-improvement-review-apply-flow\" data-window-order=\"3\" data-selected-proposal=\"PR-044\" data-approval-gated=\"true\" data-window-chrome=\"compact\" data-review-action-placement=\"actions-before-detail\" data-review-audit-priority=\"first-viewport-recent-history\" data-review-density=\"audit-visible\"",
        "id=\"tau-ops-harness-audit-log\" data-audit-row-count=\"4\" data-audit-source=\"fallback\" data-audit-priority=\"first-viewport-recent-proof\" data-audit-density=\"compact-scroll\" data-audit-visible-columns=\"time,action,item,result\"",
        "#tau-ops-harness-audit-log {\n                                max-height: 104px;\n                                overflow: hidden;",
        "#tau-ops-harness-audit-log .tau-harness-table-wrap {\n                                max-height: 64px;\n                                overflow: auto;",
        "#tau-ops-harness-audit-log table {\n                                min-width: 0;",
        "#tau-ops-harness-audit-log td:nth-child(2),\n                            #tau-ops-harness-audit-log td:nth-child(4) {\n                                display: none;",
        "data-action=\"dry-run\" data-result=\"passed\" data-timestamp-unix-ms=\"\"",
        "<td>May 15, 10:11</td><td>Operator</td><td>Dry Run</td><td>Prompt</td><td>PR-044</td><td>Passed</td>",
        "data-action=\"apply\" data-result=\"applied\" data-timestamp-unix-ms=\"\"",
        "<td>May 15, 09:42</td><td>Operator</td><td>Apply</td><td>Config</td><td>CL-031</td><td>Applied</td>",
    ] {
        assert!(
            html.contains(marker),
            "missing compact recent audit marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3780_c01_c02_c03_harness_keeps_all_verification_gates_visible() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let gates_index = html
        .find("id=\"tau-ops-harness-verification-gates\"")
        .expect("verification gates section should render");
    let memory_index = html
        .find("id=\"tau-ops-harness-memory-learning\"")
        .expect("memory proof section should render");
    let artifacts_index = html
        .find("id=\"tau-ops-harness-artifacts\"")
        .expect("artifacts proof section should render");

    assert!(
        gates_index < memory_index,
        "memory proof output should remain after verification gates"
    );
    assert!(
        memory_index < artifacts_index,
        "artifacts proof output should remain after memory proof output"
    );

    let mut previous_gate_index = gates_index;
    for gate_id in ["VG-01", "VG-02", "VG-03", "VG-04", "VG-05"] {
        let marker = format!("data-gate-id=\"{gate_id}\"");
        let gate_index = html[previous_gate_index..]
            .find(&marker)
            .unwrap_or_else(|| panic!("missing ordered verification gate marker `{marker}`"))
            + previous_gate_index;
        assert!(
            previous_gate_index <= gate_index,
            "gate `{gate_id}` should remain in verification order"
        );
        previous_gate_index = gate_index;
    }

    for marker in [
        "id=\"tau-ops-harness-verification-gates\" data-gate-count=\"5\" data-failed-gate-count=\"1\" data-proof-secondary-priority=\"first-screen\" data-proof-detail-budget=\"compact-scroll\" data-gate-visibility=\"all-gates-first-viewport\" data-gate-layout=\"two-column-compact\"",
        "#tau-ops-harness-verification-gates[data-gate-visibility=\"all-gates-first-viewport\"] {\n                                overflow: hidden;",
        "#tau-ops-harness-verification-gates[data-gate-visibility=\"all-gates-first-viewport\"] ul {\n                                display: grid;\n                                grid-template-columns: repeat(2, minmax(0, 1fr));",
        "#tau-ops-harness-verification-gates[data-gate-visibility=\"all-gates-first-viewport\"] li {\n                                min-width: 0;\n                                justify-content: flex-start;",
        "id=\"tau-ops-harness-gate-learning\" data-gate-id=\"VG-05\" data-gate-status=\"pending\">Learning proof</li>",
        "id=\"tau-ops-harness-memory-learning\" data-memory-hits=\"12\" data-learning-records=\"2\" data-last-memory-write=\"10:20:55\" data-proof-footer-priority=\"first-viewport\"",
        "id=\"tau-ops-harness-artifacts\" data-artifact-count=\"3\" data-proof-footer-priority=\"first-viewport\"",
    ] {
        assert!(
            html.contains(marker),
            "missing all-gates visibility marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3781_c01_c02_c03_harness_uses_clean_active_mission_scroll_boundary() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let missions_index = html
        .find("id=\"tau-ops-harness-active-missions\"")
        .expect("active missions section should render");
    let benchmark_index = html
        .find("id=\"tau-ops-harness-benchmark-panel\"")
        .expect("benchmark panel should render");

    assert!(
        missions_index < benchmark_index,
        "benchmark panel should remain directly after active missions"
    );

    for row_id in 0..=4 {
        let marker = format!("id=\"tau-ops-harness-mission-row-{row_id}\"");
        assert!(
            html.contains(&marker),
            "mission row `{row_id}` should remain available inside the scroll region"
        );
    }

    for marker in [
        "id=\"tau-ops-harness-active-missions\"",
        "data-active-count=\"5\"",
        "data-running-count=\"3\"",
        "data-blocked-count=\"1\"",
        "data-first-viewport-budget=\"benchmark-visible\"",
        "data-active-mission-scroll-boundary=\"whole-row\"",
        "data-active-mission-visible-rows=\"3\"",
        "data-scroll-region=\"active-missions\" data-scroll-boundary=\"whole-row\"",
        "#tau-ops-harness-active-missions[data-active-mission-scroll-boundary=\"whole-row\"] .tau-harness-table-wrap {\n                                max-height: 388px;\n                                overflow: auto;",
        "#tau-ops-harness-active-missions[data-active-mission-scroll-boundary=\"whole-row\"] #tau-ops-harness-missions-table tbody tr {\n                                scroll-snap-align: start;",
        "#tau-ops-harness-active-missions[data-active-mission-scroll-boundary=\"whole-row\"] #tau-ops-harness-missions-table td {\n                                vertical-align: top;",
        "id=\"tau-ops-harness-benchmark-panel\" data-benchmark-id=\"m334-tranche-one-autonomy\"",
    ] {
        assert!(
            html.contains(marker),
            "missing active mission whole-row boundary marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3782_c01_c02_c03_harness_left_tables_do_not_overflow_column() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let missions_index = html
        .find("id=\"tau-ops-harness-active-missions\"")
        .expect("active missions section should render");
    let benchmark_index = html
        .find("id=\"tau-ops-harness-benchmark-panel\"")
        .expect("benchmark panel should render");

    assert!(
        missions_index < benchmark_index,
        "benchmark panel should remain after active missions"
    );

    for marker in [
        "id=\"tau-ops-harness-active-missions\"",
        "data-active-count=\"5\"",
        "data-running-count=\"3\"",
        "data-blocked-count=\"1\"",
        "data-left-table-fit=\"compact-no-overflow\"",
        "data-horizontal-overflow-budget=\"none\"",
        "id=\"tau-ops-harness-benchmark-panel\" data-benchmark-id=\"m334-tranche-one-autonomy\" data-proof-artifact=\"/artifacts/bench/m334/latest.json\" data-task-count=\"4\" data-pass-count=\"4\" data-failed-gates=\"none\" data-proof-source=\"fallback\" data-first-viewport-anchor=\"canonical-benchmark\" data-left-table-fit=\"compact-no-overflow\" data-horizontal-overflow-budget=\"none\"",
        "#tau-ops-harness-active-missions[data-left-table-fit=\"compact-no-overflow\"] #tau-ops-harness-missions-table,\n                            #tau-ops-harness-benchmark-panel[data-left-table-fit=\"compact-no-overflow\"] #tau-ops-harness-benchmark-table {\n                                min-width: 0;\n                                table-layout: fixed;",
        "#tau-ops-harness-active-missions[data-left-table-fit=\"compact-no-overflow\"] #tau-ops-harness-missions-table th:nth-child(n+4),",
        "#tau-ops-harness-active-missions[data-left-table-fit=\"compact-no-overflow\"] #tau-ops-harness-missions-table td:nth-child(n+4) {\n                                display: none;",
        "#tau-ops-harness-active-missions[data-left-table-fit=\"compact-no-overflow\"] #tau-ops-harness-missions-table th,\n                            #tau-ops-harness-active-missions[data-left-table-fit=\"compact-no-overflow\"] #tau-ops-harness-missions-table td,\n                            #tau-ops-harness-benchmark-panel[data-left-table-fit=\"compact-no-overflow\"] #tau-ops-harness-benchmark-table td {\n                                white-space: normal;",
        "data-mission-state-chip=\"running\"",
        "data-mission-gate-chip=\"needs-review\"",
    ] {
        assert!(
            html.contains(marker),
            "missing left table no-overflow marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3783_c01_c02_c03_harness_review_pane_contains_compact_proof_rows() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let queue_index = html
        .find("id=\"tau-ops-harness-learning-queue\"")
        .expect("learning queue should render");
    let detail_index = html
        .find("id=\"tau-ops-harness-proposal-detail\"")
        .expect("proposal detail should render");
    let audit_index = html
        .find("id=\"tau-ops-harness-audit-log\"")
        .expect("audit log should render");

    assert!(
        queue_index < detail_index && detail_index < audit_index,
        "review proof sections should remain in queue, detail, audit order"
    );

    for marker in [
        "id=\"tau-ops-harness-self-improvement-window\" data-window=\"self-improvement-review-apply-flow\" data-window-order=\"3\" data-selected-proposal=\"PR-044\" data-approval-gated=\"true\" data-window-chrome=\"compact\" data-review-action-placement=\"actions-before-detail\" data-review-audit-priority=\"first-viewport-recent-history\" data-review-density=\"audit-visible\" data-review-overflow-contract=\"contained-proof-rows\"",
        "id=\"tau-ops-harness-learning-queue\" data-queue-count=\"4\" data-queue-density=\"all-items-visible\" data-queue-overflow-budget=\"none\"",
        "id=\"tau-ops-harness-proposal-detail\" data-proposal-id=\"PR-044\" data-learning-record=\"LR-044\" data-target-type=\"Prompt\" data-target-path=\"prompts/research_to_doc/system.md\" data-proposal-detail-priority=\"first-viewport-summary\" data-proposal-detail-density=\"compact-scroll\" data-proposal-detail-overflow-budget=\"contained\" data-proposal-visible-rows=\"7\"",
        "id=\"tau-ops-harness-audit-log\" data-audit-row-count=\"4\" data-audit-source=\"fallback\" data-audit-priority=\"first-viewport-recent-proof\" data-audit-density=\"compact-scroll\" data-audit-visible-columns=\"time,action,item,result\" data-audit-overflow-budget=\"all-rows-visible\"",
        "#tau-ops-harness-learning-queue[data-queue-readability=\"full-labels\"] ul {\n                                display: grid;\n                                grid-template-columns: minmax(0, 1fr);",
        "#tau-ops-harness-proposal-detail[data-proposal-detail-overflow-budget=\"contained\"] dl {\n                                gap: 3px 10px;",
        "#tau-ops-harness-proposal-detail[data-proposal-detail-overflow-budget=\"contained\"] a {\n                                min-height: 0;\n                                padding: 0;",
        "#tau-ops-harness-audit-log[data-audit-overflow-budget=\"all-rows-visible\"] td {\n                                padding: 2px 5px;",
        "data-learning-id=\"PR-045\" data-status=\"proposal\" data-selected=\"false\" data-actionable=\"true\"",
        "href=\"/ops/harness?theme=dark&amp;sidebar=expanded&amp;session=default&amp;proposal_id=PR-045\" data-proposal-link=\"PR-045\" aria-current=\"false\"",
        "<span class=\"tau-harness-queue-label\">Skill patch for benchmark artifact naming</span>",
        "<span class=\"tau-harness-queue-status\">Proposal</span>",
        "<dt>Test Evidence</dt><dd><a href=\"/evidence/pr-044-dryrun.json\">evidence/pr-044-dryrun.json</a></dd>",
        "data-action=\"reject\" data-result=\"rejected\" data-timestamp-unix-ms=\"\"",
        "<td>May 15, 08:33</td><td>Operator</td><td>Reject</td><td>Prompt</td><td>PR-029</td><td>Rejected</td>",
    ] {
        assert!(
            html.contains(marker),
            "missing right review contained proof marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3784_c01_c02_c03_harness_center_proof_evidence_stays_contained() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let evidence_index = html
        .find("id=\"tau-ops-harness-tool-evidence\"")
        .expect("tool evidence section should render");
    let acceptance_index = html
        .find("id=\"tau-ops-harness-acceptance\"")
        .expect("acceptance criteria section should render");
    let gates_index = html
        .find("id=\"tau-ops-harness-verification-gates\"")
        .expect("verification gates section should render");

    assert!(
        evidence_index < acceptance_index && acceptance_index < gates_index,
        "center proof sections should remain in evidence, acceptance, gate order"
    );

    for marker in [
        "id=\"tau-ops-harness-tool-evidence\" data-tool-call-count=\"8\" data-compact-evidence-breakpoint=\"1400px\" data-compact-call-id-visibility=\"hidden-at-1400px\" data-proof-evidence-priority=\"first-screen\" data-tool-evidence-fit=\"compact-no-overflow\" data-tool-evidence-overflow-budget=\"none\" data-tool-evidence-visible-columns=\"tool,plan-node,runtime,status,artifact\"",
        "id=\"tau-ops-harness-acceptance\" data-acceptance-met=\"3\" data-acceptance-total=\"5\" data-proof-detail-budget=\"compact-scroll\" data-acceptance-overflow-budget=\"all-criteria-visible\" data-acceptance-layout=\"compact-contained\"",
        "#tau-ops-harness-tool-evidence[data-tool-evidence-fit=\"compact-no-overflow\"] table {\n                                min-width: 0;\n                                width: 100%;\n                                table-layout: fixed;",
        "#tau-ops-harness-tool-evidence[data-tool-evidence-fit=\"compact-no-overflow\"] th:nth-child(2),\n                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit=\"compact-no-overflow\"] td:nth-child(2) {\n                                display: none;",
        "#tau-ops-harness-tool-evidence[data-tool-evidence-fit=\"compact-no-overflow\"] th,\n                            #tau-ops-harness-tool-evidence[data-tool-evidence-fit=\"compact-no-overflow\"] td {\n                                overflow: hidden;\n                                text-overflow: ellipsis;",
        "#tau-ops-harness-acceptance[data-acceptance-overflow-budget=\"all-criteria-visible\"] ul {\n                                display: grid;\n                                grid-template-columns: minmax(0, 1fr);",
        "#tau-ops-harness-acceptance[data-acceptance-overflow-budget=\"all-criteria-visible\"] li {\n                                min-width: 0;\n                                width: 100%;",
        "data-tool=\"report.write\" data-status=\"running\"><td>report.write</td><td>c1a2b9</td><td>Verify</td><td>00:01:21</td><td>running</td><td>/artifacts/report.md</td>",
        "data-ac-id=\"VG-05\" data-ac-status=\"pending\">Benchmark proof emitted</li>",
    ] {
        assert!(
            html.contains(marker),
            "missing center proof containment marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3785_c01_c02_c03_harness_review_queue_labels_are_readable() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let queue_index = html
        .find("id=\"tau-ops-harness-learning-queue\"")
        .expect("learning queue should render");
    let actions_index = html
        .find("id=\"tau-ops-harness-operator-actions\"")
        .expect("operator actions should render");

    assert!(
        queue_index < actions_index,
        "learning queue should remain before operator actions"
    );

    let mut previous_index = queue_index;
    for item in [
        "data-learning-id=\"LR-219\" data-status=\"needs-review\" data-selected=\"false\" data-actionable=\"false\"",
        "data-learning-id=\"LR-220\" data-status=\"needs-review\" data-selected=\"false\" data-actionable=\"false\"",
        "data-learning-id=\"PR-044\" data-status=\"proposal\" data-selected=\"true\" data-actionable=\"true\"",
        "data-learning-id=\"PR-045\" data-status=\"proposal\" data-selected=\"false\" data-actionable=\"true\"",
    ] {
        let item_index = html
            .find(item)
            .unwrap_or_else(|| panic!("missing readable queue item `{item}`"));
        assert!(
            item_index >= previous_index,
            "queue item `{item}` should preserve DOM order"
        );
        previous_index = item_index;
    }

    for marker in [
        "id=\"tau-ops-harness-learning-queue\" data-queue-count=\"4\" data-queue-density=\"all-items-visible\" data-queue-overflow-budget=\"none\" data-queue-readability=\"full-labels\" data-queue-layout=\"single-column-readable\" data-queue-truncation-budget=\"none\" data-queue-navigation=\"proposal-links\" data-queue-source=\"fallback\"",
        "<span class=\"tau-harness-queue-label\">Retry storm in document synthesis</span>",
        "<span class=\"tau-harness-queue-label\">Missing memory write after verification</span>",
        "href=\"/ops/harness?theme=dark&amp;sidebar=expanded&amp;session=default&amp;proposal_id=PR-044\" data-proposal-link=\"PR-044\" aria-current=\"page\"",
        "href=\"/ops/harness?theme=dark&amp;sidebar=expanded&amp;session=default&amp;proposal_id=PR-045\" data-proposal-link=\"PR-045\" aria-current=\"false\"",
        "<span class=\"tau-harness-queue-status\">Needs Review</span>",
        "<span class=\"tau-harness-queue-status\">Proposal</span>",
        "#tau-ops-harness-learning-queue[data-queue-readability=\"full-labels\"] ul {\n                                display: grid;\n                                grid-template-columns: minmax(0, 1fr);",
        "#tau-ops-harness-learning-queue[data-queue-readability=\"full-labels\"] li {\n                                width: 100%;\n                                min-width: 0;",
        "#tau-ops-harness-learning-queue[data-queue-readability=\"full-labels\"] li {\n                                width: 100%;\n                                min-width: 0;\n                                overflow: hidden;\n                                white-space: nowrap;",
        "#tau-ops-harness-learning-queue li[data-selected=\"true\"] {\n                                border-color: rgba(89, 151, 255, .82);",
    ] {
        assert!(
            html.contains(marker),
            "missing readable review queue marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3786_c01_c02_c03_harness_proof_header_metadata_does_not_wrap() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let header_index = html
        .find("id=\"tau-ops-harness-proof-header\"")
        .expect("proof header should render");
    let dag_index = html
        .find("id=\"tau-ops-harness-plan-dag\"")
        .expect("plan dag should render");

    assert!(
        header_index < dag_index,
        "proof metadata should remain before the plan DAG"
    );

    for marker in [
        "id=\"tau-ops-harness-proof-header\"",
        "data-compact-metadata-breakpoint=\"1400px\" data-metadata-fit=\"no-wrap\" data-run-id-wrap=\"single-line\" data-metadata-value-overflow-budget=\"none\"",
        "<dt>Run ID</dt><dd>run_8f3a2</dd>",
        "<dt>Elapsed</dt><dd>01:42:18</dd>",
        "<dt>Tool Budget</dt><dd>42/60</dd>",
        "<dt>Cost</dt><dd>$3.82</dd>",
        "<dt>Retry Count</dt><dd>1</dd>",
        "#tau-ops-harness-proof-header[data-metadata-fit=\"no-wrap\"] dl {\n                                grid-template-columns: minmax(5.5rem, max-content) minmax(4.75rem, max-content);",
        "#tau-ops-harness-proof-header[data-metadata-fit=\"no-wrap\"] dt,\n                            #tau-ops-harness-proof-header[data-metadata-fit=\"no-wrap\"] dd {\n                                white-space: nowrap;\n                                overflow-wrap: normal;",
    ] {
        assert!(
            html.contains(marker),
            "missing proof header no-wrap marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3787_c01_c02_c03_harness_proposal_patch_summary_is_readable() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let detail_index = html
        .find("id=\"tau-ops-harness-proposal-detail\"")
        .expect("proposal detail should render");
    let audit_index = html
        .find("id=\"tau-ops-harness-audit-log\"")
        .expect("audit log should render");

    assert!(
        detail_index < audit_index,
        "proposal detail should remain before audit history"
    );

    for row in [
        "<dt>Dry-run Result</dt><dd data-result=\"passed\">Tests passed (18/18)</dd>",
        "<dt>Safety Check</dt><dd data-result=\"passed\">Passed</dd>",
        "<dt>Rollback Plan</dt><dd>Revert to previous prompt version</dd>",
        "<dt>Patch Summary</dt><dd data-proposal-row=\"patch-summary\" data-summary-fit=\"full-text\">Compress system prompt by removing redundant instructions and examples.</dd>",
        "<dt>Failure Observed</dt><dd>Token overrun during research-to-doc tasks</dd>",
        "<dt>Root Cause</dt><dd>Verbose prompts with redundant context</dd>",
        "<dt>Test Evidence</dt><dd><a href=\"/evidence/pr-044-dryrun.json\">evidence/pr-044-dryrun.json</a></dd>",
    ] {
        assert!(
            html.contains(row),
            "proposal detail should retain row `{row}`"
        );
    }

    for marker in [
        "id=\"tau-ops-harness-proposal-detail\" data-proposal-id=\"PR-044\" data-learning-record=\"LR-044\" data-target-type=\"Prompt\" data-target-path=\"prompts/research_to_doc/system.md\" data-proposal-detail-priority=\"first-viewport-summary\" data-proposal-detail-density=\"compact-scroll\" data-proposal-detail-overflow-budget=\"contained\" data-proposal-visible-rows=\"7\" data-proposal-summary-fit=\"full-text\" data-proposal-summary-overflow-budget=\"none\"",
        "#tau-ops-harness-proposal-detail[data-proposal-summary-fit=\"full-text\"] dl {\n                                gap: 2px 10px;",
        "#tau-ops-harness-proposal-detail[data-proposal-summary-fit=\"full-text\"] dd[data-summary-fit=\"full-text\"] {\n                                white-space: normal;\n                                overflow-wrap: normal;",
    ] {
        assert!(
            html.contains(marker),
            "missing proposal summary readability marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3788_c01_c02_c03_harness_kpi_labels_avoid_mid_word_breaks() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "id=\"tau-ops-harness-kpi-grid\" data-kpi-card-count=\"4\" data-kpi-label-fit=\"word-boundary\" data-kpi-label-overflow-budget=\"none\"",
        "id=\"tau-ops-harness-kpi-verifications\" data-harness-kpi-card=\"pending-verifications\" data-kpi-value=\"3\" data-kpi-heading-fit=\"word-boundary\"",
        "<h4 aria-label=\"Pending Verifications\"><span>Pending</span><span>Verifications</span></h4>",
        "#tau-ops-harness-kpi-grid[data-kpi-label-fit=\"word-boundary\"] h4 {\n                                overflow-wrap: normal;\n                                word-break: normal;",
        "#tau-ops-harness-kpi-grid[data-kpi-label-fit=\"word-boundary\"] h4 span {\n                                display: block;",
    ] {
        assert!(
            html.contains(marker),
            "KPI labels should preserve word-boundary fit marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3789_c01_c02_c03_harness_benchmark_categories_are_operator_readable() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "id=\"tau-ops-harness-benchmark-panel\" data-benchmark-id=\"m334-tranche-one-autonomy\" data-proof-artifact=\"/artifacts/bench/m334/latest.json\" data-task-count=\"4\" data-pass-count=\"4\" data-failed-gates=\"none\" data-proof-source=\"fallback\" data-first-viewport-anchor=\"canonical-benchmark\" data-left-table-fit=\"compact-no-overflow\" data-horizontal-overflow-budget=\"none\" data-category-label-fit=\"operator-readable\" data-category-overflow-budget=\"none\"",
        "data-category=\"greenfield_build\" data-task-count=\"1\" data-last-run=\"4/4 pass\" data-pass-rate=\"100\"",
        "<td data-category-label=\"Greenfield build\"><span class=\"tau-harness-benchmark-category-label\">Greenfield build</span></td>",
        "<td data-category-label=\"Research design\"><span class=\"tau-harness-benchmark-category-label\">Research design</span></td>",
        "<td data-category-label=\"Data to deliverable\"><span class=\"tau-harness-benchmark-category-label\">Data to deliverable</span></td>",
        "#tau-ops-harness-benchmark-panel[data-category-label-fit=\"operator-readable\"] #tau-ops-harness-benchmark-table td:first-child {\n                                white-space: normal;\n                                overflow-wrap: normal;",
        "#tau-ops-harness-benchmark-panel[data-category-label-fit=\"operator-readable\"] .tau-harness-benchmark-category-label {\n                                display: block;",
    ] {
        assert!(
            html.contains(marker),
            "benchmark categories should preserve readable label marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3790_c01_c02_c03_harness_tool_evidence_shows_memory_tool_names() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "id=\"tau-ops-harness-tool-evidence\" data-tool-call-count=\"8\" data-compact-evidence-breakpoint=\"1400px\" data-compact-call-id-visibility=\"hidden-at-1400px\" data-proof-evidence-priority=\"first-screen\" data-tool-evidence-fit=\"compact-no-overflow\" data-tool-evidence-overflow-budget=\"none\" data-tool-evidence-visible-columns=\"tool,plan-node,runtime,status,artifact\" data-tool-label-fit=\"full-memory-tool-names\" data-tool-column-overflow-budget=\"none\"",
        "data-tool=\"memory.search\" data-status=\"passed\"><td>memory.search</td><td>c1a2b7f</td><td>Memory Write</td><td>00:00:48</td><td>passed</td><td>/artifacts/memory.json</td>",
        "data-tool=\"memory.write\" data-status=\"passed\"><td>memory.write</td><td>c1a2b8e</td><td>Memory Write</td><td>00:00:36</td><td>passed</td><td>/artifacts/learning.json</td>",
        "#tau-ops-harness-tool-evidence[data-tool-label-fit=\"full-memory-tool-names\"] th:nth-child(1),\n                            #tau-ops-harness-tool-evidence[data-tool-label-fit=\"full-memory-tool-names\"] td:nth-child(1) {\n                                width: 96px;",
        "#tau-ops-harness-tool-evidence[data-tool-label-fit=\"full-memory-tool-names\"] table {\n                                font-size: .66rem;",
        "#tau-ops-harness-tool-evidence[data-tool-label-fit=\"full-memory-tool-names\"] th:nth-child(3),\n                            #tau-ops-harness-tool-evidence[data-tool-label-fit=\"full-memory-tool-names\"] td:nth-child(3) {\n                                width: 86px;",
    ] {
        assert!(
            html.contains(marker),
            "Tool evidence should preserve full memory tool label marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3791_c01_c02_c03_harness_proposal_detail_has_no_vertical_clip() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "id=\"tau-ops-harness-proposal-detail\" data-proposal-id=\"PR-044\" data-learning-record=\"LR-044\" data-target-type=\"Prompt\" data-target-path=\"prompts/research_to_doc/system.md\" data-proposal-detail-priority=\"first-viewport-summary\" data-proposal-detail-density=\"compact-scroll\" data-proposal-detail-overflow-budget=\"contained\" data-proposal-visible-rows=\"7\" data-proposal-summary-fit=\"full-text\" data-proposal-summary-overflow-budget=\"none\" data-proposal-detail-vertical-overflow-budget=\"none\" data-proposal-detail-max-height=\"132px\"",
        "#tau-ops-harness-proposal-detail[data-proposal-detail-vertical-overflow-budget=\"none\"] {\n                                max-height: 132px;",
        "<dt>Dry-run Result</dt><dd data-result=\"passed\">Tests passed (18/18)</dd>",
        "<dt>Patch Summary</dt><dd data-proposal-row=\"patch-summary\" data-summary-fit=\"full-text\">Compress system prompt by removing redundant instructions and examples.</dd>",
        "<dt>Test Evidence</dt><dd><a href=\"/evidence/pr-044-dryrun.json\">evidence/pr-044-dryrun.json</a></dd>",
    ] {
        assert!(
            html.contains(marker),
            "proposal detail should preserve no-clip marker `{marker}`"
        );
    }
}

#[test]
fn functional_harness_self_improvement_proof_surfaces_completed_mission_state() {
    let harness = TauOpsDashboardHarnessSnapshot {
        selected_proposal_id: "PR-045".to_string(),
        self_improvement_proof: TauOpsDashboardHarnessSelfImprovementProof {
            source: "state".to_string(),
            mission_id: "ops-harness-self-improve-pr-045".to_string(),
            mission_status: "completed".to_string(),
            plan_completed_count: 5,
            plan_total_count: 5,
            gate_passed_count: 3,
            gate_total_count: 3,
            memory_hit_count: 1,
            artifact_count: 3,
            final_learning_summary: "Applied PR-045 and updated curator state for LR-045."
                .to_string(),
            final_learning_records: vec!["LR-045".to_string()],
            plan_rows: vec![TauOpsDashboardHarnessProofRow {
                item_id: "curate-learning".to_string(),
                status_key: "completed".to_string(),
                label: "Update the curator record after successful apply.".to_string(),
            }],
            gate_rows: vec![TauOpsDashboardHarnessProofRow {
                item_id: "VG-APPLY".to_string(),
                status_key: "passed".to_string(),
                label: "Target update is applied and curator state is updated.".to_string(),
            }],
            artifact_rows: vec![TauOpsDashboardHarnessProofRow {
                item_id: "target:skills/benchmark_artifacts/SKILL.md".to_string(),
                status_key: "skill".to_string(),
                label: "skills/benchmark_artifacts/SKILL.md".to_string(),
            }],
        },
        ..TauOpsDashboardHarnessSnapshot::default()
    };
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        active_route: TauOpsDashboardRoute::Harness,
        harness,
        ..TauOpsDashboardShellContext::default()
    });

    let detail_index = html
        .find("id=\"tau-ops-harness-proposal-detail\"")
        .expect("proposal detail should render");
    let proof_index = html
        .find("id=\"tau-ops-harness-self-improvement-proof\"")
        .expect("self-improvement proof should render");
    let audit_index = html
        .find("id=\"tau-ops-harness-audit-log\"")
        .expect("audit log should render");
    assert!(
        detail_index < proof_index && proof_index < audit_index,
        "completed mission proof should sit between selected proposal detail and audit history"
    );

    for marker in [
        "id=\"tau-ops-harness-self-improvement-proof\" data-proof-source=\"state\" data-mission-id=\"ops-harness-self-improve-pr-045\" data-mission-status=\"completed\" data-plan-completed=\"5\" data-plan-total=\"5\" data-gates-passed=\"3\" data-gates-total=\"3\" data-memory-hits=\"1\" data-artifact-count=\"3\" data-final-learning-records=\"LR-045\"",
        "Applied PR-045 and updated curator state for LR-045.",
        "data-proof-row=\"plan\" data-proof-id=\"curate-learning\" data-proof-status=\"completed\">Update the curator record after successful apply.</li>",
        "data-proof-row=\"gate\" data-proof-id=\"VG-APPLY\" data-proof-status=\"passed\">VG-APPLY: Target update is applied and curator state is updated.</li>",
        "data-proof-row=\"artifact\" data-proof-id=\"target:skills/benchmark_artifacts/SKILL.md\" data-proof-status=\"skill\">skills/benchmark_artifacts/SKILL.md</li>",
    ] {
        assert!(
            html.contains(marker),
            "missing completed self-improvement proof marker `{marker}`"
        );
    }
}

#[test]
fn functional_harness_draft_mission_exposes_start_action() {
    let harness = TauOpsDashboardHarnessSnapshot {
        selected_proposal_id: "PR-045".to_string(),
        mission_rows: vec![TauOpsDashboardHarnessMissionRow {
            mission_id: "mission-draft-123".to_string(),
            title: "PR-045 Skill patch for benchmark artifact naming".to_string(),
            status_key: "draft".to_string(),
            status_label: "Draft".to_string(),
            gate_status_key: "pending".to_string(),
            gate_label: "0/3 gates".to_string(),
            acceptance_label: "0/3".to_string(),
            plan_progress: 0,
            tool_budget: "0/40".to_string(),
            memory_hits: 0,
            verification_state: "pending".to_string(),
            last_checkpoint: "Draft mission saved before execution.".to_string(),
            artifact_count: 1,
        }],
        ..TauOpsDashboardHarnessSnapshot::default()
    };
    let context = TauOpsDashboardShellContext {
        active_route: TauOpsDashboardRoute::Harness,
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "ops-harness-context".to_string(),
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness,
        ..TauOpsDashboardShellContext::default()
    };
    let html = render_tau_ops_dashboard_shell_with_context(context);

    for marker in [
        "id=\"tau-ops-harness-start-mission-form-0\"",
        "/ops/harness/missions/mission-draft-123/start?theme=dark",
        "sidebar=expanded",
        "session=ops-harness-context",
        "proposal_id=PR-045",
        "method=\"post\"",
        "data-action-contract=\"mission-start-dry-run\"",
        "data-preserves-shell-context=\"true\"",
        "id=\"tau-ops-harness-start-mission-0\"",
        "type=\"submit\" data-action=\"start-mission\"",
        "data-mission-id=\"mission-draft-123\"",
        "data-action-contract=\"coding-agent-dry-run\"",
        "data-mission-state-chip=\"draft\"",
        "data-mission-gate-chip=\"pending\"",
    ] {
        assert!(
            html.contains(marker),
            "draft mission should expose start action marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3792_c01_c02_c03_harness_proof_pane_fits_in_app_browser_width() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "id=\"tau-ops-harness-proof-window\" data-window=\"mission-detail-proof-view\" data-window-order=\"2\" data-run-id=\"run_8f3a2\" data-mission-status=\"running\" data-tool-budget=\"42/60\" data-window-chrome=\"compact\" data-narrow-proof-fit=\"no-hidden-overflow\"",
        "id=\"tau-ops-harness-acceptance\" data-acceptance-met=\"3\" data-acceptance-total=\"5\" data-proof-detail-budget=\"compact-scroll\" data-acceptance-overflow-budget=\"all-criteria-visible\" data-acceptance-layout=\"compact-contained\" data-narrow-label-fit=\"full-labels-at-1400px\"",
        "id=\"tau-ops-harness-verification-gates\" data-gate-count=\"5\" data-failed-gate-count=\"1\" data-proof-secondary-priority=\"first-screen\" data-proof-detail-budget=\"compact-scroll\" data-gate-visibility=\"all-gates-first-viewport\" data-gate-layout=\"two-column-compact\" data-narrow-height-budget=\"no-hidden-overflow\"",
        "#tau-ops-harness-acceptance[data-narrow-label-fit=\"full-labels-at-1400px\"][data-acceptance-overflow-budget=\"all-criteria-visible\"] li {\n                                    font-size: .59rem;",
        "#tau-ops-harness-verification-gates[data-narrow-height-budget=\"no-hidden-overflow\"][data-gate-visibility=\"all-gates-first-viewport\"] {\n                                    max-height: 168px;",
        "Registry loads plugins deterministically",
        "Hot reload preserves active sessions",
        "id=\"tau-ops-harness-gate-learning\" data-gate-id=\"VG-05\" data-gate-status=\"pending\">Learning proof</li>",
    ] {
        assert!(
            html.contains(marker),
            "proof pane should fit the in-app browser width marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3775_c01_c02_c03_harness_keeps_benchmark_panel_in_left_first_viewport() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let missions_index = html
        .find("id=\"tau-ops-harness-active-missions\"")
        .expect("active missions section should render");
    let benchmark_index = html
        .find("id=\"tau-ops-harness-benchmark-panel\"")
        .expect("benchmark panel should render");

    assert!(
        missions_index < benchmark_index,
        "benchmark panel should remain directly after active missions"
    );

    for marker in [
        "id=\"tau-ops-harness-active-missions\"",
        "data-active-count=\"5\"",
        "data-running-count=\"3\"",
        "data-blocked-count=\"1\"",
        "data-first-viewport-budget=\"benchmark-visible\"",
        "data-scroll-region=\"active-missions\"",
        "#tau-ops-harness-active-missions {\n                                max-height: 480px;",
        "#tau-ops-harness-active-missions .tau-harness-table-wrap {\n                                max-height: 432px;\n                                overflow: auto;",
        "id=\"tau-ops-harness-benchmark-panel\" data-benchmark-id=\"m334-tranche-one-autonomy\" data-proof-artifact=\"/artifacts/bench/m334/latest.json\" data-task-count=\"4\" data-pass-count=\"4\" data-failed-gates=\"none\" data-proof-source=\"fallback\" data-first-viewport-anchor=\"canonical-benchmark\"",
        "id=\"tau-ops-harness-run-benchmark-form\" action=\"/ops/harness/run-benchmark?theme=dark&amp;sidebar=expanded&amp;session=default&amp;proposal_id=PR-044\" method=\"post\" data-command=\"tau_agent_harness\" data-preserves-shell-context=\"true\"",
        "data-mission-state-chip=\"running\"",
        "data-mission-gate-chip=\"needs-review\"",
    ] {
        assert!(
            html.contains(marker),
            "missing first-viewport benchmark marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3776_c01_c02_c03_harness_keeps_memory_and_artifacts_in_proof_viewport() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    let log_index = html
        .find("id=\"tau-ops-harness-operator-log\"")
        .expect("operator log section should render");
    let acceptance_index = html
        .find("id=\"tau-ops-harness-acceptance\"")
        .expect("acceptance section should render");
    let gates_index = html
        .find("id=\"tau-ops-harness-verification-gates\"")
        .expect("verification gates section should render");
    let memory_index = html
        .find("id=\"tau-ops-harness-memory-learning\"")
        .expect("memory learning section should render");
    let artifacts_index = html
        .find("id=\"tau-ops-harness-artifacts\"")
        .expect("artifacts section should render");

    assert!(
        log_index < acceptance_index,
        "operator log should remain before acceptance detail"
    );
    assert!(
        acceptance_index < gates_index,
        "acceptance detail should remain before verification gates"
    );
    assert!(
        gates_index < memory_index,
        "verification gates should remain before memory proof output"
    );
    assert!(
        memory_index < artifacts_index,
        "memory proof output should remain before artifacts"
    );

    for marker in [
        "id=\"tau-ops-harness-acceptance\" data-acceptance-met=\"3\" data-acceptance-total=\"5\" data-proof-detail-budget=\"compact-scroll\"",
        "id=\"tau-ops-harness-verification-gates\" data-gate-count=\"5\" data-failed-gate-count=\"1\" data-proof-secondary-priority=\"first-screen\" data-proof-detail-budget=\"compact-scroll\"",
        "id=\"tau-ops-harness-memory-learning\" data-memory-hits=\"12\" data-learning-records=\"2\" data-last-memory-write=\"10:20:55\" data-proof-footer-priority=\"first-viewport\"",
        "id=\"tau-ops-harness-artifacts\" data-artifact-count=\"3\" data-proof-footer-priority=\"first-viewport\"",
        "#tau-ops-harness-acceptance,\n                            #tau-ops-harness-verification-gates {\n                                max-height: 160px;\n                                overflow: auto;",
        "#tau-ops-harness-memory-learning,\n                            #tau-ops-harness-artifacts {\n                                min-height: 0;",
    ] {
        assert!(
            html.contains(marker),
            "missing first-viewport proof footer marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3759_c02_c03_harness_static_preview_guard_preserves_gateway_forms() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops/harness");

    for marker in [
        "id=\"tau-ops-harness-run-benchmark-form\" action=\"/ops/harness/run-benchmark?theme=dark&amp;sidebar=expanded&amp;session=default&amp;proposal_id=PR-044\" method=\"post\" data-command=\"tau_agent_harness\" data-preserves-shell-context=\"true\"",
        "id=\"tau-ops-harness-approve-form\" action=\"/ops/harness/proposals/PR-044/approve?theme=dark&amp;sidebar=expanded&amp;session=default\" method=\"post\"",
        "id=\"tau-ops-harness-approve-theme\" type=\"hidden\" name=\"theme\" value=\"dark\"",
        "id=\"tau-ops-harness-approve-session\" type=\"hidden\" name=\"session\" value=\"default\"",
        "id=\"tau-ops-harness-reject-form\" action=\"/ops/harness/proposals/PR-044/reject?theme=dark&amp;sidebar=expanded&amp;session=default\" method=\"post\"",
        "id=\"tau-ops-harness-dry-run-form\" action=\"/ops/harness/proposals/PR-044/dry-run?theme=dark&amp;sidebar=expanded&amp;session=default\" method=\"post\"",
        "id=\"tau-ops-harness-preview-status\" data-preview-status=\"idle\" hidden",
        "id=\"tau-ops-harness-preview-guard\" data-preview-submit-guard=\"file-protocol-post-forms\"",
        "window.location.protocol !== \"file:\"",
        "form.setAttribute(\"data-preview-submit-blocked\", \"true\")",
        "panel.contains(form)",
    ] {
        assert!(
            html.contains(marker),
            "missing harness static preview guard marker `{marker}`"
        );
    }
}

#[test]
fn regression_spec_3756_c03_non_harness_routes_hide_harness_panel() {
    let html = render_tau_ops_dashboard_shell_for_route("/ops");

    assert!(html.contains(
        "id=\"tau-ops-harness-panel\" data-route=\"/ops/harness\" data-component=\"MissionHarnessWorkspace\" data-design-template=\"three-window-agent-harness\" aria-hidden=\"true\" data-panel-visible=\"false\""
    ));
    assert!(
        html.contains("id=\"tau-ops-command-center\" data-route=\"/ops\" aria-hidden=\"false\"")
    );
}

#[test]
fn functional_spec_2798_c01_c02_c03_shell_exposes_responsive_and_theme_contract_markers() {
    let html = render_tau_ops_dashboard_shell();
    assert!(html.contains("id=\"tau-ops-shell-controls\""));
    assert!(html.contains("id=\"tau-ops-sidebar-toggle\""));
    assert!(html.contains("id=\"tau-ops-sidebar-hamburger\""));
    assert!(html.contains("data-sidebar-mobile-default=\"collapsed\""));
    assert!(html.contains("data-sidebar-state=\"expanded\""));
    assert!(html.contains("data-theme=\"dark\""));
    assert!(html.contains("id=\"tau-ops-theme-toggle-dark\""));
    assert!(html.contains("id=\"tau-ops-theme-toggle-light\""));
}

#[test]
fn functional_spec_2798_c02_shell_sidebar_collapsed_state_updates_toggle_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });
    assert!(html.contains("data-sidebar-state=\"collapsed\""));
    assert!(html.contains("data-sidebar-target-state=\"expanded\""));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(html.contains("href=\"/ops?theme=dark&amp;sidebar=expanded\""));
}

#[test]
fn functional_spec_2798_c03_shell_light_theme_state_updates_theme_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });
    assert!(html.contains("data-theme=\"light\""));
    assert!(html.contains(
        "id=\"tau-ops-theme-toggle-dark\" data-theme-option=\"dark\" aria-pressed=\"false\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-theme-toggle-light\" data-theme-option=\"light\" aria-pressed=\"true\""
    ));
    assert!(html.contains("href=\"/ops/chat?theme=dark&amp;sidebar=expanded\""));
}

#[test]
fn functional_spec_2830_c01_chat_route_renders_send_form_and_fallback_transcript_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-chat-panel\" data-route=\"/ops/chat\" aria-hidden=\"false\" data-active-session-key=\"default\""));
    assert!(html.contains(
        "id=\"tau-ops-chat-session-summary\" data-active-session-key=\"default\" data-entry-count=\"0\" data-total-tokens=\"0\" data-validation-state=\"valid\" data-updated-unix-ms=\"0\""
    ));
    assert!(html.contains("data-latest-message-index=\"none\""));
    assert!(
        html.contains("id=\"tau-ops-chat-session-actions\" aria-label=\"Chat session actions\"")
    );
    assert!(html.contains("id=\"tau-ops-chat-open-session-detail\" href=\"/ops/sessions/default?theme=dark&amp;sidebar=expanded&amp;session=default\""));
    assert!(html.contains("id=\"tau-ops-chat-jump-latest\" href=\"#tau-ops-chat-transcript\""));
    assert!(html
        .contains("id=\"tau-ops-chat-session-updated-label\" data-updated-unix-ms=\"0\">never<"));
    assert!(html.contains("Session Summary"));
    assert!(html.contains("id=\"tau-ops-chat-send-form\" action=\"/ops/chat/send\" method=\"post\" data-session-key=\"default\""));
    assert!(html.contains(
        "id=\"tau-ops-chat-session-key\" type=\"hidden\" name=\"session_key\" value=\"default\""
    ));
    assert!(
        html.contains("id=\"tau-ops-chat-theme\" type=\"hidden\" name=\"theme\" value=\"dark\"")
    );
    assert!(html.contains(
        "id=\"tau-ops-chat-sidebar\" type=\"hidden\" name=\"sidebar\" value=\"expanded\""
    ));
    assert!(html.contains("id=\"tau-ops-chat-transcript\" data-message-count=\"1\""));
    assert!(html.contains("id=\"tau-ops-chat-message-row-0\" data-message-role=\"system\""));
    assert!(html.contains("No chat messages yet."));
    assert!(html.contains(
        "id=\"tau-ops-chat-latest-turn\" data-latest-turn-visible=\"false\" aria-hidden=\"true\""
    ));
}

#[test]
fn functional_spec_2830_c02_chat_route_renders_snapshot_message_rows_for_active_session() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "session-42".to_string(),
            send_form_action: "/ops/chat/send".to_string(),
            send_form_method: "post".to_string(),
            session_options: vec![],
            session_detail_route: "/ops/sessions/session-42".to_string(),
            message_rows: vec![
                TauOpsDashboardChatMessageRow {
                    role: "user".to_string(),
                    content: "first message".to_string(),
                },
                TauOpsDashboardChatMessageRow {
                    role: "assistant".to_string(),
                    content: "second message".to_string(),
                },
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("data-active-session-key=\"session-42\""));
    assert!(html.contains(
        "id=\"tau-ops-chat-session-summary\" data-active-session-key=\"session-42\" data-entry-count=\"2\" data-total-tokens=\"0\" data-validation-state=\"valid\" data-updated-unix-ms=\"0\""
    ));
    assert!(html.contains("data-latest-message-index=\"1\""));
    assert!(html.contains("id=\"tau-ops-chat-open-session-detail\" href=\"/ops/sessions/session-42?theme=light&amp;sidebar=collapsed&amp;session=session-42\""));
    assert!(html.contains("id=\"tau-ops-chat-jump-latest\" href=\"#tau-ops-chat-message-row-1\""));
    assert!(html.contains("id=\"tau-ops-chat-send-form\" action=\"/ops/chat/send\" method=\"post\" data-session-key=\"session-42\""));
    assert!(
        html.contains("id=\"tau-ops-chat-theme\" type=\"hidden\" name=\"theme\" value=\"light\"")
    );
    assert!(html.contains(
        "id=\"tau-ops-chat-sidebar\" type=\"hidden\" name=\"sidebar\" value=\"collapsed\""
    ));
    assert!(html.contains("id=\"tau-ops-chat-transcript\" data-message-count=\"2\""));
    assert!(html.contains("id=\"tau-ops-chat-message-row-0\" data-message-role=\"user\""));
    assert!(html.contains("id=\"tau-ops-chat-message-row-1\" data-message-role=\"assistant\""));
    assert!(html.contains("first message"));
    assert!(html.contains("second message"));
    assert!(html.contains(
        "id=\"tau-ops-chat-latest-turn\" data-latest-turn-visible=\"true\" aria-hidden=\"false\""
    ));
    assert!(html.contains("data-latest-user-index=\"0\""));
    assert!(html.contains("data-latest-assistant-index=\"1\""));
    assert!(html.contains("id=\"tau-ops-chat-latest-user\" data-message-role=\"user\""));
    assert!(html.contains("id=\"tau-ops-chat-latest-assistant\" data-message-role=\"assistant\""));
    assert!(html.contains("#tau-ops-chat-panel[aria-hidden=\"false\"]"));
    assert!(html.contains("grid-template-columns: minmax(0, 1fr);"));
    assert!(html.contains("max-width: 720px;"));
    assert!(html.contains("#tau-ops-chat-session-summary"));
    assert!(html.contains("#tau-ops-chat-transcript {"));
    assert!(html.contains("grid-template-columns: minmax(0, 1fr);"));
    assert!(html.contains("overflow-x: hidden;"));
    assert!(html.contains("max-width: 420px;"));
    assert!(html.contains("#tau-ops-protected-shell #tau-ops-chat-new-session-form"));
    assert!(html.contains("#tau-ops-protected-shell #tau-ops-chat-send-form"));
    assert!(html.contains("list-style: none;"));
    assert!(html.contains("width: min(760px, 100%);"));
    assert!(html.contains("max-width: min(760px, 100%);"));
    assert!(html.contains("min-width: 0;"));
    assert!(html.contains("word-break: break-word;"));
    assert!(!html.contains("grid-template-columns: minmax(220px, 280px) minmax(0, 1fr);"));
    assert!(html.contains("#tau-ops-chat-transcript [data-token-stream=\"assistant\"]"));
    assert!(html.contains("display: none;"));
}

#[test]
fn functional_spec_2872_c01_chat_route_renders_new_session_form_contract_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "chat-c01".to_string(),
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-chat-new-session-form\" action=\"/ops/chat/new\" method=\"post\" data-active-session-key=\"chat-c01\""
        ));
    assert!(html.contains(
        "id=\"tau-ops-chat-new-session-key\" type=\"text\" name=\"session_key\" value=\"\""
    ));
    assert!(html
        .contains("id=\"tau-ops-chat-new-theme\" type=\"hidden\" name=\"theme\" value=\"light\""));
    assert!(html.contains(
        "id=\"tau-ops-chat-new-sidebar\" type=\"hidden\" name=\"sidebar\" value=\"collapsed\""
    ));
    assert!(html.contains("id=\"tau-ops-chat-new-session-button\" type=\"submit\""));
}

#[test]
fn functional_spec_2881_c01_chat_route_renders_multiline_compose_contract_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "chat-multiline".to_string(),
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-chat-input\" name=\"message\" placeholder=\"Type a message for the active session\" rows=\"4\" data-multiline-enabled=\"true\" data-newline-shortcut=\"shift-enter\""
        ));
    assert!(html.contains(
        "id=\"tau-ops-chat-input-shortcut-hint\" data-shortcut-contract=\"shift-enter\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-chat-compose-shortcuts\" data-submit-shortcut=\"enter\" data-newline-shortcut=\"shift-enter\""
    ));
    assert!(html.contains("form.requestSubmit()"));
    assert!(html.contains("data-submit-blocked\", \"empty-message\""));
}

#[test]
fn functional_spec_2862_c01_c02_c03_chat_route_renders_token_counter_marker_contract() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "session-usage".to_string(),
            send_form_action: "/ops/chat/send".to_string(),
            send_form_method: "post".to_string(),
            session_options: vec![],
            message_rows: vec![],
            session_detail_usage_input_tokens: 13,
            session_detail_usage_output_tokens: 21,
            session_detail_usage_total_tokens: 34,
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-chat-panel\" data-route=\"/ops/chat\" aria-hidden=\"false\" data-active-session-key=\"session-usage\" data-panel-visible=\"true\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-chat-token-counter\" data-session-key=\"session-usage\" data-input-tokens=\"13\" data-output-tokens=\"21\" data-total-tokens=\"34\""
        ));
}

#[test]
fn regression_spec_2862_c04_non_chat_routes_keep_hidden_chat_token_counter_marker_contract() {
    let ops_html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "chat-c01".to_string(),
            session_detail_usage_input_tokens: 0,
            session_detail_usage_output_tokens: 0,
            session_detail_usage_total_tokens: 0,
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });
    assert!(ops_html.contains(
            "id=\"tau-ops-chat-panel\" data-route=\"/ops/chat\" aria-hidden=\"true\" data-active-session-key=\"chat-c01\" data-panel-visible=\"false\""
        ));
    assert!(ops_html.contains(
            "id=\"tau-ops-chat-token-counter\" data-session-key=\"chat-c01\" data-input-tokens=\"0\" data-output-tokens=\"0\" data-total-tokens=\"0\""
        ));

    let sessions_html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Sessions,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "chat-c01".to_string(),
            session_detail_usage_input_tokens: 0,
            session_detail_usage_output_tokens: 0,
            session_detail_usage_total_tokens: 0,
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });
    assert!(sessions_html.contains(
            "id=\"tau-ops-chat-panel\" data-route=\"/ops/chat\" aria-hidden=\"true\" data-active-session-key=\"chat-c01\" data-panel-visible=\"false\""
        ));
    assert!(sessions_html.contains(
            "id=\"tau-ops-chat-token-counter\" data-session-key=\"chat-c01\" data-input-tokens=\"0\" data-output-tokens=\"0\" data-total-tokens=\"0\""
        ));
}

#[test]
fn functional_spec_2866_c01_c02_chat_route_renders_inline_tool_card_for_tool_rows_only() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "chat-tool-session".to_string(),
            message_rows: vec![
                TauOpsDashboardChatMessageRow {
                    role: "user".to_string(),
                    content: "run tool".to_string(),
                },
                TauOpsDashboardChatMessageRow {
                    role: "tool".to_string(),
                    content: "{\"result\":\"ok\"}".to_string(),
                },
                TauOpsDashboardChatMessageRow {
                    role: "assistant".to_string(),
                    content: "tool completed".to_string(),
                },
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-chat-message-row-1\" data-message-role=\"tool\""));
    assert!(html.contains(
        "id=\"tau-ops-chat-tool-card-1\" data-tool-card=\"true\" data-inline-result=\"true\""
    ));
    assert!(!html.contains("id=\"tau-ops-chat-tool-card-0\""));
    assert!(!html.contains("id=\"tau-ops-chat-tool-card-2\""));
}

#[test]
fn regression_spec_2866_c04_non_chat_routes_keep_hidden_chat_tool_card_markers() {
    let ops_html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "chat-tool-session".to_string(),
            message_rows: vec![TauOpsDashboardChatMessageRow {
                role: "tool".to_string(),
                content: "{\"result\":\"ok\"}".to_string(),
            }],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });
    assert!(ops_html.contains(
            "id=\"tau-ops-chat-panel\" data-route=\"/ops/chat\" aria-hidden=\"true\" data-active-session-key=\"chat-tool-session\" data-panel-visible=\"false\""
        ));
    assert!(ops_html.contains(
        "id=\"tau-ops-chat-tool-card-0\" data-tool-card=\"true\" data-inline-result=\"true\""
    ));

    let sessions_html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Sessions,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "chat-tool-session".to_string(),
            message_rows: vec![TauOpsDashboardChatMessageRow {
                role: "tool".to_string(),
                content: "{\"result\":\"ok\"}".to_string(),
            }],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });
    assert!(sessions_html.contains(
            "id=\"tau-ops-chat-panel\" data-route=\"/ops/chat\" aria-hidden=\"true\" data-active-session-key=\"chat-tool-session\" data-panel-visible=\"false\""
        ));
    assert!(sessions_html.contains(
        "id=\"tau-ops-chat-tool-card-0\" data-tool-card=\"true\" data-inline-result=\"true\""
    ));
}

#[test]
fn functional_spec_2870_c01_c02_chat_route_renders_markdown_and_code_markers() {
    let markdown_code_message = "## Build report\n- item one\n[docs](https://example.com)\n|k|v|\n|---|---|\n|a|b|\n```rust\nfn main() {}\n```";
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "chat-markdown-code".to_string(),
            message_rows: vec![
                TauOpsDashboardChatMessageRow {
                    role: "user".to_string(),
                    content: "show report".to_string(),
                },
                TauOpsDashboardChatMessageRow {
                    role: "assistant".to_string(),
                    content: markdown_code_message.to_string(),
                },
                TauOpsDashboardChatMessageRow {
                    role: "assistant".to_string(),
                    content: "plain response".to_string(),
                },
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-chat-message-row-1\" data-message-role=\"assistant\""));
    assert!(html.contains("id=\"tau-ops-chat-markdown-1\" data-markdown-rendered=\"true\""));
    assert!(html.contains(
            "id=\"tau-ops-chat-code-block-1\" data-code-block=\"true\" data-language=\"rust\" data-code=\"fn main() {}\""
        ));
    assert!(!html.contains("id=\"tau-ops-chat-markdown-0\""));
    assert!(!html.contains("id=\"tau-ops-chat-code-block-0\""));
    assert!(!html.contains("id=\"tau-ops-chat-code-block-2\""));
}

#[test]
fn regression_spec_2870_c04_non_chat_routes_keep_hidden_markdown_and_code_markers() {
    let markdown_code_message = "## Build report\n- item one\n[docs](https://example.com)\n|k|v|\n|---|---|\n|a|b|\n```rust\nfn main() {}\n```";
    let ops_html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "chat-markdown-code".to_string(),
            message_rows: vec![TauOpsDashboardChatMessageRow {
                role: "assistant".to_string(),
                content: markdown_code_message.to_string(),
            }],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });
    assert!(ops_html.contains(
            "id=\"tau-ops-chat-panel\" data-route=\"/ops/chat\" aria-hidden=\"true\" data-active-session-key=\"chat-markdown-code\" data-panel-visible=\"false\""
        ));
    assert!(ops_html.contains("id=\"tau-ops-chat-markdown-0\" data-markdown-rendered=\"true\""));
    assert!(ops_html.contains(
            "id=\"tau-ops-chat-code-block-0\" data-code-block=\"true\" data-language=\"rust\" data-code=\"fn main() {}\""
        ));

    let sessions_html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Sessions,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "chat-markdown-code".to_string(),
            message_rows: vec![TauOpsDashboardChatMessageRow {
                role: "assistant".to_string(),
                content: markdown_code_message.to_string(),
            }],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });
    assert!(sessions_html.contains(
            "id=\"tau-ops-chat-panel\" data-route=\"/ops/chat\" aria-hidden=\"true\" data-active-session-key=\"chat-markdown-code\" data-panel-visible=\"false\""
        ));
    assert!(
        sessions_html.contains("id=\"tau-ops-chat-markdown-0\" data-markdown-rendered=\"true\"")
    );
    assert!(sessions_html.contains(
            "id=\"tau-ops-chat-code-block-0\" data-code-block=\"true\" data-language=\"rust\" data-code=\"fn main() {}\""
        ));
}

#[test]
fn functional_spec_2834_c01_chat_route_renders_session_selector_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-chat-session-selector\" data-active-session-key=\"default\" data-option-count=\"1\""
        ));
    assert!(html.contains("id=\"tau-ops-chat-session-options\""));
    assert!(html.contains(
        "id=\"tau-ops-chat-session-option-0\" data-session-key=\"default\" data-selected=\"true\""
    ));
    assert!(html.contains("data-session-link=\"default\""));
    assert!(html.contains("href=\"/ops/chat?theme=dark&amp;sidebar=expanded&amp;session=default\""));
}

#[test]
fn functional_spec_2834_c02_chat_route_keeps_active_session_selected_in_selector_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "session-beta".to_string(),
            send_form_action: "/ops/chat/send".to_string(),
            send_form_method: "post".to_string(),
            session_options: vec![],
            message_rows: vec![TauOpsDashboardChatMessageRow {
                role: "user".to_string(),
                content: "chat from beta".to_string(),
            }],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html
        .contains("id=\"tau-ops-chat-session-selector\" data-active-session-key=\"session-beta\""));
    assert!(html.contains(
            "id=\"tau-ops-chat-session-option-0\" data-session-key=\"session-beta\" data-selected=\"true\""
        ));
    assert!(html
        .contains("href=\"/ops/chat?theme=light&amp;sidebar=collapsed&amp;session=session-beta\""));
    assert!(html.contains(
            "id=\"tau-ops-chat-session-key\" type=\"hidden\" name=\"session_key\" value=\"session-beta\""
        ));
    assert!(html.contains("chat from beta"));
}

#[test]
fn functional_spec_2834_c03_chat_route_adds_missing_active_session_option_marker() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "session-zeta".to_string(),
            send_form_action: "/ops/chat/send".to_string(),
            send_form_method: "post".to_string(),
            session_options: vec![TauOpsDashboardChatSessionOptionRow {
                session_key: "session-alpha".to_string(),
                selected: false,
                entry_count: 0,
                usage_total_tokens: 0,
                validation_is_valid: true,
                updated_unix_ms: 0,
            }],
            message_rows: vec![TauOpsDashboardChatMessageRow {
                role: "user".to_string(),
                content: "zeta transcript".to_string(),
            }],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-chat-session-selector\" data-active-session-key=\"session-zeta\" data-option-count=\"2\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-chat-session-option-0\" data-session-key=\"session-alpha\" data-selected=\"false\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-chat-session-option-1\" data-session-key=\"session-zeta\" data-selected=\"true\""
        ));
}

#[test]
fn functional_spec_2901_c01_c03_chat_route_renders_assistant_token_stream_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "chat-stream".to_string(),
            message_rows: vec![
                TauOpsDashboardChatMessageRow {
                    role: "user".to_string(),
                    content: "operator request".to_string(),
                },
                TauOpsDashboardChatMessageRow {
                    role: "assistant".to_string(),
                    content: "stream one two".to_string(),
                },
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-chat-message-row-1\" data-message-role=\"assistant\" data-assistant-token-stream=\"true\" data-token-count=\"3\""
        ));
    assert!(html.contains(
        "id=\"tau-ops-chat-token-stream-1\" data-token-stream=\"assistant\" data-token-count=\"3\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-chat-token-1-0\" data-token-index=\"0\" data-token-value=\"stream\""
    ));
    assert!(html
        .contains("id=\"tau-ops-chat-token-1-1\" data-token-index=\"1\" data-token-value=\"one\""));
    assert!(html
        .contains("id=\"tau-ops-chat-token-1-2\" data-token-index=\"2\" data-token-value=\"two\""));
    assert!(!html.contains("id=\"tau-ops-chat-token-stream-0\""));
}

#[test]
fn functional_spec_2905_c01_c03_memory_route_renders_search_panel_and_empty_state_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Memory,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-memory-panel\" data-route=\"/ops/memory\" aria-hidden=\"false\" data-panel-visible=\"true\" data-query=\"\" data-result-count=\"0\""
        ));
    assert!(html.contains(
        "id=\"tau-ops-memory-scope-summary\" data-session-key=\"default\" data-result-count=\"0\" data-query=\"\" data-workspace-id=\"\" data-channel-id=\"\" data-actor-id=\"\" data-memory-type=\"\" data-create-status=\"idle\""
    ));
    assert!(html.contains("Memory Scope"));
    assert!(html.contains("all entries"));
    assert!(html.contains("id=\"tau-ops-memory-open-graph\" href=\"/ops/memory-graph?theme=light&amp;sidebar=collapsed&amp;session=default&amp;workspace_id=&amp;channel_id=&amp;actor_id=&amp;memory_type=\""));
    assert!(html.contains("id=\"tau-ops-memory-open-session\" href=\"/ops/sessions/default?theme=light&amp;sidebar=collapsed&amp;session=default\""));
    assert!(
        html.contains("id=\"tau-ops-memory-search-form\" action=\"/ops/memory\" method=\"get\"")
    );
    assert!(html.contains("id=\"tau-ops-memory-query\" type=\"search\" name=\"query\" value=\"\""));
    assert!(html.contains("id=\"tau-ops-memory-results\" data-result-count=\"0\""));
    assert!(html.contains("id=\"tau-ops-memory-empty-state\" data-empty-state=\"true\""));
}

#[test]
fn functional_spec_2909_c01_c03_memory_route_renders_scope_filter_controls() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Memory,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-memory-workspace-filter\" type=\"text\" name=\"workspace_id\" value=\"\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-channel-filter\" type=\"text\" name=\"channel_id\" value=\"\""
    ));
    assert!(html
        .contains("id=\"tau-ops-memory-actor-filter\" type=\"text\" name=\"actor_id\" value=\"\""));
}

#[test]
fn functional_spec_2913_c01_c03_memory_route_renders_type_filter_control() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Memory,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-memory-type-filter\" type=\"text\" name=\"memory_type\" value=\"\""
    ));
}

#[test]
fn functional_spec_2917_c01_c03_memory_route_renders_create_form_and_status_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Memory,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-memory-panel\" data-route=\"/ops/memory\" aria-hidden=\"false\" data-panel-visible=\"true\" data-query=\"\" data-result-count=\"0\" data-workspace-id=\"\" data-channel-id=\"\" data-actor-id=\"\" data-memory-type=\"\" data-create-status=\"idle\" data-created-memory-id=\"\""
        ));
    assert!(
            html.contains("id=\"tau-ops-memory-create-status\" data-create-status=\"idle\" data-created-memory-id=\"\"")
        );
    assert!(
        html.contains("id=\"tau-ops-memory-create-form\" action=\"/ops/memory\" method=\"post\"")
    );
    assert!(html.contains(
        "id=\"tau-ops-memory-create-entry-id\" type=\"text\" name=\"entry_id\" value=\"\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-create-summary\" type=\"text\" name=\"summary\" value=\"\""
    ));
    assert!(
        html.contains("id=\"tau-ops-memory-create-tags\" type=\"text\" name=\"tags\" value=\"\"")
    );
    assert!(
        html.contains("id=\"tau-ops-memory-create-facts\" type=\"text\" name=\"facts\" value=\"\"")
    );
    assert!(html.contains(
            "id=\"tau-ops-memory-create-source-event-key\" type=\"text\" name=\"source_event_key\" value=\"\""
        ));
    assert!(html.contains(
        "id=\"tau-ops-memory-create-workspace-id\" type=\"text\" name=\"workspace_id\" value=\"\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-create-channel-id\" type=\"text\" name=\"channel_id\" value=\"\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-create-actor-id\" type=\"text\" name=\"actor_id\" value=\"\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-create-memory-type\" type=\"text\" name=\"memory_type\" value=\"\""
    ));
    assert!(html.contains(
            "id=\"tau-ops-memory-create-importance\" type=\"number\" step=\"0.01\" name=\"importance\" value=\"\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-memory-create-relation-target-id\" type=\"text\" name=\"relation_target_id\" value=\"\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-memory-create-relation-type\" type=\"text\" name=\"relation_type\" value=\"\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-memory-create-relation-weight\" type=\"number\" step=\"0.01\" name=\"relation_weight\" value=\"\""
        ));
}

#[test]
fn functional_spec_2921_c01_c03_memory_route_renders_edit_form_and_status_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Memory,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-memory-edit-status\" data-edit-status=\"idle\" data-edited-memory-id=\"\""
    ));
    assert!(html.contains("id=\"tau-ops-memory-edit-form\" action=\"/ops/memory\" method=\"post\""));
    assert!(html.contains(
        "id=\"tau-ops-memory-edit-operation\" type=\"hidden\" name=\"operation\" value=\"edit\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-edit-entry-id\" type=\"text\" name=\"entry_id\" value=\"\""
    ));
    assert!(html
        .contains("id=\"tau-ops-memory-edit-summary\" type=\"text\" name=\"summary\" value=\"\""));
    assert!(html.contains(
        "id=\"tau-ops-memory-edit-memory-type\" type=\"text\" name=\"memory_type\" value=\"\""
    ));
    assert!(html.contains(
            "id=\"tau-ops-memory-edit-importance\" type=\"number\" step=\"0.01\" name=\"importance\" value=\"\""
        ));
}

#[test]
fn regression_spec_2921_memory_edit_status_updated_renders_updated_message_marker() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Memory,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            memory_create_status: "updated".to_string(),
            memory_create_created_entry_id: "mem-edit-1".to_string(),
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    let edit_status_marker = "id=\"tau-ops-memory-edit-status\" data-edit-status=\"updated\" data-edited-memory-id=\"mem-edit-1\"";
    assert!(html.contains(edit_status_marker));
    let edit_section = &html[html
        .find(edit_status_marker)
        .expect("edit status marker should be rendered when status is updated")..];
    assert!(edit_section.contains(">Memory entry updated.</p>"));
}

#[test]
fn regression_spec_2917_memory_create_status_created_renders_created_message_marker() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Memory,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            memory_create_status: "created".to_string(),
            memory_create_created_entry_id: "mem-create-1".to_string(),
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-memory-create-status\" data-create-status=\"created\" data-created-memory-id=\"mem-create-1\""
        ));
    assert!(html.contains("Memory entry created."));
}

#[test]
fn regression_spec_2917_memory_create_status_updated_renders_updated_message_marker() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Memory,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            memory_create_status: "updated".to_string(),
            memory_create_created_entry_id: "mem-create-1".to_string(),
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-memory-create-status\" data-create-status=\"updated\" data-created-memory-id=\"mem-create-1\""
        ));
    assert!(html.contains("Memory entry updated."));
}

#[test]
fn functional_spec_3060_c01_memory_route_renders_delete_form_and_confirmation_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Memory,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-memory-panel\" data-route=\"/ops/memory\" aria-hidden=\"false\" data-panel-visible=\"true\" data-query=\"\" data-result-count=\"0\" data-workspace-id=\"\" data-channel-id=\"\" data-actor-id=\"\" data-memory-type=\"\" data-create-status=\"idle\" data-created-memory-id=\"\" data-edit-status=\"idle\" data-edited-memory-id=\"\" data-delete-status=\"idle\" data-deleted-memory-id=\"\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-delete-status\" data-delete-status=\"idle\" data-deleted-memory-id=\"\""
    ));
    assert!(
        html.contains("id=\"tau-ops-memory-delete-form\" action=\"/ops/memory\" method=\"post\"")
    );
    assert!(html.contains(
        "id=\"tau-ops-memory-delete-operation\" type=\"hidden\" name=\"operation\" value=\"delete\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-delete-entry-id\" type=\"text\" name=\"entry_id\" value=\"\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-delete-confirm\" type=\"checkbox\" name=\"confirm_delete\" value=\"true\""
    ));
}

#[test]
fn regression_spec_3060_c04_non_memory_routes_keep_hidden_memory_delete_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-memory-panel\" data-route=\"/ops/memory\" aria-hidden=\"true\" data-panel-visible=\"false\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-delete-status\" data-delete-status=\"idle\" data-deleted-memory-id=\"\""
    ));
    assert!(
        html.contains("id=\"tau-ops-memory-delete-form\" action=\"/ops/memory\" method=\"post\"")
    );
}

#[test]
fn functional_spec_3064_c01_memory_route_renders_detail_panel_default_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Memory,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-memory-detail-panel\" data-detail-visible=\"false\" data-memory-id=\"\" data-memory-type=\"\" data-embedding-source=\"\" data-embedding-model=\"\" data-embedding-reason-code=\"\" data-embedding-dimensions=\"0\" data-relation-count=\"0\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-detail-embedding\" data-embedding-source=\"\" data-embedding-model=\"\" data-embedding-reason-code=\"\" data-embedding-dimensions=\"0\""
    ));
    assert!(html.contains("id=\"tau-ops-memory-relations\" data-relation-count=\"0\""));
    assert!(html.contains("id=\"tau-ops-memory-relations-empty-state\" data-empty-state=\"true\""));
}

#[test]
fn regression_spec_3064_c04_non_memory_routes_keep_hidden_detail_panel_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-memory-panel\" data-route=\"/ops/memory\" aria-hidden=\"true\" data-panel-visible=\"false\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-detail-panel\" data-detail-visible=\"false\" data-memory-id=\"\""
    ));
}

#[test]
fn functional_spec_3068_c01_memory_graph_route_renders_graph_panel_default_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::MemoryGraph,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-memory-graph-panel\" data-route=\"/ops/memory-graph\" aria-hidden=\"false\" data-panel-visible=\"true\" data-node-count=\"0\" data-edge-count=\"0\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-graph-scope-summary\" data-session-key=\"default\" data-node-count=\"0\" data-edge-count=\"0\" data-scope-memory-type=\"\" data-filter-memory-type=\"all\" data-filter-relation-type=\"all\""
    ));
    assert!(html.contains("Graph Scope"));
    assert!(html.contains("scope type all types"));
    assert!(html.contains("empty graph"));
    assert!(html.contains("id=\"tau-ops-memory-graph-open-memory\" href=\"/ops/memory?theme=light&amp;sidebar=collapsed&amp;session=default&amp;workspace_id=&amp;channel_id=&amp;actor_id=&amp;memory_type=\""));
    assert!(html.contains("id=\"tau-ops-memory-graph-open-session\" href=\"/ops/sessions/default?theme=light&amp;sidebar=collapsed&amp;session=default\""));
    assert!(html.contains("id=\"tau-ops-memory-graph-nodes\" data-node-count=\"0\""));
    assert!(html.contains("id=\"tau-ops-memory-graph-edges\" data-edge-count=\"0\""));
    assert!(html.contains("id=\"tau-ops-memory-graph-empty-state\" data-empty-state=\"true\""));
}

#[test]
fn regression_spec_3068_c03_non_memory_graph_routes_keep_hidden_graph_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-memory-graph-panel\" data-route=\"/ops/memory-graph\" aria-hidden=\"true\" data-panel-visible=\"false\""
    ));
    assert!(html.contains("id=\"tau-ops-memory-graph-nodes\" data-node-count=\"0\""));
    assert!(html.contains("id=\"tau-ops-memory-graph-edges\" data-edge-count=\"0\""));
}

#[test]
fn functional_spec_3070_c01_c02_memory_graph_route_renders_node_size_markers_from_importance() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::MemoryGraph,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            memory_graph_node_rows: vec![
                TauOpsDashboardMemoryGraphNodeRow {
                    memory_id: "mem-size-low".to_string(),
                    memory_type: "fact".to_string(),
                    importance: "0.1000".to_string(),
                },
                TauOpsDashboardMemoryGraphNodeRow {
                    memory_id: "mem-size-high".to_string(),
                    memory_type: "goal".to_string(),
                    importance: "0.9000".to_string(),
                },
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-memory-graph-node-0\" data-memory-id=\"mem-size-low\" data-memory-type=\"fact\" data-importance=\"0.1000\" data-node-size-bucket=\"small\" data-node-size-px=\"13.60\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-graph-node-1\" data-memory-id=\"mem-size-high\" data-memory-type=\"goal\" data-importance=\"0.9000\" data-node-size-bucket=\"large\" data-node-size-px=\"26.40\""
    ));
}

#[test]
fn functional_spec_3078_c02_memory_graph_route_renders_node_color_markers_from_memory_type() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::MemoryGraph,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            memory_graph_node_rows: vec![
                TauOpsDashboardMemoryGraphNodeRow {
                    memory_id: "mem-color-fact".to_string(),
                    memory_type: "fact".to_string(),
                    importance: "0.5000".to_string(),
                },
                TauOpsDashboardMemoryGraphNodeRow {
                    memory_id: "mem-color-event".to_string(),
                    memory_type: "event".to_string(),
                    importance: "0.5000".to_string(),
                },
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-memory-graph-node-0\" data-memory-id=\"mem-color-fact\" data-memory-type=\"fact\" data-importance=\"0.5000\" data-node-size-bucket=\"medium\" data-node-size-px=\"20.00\" data-node-color-token=\"fact\" data-node-color-hex=\"#2563eb\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-graph-node-1\" data-memory-id=\"mem-color-event\" data-memory-type=\"event\" data-importance=\"0.5000\" data-node-size-bucket=\"medium\" data-node-size-px=\"20.00\" data-node-color-token=\"event\" data-node-color-hex=\"#7c3aed\""
    ));
}

#[test]
fn functional_spec_3082_c02_memory_graph_route_renders_edge_style_markers_from_relation_type() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::MemoryGraph,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            memory_graph_edge_rows: vec![
                TauOpsDashboardMemoryGraphEdgeRow {
                    source_memory_id: "mem-edge-a".to_string(),
                    target_memory_id: "mem-edge-b".to_string(),
                    relation_type: "related_to".to_string(),
                    effective_weight: "0.4200".to_string(),
                },
                TauOpsDashboardMemoryGraphEdgeRow {
                    source_memory_id: "mem-edge-b".to_string(),
                    target_memory_id: "mem-edge-c".to_string(),
                    relation_type: "updates".to_string(),
                    effective_weight: "0.5500".to_string(),
                },
                TauOpsDashboardMemoryGraphEdgeRow {
                    source_memory_id: "mem-edge-c".to_string(),
                    target_memory_id: "mem-edge-d".to_string(),
                    relation_type: "contradicts".to_string(),
                    effective_weight: "0.6600".to_string(),
                },
                TauOpsDashboardMemoryGraphEdgeRow {
                    source_memory_id: "mem-edge-d".to_string(),
                    target_memory_id: "mem-edge-e".to_string(),
                    relation_type: "depends_on".to_string(),
                    effective_weight: "0.7700".to_string(),
                },
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-memory-graph-edge-0\" data-source-memory-id=\"mem-edge-a\" data-target-memory-id=\"mem-edge-b\" data-relation-type=\"related_to\" data-relation-weight=\"0.4200\" data-edge-style-token=\"solid\" data-edge-stroke-dasharray=\"none\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-graph-edge-1\" data-source-memory-id=\"mem-edge-b\" data-target-memory-id=\"mem-edge-c\" data-relation-type=\"updates\" data-relation-weight=\"0.5500\" data-edge-style-token=\"dashed\" data-edge-stroke-dasharray=\"6 4\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-graph-edge-2\" data-source-memory-id=\"mem-edge-c\" data-target-memory-id=\"mem-edge-d\" data-relation-type=\"contradicts\" data-relation-weight=\"0.6600\" data-edge-style-token=\"dotted\" data-edge-stroke-dasharray=\"2 4\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-graph-edge-3\" data-source-memory-id=\"mem-edge-d\" data-target-memory-id=\"mem-edge-e\" data-relation-type=\"depends_on\" data-relation-weight=\"0.7700\" data-edge-style-token=\"dashed\" data-edge-stroke-dasharray=\"6 4\""
    ));
}

#[test]
fn functional_spec_3086_c02_memory_graph_route_renders_selected_node_detail_panel_contracts() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::MemoryGraph,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "ops-memory-graph-detail".to_string(),
            memory_search_workspace_id: "workspace-detail-graph".to_string(),
            memory_search_channel_id: "channel-detail-graph".to_string(),
            memory_search_actor_id: "operator".to_string(),
            memory_search_memory_type: "goal".to_string(),
            memory_detail_visible: true,
            memory_detail_selected_entry_id: "mem-detail-graph".to_string(),
            memory_detail_summary: "Graph detail summary".to_string(),
            memory_detail_memory_type: "goal".to_string(),
            memory_graph_node_rows: vec![
                TauOpsDashboardMemoryGraphNodeRow {
                    memory_id: "mem-detail-graph".to_string(),
                    memory_type: "goal".to_string(),
                    importance: "0.7000".to_string(),
                },
                TauOpsDashboardMemoryGraphNodeRow {
                    memory_id: "mem-other-graph".to_string(),
                    memory_type: "goal".to_string(),
                    importance: "0.4000".to_string(),
                },
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-memory-graph-node-0\" data-memory-id=\"mem-detail-graph\""));
    assert!(html.contains("id=\"tau-ops-memory-graph-node-1\" data-memory-id=\"mem-other-graph\""));
    assert!(html.contains("data-node-selected=\"true\""));
    assert!(html.contains("data-node-selected=\"false\""));
    assert!(html.contains("data-node-detail-href=\"/ops/memory-graph?theme=light"));
    assert!(html.contains("detail_memory_id=mem-detail-graph"));
    assert!(html.contains("detail_memory_id=mem-other-graph"));
    assert!(html.contains(
        "id=\"tau-ops-memory-graph-detail-panel\" data-detail-visible=\"true\" data-memory-id=\"mem-detail-graph\" data-memory-type=\"goal\" data-relation-count=\"0\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-memory-graph-detail-summary\" data-memory-id=\"mem-detail-graph\">Graph detail summary"
    ));
    assert!(html
        .contains("id=\"tau-ops-memory-graph-detail-open-memory\" href=\"/ops/memory?theme=light"));
    assert!(html.contains("id=\"tau-ops-memory-graph-detail-open-memory\""));
    assert!(html.contains("data-detail-memory-id=\"mem-detail-graph\""));
}

#[test]
fn functional_spec_3090_c02_memory_graph_route_marks_connected_edges_and_neighbors_for_focus() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::MemoryGraph,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            memory_detail_visible: true,
            memory_detail_selected_entry_id: "mem-focus".to_string(),
            memory_graph_node_rows: vec![
                TauOpsDashboardMemoryGraphNodeRow {
                    memory_id: "mem-focus".to_string(),
                    memory_type: "goal".to_string(),
                    importance: "0.7000".to_string(),
                },
                TauOpsDashboardMemoryGraphNodeRow {
                    memory_id: "mem-neighbor".to_string(),
                    memory_type: "fact".to_string(),
                    importance: "0.5000".to_string(),
                },
                TauOpsDashboardMemoryGraphNodeRow {
                    memory_id: "mem-unrelated".to_string(),
                    memory_type: "event".to_string(),
                    importance: "0.5000".to_string(),
                },
            ],
            memory_graph_edge_rows: vec![
                TauOpsDashboardMemoryGraphEdgeRow {
                    source_memory_id: "mem-focus".to_string(),
                    target_memory_id: "mem-neighbor".to_string(),
                    relation_type: "related_to".to_string(),
                    effective_weight: "0.4200".to_string(),
                },
                TauOpsDashboardMemoryGraphEdgeRow {
                    source_memory_id: "mem-neighbor".to_string(),
                    target_memory_id: "mem-unrelated".to_string(),
                    relation_type: "updates".to_string(),
                    effective_weight: "0.2000".to_string(),
                },
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-memory-graph-node-0\" data-memory-id=\"mem-focus\""));
    assert!(html.contains("id=\"tau-ops-memory-graph-node-1\" data-memory-id=\"mem-neighbor\""));
    assert!(html.contains("id=\"tau-ops-memory-graph-node-2\" data-memory-id=\"mem-unrelated\""));
    assert!(html.contains("data-node-selected=\"true\" data-node-hover-neighbor=\"true\""));
    assert!(html.contains("data-node-hover-neighbor=\"false\""));
    assert!(html.contains(
        "data-source-memory-id=\"mem-focus\" data-target-memory-id=\"mem-neighbor\" data-relation-type=\"related_to\" data-relation-weight=\"0.4200\" data-edge-style-token=\"solid\" data-edge-stroke-dasharray=\"none\" data-edge-hover-highlighted=\"true\""
    ));
    assert!(html.contains(
        "data-source-memory-id=\"mem-neighbor\" data-target-memory-id=\"mem-unrelated\" data-relation-type=\"updates\" data-relation-weight=\"0.2000\" data-edge-style-token=\"dashed\" data-edge-stroke-dasharray=\"6 4\" data-edge-hover-highlighted=\"false\""
    ));
}

#[test]
fn functional_spec_3094_c01_c02_memory_graph_route_renders_default_zoom_markers_and_actions() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::MemoryGraph,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "ops-zoom".to_string(),
            memory_search_workspace_id: "workspace-zoom".to_string(),
            memory_search_channel_id: "channel-zoom".to_string(),
            memory_search_actor_id: "operator".to_string(),
            memory_search_memory_type: "goal".to_string(),
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-memory-graph-zoom-controls\" data-zoom-level=\"1.00\" data-zoom-min=\"0.25\" data-zoom-max=\"2.00\" data-zoom-step=\"0.10\""
    ));
    assert!(html.contains("id=\"tau-ops-memory-graph-zoom-in\""));
    assert!(html.contains("data-zoom-action=\"in\""));
    assert!(html.contains("graph_zoom=1.10"));
    assert!(html.contains("id=\"tau-ops-memory-graph-zoom-out\""));
    assert!(html.contains("data-zoom-action=\"out\""));
    assert!(html.contains("graph_zoom=0.90"));
}

#[test]
fn functional_spec_3099_c01_c02_memory_graph_route_renders_default_pan_markers_and_actions() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::MemoryGraph,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "ops-pan".to_string(),
            memory_search_workspace_id: "workspace-pan".to_string(),
            memory_search_channel_id: "channel-pan".to_string(),
            memory_search_actor_id: "operator".to_string(),
            memory_search_memory_type: "goal".to_string(),
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-memory-graph-pan-controls\" data-pan-x=\"0.00\" data-pan-y=\"0.00\" data-pan-step=\"25.00\""
    ));
    assert!(html.contains("id=\"tau-ops-memory-graph-pan-left\""));
    assert!(html.contains("data-pan-action=\"left\""));
    assert!(html.contains("graph_pan_x=-25.00"));
    assert!(html.contains("id=\"tau-ops-memory-graph-pan-right\""));
    assert!(html.contains("data-pan-action=\"right\""));
    assert!(html.contains("graph_pan_x=25.00"));
    assert!(html.contains("id=\"tau-ops-memory-graph-pan-up\""));
    assert!(html.contains("data-pan-action=\"up\""));
    assert!(html.contains("graph_pan_y=-25.00"));
    assert!(html.contains("id=\"tau-ops-memory-graph-pan-down\""));
    assert!(html.contains("data-pan-action=\"down\""));
    assert!(html.contains("graph_pan_y=25.00"));
}

#[test]
fn functional_spec_3103_c01_c02_memory_graph_route_renders_filter_controls_and_contract_links() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::MemoryGraph,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "ops-filter".to_string(),
            memory_search_workspace_id: "workspace-filter".to_string(),
            memory_search_channel_id: "channel-filter".to_string(),
            memory_search_actor_id: "operator".to_string(),
            memory_search_memory_type: "goal".to_string(),
            memory_graph_node_rows: vec![
                TauOpsDashboardMemoryGraphNodeRow {
                    memory_id: "goal-1".to_string(),
                    memory_type: "goal".to_string(),
                    importance: "0.9000".to_string(),
                },
                TauOpsDashboardMemoryGraphNodeRow {
                    memory_id: "goal-2".to_string(),
                    memory_type: "goal".to_string(),
                    importance: "0.7000".to_string(),
                },
                TauOpsDashboardMemoryGraphNodeRow {
                    memory_id: "fact-1".to_string(),
                    memory_type: "fact".to_string(),
                    importance: "0.5000".to_string(),
                },
            ],
            memory_graph_edge_rows: vec![
                TauOpsDashboardMemoryGraphEdgeRow {
                    source_memory_id: "goal-1".to_string(),
                    target_memory_id: "goal-2".to_string(),
                    relation_type: "related_to".to_string(),
                    effective_weight: "0.8000".to_string(),
                },
                TauOpsDashboardMemoryGraphEdgeRow {
                    source_memory_id: "goal-1".to_string(),
                    target_memory_id: "fact-1".to_string(),
                    relation_type: "contradicts".to_string(),
                    effective_weight: "0.6000".to_string(),
                },
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-memory-graph-filter-controls\" data-filter-memory-type=\"all\" data-filter-relation-type=\"all\""
    ));
    assert!(html.contains("id=\"tau-ops-memory-graph-filter-memory-type-all\""));
    assert!(html.contains("id=\"tau-ops-memory-graph-filter-memory-type-goal\""));
    assert!(html.contains("id=\"tau-ops-memory-graph-filter-relation-type-all\""));
    assert!(html.contains("id=\"tau-ops-memory-graph-filter-relation-type-related-to\""));
    assert!(html.contains("graph_filter_memory_type=goal"));
    assert!(html.contains("graph_filter_relation_type=related_to"));
}

#[test]
fn functional_spec_3106_c01_c03_tools_route_renders_inventory_panel_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::ToolsJobs,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-tools-panel\" data-route=\"/ops/tools-jobs\" aria-hidden=\"false\" data-panel-visible=\"true\" data-total-tools=\"0\""
    ));
    assert!(html.contains("id=\"tau-ops-tools-inventory-summary\" data-total-tools=\"0\""));
    assert!(html.contains("id=\"tau-ops-tools-inventory-table\" data-row-count=\"0\""));
    assert!(html.contains("id=\"tau-ops-tools-inventory-empty-state\" data-empty-state=\"true\""));
}

#[test]
fn functional_spec_3106_c02_tools_route_renders_deterministic_inventory_rows() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::ToolsJobs,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            tools_inventory_rows: vec![
                TauOpsDashboardToolInventoryRow {
                    tool_name: "memory_search".to_string(),
                    category: "Memory".to_string(),
                    policy: "allowed".to_string(),
                    usage_count: 4,
                    error_rate: "0.25".to_string(),
                    avg_latency_ms: "18.40".to_string(),
                    last_used_unix_ms: 1002,
                },
                TauOpsDashboardToolInventoryRow {
                    tool_name: "bash".to_string(),
                    category: "Code".to_string(),
                    policy: "allowed".to_string(),
                    usage_count: 6,
                    error_rate: "0.00".to_string(),
                    avg_latency_ms: "12.00".to_string(),
                    last_used_unix_ms: 1003,
                },
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-tools-panel\" data-route=\"/ops/tools-jobs\" aria-hidden=\"false\" data-panel-visible=\"true\" data-total-tools=\"2\""
    ));
    assert!(html.contains("id=\"tau-ops-tools-inventory-table\" data-row-count=\"2\""));
    assert!(html.contains(
        "id=\"tau-ops-tools-inventory-row-0\" data-tool-name=\"bash\" data-tool-category=\"Code\" data-tool-policy=\"allowed\" data-usage-count=\"6\" data-error-rate=\"0.00\" data-avg-latency-ms=\"12.00\" data-last-used-unix-ms=\"1003\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-tools-inventory-row-1\" data-tool-name=\"memory_search\" data-tool-category=\"Memory\" data-tool-policy=\"allowed\" data-usage-count=\"4\" data-error-rate=\"0.25\" data-avg-latency-ms=\"18.40\" data-last-used-unix-ms=\"1002\""
    ));
    assert!(!html.contains("id=\"tau-ops-tools-inventory-empty-state\""));
}

#[test]
fn regression_spec_3106_c04_non_tools_routes_keep_hidden_tools_panel_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-tools-panel\" data-route=\"/ops/tools-jobs\" aria-hidden=\"true\" data-panel-visible=\"false\" data-total-tools=\"0\""
    ));
    assert!(html.contains("id=\"tau-ops-tools-inventory-summary\" data-total-tools=\"0\""));
    assert!(html.contains("id=\"tau-ops-tools-inventory-table\" data-row-count=\"0\""));
}

#[test]
fn functional_spec_3112_c01_c02_tools_route_renders_tool_detail_metadata_and_policy_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::ToolsJobs,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            tools_inventory_rows: vec![TauOpsDashboardToolInventoryRow {
                tool_name: "bash".to_string(),
                category: "Code".to_string(),
                policy: "allowed".to_string(),
                usage_count: 6,
                error_rate: "0.00".to_string(),
                avg_latency_ms: "12.00".to_string(),
                last_used_unix_ms: 1003,
            }],
            tool_detail_selected_tool_name: "bash".to_string(),
            tool_detail_description: "Runs shell commands.".to_string(),
            tool_detail_parameter_schema: "{\"type\":\"object\",\"properties\":{}}".to_string(),
            tool_detail_policy_timeout_ms: 120_000,
            tool_detail_policy_max_output_chars: 32_768,
            tool_detail_policy_sandbox_mode: "default".to_string(),
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-tool-detail-panel\" data-selected-tool=\"bash\" data-detail-visible=\"true\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-tool-detail-metadata\" data-tool-name=\"bash\" data-parameter-schema=\"{&quot;type&quot;:&quot;object&quot;,&quot;properties&quot;:{}}\""
    ));
    assert!(html.contains("id=\"tau-ops-tool-detail-description\""));
    assert!(html.contains("Runs shell commands."));
    assert!(html.contains(
        "id=\"tau-ops-tool-detail-policy\" data-timeout-ms=\"120000\" data-max-output-chars=\"32768\" data-sandbox-mode=\"default\""
    ));
}

#[test]
fn functional_spec_3112_c03_tools_route_renders_usage_histogram_and_recent_invocation_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::ToolsJobs,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            tools_inventory_rows: vec![TauOpsDashboardToolInventoryRow {
                tool_name: "bash".to_string(),
                category: "Code".to_string(),
                policy: "allowed".to_string(),
                usage_count: 6,
                error_rate: "0.00".to_string(),
                avg_latency_ms: "12.00".to_string(),
                last_used_unix_ms: 1003,
            }],
            tool_detail_selected_tool_name: "bash".to_string(),
            tool_detail_usage_histogram_rows: vec![
                TauOpsDashboardToolUsageHistogramRow {
                    hour_offset: 0,
                    call_count: 6,
                },
                TauOpsDashboardToolUsageHistogramRow {
                    hour_offset: 1,
                    call_count: 4,
                },
            ],
            tool_detail_recent_invocation_rows: vec![TauOpsDashboardToolInvocationRow {
                timestamp_unix_ms: 1_700_000_123_000,
                args_summary: "{\"command\":\"ls\"}".to_string(),
                result_summary: "exit=0".to_string(),
                duration_ms: 18,
                status: "success".to_string(),
            }],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-tool-detail-usage-histogram\" data-bucket-count=\"2\""));
    assert!(html.contains(
        "id=\"tau-ops-tool-detail-usage-bucket-0\" data-hour-offset=\"0\" data-call-count=\"6\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-tool-detail-usage-bucket-1\" data-hour-offset=\"1\" data-call-count=\"4\""
    ));
    assert!(html.contains("id=\"tau-ops-tool-detail-invocations\" data-row-count=\"1\""));
    assert!(html.contains(
        "id=\"tau-ops-tool-detail-invocation-row-0\" data-timestamp-unix-ms=\"1700000123000\" data-args-summary=\"{&quot;command&quot;:&quot;ls&quot;}\" data-result-summary=\"exit=0\" data-duration-ms=\"18\" data-status=\"success\""
    ));
}

#[test]
fn regression_spec_3112_c04_non_tools_routes_keep_hidden_tool_detail_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-tool-detail-panel\" data-selected-tool=\"\" data-detail-visible=\"false\""
    ));
    assert!(html.contains("id=\"tau-ops-tool-detail-usage-histogram\" data-bucket-count=\"0\""));
    assert!(html.contains("id=\"tau-ops-tool-detail-invocations\" data-row-count=\"0\""));
}

#[test]
fn functional_spec_3116_c01_c02_tools_route_renders_jobs_summary_and_rows() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::ToolsJobs,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            jobs_rows: vec![
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
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html
        .contains("id=\"tau-ops-jobs-panel\" data-panel-visible=\"true\" data-total-jobs=\"3\""));
    assert!(html.contains(
        "id=\"tau-ops-jobs-summary\" data-running-count=\"1\" data-completed-count=\"1\" data-failed-count=\"1\""
    ));
    assert!(html.contains("id=\"tau-ops-jobs-table\" data-row-count=\"3\""));
    assert!(html.contains(
        "id=\"tau-ops-jobs-row-0\" data-job-id=\"job-001\" data-job-name=\"memory-index\" data-job-status=\"running\" data-started-unix-ms=\"1000\" data-finished-unix-ms=\"0\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-jobs-row-1\" data-job-id=\"job-002\" data-job-name=\"session-prune\" data-job-status=\"completed\" data-started-unix-ms=\"900\" data-finished-unix-ms=\"950\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-jobs-row-2\" data-job-id=\"job-003\" data-job-name=\"connector-retry\" data-job-status=\"failed\" data-started-unix-ms=\"800\" data-finished-unix-ms=\"820\""
    ));
    assert!(!html.contains("id=\"tau-ops-jobs-empty-state\""));
}

#[test]
fn regression_spec_3116_c04_non_tools_routes_keep_hidden_jobs_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html
        .contains("id=\"tau-ops-jobs-panel\" data-panel-visible=\"false\" data-total-jobs=\"0\""));
    assert!(html.contains(
        "id=\"tau-ops-jobs-summary\" data-running-count=\"0\" data-completed-count=\"0\" data-failed-count=\"0\""
    ));
    assert!(html.contains("id=\"tau-ops-jobs-table\" data-row-count=\"0\""));
}

#[test]
fn functional_spec_3120_c01_c02_tools_route_renders_job_detail_output_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::ToolsJobs,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            jobs_rows: vec![
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
            ],
            job_detail_selected_job_id: "job-002".to_string(),
            job_detail_status: "completed".to_string(),
            job_detail_duration_ms: 50,
            job_detail_stdout: "prune complete".to_string(),
            job_detail_stderr: String::new(),
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-job-detail-panel\" data-selected-job-id=\"job-002\" data-detail-visible=\"true\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-job-detail-metadata\" data-job-id=\"job-002\" data-job-status=\"completed\" data-duration-ms=\"50\""
    ));
    assert!(html.contains("id=\"tau-ops-job-detail-stdout\" data-output-bytes=\"14\""));
    assert!(html.contains("prune complete"));
    assert!(html.contains("id=\"tau-ops-job-detail-stderr\" data-output-bytes=\"0\""));
}

#[test]
fn regression_spec_3120_c04_non_tools_routes_keep_hidden_job_detail_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-job-detail-panel\" data-selected-job-id=\"\" data-detail-visible=\"false\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-job-detail-metadata\" data-job-id=\"\" data-job-status=\"\" data-duration-ms=\"0\""
    ));
    assert!(html.contains("id=\"tau-ops-job-detail-stdout\" data-output-bytes=\"0\""));
    assert!(html.contains("id=\"tau-ops-job-detail-stderr\" data-output-bytes=\"0\""));
}

#[test]
fn functional_spec_3124_c01_c02_tools_route_renders_job_cancel_action_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::ToolsJobs,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            jobs_rows: vec![
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
                    job_status: "cancelled".to_string(),
                    started_unix_ms: 800,
                    finished_unix_ms: 805,
                },
            ],
            job_detail_selected_job_id: "job-003".to_string(),
            job_detail_status: "cancelled".to_string(),
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-jobs-cancel-0\" data-action=\"cancel-job\" data-job-id=\"job-001\" data-cancel-enabled=\"true\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-jobs-cancel-1\" data-action=\"cancel-job\" data-job-id=\"job-002\" data-cancel-enabled=\"false\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-jobs-cancel-2\" data-action=\"cancel-job\" data-job-id=\"job-003\" data-cancel-enabled=\"false\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-job-cancel-panel\" data-requested-job-id=\"job-003\" data-cancel-status=\"cancelled\" data-panel-visible=\"true\" data-cancel-endpoint-template=\"/gateway/jobs/{job_id}/cancel\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-job-cancel-submit\" data-action=\"cancel-job\" data-job-id=\"job-003\" data-cancel-enabled=\"false\""
    ));
}

#[test]
fn regression_spec_3124_c04_non_tools_routes_keep_hidden_job_cancel_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-job-cancel-panel\" data-requested-job-id=\"\" data-cancel-status=\"idle\" data-panel-visible=\"false\" data-cancel-endpoint-template=\"/gateway/jobs/{job_id}/cancel\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-job-cancel-submit\" data-action=\"cancel-job\" data-job-id=\"\" data-cancel-enabled=\"false\""
    ));
}

#[test]
fn functional_spec_3128_c01_c02_channels_route_renders_panel_summary_and_rows() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Channels,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            connector_health_rows: vec![
                TauOpsDashboardConnectorHealthRow {
                    channel: "telegram".to_string(),
                    mode: "polling".to_string(),
                    liveness: "open".to_string(),
                    events_ingested: 6,
                    provider_failures: 2,
                },
                TauOpsDashboardConnectorHealthRow {
                    channel: "discord".to_string(),
                    mode: "gateway".to_string(),
                    liveness: "offline".to_string(),
                    events_ingested: 0,
                    provider_failures: 1,
                },
            ],
            ..TauOpsDashboardCommandCenterSnapshot::default()
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-channels-panel\" data-route=\"/ops/channels\" aria-hidden=\"false\" data-panel-visible=\"true\" data-channel-count=\"2\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-channels-summary\" data-online-count=\"1\" data-offline-count=\"1\" data-degraded-count=\"0\""
    ));
    assert!(html.contains("id=\"tau-ops-channels-table\" data-row-count=\"2\""));
    assert!(html.contains(
        "id=\"tau-ops-channels-row-0\" data-channel=\"telegram\" data-mode=\"polling\" data-liveness=\"open\" data-events-ingested=\"6\" data-provider-failures=\"2\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-channels-row-1\" data-channel=\"discord\" data-mode=\"gateway\" data-liveness=\"offline\" data-events-ingested=\"0\" data-provider-failures=\"1\""
    ));
}

#[test]
fn regression_spec_3128_c04_non_channels_routes_keep_hidden_channels_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-channels-panel\" data-route=\"/ops/channels\" aria-hidden=\"true\" data-panel-visible=\"false\" data-channel-count=\"1\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-channels-summary\" data-online-count=\"0\" data-offline-count=\"1\" data-degraded-count=\"0\""
    ));
    assert!(html.contains("id=\"tau-ops-channels-table\" data-row-count=\"1\""));
}

#[test]
fn functional_spec_3132_c01_c02_channels_route_renders_channel_action_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Channels,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            connector_health_rows: vec![
                TauOpsDashboardConnectorHealthRow {
                    channel: "telegram".to_string(),
                    mode: "polling".to_string(),
                    liveness: "open".to_string(),
                    events_ingested: 6,
                    provider_failures: 2,
                },
                TauOpsDashboardConnectorHealthRow {
                    channel: "discord".to_string(),
                    mode: "gateway".to_string(),
                    liveness: "offline".to_string(),
                    events_ingested: 0,
                    provider_failures: 1,
                },
            ],
            ..TauOpsDashboardCommandCenterSnapshot::default()
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-channels-login-0\" data-action=\"channel-login\" data-channel=\"telegram\" data-action-enabled=\"false\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-channels-logout-0\" data-action=\"channel-logout\" data-channel=\"telegram\" data-action-enabled=\"true\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-channels-probe-0\" data-action=\"channel-probe\" data-channel=\"telegram\" data-action-enabled=\"true\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-channels-login-1\" data-action=\"channel-login\" data-channel=\"discord\" data-action-enabled=\"true\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-channels-logout-1\" data-action=\"channel-logout\" data-channel=\"discord\" data-action-enabled=\"false\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-channels-probe-1\" data-action=\"channel-probe\" data-channel=\"discord\" data-action-enabled=\"true\""
    ));
}

#[test]
fn functional_spec_3797_c01_channels_route_exposes_operator_kpi_contract() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Channels,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            connector_health_rows: vec![
                TauOpsDashboardConnectorHealthRow {
                    channel: "telegram".to_string(),
                    mode: "polling".to_string(),
                    liveness: "open".to_string(),
                    events_ingested: 6,
                    provider_failures: 2,
                },
                TauOpsDashboardConnectorHealthRow {
                    channel: "discord".to_string(),
                    mode: "gateway".to_string(),
                    liveness: "offline".to_string(),
                    events_ingested: 0,
                    provider_failures: 1,
                },
            ],
            ..TauOpsDashboardCommandCenterSnapshot::default()
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    for marker in [
        "id=\"tau-ops-channels-panel\" data-route=\"/ops/channels\" aria-hidden=\"false\" data-panel-visible=\"true\" data-channel-count=\"2\" data-visual-contract=\"channel-operator-console\"",
        "id=\"tau-ops-channels-header\" data-layout=\"summary-with-kpis\"",
        "id=\"tau-ops-channels-kpi-grid\" data-card-count=\"3\"",
        "id=\"tau-ops-channels-online-card\" data-kpi=\"online\" data-count=\"1\"",
        "id=\"tau-ops-channels-offline-card\" data-kpi=\"offline\" data-count=\"1\"",
        "id=\"tau-ops-channels-degraded-card\" data-kpi=\"degraded\" data-count=\"0\"",
        "data-nav-item=\"channels\" href=\"/ops/channels\" data-harness-rail-label=\"Channels\" aria-current=\"page\">Channels</a>",
    ] {
        assert!(
            html.contains(marker),
            "missing channels operator KPI marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3797_c02_channels_route_groups_actions_as_controls() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Channels,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            connector_health_rows: vec![TauOpsDashboardConnectorHealthRow {
                channel: "telegram".to_string(),
                mode: "polling".to_string(),
                liveness: "open".to_string(),
                events_ingested: 6,
                provider_failures: 2,
            }],
            ..TauOpsDashboardCommandCenterSnapshot::default()
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    for marker in [
        "id=\"tau-ops-channels-table-wrap\"",
        "class=\"tau-ops-table-wrap\"",
        "data-horizontal-overflow=\"contained\"",
        "class=\"tau-ops-channel-actions\"",
        "data-action-count=\"3\"",
        "data-hit-target-contract=\"separate-action-buttons\"",
        "data-submit-contract=\"clicked-button-action\"",
        "data-column=\"actions\"",
        "grid-template-columns: repeat(3, minmax(78px, 1fr));",
        "id=\"tau-ops-channels-login-0\" data-action=\"channel-login\" data-channel=\"telegram\" data-action-enabled=\"false\" type=\"submit\" name=\"action\" value=\"login\" aria-disabled=\"true\"",
        "id=\"tau-ops-channels-logout-0\" data-action=\"channel-logout\" data-channel=\"telegram\" data-action-enabled=\"true\" type=\"submit\" name=\"action\" value=\"logout\" aria-disabled=\"false\"",
        "#tau-ops-channels-panel button[data-action^=\"channel-\"]",
        "#tau-ops-channels-panel button[data-action^=\"channel-\"][data-action-enabled=\"false\"]",
    ] {
        assert!(
            html.contains(marker),
            "missing channels action-control marker `{marker}`"
        );
    }
}

#[test]
fn functional_spec_3798_c02_channels_route_posts_lifecycle_action_forms() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Channels,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            connector_health_rows: vec![TauOpsDashboardConnectorHealthRow {
                channel: "telegram".to_string(),
                mode: "polling".to_string(),
                liveness: "open".to_string(),
                events_ingested: 6,
                provider_failures: 2,
            }],
            ..TauOpsDashboardCommandCenterSnapshot::default()
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    for marker in [
        "id=\"tau-ops-channels-action-status\" data-channel-action-status=\"idle\" data-channel-action=\"none\" data-channel-action-channel=\"none\" data-channel-action-reason=\"none\"",
        "id=\"tau-ops-channels-action-form-0\" action=\"/ops/channels/action\" method=\"post\" data-action=\"channel-lifecycle\" data-channel=\"telegram\" data-action-enabled=\"true\" data-submit-contract=\"clicked-button-action\"",
        "id=\"tau-ops-channels-action-0-channel\" type=\"hidden\" name=\"channel\" value=\"telegram\"",
        "id=\"tau-ops-channels-action-0-theme\" type=\"hidden\" name=\"theme\" value=\"light\"",
        "id=\"tau-ops-channels-action-0-sidebar\" type=\"hidden\" name=\"sidebar\" value=\"collapsed\"",
        "id=\"tau-ops-channels-logout-0\" data-action=\"channel-logout\" data-channel=\"telegram\" data-action-enabled=\"true\" type=\"submit\" name=\"action\" value=\"logout\" aria-disabled=\"false\"",
    ] {
        assert!(
            html.contains(marker),
            "missing channels lifecycle form marker `{marker}`"
        );
    }
}

#[test]
fn regression_spec_3132_c04_non_channels_routes_keep_hidden_channel_action_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-channels-login-0\" data-action=\"channel-login\" data-channel=\"none\" data-action-enabled=\"true\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-channels-logout-0\" data-action=\"channel-logout\" data-channel=\"none\" data-action-enabled=\"false\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-channels-probe-0\" data-action=\"channel-probe\" data-channel=\"none\" data-action-enabled=\"true\""
    ));
}

#[test]
fn functional_spec_3140_c01_config_route_renders_configuration_panel_contracts() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Config,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-config-panel\" data-route=\"/ops/config\" aria-hidden=\"false\" data-panel-visible=\"true\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-config-endpoints\" data-config-get-endpoint=\"/gateway/config\" data-config-patch-endpoint=\"/gateway/config\""
    ));
}

#[test]
fn functional_spec_3140_c02_training_route_renders_training_panel_contracts() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Training,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-training-panel\" data-route=\"/ops/training\" aria-hidden=\"false\" data-panel-visible=\"true\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-training-endpoints\" data-training-status-endpoint=\"/gateway/training/status\" data-training-rollouts-endpoint=\"/gateway/training/rollouts\" data-training-config-endpoint=\"/gateway/training/config\""
    ));
}

#[test]
fn functional_spec_3140_c03_safety_and_diagnostics_routes_render_panel_contracts() {
    let safety_html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Safety,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });
    assert!(safety_html.contains(
        "id=\"tau-ops-safety-panel\" data-route=\"/ops/safety\" aria-hidden=\"false\" data-panel-visible=\"true\""
    ));
    assert!(safety_html.contains(
        "id=\"tau-ops-safety-endpoints\" data-safety-policy-get-endpoint=\"/gateway/safety/policy\" data-safety-policy-put-endpoint=\"/gateway/safety/policy\" data-safety-rules-get-endpoint=\"/gateway/safety/rules\" data-safety-rules-put-endpoint=\"/gateway/safety/rules\" data-safety-test-endpoint=\"/gateway/safety/test\""
    ));

    let diagnostics_html =
        render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
            auth_mode: TauOpsDashboardAuthMode::Token,
            active_route: TauOpsDashboardRoute::Diagnostics,
            theme: TauOpsDashboardTheme::Light,
            sidebar_state: TauOpsDashboardSidebarState::Collapsed,
            command_center: TauOpsDashboardCommandCenterSnapshot::default(),
            chat: TauOpsDashboardChatSnapshot::default(),
            harness: TauOpsDashboardHarnessSnapshot::default(),
        });
    assert!(diagnostics_html.contains(
        "id=\"tau-ops-diagnostics-panel\" data-route=\"/ops/diagnostics\" aria-hidden=\"false\" data-panel-visible=\"true\""
    ));
    assert!(diagnostics_html.contains(
        "id=\"tau-ops-diagnostics-endpoints\" data-audit-summary-endpoint=\"/gateway/audit/summary\" data-audit-log-endpoint=\"/gateway/audit/log\" data-ui-telemetry-endpoint=\"/gateway/ui/telemetry\""
    ));
}

#[test]
fn regression_spec_3140_c05_non_target_routes_keep_hidden_route_panels() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-config-panel\" data-route=\"/ops/config\" aria-hidden=\"true\" data-panel-visible=\"false\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-training-panel\" data-route=\"/ops/training\" aria-hidden=\"true\" data-panel-visible=\"false\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-safety-panel\" data-route=\"/ops/safety\" aria-hidden=\"true\" data-panel-visible=\"false\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-diagnostics-panel\" data-route=\"/ops/diagnostics\" aria-hidden=\"true\" data-panel-visible=\"false\""
    ));
}

#[test]
fn functional_spec_3144_c01_config_route_renders_profile_control_contracts() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Config,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-config-profile-controls\" data-model-ref=\"gpt-4.1-mini\" data-fallback-model-count=\"2\" data-system-prompt-chars=\"0\" data-max-turns=\"64\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-config-profile-model-ref\" name=\"model_ref\" data-control=\"select\""
    ));
    assert!(html
        .contains("id=\"tau-ops-config-profile-fallback-models\" data-control=\"ordered-list\""));
    assert!(html.contains(
        "id=\"tau-ops-config-profile-system-prompt\" name=\"system_prompt\" data-control=\"textarea\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-config-profile-max-turns\" name=\"max_turns\" data-control=\"number\""
    ));
}

#[test]
fn functional_spec_3144_c02_config_route_renders_policy_control_contracts() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Config,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-config-policy-controls\" data-tool-policy-preset=\"balanced\" data-bash-profile=\"balanced\" data-os-sandbox-mode=\"auto\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-config-policy-limits\" data-bash-timeout-ms=\"120000\" data-max-command-length=\"8192\" data-max-tool-output-bytes=\"32768\" data-max-file-read-bytes=\"262144\" data-max-file-write-bytes=\"262144\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-config-policy-heartbeat\" data-runtime-heartbeat-enabled=\"true\" data-runtime-heartbeat-interval-ms=\"5000\" data-runtime-self-repair-enabled=\"true\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-config-policy-compaction\" data-warn-threshold=\"70\" data-aggressive-threshold=\"85\" data-emergency-threshold=\"95\""
    ));
}

#[test]
fn regression_spec_3144_c04_non_config_routes_keep_config_controls_hidden() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-config-panel\" data-route=\"/ops/config\" aria-hidden=\"true\" data-panel-visible=\"false\""
    ));
    assert!(html.contains("id=\"tau-ops-config-profile-controls\" data-model-ref=\"gpt-4.1-mini\""));
    assert!(
        html.contains("id=\"tau-ops-config-policy-controls\" data-tool-policy-preset=\"balanced\"")
    );
}

#[test]
fn functional_spec_3148_c01_training_route_renders_status_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Training,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            rollout_gate: "pass".to_string(),
            ..TauOpsDashboardCommandCenterSnapshot::default()
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-training-status\" data-status=\"running\" data-gate=\"pass\" data-store-path=\".tau/training/rl.sqlite\" data-update-interval-rollouts=\"8\" data-max-rollouts-per-update=\"64\" data-failure-streak=\"0/3\""
    ));
}

#[test]
fn functional_spec_3148_c02_training_route_renders_rollout_and_optimizer_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Training,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-training-rollouts\" data-rollout-count=\"3\" data-last-rollout-id=\"142\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-training-optimizer\" data-mean-total-loss=\"0.023\" data-approx-kl=\"0.0012\" data-early-stop=\"false\""
    ));
}

#[test]
fn functional_spec_3148_c03_training_route_renders_action_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Training,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-training-actions\" data-pause-endpoint=\"/gateway/training/config\" data-reset-endpoint=\"/gateway/training/config\" data-export-endpoint=\"/gateway/training/rollouts\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-training-action-pause\" data-action=\"pause-training\" data-action-enabled=\"true\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-training-action-reset\" data-action=\"reset-store\" data-action-enabled=\"true\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-training-action-export\" data-action=\"export-data\" data-action-enabled=\"true\""
    ));
}

#[test]
fn regression_spec_3148_c05_non_training_routes_keep_training_panel_hidden() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-training-panel\" data-route=\"/ops/training\" aria-hidden=\"true\" data-panel-visible=\"false\""
    ));
    assert!(
        html.contains("id=\"tau-ops-training-status\" data-status=\"running\" data-gate=\"hold\"")
    );
}

#[test]
fn functional_spec_2838_c01_c02_c03_sessions_route_renders_sessions_panel_list_rows_and_links() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Sessions,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "session-beta".to_string(),
            send_form_action: "/ops/chat/send".to_string(),
            send_form_method: "post".to_string(),
            session_options: vec![
                TauOpsDashboardChatSessionOptionRow {
                    session_key: "session-alpha".to_string(),
                    selected: false,
                    entry_count: 0,
                    usage_total_tokens: 0,
                    validation_is_valid: true,
                    updated_unix_ms: 0,
                },
                TauOpsDashboardChatSessionOptionRow {
                    session_key: "session-beta".to_string(),
                    selected: true,
                    entry_count: 0,
                    usage_total_tokens: 0,
                    validation_is_valid: true,
                    updated_unix_ms: 0,
                },
            ],
            message_rows: vec![],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-sessions-panel\" data-route=\"/ops/sessions\" aria-hidden=\"false\""
    ));
    assert!(html.contains("id=\"tau-ops-sessions-list\" data-session-count=\"2\""));
    assert!(html.contains(
        "id=\"tau-ops-sessions-row-0\" data-session-key=\"session-alpha\" data-selected=\"false\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-sessions-row-1\" data-session-key=\"session-beta\" data-selected=\"true\""
    ));
    assert!(html.contains(
        "href=\"/ops/chat?theme=light&amp;sidebar=collapsed&amp;session=session-alpha\""
    ));
    assert!(html
        .contains("href=\"/ops/chat?theme=light&amp;sidebar=collapsed&amp;session=session-beta\""));
}

#[test]
fn functional_spec_2838_c04_sessions_route_renders_empty_state_marker_when_no_sessions_discovered()
{
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Sessions,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "default".to_string(),
            send_form_action: "/ops/chat/send".to_string(),
            send_form_method: "post".to_string(),
            session_options: vec![],
            message_rows: vec![],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-sessions-panel\" data-route=\"/ops/sessions\" aria-hidden=\"false\""
    ));
    assert!(html.contains("id=\"tau-ops-sessions-list\" data-session-count=\"0\""));
    assert!(html.contains("id=\"tau-ops-sessions-empty-state\" data-empty-state=\"true\""));
    assert!(html.contains("No sessions discovered yet."));
}

#[test]
fn functional_spec_2893_c01_sessions_route_renders_row_metadata_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Sessions,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "session-beta".to_string(),
            send_form_action: "/ops/chat/send".to_string(),
            send_form_method: "post".to_string(),
            session_options: vec![
                TauOpsDashboardChatSessionOptionRow {
                    session_key: "session-alpha".to_string(),
                    selected: false,
                    entry_count: 0,
                    usage_total_tokens: 0,
                    validation_is_valid: true,
                    updated_unix_ms: 0,
                },
                TauOpsDashboardChatSessionOptionRow {
                    session_key: "session-beta".to_string(),
                    selected: true,
                    entry_count: 0,
                    usage_total_tokens: 0,
                    validation_is_valid: true,
                    updated_unix_ms: 0,
                },
            ],
            message_rows: vec![],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-sessions-row-0\" data-session-key=\"session-alpha\" data-selected=\"false\" data-entry-count=\"0\" data-total-tokens=\"0\" data-is-valid=\"true\" data-updated-unix-ms=\"0\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-sessions-row-1\" data-session-key=\"session-beta\" data-selected=\"true\" data-entry-count=\"0\" data-total-tokens=\"0\" data-is-valid=\"true\" data-updated-unix-ms=\"0\""
        ));
}

#[test]
fn functional_spec_2842_c01_c03_c05_sessions_route_renders_detail_panel_and_empty_timeline_contracts(
) {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Sessions,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "session-empty".to_string(),
            send_form_action: "/ops/chat/send".to_string(),
            send_form_method: "post".to_string(),
            session_options: vec![TauOpsDashboardChatSessionOptionRow {
                session_key: "session-empty".to_string(),
                selected: true,
                entry_count: 0,
                usage_total_tokens: 0,
                validation_is_valid: true,
                updated_unix_ms: 0,
            }],
            message_rows: vec![],
            session_detail_visible: true,
            session_detail_route: "/ops/sessions/session-empty".to_string(),
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-session-detail-panel\" data-route=\"/ops/sessions/session-empty\" data-session-key=\"session-empty\" aria-hidden=\"false\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-session-validation-report\" data-entries=\"0\" data-duplicates=\"0\" data-invalid-parent=\"0\" data-cycles=\"0\" data-is-valid=\"true\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-session-usage-summary\" data-input-tokens=\"0\" data-output-tokens=\"0\" data-total-tokens=\"0\" data-estimated-cost-usd=\"0.000000\""
        ));
    assert!(html.contains("id=\"tau-ops-session-message-timeline\" data-entry-count=\"0\""));
    assert!(html.contains("id=\"tau-ops-session-message-empty-state\" data-empty-state=\"true\""));
}

#[test]
fn functional_spec_2842_c02_c04_sessions_route_renders_detail_timeline_rows_and_usage_contracts() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Sessions,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "session-alpha".to_string(),
            send_form_action: "/ops/chat/send".to_string(),
            send_form_method: "post".to_string(),
            session_options: vec![TauOpsDashboardChatSessionOptionRow {
                session_key: "session-alpha".to_string(),
                selected: true,
                entry_count: 0,
                usage_total_tokens: 0,
                validation_is_valid: true,
                updated_unix_ms: 0,
            }],
            message_rows: vec![
                TauOpsDashboardChatMessageRow {
                    role: "user".to_string(),
                    content: "first detail message".to_string(),
                },
                TauOpsDashboardChatMessageRow {
                    role: "assistant".to_string(),
                    content: "second detail message".to_string(),
                },
            ],
            session_detail_visible: true,
            session_detail_route: "/ops/sessions/session-alpha".to_string(),
            session_detail_timeline_rows: vec![
                TauOpsDashboardSessionTimelineRow {
                    entry_id: 0,
                    role: "user".to_string(),
                    content: "first detail message".to_string(),
                },
                TauOpsDashboardSessionTimelineRow {
                    entry_id: 1,
                    role: "assistant".to_string(),
                    content: "second detail message".to_string(),
                },
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-session-message-timeline\" data-entry-count=\"2\""));
    assert!(html.contains(
        "id=\"tau-ops-session-message-row-0\" data-entry-id=\"0\" data-message-role=\"user\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-session-message-row-1\" data-entry-id=\"1\" data-message-role=\"assistant\""
    ));
    assert!(html.contains("first detail message"));
    assert!(html.contains("second detail message"));
    assert!(html.contains(
            "id=\"tau-ops-session-usage-summary\" data-input-tokens=\"0\" data-output-tokens=\"0\" data-total-tokens=\"0\" data-estimated-cost-usd=\"0.000000\""
        ));
}

#[test]
fn functional_spec_2897_c01_c02_session_detail_timeline_exposes_complete_content_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Sessions,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "session-coverage".to_string(),
            send_form_action: "/ops/chat/send".to_string(),
            send_form_method: "post".to_string(),
            session_options: vec![TauOpsDashboardChatSessionOptionRow {
                session_key: "session-coverage".to_string(),
                selected: true,
                entry_count: 0,
                usage_total_tokens: 0,
                validation_is_valid: true,
                updated_unix_ms: 0,
            }],
            message_rows: vec![],
            session_detail_visible: true,
            session_detail_route: "/ops/sessions/session-coverage".to_string(),
            session_detail_timeline_rows: vec![
                TauOpsDashboardSessionTimelineRow {
                    entry_id: 0,
                    role: "system".to_string(),
                    content: "system coverage message".to_string(),
                },
                TauOpsDashboardSessionTimelineRow {
                    entry_id: 1,
                    role: "user".to_string(),
                    content: "user coverage message".to_string(),
                },
                TauOpsDashboardSessionTimelineRow {
                    entry_id: 2,
                    role: "assistant".to_string(),
                    content: "assistant coverage message".to_string(),
                },
                TauOpsDashboardSessionTimelineRow {
                    entry_id: 3,
                    role: "tool".to_string(),
                    content: "tool coverage output".to_string(),
                },
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-session-message-timeline\" data-entry-count=\"4\""));
    assert!(html.contains(
            "id=\"tau-ops-session-message-row-0\" data-entry-id=\"0\" data-message-role=\"system\" data-message-content=\"system coverage message\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-session-message-row-1\" data-entry-id=\"1\" data-message-role=\"user\" data-message-content=\"user coverage message\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-session-message-row-2\" data-entry-id=\"2\" data-message-role=\"assistant\" data-message-content=\"assistant coverage message\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-session-message-row-3\" data-entry-id=\"3\" data-message-role=\"tool\" data-message-content=\"tool coverage output\""
        ));
}

#[test]
fn functional_spec_2846_c01_c04_c05_sessions_route_renders_graph_panel_summary_and_empty_state() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Sessions,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "session-empty".to_string(),
            session_detail_visible: true,
            session_detail_route: "/ops/sessions/session-empty".to_string(),
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-session-graph-panel\" data-route=\"/ops/sessions/session-empty\" data-session-key=\"session-empty\" aria-hidden=\"false\""
        ));
    assert!(html.contains("id=\"tau-ops-session-graph-nodes\" data-node-count=\"0\""));
    assert!(html.contains("id=\"tau-ops-session-graph-edges\" data-edge-count=\"0\""));
    assert!(html.contains("id=\"tau-ops-session-graph-empty-state\" data-empty-state=\"true\""));
}

#[test]
fn functional_spec_2846_c02_c03_sessions_route_renders_graph_node_and_edge_rows() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Sessions,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "session-graph".to_string(),
            session_detail_visible: true,
            session_detail_route: "/ops/sessions/session-graph".to_string(),
            session_graph_node_rows: vec![
                TauOpsDashboardSessionGraphNodeRow {
                    entry_id: 1,
                    role: "system".to_string(),
                },
                TauOpsDashboardSessionGraphNodeRow {
                    entry_id: 2,
                    role: "user".to_string(),
                },
                TauOpsDashboardSessionGraphNodeRow {
                    entry_id: 3,
                    role: "assistant".to_string(),
                },
            ],
            session_graph_edge_rows: vec![
                TauOpsDashboardSessionGraphEdgeRow {
                    source_entry_id: 1,
                    target_entry_id: 2,
                },
                TauOpsDashboardSessionGraphEdgeRow {
                    source_entry_id: 2,
                    target_entry_id: 3,
                },
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-session-graph-nodes\" data-node-count=\"3\""));
    assert!(html.contains("id=\"tau-ops-session-graph-edges\" data-edge-count=\"2\""));
    assert!(html.contains(
        "id=\"tau-ops-session-graph-node-0\" data-entry-id=\"1\" data-message-role=\"system\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-session-graph-node-1\" data-entry-id=\"2\" data-message-role=\"user\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-session-graph-node-2\" data-entry-id=\"3\" data-message-role=\"assistant\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-session-graph-edge-0\" data-source-entry-id=\"1\" data-target-entry-id=\"2\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-session-graph-edge-1\" data-source-entry-id=\"2\" data-target-entry-id=\"3\""
    ));
}

#[test]
fn functional_spec_2885_c01_sessions_route_renders_timeline_row_branch_form_contracts() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Sessions,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "session-branch-source".to_string(),
            session_detail_visible: true,
            session_detail_route: "/ops/sessions/session-branch-source".to_string(),
            session_detail_timeline_rows: vec![
                TauOpsDashboardSessionTimelineRow {
                    entry_id: 7,
                    role: "user".to_string(),
                    content: "branch anchor message".to_string(),
                },
                TauOpsDashboardSessionTimelineRow {
                    entry_id: 8,
                    role: "assistant".to_string(),
                    content: "downstream reply".to_string(),
                },
            ],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-session-branch-form-0\" action=\"/ops/sessions/branch\" method=\"post\" data-source-session-key=\"session-branch-source\" data-entry-id=\"7\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-session-branch-source-session-key-0\" type=\"hidden\" name=\"source_session_key\" value=\"session-branch-source\""
        ));
    assert!(html.contains(
        "id=\"tau-ops-session-branch-entry-id-0\" type=\"hidden\" name=\"entry_id\" value=\"7\""
    ));
    assert!(
            html.contains("id=\"tau-ops-session-branch-target-session-key-0\" type=\"text\" name=\"target_session_key\" value=\"\"")
        );
    assert!(html.contains(
        "id=\"tau-ops-session-branch-theme-0\" type=\"hidden\" name=\"theme\" value=\"dark\""
    ));
    assert!(html.contains(
            "id=\"tau-ops-session-branch-sidebar-0\" type=\"hidden\" name=\"sidebar\" value=\"expanded\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-session-branch-submit-0\" type=\"submit\" data-confirmation-required=\"true\""
        ));
}

#[test]
fn functional_spec_2889_c01_sessions_route_renders_reset_confirmation_form_contracts() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Sessions,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "session-reset-target".to_string(),
            session_detail_visible: true,
            session_detail_route: "/ops/sessions/session-reset-target".to_string(),
            session_detail_timeline_rows: vec![TauOpsDashboardSessionTimelineRow {
                entry_id: 3,
                role: "assistant".to_string(),
                content: "reset candidate row".to_string(),
            }],
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-session-reset-form\" action=\"/ops/sessions/session-reset-target\" method=\"post\" data-session-key=\"session-reset-target\" data-confirmation-required=\"true\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-session-reset-session-key\" type=\"hidden\" name=\"session_key\" value=\"session-reset-target\""
        ));
    assert!(html.contains(
        "id=\"tau-ops-session-reset-theme\" type=\"hidden\" name=\"theme\" value=\"light\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-session-reset-sidebar\" type=\"hidden\" name=\"sidebar\" value=\"collapsed\""
    ));
    assert!(html.contains(
            "id=\"tau-ops-session-reset-confirm\" type=\"hidden\" name=\"confirm_reset\" value=\"true\""
        ));
    assert!(html.contains(
        "id=\"tau-ops-session-reset-submit\" type=\"submit\" data-confirmation-required=\"true\""
    ));
}

#[test]
fn regression_spec_2842_session_detail_panel_stays_hidden_on_non_sessions_route() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            active_session_key: "session-alpha".to_string(),
            session_detail_visible: true,
            session_detail_route: "/ops/sessions/session-alpha".to_string(),
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-session-detail-panel\" data-route=\"/ops/sessions/session-alpha\" data-session-key=\"session-alpha\" aria-hidden=\"true\""
        ));
}

#[test]
fn functional_spec_2806_c01_c02_c03_command_center_snapshot_markers_render() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            health_state: "healthy".to_string(),
            health_reason: "no recent transport failures observed".to_string(),
            rollout_gate: "hold".to_string(),
            control_mode: "paused".to_string(),
            control_paused: true,
            action_pause_enabled: false,
            action_resume_enabled: true,
            action_refresh_enabled: true,
            last_action_request_id: "dashboard-action-90210".to_string(),
            last_action_name: "pause".to_string(),
            last_action_actor: "ops-user".to_string(),
            last_action_reason: "maintenance".to_string(),
            last_action_timestamp_unix_ms: 90210,
            timeline_range: "1h".to_string(),
            timeline_point_count: 9,
            timeline_last_timestamp_unix_ms: 811,
            queue_depth: 3,
            failure_streak: 1,
            processed_case_count: 8,
            alert_count: 2,
            widget_count: 6,
            timeline_cycle_count: 9,
            timeline_invalid_cycle_count: 1,
            primary_alert_code: "dashboard_queue_backlog".to_string(),
            primary_alert_severity: "warning".to_string(),
            primary_alert_message: "runtime backlog detected (queue_depth=3)".to_string(),
            alert_feed_rows: vec![],
            connector_health_rows: vec![],
            channel_action_status: "idle".to_string(),
            channel_action: "none".to_string(),
            channel_action_channel: "none".to_string(),
            channel_action_reason: "none".to_string(),
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("data-health-state=\"healthy\""));
    assert!(html.contains("data-health-reason=\"no recent transport failures observed\""));
    assert_eq!(html.matches("data-kpi-card=").count(), 6);
    assert!(html.contains("data-kpi-card=\"queue-depth\" data-kpi-value=\"3\""));
    assert!(html.contains("data-kpi-card=\"failure-streak\" data-kpi-value=\"1\""));
    assert!(html.contains("data-kpi-card=\"processed-cases\" data-kpi-value=\"8\""));
    assert!(html.contains("data-kpi-card=\"alert-count\" data-kpi-value=\"2\""));
    assert!(html.contains("data-kpi-card=\"widget-count\" data-kpi-value=\"6\""));
    assert!(html.contains("data-kpi-card=\"timeline-cycles\" data-kpi-value=\"9\""));
    assert!(html.contains("data-alert-count=\"2\""));
    assert!(html.contains("data-primary-alert-code=\"dashboard_queue_backlog\""));
    assert!(html.contains("data-primary-alert-severity=\"warning\""));
    assert!(html.contains("runtime backlog detected (queue_depth=3)"));
    assert!(html.contains("data-timeline-cycle-count=\"9\""));
    assert!(html.contains("data-timeline-invalid-cycle-count=\"1\""));
    assert!(html.contains("data-control-mode=\"paused\""));
    assert!(html.contains("data-rollout-gate=\"hold\""));
    assert!(html.contains("data-control-paused=\"true\""));
    assert!(html.contains("id=\"tau-ops-control-action-pause\" data-action-enabled=\"false\""));
    assert!(html.contains("id=\"tau-ops-control-action-resume\" data-action-enabled=\"true\""));
    assert!(html.contains("id=\"tau-ops-control-action-refresh\" data-action-enabled=\"true\""));
    assert!(html.contains("data-last-action-request-id=\"dashboard-action-90210\""));
    assert!(html.contains("data-last-action-name=\"pause\""));
    assert!(html.contains("data-last-action-actor=\"ops-user\""));
    assert!(html.contains("data-last-action-reason=\"maintenance\""));
    assert!(html.contains("data-last-action-timestamp=\"90210\""));
    assert!(html.contains("id=\"tau-ops-queue-timeline-chart\""));
    assert!(html.contains("data-component=\"TimelineChart\""));
    assert!(html.contains("data-timeline-range=\"1h\""));
    assert!(html.contains("data-timeline-point-count=\"9\""));
    assert!(html.contains("data-timeline-last-timestamp=\"811\""));
}

#[test]
fn functional_spec_2854_c01_command_center_panel_visible_on_ops_route() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(
        html.contains("id=\"tau-ops-command-center\" data-route=\"/ops\" aria-hidden=\"false\"")
    );
}

#[test]
fn functional_spec_2854_c02_c03_command_center_panel_hidden_on_non_ops_routes() {
    let chat_html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });
    assert!(chat_html
        .contains("id=\"tau-ops-command-center\" data-route=\"/ops\" aria-hidden=\"true\""));

    let sessions_html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Sessions,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });
    assert!(sessions_html
        .contains("id=\"tau-ops-command-center\" data-route=\"/ops\" aria-hidden=\"true\""));
}

#[test]
fn functional_spec_2858_c01_c03_chat_route_panel_visibility_state_contracts() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Chat,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-chat-panel\" data-route=\"/ops/chat\" aria-hidden=\"false\" data-active-session-key=\"default\" data-panel-visible=\"true\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-sessions-panel\" data-route=\"/ops/sessions\" aria-hidden=\"true\" data-panel-visible=\"false\""
        ));
}

#[test]
fn functional_spec_2858_c02_c04_sessions_route_panel_visibility_state_contracts() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Sessions,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-chat-panel\" data-route=\"/ops/chat\" aria-hidden=\"true\" data-active-session-key=\"default\" data-panel-visible=\"false\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-sessions-panel\" data-route=\"/ops/sessions\" aria-hidden=\"false\" data-panel-visible=\"true\""
        ));
}

#[test]
fn regression_spec_2858_c05_ops_route_panels_remain_hidden_with_visibility_state_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
            "id=\"tau-ops-chat-panel\" data-route=\"/ops/chat\" aria-hidden=\"true\" data-active-session-key=\"default\" data-panel-visible=\"false\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-sessions-panel\" data-route=\"/ops/sessions\" aria-hidden=\"true\" data-panel-visible=\"false\""
        ));
}

#[test]
fn functional_spec_2810_c01_c02_c03_command_center_control_markers_render() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            health_state: "healthy".to_string(),
            health_reason: "operator pause action is active".to_string(),
            rollout_gate: "hold".to_string(),
            control_mode: "paused".to_string(),
            control_paused: true,
            action_pause_enabled: false,
            action_resume_enabled: true,
            action_refresh_enabled: true,
            last_action_request_id: "dashboard-action-90210".to_string(),
            last_action_name: "pause".to_string(),
            last_action_actor: "ops-user".to_string(),
            last_action_reason: "maintenance".to_string(),
            last_action_timestamp_unix_ms: 90210,
            timeline_range: "1h".to_string(),
            timeline_point_count: 2,
            timeline_last_timestamp_unix_ms: 811,
            queue_depth: 1,
            failure_streak: 0,
            processed_case_count: 2,
            alert_count: 2,
            widget_count: 2,
            timeline_cycle_count: 2,
            timeline_invalid_cycle_count: 1,
            primary_alert_code: "dashboard_queue_backlog".to_string(),
            primary_alert_severity: "warning".to_string(),
            primary_alert_message: "runtime backlog detected (queue_depth=1)".to_string(),
            alert_feed_rows: vec![],
            connector_health_rows: vec![],
            channel_action_status: "idle".to_string(),
            channel_action: "none".to_string(),
            channel_action_channel: "none".to_string(),
            channel_action_reason: "none".to_string(),
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-control-panel\""));
    assert!(html.contains("data-control-mode=\"paused\""));
    assert!(html.contains("data-rollout-gate=\"hold\""));
    assert!(html.contains("data-control-paused=\"true\""));
    assert!(html.contains("id=\"tau-ops-control-action-pause\" data-action-enabled=\"false\""));
    assert!(html.contains("id=\"tau-ops-control-action-resume\" data-action-enabled=\"true\""));
    assert!(html.contains("id=\"tau-ops-control-action-refresh\" data-action-enabled=\"true\""));
    assert!(html.contains("data-last-action-request-id=\"dashboard-action-90210\""));
    assert!(html.contains("data-last-action-name=\"pause\""));
    assert!(html.contains("data-last-action-actor=\"ops-user\""));
    assert!(html.contains("data-last-action-reason=\"maintenance\""));
    assert!(html.contains("data-last-action-timestamp=\"90210\""));
}

#[test]
fn functional_spec_2826_c01_c02_control_actions_expose_confirmation_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            health_state: "healthy".to_string(),
            health_reason: "operator controls are ready".to_string(),
            rollout_gate: "pass".to_string(),
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
            timeline_point_count: 1,
            timeline_last_timestamp_unix_ms: 811,
            queue_depth: 0,
            failure_streak: 0,
            processed_case_count: 1,
            alert_count: 1,
            widget_count: 1,
            timeline_cycle_count: 1,
            timeline_invalid_cycle_count: 0,
            primary_alert_code: "dashboard_healthy".to_string(),
            primary_alert_severity: "info".to_string(),
            primary_alert_message: "dashboard runtime health is nominal".to_string(),
            alert_feed_rows: vec![],
            connector_health_rows: vec![],
            channel_action_status: "idle".to_string(),
            channel_action: "none".to_string(),
            channel_action_channel: "none".to_string(),
            channel_action_reason: "none".to_string(),
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-control-action-pause\""));
    assert!(html.contains(
        "id=\"tau-ops-control-actions\" data-action-count=\"3\" data-action-endpoint=\"/ops/control-action\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-control-action-form-pause\" action=\"/ops/control-action\" method=\"post\" data-action=\"pause\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-control-action-pause-value\" type=\"hidden\" name=\"action\" value=\"pause\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-control-action-form-resume\" action=\"/ops/control-action\" method=\"post\" data-action=\"resume\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-control-action-form-refresh\" action=\"/ops/control-action\" method=\"post\" data-action=\"refresh\""
    ));
    assert!(html.contains(
            "id=\"tau-ops-control-action-pause\" data-action-enabled=\"true\" data-action=\"pause\" data-confirm-required=\"true\" data-confirm-title=\"Confirm pause action\" data-confirm-body=\"Pause command-center processing until resumed.\" data-confirm-verb=\"pause\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-control-action-resume\" data-action-enabled=\"false\" data-action=\"resume\" data-confirm-required=\"true\" data-confirm-title=\"Confirm resume action\" data-confirm-body=\"Resume command-center processing.\" data-confirm-verb=\"resume\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-control-action-refresh\" data-action-enabled=\"true\" data-action=\"refresh\" data-confirm-required=\"true\" data-confirm-title=\"Confirm refresh action\" data-confirm-body=\"Refresh command-center state from latest runtime artifacts.\" data-confirm-verb=\"refresh\""
        ));
}

#[test]
fn functional_spec_3466_c04_control_action_status_panel_renders_marker_contracts() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot {
            control_action_status: "failed".to_string(),
            control_action: "none".to_string(),
            control_action_reason: "invalid_dashboard_action".to_string(),
            ..TauOpsDashboardChatSnapshot::default()
        },
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-control-action-status\" data-control-action-status=\"failed\" data-control-action=\"none\" data-control-action-reason=\"invalid_dashboard_action\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-control-action-status-message\">Failed to apply none action (invalid_dashboard_action)."
    ));
}

#[test]
fn regression_spec_3466_c05_control_action_status_panel_defaults_to_idle_contract_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains(
        "id=\"tau-ops-control-action-status\" data-control-action-status=\"idle\" data-control-action=\"none\" data-control-action-reason=\"none\""
    ));
    assert!(html
        .contains("id=\"tau-ops-control-action-status-message\">No control action submitted yet."));
}

#[test]
fn functional_spec_3478_c01_last_action_section_renders_readable_detail_rows() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            last_action_request_id: "dashboard-action-90210".to_string(),
            last_action_name: "pause".to_string(),
            last_action_actor: "ops-user".to_string(),
            last_action_timestamp_unix_ms: 90210,
            ..TauOpsDashboardCommandCenterSnapshot::default()
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-control-last-action\""));
    assert!(
        html.contains("id=\"tau-ops-last-action-request-id\">request.id: dashboard-action-90210")
    );
    assert!(html.contains("id=\"tau-ops-last-action-name\">action: pause"));
    assert!(html.contains("id=\"tau-ops-last-action-actor\">actor: ops-user"));
    assert!(html.contains("id=\"tau-ops-last-action-timestamp\">timestamp: 90210"));
}

#[test]
fn regression_spec_3478_c02_last_action_section_defaults_to_fallback_rows() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-last-action-request-id\">request.id: none"));
    assert!(html.contains("id=\"tau-ops-last-action-name\">action: none"));
    assert!(html.contains("id=\"tau-ops-last-action-actor\">actor: none"));
    assert!(html.contains("id=\"tau-ops-last-action-timestamp\">timestamp: 0"));
}

#[test]
fn functional_spec_3482_c01_last_action_section_exposes_reason_row_and_marker_contracts() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            last_action_request_id: "dashboard-action-90210".to_string(),
            last_action_name: "pause".to_string(),
            last_action_actor: "ops-user".to_string(),
            last_action_reason: "maintenance".to_string(),
            last_action_timestamp_unix_ms: 90210,
            ..TauOpsDashboardCommandCenterSnapshot::default()
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-control-last-action\""));
    assert!(html.contains("data-last-action-reason=\"maintenance\""));
    assert!(html.contains("id=\"tau-ops-last-action-reason\">reason: maintenance"));
}

#[test]
fn regression_spec_3482_c02_last_action_reason_row_defaults_to_none() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot::default(),
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("data-last-action-reason=\"none\""));
    assert!(html.contains("id=\"tau-ops-last-action-reason\">reason: none"));
}

#[test]
fn functional_spec_2814_c01_c02_c03_timeline_chart_and_range_markers_render() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            health_state: "healthy".to_string(),
            health_reason: "no recent transport failures observed".to_string(),
            rollout_gate: "pass".to_string(),
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
            timeline_range: "6h".to_string(),
            timeline_point_count: 2,
            timeline_last_timestamp_unix_ms: 811,
            queue_depth: 1,
            failure_streak: 0,
            processed_case_count: 2,
            alert_count: 2,
            widget_count: 2,
            timeline_cycle_count: 2,
            timeline_invalid_cycle_count: 1,
            primary_alert_code: "dashboard_queue_backlog".to_string(),
            primary_alert_severity: "warning".to_string(),
            primary_alert_message: "runtime backlog detected (queue_depth=1)".to_string(),
            alert_feed_rows: vec![],
            connector_health_rows: vec![],
            channel_action_status: "idle".to_string(),
            channel_action: "none".to_string(),
            channel_action_channel: "none".to_string(),
            channel_action_reason: "none".to_string(),
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-queue-timeline-chart\""));
    assert!(html.contains("data-component=\"TimelineChart\""));
    assert!(html.contains("data-timeline-range=\"6h\""));
    assert!(html.contains("data-timeline-point-count=\"2\""));
    assert!(html.contains("data-timeline-last-timestamp=\"811\""));
    assert!(html.contains(
        "id=\"tau-ops-timeline-range-1h\" data-range-option=\"1h\" data-range-selected=\"false\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-timeline-range-6h\" data-range-option=\"6h\" data-range-selected=\"true\""
    ));
    assert!(html.contains(
        "id=\"tau-ops-timeline-range-24h\" data-range-option=\"24h\" data-range-selected=\"false\""
    ));
    assert!(html.contains("href=\"/ops?theme=light&amp;sidebar=collapsed&amp;range=1h\""));
    assert!(html.contains("href=\"/ops?theme=light&amp;sidebar=collapsed&amp;range=6h\""));
    assert!(html.contains("href=\"/ops?theme=light&amp;sidebar=collapsed&amp;range=24h\""));
}

#[test]
fn functional_spec_2850_c01_c02_c04_recent_cycles_table_renders_panel_and_summary_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Light,
        sidebar_state: TauOpsDashboardSidebarState::Collapsed,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            timeline_range: "6h".to_string(),
            timeline_point_count: 2,
            timeline_last_timestamp_unix_ms: 811,
            timeline_cycle_count: 2,
            timeline_invalid_cycle_count: 1,
            ..TauOpsDashboardCommandCenterSnapshot::default()
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(
        html.contains("id=\"tau-ops-data-table\" data-route=\"/ops\" data-timeline-range=\"6h\"")
    );
    assert!(html.contains(
            "id=\"tau-ops-timeline-summary-row\" data-row-kind=\"summary\" data-last-timestamp=\"811\" data-point-count=\"2\" data-cycle-count=\"2\" data-invalid-cycle-count=\"1\""
        ));
    assert!(!html.contains("id=\"tau-ops-timeline-empty-row\""));
}

#[test]
fn functional_spec_2850_c03_recent_cycles_table_renders_empty_state_marker() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            timeline_range: "1h".to_string(),
            timeline_point_count: 0,
            timeline_last_timestamp_unix_ms: 0,
            timeline_cycle_count: 0,
            timeline_invalid_cycle_count: 0,
            ..TauOpsDashboardCommandCenterSnapshot::default()
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(
        html.contains("id=\"tau-ops-data-table\" data-route=\"/ops\" data-timeline-range=\"1h\"")
    );
    assert!(html.contains(
            "id=\"tau-ops-timeline-summary-row\" data-row-kind=\"summary\" data-last-timestamp=\"0\" data-point-count=\"0\" data-cycle-count=\"0\" data-invalid-cycle-count=\"0\""
        ));
    assert!(html.contains("id=\"tau-ops-timeline-empty-row\" data-empty-state=\"true\""));
}

#[test]
fn functional_spec_2818_c01_c02_alert_feed_row_markers_render_for_snapshot_alerts() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            health_state: "degraded".to_string(),
            health_reason: "runtime backlog detected".to_string(),
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
            timeline_point_count: 1,
            timeline_last_timestamp_unix_ms: 900,
            queue_depth: 1,
            failure_streak: 0,
            processed_case_count: 1,
            alert_count: 2,
            widget_count: 1,
            timeline_cycle_count: 1,
            timeline_invalid_cycle_count: 0,
            primary_alert_code: "dashboard_queue_backlog".to_string(),
            primary_alert_severity: "warning".to_string(),
            primary_alert_message: "runtime backlog detected (queue_depth=1)".to_string(),
            alert_feed_rows: vec![
                TauOpsDashboardAlertFeedRow {
                    code: "dashboard_queue_backlog".to_string(),
                    severity: "warning".to_string(),
                    message: "runtime backlog detected (queue_depth=1)".to_string(),
                },
                TauOpsDashboardAlertFeedRow {
                    code: "dashboard_cycle_log_invalid_lines".to_string(),
                    severity: "warning".to_string(),
                    message: "runtime events log contains 1 malformed line(s)".to_string(),
                },
            ],
            connector_health_rows: vec![],
            channel_action_status: "idle".to_string(),
            channel_action: "none".to_string(),
            channel_action_channel: "none".to_string(),
            channel_action_reason: "none".to_string(),
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-alert-feed-list\""));
    assert!(html.contains(
            "id=\"tau-ops-alert-row-0\" data-alert-code=\"dashboard_queue_backlog\" data-alert-severity=\"warning\""
        ));
    assert!(html.contains(
            "id=\"tau-ops-alert-row-1\" data-alert-code=\"dashboard_cycle_log_invalid_lines\" data-alert-severity=\"warning\""
        ));
    assert!(html.contains("runtime backlog detected (queue_depth=1)"));
}

#[test]
fn functional_spec_2818_c03_alert_feed_row_markers_render_nominal_fallback_alert() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            health_state: "healthy".to_string(),
            health_reason: "dashboard runtime health is nominal".to_string(),
            rollout_gate: "pass".to_string(),
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
            timeline_point_count: 1,
            timeline_last_timestamp_unix_ms: 900,
            queue_depth: 0,
            failure_streak: 0,
            processed_case_count: 1,
            alert_count: 1,
            widget_count: 1,
            timeline_cycle_count: 1,
            timeline_invalid_cycle_count: 0,
            primary_alert_code: "dashboard_healthy".to_string(),
            primary_alert_severity: "info".to_string(),
            primary_alert_message: "dashboard runtime health is nominal".to_string(),
            alert_feed_rows: vec![TauOpsDashboardAlertFeedRow {
                code: "dashboard_healthy".to_string(),
                severity: "info".to_string(),
                message: "dashboard runtime health is nominal".to_string(),
            }],
            connector_health_rows: vec![],
            channel_action_status: "idle".to_string(),
            channel_action: "none".to_string(),
            channel_action_channel: "none".to_string(),
            channel_action_reason: "none".to_string(),
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-alert-feed-list\""));
    assert!(html.contains(
            "id=\"tau-ops-alert-row-0\" data-alert-code=\"dashboard_healthy\" data-alert-severity=\"info\""
        ));
    assert!(html.contains("dashboard runtime health is nominal"));
}

#[test]
fn functional_spec_2822_c03_connector_health_table_renders_fallback_row_markers() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            health_state: "healthy".to_string(),
            health_reason: "dashboard runtime health is nominal".to_string(),
            rollout_gate: "pass".to_string(),
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
            timeline_point_count: 1,
            timeline_last_timestamp_unix_ms: 900,
            queue_depth: 0,
            failure_streak: 0,
            processed_case_count: 1,
            alert_count: 1,
            widget_count: 1,
            timeline_cycle_count: 1,
            timeline_invalid_cycle_count: 0,
            primary_alert_code: "dashboard_healthy".to_string(),
            primary_alert_severity: "info".to_string(),
            primary_alert_message: "dashboard runtime health is nominal".to_string(),
            alert_feed_rows: vec![TauOpsDashboardAlertFeedRow {
                code: "dashboard_healthy".to_string(),
                severity: "info".to_string(),
                message: "dashboard runtime health is nominal".to_string(),
            }],
            connector_health_rows: vec![],
            channel_action_status: "idle".to_string(),
            channel_action: "none".to_string(),
            channel_action_channel: "none".to_string(),
            channel_action_reason: "none".to_string(),
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-connector-health-table\""));
    assert!(html.contains("id=\"tau-ops-connector-table-body\""));
    assert!(html.contains(
            "id=\"tau-ops-connector-row-0\" data-channel=\"none\" data-mode=\"unknown\" data-liveness=\"unknown\" data-events-ingested=\"0\" data-provider-failures=\"0\""
        ));
}

#[test]
fn functional_spec_2822_c01_c02_connector_health_table_rows_render_for_snapshot_connectors() {
    let html = render_tau_ops_dashboard_shell_with_context(TauOpsDashboardShellContext {
        auth_mode: TauOpsDashboardAuthMode::Token,
        active_route: TauOpsDashboardRoute::Ops,
        theme: TauOpsDashboardTheme::Dark,
        sidebar_state: TauOpsDashboardSidebarState::Expanded,
        command_center: TauOpsDashboardCommandCenterSnapshot {
            health_state: "degraded".to_string(),
            health_reason: "connector retry in progress".to_string(),
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
            timeline_point_count: 1,
            timeline_last_timestamp_unix_ms: 900,
            queue_depth: 0,
            failure_streak: 0,
            processed_case_count: 1,
            alert_count: 1,
            widget_count: 1,
            timeline_cycle_count: 1,
            timeline_invalid_cycle_count: 0,
            primary_alert_code: "dashboard_healthy".to_string(),
            primary_alert_severity: "info".to_string(),
            primary_alert_message: "dashboard runtime health is nominal".to_string(),
            alert_feed_rows: vec![TauOpsDashboardAlertFeedRow {
                code: "dashboard_healthy".to_string(),
                severity: "info".to_string(),
                message: "dashboard runtime health is nominal".to_string(),
            }],
            connector_health_rows: vec![TauOpsDashboardConnectorHealthRow {
                channel: "telegram".to_string(),
                mode: "polling".to_string(),
                liveness: "open".to_string(),
                events_ingested: 6,
                provider_failures: 2,
            }],
            channel_action_status: "idle".to_string(),
            channel_action: "none".to_string(),
            channel_action_channel: "none".to_string(),
            channel_action_reason: "none".to_string(),
        },
        chat: TauOpsDashboardChatSnapshot::default(),
        harness: TauOpsDashboardHarnessSnapshot::default(),
    });

    assert!(html.contains("id=\"tau-ops-connector-health-table\""));
    assert!(html.contains("id=\"tau-ops-connector-table-body\""));
    assert!(html.contains(
            "id=\"tau-ops-connector-row-0\" data-channel=\"telegram\" data-mode=\"polling\" data-liveness=\"open\" data-events-ingested=\"6\" data-provider-failures=\"2\""
        ));
}
