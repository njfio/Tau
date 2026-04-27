use super::*;

#[tokio::test]
async fn functional_spec_2806_c01_c02_c03_ops_shell_command_center_markers_reflect_dashboard_snapshot(
) {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!("http://{addr}/ops"))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");

    assert!(body.contains("data-health-state=\"healthy\""));
    assert!(body.contains("data-health-reason=\"no recent transport failures observed\""));
    assert_eq!(body.matches("data-kpi-card=").count(), 6);
    assert!(body.contains("data-kpi-card=\"queue-depth\" data-kpi-value=\"1\""));
    assert!(body.contains("data-kpi-card=\"failure-streak\" data-kpi-value=\"0\""));
    assert!(body.contains("data-kpi-card=\"processed-cases\" data-kpi-value=\"2\""));
    assert!(body.contains("data-kpi-card=\"alert-count\" data-kpi-value=\"2\""));
    assert!(body.contains("data-kpi-card=\"widget-count\" data-kpi-value=\"2\""));
    assert!(body.contains("data-kpi-card=\"timeline-cycles\" data-kpi-value=\"2\""));
    assert!(body.contains("data-alert-count=\"2\""));
    assert!(body.contains("data-primary-alert-code=\"dashboard_queue_backlog\""));
    assert!(body.contains("data-primary-alert-severity=\"warning\""));
    assert!(body.contains("runtime backlog detected (queue_depth=1)"));
    assert!(body.contains("data-timeline-cycle-count=\"2\""));
    assert!(body.contains("data-timeline-invalid-cycle-count=\"1\""));

    handle.abort();
}

#[tokio::test]
async fn functional_spec_2854_c01_ops_shell_command_center_panel_visible_on_ops_route() {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!("http://{addr}/ops"))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");

    assert!(
        body.contains("id=\"tau-ops-command-center\" data-route=\"/ops\" aria-hidden=\"false\"")
    );

    handle.abort();
}

#[tokio::test]
async fn integration_spec_2854_c02_c03_command_center_panel_hidden_on_chat_and_sessions_routes() {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let chat_response = client
        .get(format!(
            "http://{addr}/ops/chat?theme=light&sidebar=collapsed&session=chat-c01"
        ))
        .send()
        .await
        .expect("ops chat shell request");
    assert_eq!(chat_response.status(), StatusCode::OK);
    let chat_body = chat_response
        .text()
        .await
        .expect("read ops chat shell body");
    assert!(chat_body
        .contains("id=\"tau-ops-command-center\" data-route=\"/ops\" aria-hidden=\"true\""));

    let sessions_response = client
        .get(format!(
            "http://{addr}/ops/sessions?theme=dark&sidebar=expanded&session=chat-c01"
        ))
        .send()
        .await
        .expect("ops sessions shell request");
    assert_eq!(sessions_response.status(), StatusCode::OK);
    let sessions_body = sessions_response
        .text()
        .await
        .expect("read ops sessions shell body");
    assert!(sessions_body
        .contains("id=\"tau-ops-command-center\" data-route=\"/ops\" aria-hidden=\"true\""));

    handle.abort();
}

#[tokio::test]
async fn functional_spec_2810_c01_c02_c03_ops_shell_control_markers_reflect_dashboard_control_snapshot(
) {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_dashboard_control_state_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!("http://{addr}/ops"))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");

    assert!(body.contains("id=\"tau-ops-control-panel\""));
    assert!(body.contains("data-control-mode=\"paused\""));
    assert!(body.contains("data-rollout-gate=\"hold\""));
    assert!(body.contains("data-control-paused=\"true\""));
    assert!(body.contains("id=\"tau-ops-control-action-pause\""));
    assert!(body.contains("id=\"tau-ops-control-action-resume\""));
    assert!(body.contains("id=\"tau-ops-control-action-refresh\""));
    assert!(body.contains("id=\"tau-ops-control-actions\" data-action-count=\"3\" data-action-endpoint=\"/ops/control-action\""));
    assert!(body.contains("id=\"tau-ops-control-action-form-pause\" action=\"/ops/control-action\" method=\"post\" data-action=\"pause\""));
    assert!(body.contains("id=\"tau-ops-control-action-form-resume\" action=\"/ops/control-action\" method=\"post\" data-action=\"resume\""));
    assert!(body.contains("id=\"tau-ops-control-action-form-refresh\" action=\"/ops/control-action\" method=\"post\" data-action=\"refresh\""));
    assert!(body.contains("id=\"tau-ops-control-action-pause\" data-action-enabled=\"false\""));
    assert!(body.contains("id=\"tau-ops-control-action-resume\" data-action-enabled=\"true\""));
    assert!(body.contains("id=\"tau-ops-control-action-refresh\" data-action-enabled=\"true\""));
    assert!(body.contains("id=\"tau-ops-control-last-action\""));
    assert!(body.contains("data-last-action-request-id=\"dashboard-action-90210\""));
    assert!(body.contains("data-last-action-name=\"pause\""));
    assert!(body.contains("data-last-action-actor=\"ops-user\""));
    assert!(body.contains("data-last-action-reason=\"maintenance\""));
    assert!(body.contains("data-last-action-timestamp=\"90210\""));

    handle.abort();
}

#[tokio::test]
async fn functional_spec_2826_c03_ops_shell_control_markers_include_confirmation_payload() {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_dashboard_control_state_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!("http://{addr}/ops"))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");

    assert!(body.contains(
        "id=\"tau-ops-control-action-pause\" data-action-enabled=\"false\" data-action=\"pause\" data-confirm-required=\"true\" data-confirm-title=\"Confirm pause action\" data-confirm-body=\"Pause command-center processing until resumed.\" data-confirm-verb=\"pause\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-control-action-resume\" data-action-enabled=\"true\" data-action=\"resume\" data-confirm-required=\"true\" data-confirm-title=\"Confirm resume action\" data-confirm-body=\"Resume command-center processing.\" data-confirm-verb=\"resume\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-control-action-refresh\" data-action-enabled=\"true\" data-action=\"refresh\" data-confirm-required=\"true\" data-confirm-title=\"Confirm refresh action\" data-confirm-body=\"Refresh command-center state from latest runtime artifacts.\" data-confirm-verb=\"refresh\""
    ));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3478_c03_ops_shell_last_action_section_renders_readable_rows() {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_dashboard_control_state_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!("http://{addr}/ops"))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");

    assert!(body.contains("id=\"tau-ops-control-last-action\""));
    assert!(
        body.contains("id=\"tau-ops-last-action-request-id\">request.id: dashboard-action-90210")
    );
    assert!(body.contains("id=\"tau-ops-last-action-name\">action: pause"));
    assert!(body.contains("id=\"tau-ops-last-action-actor\">actor: ops-user"));
    assert!(body.contains("id=\"tau-ops-last-action-reason\">reason: maintenance"));
    assert!(body.contains("id=\"tau-ops-last-action-timestamp\">timestamp: 90210"));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3482_c03_ops_shell_last_action_reason_row_renders_fixture_reason() {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_dashboard_control_state_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!("http://{addr}/ops"))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");

    assert!(body.contains("id=\"tau-ops-control-last-action\""));
    assert!(body.contains("data-last-action-reason=\"maintenance\""));
    assert!(body.contains("id=\"tau-ops-last-action-reason\">reason: maintenance"));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3466_c03_ops_control_action_form_submits_dashboard_mutation_and_redirects_with_applied_marker(
) {
    let temp = tempdir().expect("tempdir");
    let dashboard_root = write_dashboard_runtime_fixture(temp.path());
    write_dashboard_control_state_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("client");

    let response = client
        .post(format!(
            "http://{addr}{OPS_DASHBOARD_CONTROL_ACTION_ENDPOINT}"
        ))
        .form(&[
            ("action", "resume"),
            ("reason", "ops-shell-control-panel"),
            ("theme", "light"),
            ("sidebar", "collapsed"),
        ])
        .send()
        .await
        .expect("submit ops control action");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert_eq!(
        location,
        "/ops?theme=light&sidebar=collapsed&control_action_status=applied&control_action=resume&control_action_reason=control_action_applied"
    );

    let control_state = std::fs::read_to_string(dashboard_root.join("control-state.json"))
        .expect("read dashboard control state");
    assert!(control_state.contains("\"mode\": \"running\""));
    assert!(control_state.contains("\"action\": \"resume\""));
    assert!(control_state.contains("\"actor\": \"ops-shell\""));

    let actions_log = std::fs::read_to_string(dashboard_root.join("actions-audit.jsonl"))
        .expect("read dashboard actions log");
    assert!(actions_log.contains("\"action\":\"resume\""));
    assert!(actions_log.contains("\"actor\":\"ops-shell\""));

    let redirect_response = client
        .get(format!("http://{addr}{location}"))
        .send()
        .await
        .expect("load control action redirect body");
    assert_eq!(redirect_response.status(), StatusCode::OK);
    let redirect_body = redirect_response
        .text()
        .await
        .expect("read control action redirect body");
    assert!(redirect_body.contains(
        "id=\"tau-ops-control-action-status\" data-control-action-status=\"applied\" data-control-action=\"resume\" data-control-action-reason=\"control_action_applied\""
    ));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3466_c01_ops_control_action_missing_action_redirects_with_missing_marker()
{
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_dashboard_control_state_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("client");

    let response = client
        .post(format!(
            "http://{addr}{OPS_DASHBOARD_CONTROL_ACTION_ENDPOINT}"
        ))
        .form(&[
            ("action", ""),
            ("reason", "ops-shell-control-panel"),
            ("theme", "light"),
            ("sidebar", "collapsed"),
        ])
        .send()
        .await
        .expect("submit control action form with missing action");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert_eq!(
        location,
        "/ops?theme=light&sidebar=collapsed&control_action_status=missing&control_action=none&control_action_reason=missing_action"
    );

    let redirect_response = client
        .get(format!("http://{addr}{location}"))
        .send()
        .await
        .expect("load missing-action redirect body");
    assert_eq!(redirect_response.status(), StatusCode::OK);
    let redirect_body = redirect_response
        .text()
        .await
        .expect("read missing-action redirect body");
    assert!(redirect_body.contains(
        "id=\"tau-ops-control-action-status\" data-control-action-status=\"missing\" data-control-action=\"none\" data-control-action-reason=\"missing_action\""
    ));

    handle.abort();
}

#[tokio::test]
async fn regression_spec_3466_c02_ops_control_action_invalid_action_fails_closed_with_redirect_marker(
) {
    let temp = tempdir().expect("tempdir");
    let dashboard_root = write_dashboard_runtime_fixture(temp.path());
    write_dashboard_control_state_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("client");

    let response = client
        .post(format!(
            "http://{addr}{OPS_DASHBOARD_CONTROL_ACTION_ENDPOINT}"
        ))
        .form(&[
            ("action", "explode"),
            ("reason", "ops-shell-control-panel"),
            ("theme", "light"),
            ("sidebar", "collapsed"),
        ])
        .send()
        .await
        .expect("submit control action form with invalid action");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert_eq!(
        location,
        "/ops?theme=light&sidebar=collapsed&control_action_status=failed&control_action=none&control_action_reason=invalid_dashboard_action"
    );

    let control_state = std::fs::read_to_string(dashboard_root.join("control-state.json"))
        .expect("read dashboard control state");
    assert!(control_state.contains("\"mode\": \"paused\""));
    assert!(!control_state.contains("\"action\": \"explode\""));

    let redirect_response = client
        .get(format!("http://{addr}{location}"))
        .send()
        .await
        .expect("load invalid-action redirect body");
    assert_eq!(redirect_response.status(), StatusCode::OK);
    let redirect_body = redirect_response
        .text()
        .await
        .expect("read invalid-action redirect body");
    assert!(redirect_body.contains(
        "id=\"tau-ops-control-action-status\" data-control-action-status=\"failed\" data-control-action=\"none\" data-control-action-reason=\"invalid_dashboard_action\""
    ));

    handle.abort();
}

#[tokio::test]
async fn functional_spec_2814_c01_c02_ops_shell_timeline_chart_markers_reflect_snapshot_and_range_query(
) {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops?theme=light&sidebar=collapsed&range=6h"
        ))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");

    assert!(body.contains("id=\"tau-ops-queue-timeline-chart\""));
    assert!(body.contains("data-component=\"TimelineChart\""));
    assert!(body.contains("data-timeline-point-count=\"2\""));
    assert!(body.contains("data-timeline-last-timestamp=\"811\""));
    assert!(body.contains("data-timeline-range=\"6h\""));
    assert!(body.contains("id=\"tau-ops-timeline-range-1h\""));
    assert!(body.contains("id=\"tau-ops-timeline-range-6h\""));
    assert!(body.contains("id=\"tau-ops-timeline-range-24h\""));
    assert!(body.contains(
        "id=\"tau-ops-timeline-range-1h\" data-range-option=\"1h\" data-range-selected=\"false\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-timeline-range-6h\" data-range-option=\"6h\" data-range-selected=\"true\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-timeline-range-24h\" data-range-option=\"24h\" data-range-selected=\"false\""
    ));
    assert!(body.contains("href=\"/ops?theme=light&amp;sidebar=collapsed&amp;range=1h\""));
    assert!(body.contains("href=\"/ops?theme=light&amp;sidebar=collapsed&amp;range=6h\""));
    assert!(body.contains("href=\"/ops?theme=light&amp;sidebar=collapsed&amp;range=24h\""));

    handle.abort();
}

#[tokio::test]
async fn functional_spec_2814_c03_ops_shell_timeline_range_invalid_query_defaults_to_1h() {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!("http://{addr}/ops?range=quarter"))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");

    assert!(body.contains("data-timeline-range=\"1h\""));
    assert!(body.contains(
        "id=\"tau-ops-timeline-range-1h\" data-range-option=\"1h\" data-range-selected=\"true\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-timeline-range-6h\" data-range-option=\"6h\" data-range-selected=\"false\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-timeline-range-24h\" data-range-option=\"24h\" data-range-selected=\"false\""
    ));

    handle.abort();
}

#[tokio::test]
async fn functional_spec_2850_c01_c02_c03_ops_shell_recent_cycles_table_exposes_panel_summary_and_empty_state_markers(
) {
    let temp = tempdir().expect("tempdir");
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops?theme=dark&sidebar=expanded&range=24h"
        ))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");

    assert!(
        body.contains("id=\"tau-ops-data-table\" data-route=\"/ops\" data-timeline-range=\"24h\"")
    );
    assert!(body.contains(
        "id=\"tau-ops-timeline-summary-row\" data-row-kind=\"summary\" data-last-timestamp=\"0\" data-point-count=\"0\" data-cycle-count=\"0\" data-invalid-cycle-count=\"0\""
    ));
    assert!(body.contains("id=\"tau-ops-timeline-empty-row\" data-empty-state=\"true\""));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_2850_c04_ops_shell_recent_cycles_table_hides_empty_state_when_timeline_present(
) {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops?theme=light&sidebar=collapsed&range=6h"
        ))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");

    assert!(
        body.contains("id=\"tau-ops-data-table\" data-route=\"/ops\" data-timeline-range=\"6h\"")
    );
    assert!(body.contains(
        "id=\"tau-ops-timeline-summary-row\" data-row-kind=\"summary\" data-last-timestamp=\"811\" data-point-count=\"2\" data-cycle-count=\"2\" data-invalid-cycle-count=\"1\""
    ));
    assert!(!body.contains("id=\"tau-ops-timeline-empty-row\""));

    handle.abort();
}

#[tokio::test]
async fn functional_spec_2818_c01_c02_ops_shell_alert_feed_row_markers_reflect_dashboard_snapshot()
{
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!("http://{addr}/ops"))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");

    assert!(body.contains("id=\"tau-ops-alert-feed-list\""));
    assert!(body.contains("id=\"tau-ops-alert-row-0\""));
    assert!(body.contains(
        "id=\"tau-ops-alert-row-0\" data-alert-code=\"dashboard_queue_backlog\" data-alert-severity=\"warning\""
    ));
    assert!(body.contains("runtime backlog detected (queue_depth=1)"));
    assert!(body.contains("id=\"tau-ops-alert-row-1\""));
    assert!(body.contains(
        "id=\"tau-ops-alert-row-1\" data-alert-code=\"dashboard_cycle_log_invalid_lines\" data-alert-severity=\"warning\""
    ));

    handle.abort();
}

#[tokio::test]
async fn functional_spec_2818_c03_ops_shell_alert_feed_rows_include_nominal_fallback_alert() {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture_nominal(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!("http://{addr}/ops"))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");

    assert!(body.contains("id=\"tau-ops-alert-feed-list\""));
    assert!(body.contains(
        "id=\"tau-ops-alert-row-0\" data-alert-code=\"dashboard_healthy\" data-alert-severity=\"info\""
    ));
    assert!(body.contains("dashboard runtime health is nominal"));

    handle.abort();
}

#[tokio::test]
async fn functional_spec_2822_c01_c02_ops_shell_connector_health_rows_reflect_multi_channel_connectors(
) {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    write_multi_channel_runtime_fixture(temp.path(), true);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!("http://{addr}/ops"))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");

    assert!(body.contains("id=\"tau-ops-connector-health-table\""));
    assert!(body.contains("id=\"tau-ops-connector-table-body\""));
    assert!(body.contains("id=\"tau-ops-connector-row-0\""));
    assert!(body.contains(
        "id=\"tau-ops-connector-row-0\" data-channel=\"telegram\" data-mode=\"polling\" data-liveness=\"open\" data-events-ingested=\"6\" data-provider-failures=\"2\""
    ));

    handle.abort();
}

#[tokio::test]
async fn functional_spec_2822_c03_ops_shell_connector_health_rows_include_fallback_when_state_missing(
) {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    write_multi_channel_runtime_fixture(temp.path(), false);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!("http://{addr}/ops"))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");

    assert!(body.contains("id=\"tau-ops-connector-health-table\""));
    assert!(body.contains("id=\"tau-ops-connector-table-body\""));
    assert!(body.contains(
        "id=\"tau-ops-connector-row-0\" data-channel=\"none\" data-mode=\"unknown\" data-liveness=\"unknown\" data-events-ingested=\"0\" data-provider-failures=\"0\""
    ));

    handle.abort();
}
