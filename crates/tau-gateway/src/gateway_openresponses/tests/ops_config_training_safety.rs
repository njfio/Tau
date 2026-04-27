use super::*;

#[tokio::test]
async fn integration_spec_3140_c04_ops_routes_render_config_training_safety_diagnostics_panels() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();
    let route_cases = [
        ("/ops/config", "id=\"tau-ops-config-panel\" data-route=\"/ops/config\" aria-hidden=\"false\" data-panel-visible=\"true\""),
        ("/ops/training", "id=\"tau-ops-training-panel\" data-route=\"/ops/training\" aria-hidden=\"false\" data-panel-visible=\"true\""),
        ("/ops/safety", "id=\"tau-ops-safety-panel\" data-route=\"/ops/safety\" aria-hidden=\"false\" data-panel-visible=\"true\""),
        ("/ops/diagnostics", "id=\"tau-ops-diagnostics-panel\" data-route=\"/ops/diagnostics\" aria-hidden=\"false\" data-panel-visible=\"true\""),
    ];

    for (route, expected_panel_marker) in route_cases {
        let response = client
            .get(format!(
                "http://{addr}{route}?theme=light&sidebar=collapsed&session=ops-route-contract"
            ))
            .send()
            .await
            .expect("load ops route");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.text().await.expect("read ops route body");
        assert!(
            body.contains(expected_panel_marker),
            "missing marker for route {route}"
        );
    }

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3144_c03_ops_config_route_renders_profile_policy_contract_markers() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops/config?theme=light&sidebar=collapsed&session=ops-config-contracts"
        ))
        .send()
        .await
        .expect("load ops config route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops config route body");

    assert!(body.contains(
        "id=\"tau-ops-config-profile-controls\" data-model-ref=\"gpt-4.1-mini\" data-fallback-model-count=\"2\" data-system-prompt-chars=\"0\" data-max-turns=\"64\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-config-policy-controls\" data-tool-policy-preset=\"balanced\" data-bash-profile=\"balanced\" data-os-sandbox-mode=\"auto\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-config-policy-limits\" data-bash-timeout-ms=\"120000\" data-max-command-length=\"8192\" data-max-tool-output-bytes=\"32768\" data-max-file-read-bytes=\"262144\" data-max-file-write-bytes=\"262144\""
    ));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3148_c04_ops_training_route_renders_training_contract_markers() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops/training?theme=light&sidebar=collapsed&session=ops-training-contracts"
        ))
        .send()
        .await
        .expect("load ops training route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops training route body");

    assert!(body.contains(
        "id=\"tau-ops-training-status\" data-status=\"running\" data-gate=\"hold\" data-store-path=\".tau/training/rl.sqlite\" data-update-interval-rollouts=\"8\" data-max-rollouts-per-update=\"64\" data-failure-streak=\"0/3\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-training-rollouts\" data-rollout-count=\"3\" data-last-rollout-id=\"142\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-training-optimizer\" data-mean-total-loss=\"0.023\" data-approx-kl=\"0.0012\" data-early-stop=\"false\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-training-actions\" data-pause-endpoint=\"/gateway/training/config\" data-reset-endpoint=\"/gateway/training/config\" data-export-endpoint=\"/gateway/training/rollouts\""
    ));

    handle.abort();
}
