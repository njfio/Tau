use super::*;

#[tokio::test]
async fn integration_spec_3106_c02_ops_tools_route_lists_registered_inventory_rows() {
    let temp = tempdir().expect("tempdir");
    let state = test_state_with_fixture_tools(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops/tools-jobs?theme=light&sidebar=collapsed&session=ops-tools"
        ))
        .send()
        .await
        .expect("load ops tools route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops tools route body");

    assert!(body.contains(
        "id=\"tau-ops-tools-panel\" data-route=\"/ops/tools-jobs\" aria-hidden=\"false\" data-panel-visible=\"true\" data-total-tools=\"2\""
    ));
    assert!(body.contains("id=\"tau-ops-tools-inventory-summary\" data-total-tools=\"2\""));
    assert!(body.contains("id=\"tau-ops-tools-inventory-table\" data-row-count=\"2\""));
    assert!(body.contains("id=\"tau-ops-tools-inventory-row-0\" data-tool-name=\"bash\""));
    assert!(body.contains("id=\"tau-ops-tools-inventory-row-1\" data-tool-name=\"memory_search\""));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3112_c03_ops_tools_route_renders_tool_detail_usage_contracts() {
    let temp = tempdir().expect("tempdir");
    let state = test_state_with_fixture_tools(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops/tools-jobs?theme=light&sidebar=collapsed&session=ops-tools-detail&tool=bash"
        ))
        .send()
        .await
        .expect("load ops tools detail route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .text()
        .await
        .expect("read ops tools detail route body");

    assert!(body.contains(
        "id=\"tau-ops-tool-detail-panel\" data-selected-tool=\"bash\" data-detail-visible=\"true\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-tool-detail-metadata\" data-tool-name=\"bash\" data-parameter-schema=\"{&quot;type&quot;:&quot;object&quot;,&quot;properties&quot;:{}}\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-tool-detail-policy\" data-timeout-ms=\"120000\" data-max-output-chars=\"32768\" data-sandbox-mode=\"default\""
    ));
    assert!(body.contains("id=\"tau-ops-tool-detail-usage-histogram\" data-bucket-count=\"3\""));
    assert!(body.contains(
        "id=\"tau-ops-tool-detail-usage-bucket-0\" data-hour-offset=\"0\" data-call-count=\"0\""
    ));
    assert!(body.contains("id=\"tau-ops-tool-detail-invocations\" data-row-count=\"1\""));
    assert!(body.contains(
        "id=\"tau-ops-tool-detail-invocation-row-0\" data-timestamp-unix-ms=\"0\" data-args-summary=\"{}\" data-result-summary=\"n/a\" data-duration-ms=\"0\" data-status=\"idle\""
    ));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3116_c03_ops_tools_route_renders_jobs_list_contracts() {
    let temp = tempdir().expect("tempdir");
    let state = test_state_with_fixture_tools(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops/tools-jobs?theme=light&sidebar=collapsed&session=ops-jobs"
        ))
        .send()
        .await
        .expect("load ops jobs route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops jobs route body");

    assert!(body
        .contains("id=\"tau-ops-jobs-panel\" data-panel-visible=\"true\" data-total-jobs=\"3\""));
    assert!(body.contains(
        "id=\"tau-ops-jobs-summary\" data-running-count=\"1\" data-completed-count=\"1\" data-failed-count=\"1\""
    ));
    assert!(body.contains("id=\"tau-ops-jobs-table\" data-row-count=\"3\""));
    assert!(body.contains(
        "id=\"tau-ops-jobs-row-0\" data-job-id=\"job-001\" data-job-name=\"memory-index\" data-job-status=\"running\" data-started-unix-ms=\"1000\" data-finished-unix-ms=\"0\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-jobs-row-1\" data-job-id=\"job-002\" data-job-name=\"session-prune\" data-job-status=\"completed\" data-started-unix-ms=\"900\" data-finished-unix-ms=\"950\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-jobs-row-2\" data-job-id=\"job-003\" data-job-name=\"connector-retry\" data-job-status=\"failed\" data-started-unix-ms=\"800\" data-finished-unix-ms=\"820\""
    ));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3120_c03_ops_tools_route_renders_selected_job_detail_output_contracts() {
    let temp = tempdir().expect("tempdir");
    let state = test_state_with_fixture_tools(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops/tools-jobs?theme=light&sidebar=collapsed&session=ops-job-detail&job=job-002"
        ))
        .send()
        .await
        .expect("load ops job detail route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .text()
        .await
        .expect("read ops job detail route body");

    assert!(body.contains(
        "id=\"tau-ops-job-detail-panel\" data-selected-job-id=\"job-002\" data-detail-visible=\"true\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-job-detail-metadata\" data-job-id=\"job-002\" data-job-status=\"completed\" data-duration-ms=\"50\""
    ));
    assert!(body.contains("id=\"tau-ops-job-detail-stdout\" data-output-bytes=\"14\""));
    assert!(body.contains("prune complete"));
    assert!(body.contains("id=\"tau-ops-job-detail-stderr\" data-output-bytes=\"0\""));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3124_c03_ops_tools_route_renders_job_cancel_contracts() {
    let temp = tempdir().expect("tempdir");
    let state = test_state_with_fixture_tools(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops/tools-jobs?theme=light&sidebar=collapsed&session=ops-jobs-cancel&cancel_job=job-001"
        ))
        .send()
        .await
        .expect("load ops jobs cancel route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .text()
        .await
        .expect("read ops jobs cancel route body");

    assert!(body.contains(
        "id=\"tau-ops-jobs-row-0\" data-job-id=\"job-001\" data-job-name=\"memory-index\" data-job-status=\"cancelled\" data-started-unix-ms=\"1000\" data-finished-unix-ms=\"1005\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-job-cancel-panel\" data-requested-job-id=\"job-001\" data-cancel-status=\"cancelled\" data-panel-visible=\"true\" data-cancel-endpoint-template=\"/gateway/jobs/{job_id}/cancel\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-job-cancel-submit\" data-action=\"cancel-job\" data-job-id=\"job-001\" data-cancel-enabled=\"false\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-job-detail-metadata\" data-job-id=\"job-001\" data-job-status=\"cancelled\" data-duration-ms=\"5\""
    ));
    assert!(body.contains("cancel requested"));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3128_c03_ops_channels_route_renders_channel_health_contracts() {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    write_multi_channel_runtime_fixture(temp.path(), true);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops/channels?theme=light&sidebar=collapsed&session=ops-channels"
        ))
        .send()
        .await
        .expect("load ops channels route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops channels route body");

    assert!(body.contains(
        "id=\"tau-ops-channels-panel\" data-route=\"/ops/channels\" aria-hidden=\"false\" data-panel-visible=\"true\" data-channel-count=\"1\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-channels-summary\" data-online-count=\"1\" data-offline-count=\"0\" data-degraded-count=\"0\""
    ));
    assert!(body.contains("id=\"tau-ops-channels-table\" data-row-count=\"1\""));
    assert!(body.contains(
        "id=\"tau-ops-channels-row-0\" data-channel=\"telegram\" data-mode=\"polling\" data-liveness=\"open\" data-events-ingested=\"6\" data-provider-failures=\"2\""
    ));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3132_c03_ops_channels_route_renders_channel_action_contracts() {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    write_multi_channel_runtime_fixture(temp.path(), true);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops/channels?theme=light&sidebar=collapsed&session=ops-channels-actions"
        ))
        .send()
        .await
        .expect("load ops channels actions route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .text()
        .await
        .expect("read ops channels actions route body");

    assert!(body.contains(
        "id=\"tau-ops-channels-login-0\" data-action=\"channel-login\" data-channel=\"telegram\" data-action-enabled=\"false\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-channels-logout-0\" data-action=\"channel-logout\" data-channel=\"telegram\" data-action-enabled=\"true\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-channels-probe-0\" data-action=\"channel-probe\" data-channel=\"telegram\" data-action-enabled=\"true\""
    ));

    handle.abort();
}
