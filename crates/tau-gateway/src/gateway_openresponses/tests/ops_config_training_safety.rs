use super::*;

#[derive(Debug, Default)]
struct FixtureHarnessSelfImprovementRunner;

impl GatewayOpsHarnessSelfImprovementRunner for FixtureHarnessSelfImprovementRunner {
    fn dry_run(
        &self,
        request: GatewayOpsHarnessSelfImprovementRequest,
    ) -> Result<GatewayOpsHarnessSelfImprovementResult> {
        Ok(GatewayOpsHarnessSelfImprovementResult {
            proposal_id: request.proposal_id.clone(),
            mission_id: format!("mission-{}", request.proposal_id),
            target_path: "prompts/research_to_doc/system.md".to_string(),
            result_key: "passed".to_string(),
            summary: "fixture dry-run passed".to_string(),
            artifact_path: Some(request.state_dir.join("fixture-dry-run.json")),
            applied: false,
        })
    }

    fn apply(
        &self,
        request: GatewayOpsHarnessSelfImprovementRequest,
    ) -> Result<GatewayOpsHarnessSelfImprovementResult> {
        Ok(GatewayOpsHarnessSelfImprovementResult {
            proposal_id: request.proposal_id.clone(),
            mission_id: format!("mission-{}", request.proposal_id),
            target_path: "prompts/research_to_doc/system.md".to_string(),
            result_key: "applied".to_string(),
            summary: "fixture apply completed".to_string(),
            artifact_path: Some(request.state_dir.join("fixture-apply.json")),
            applied: true,
        })
    }
}

#[tokio::test]
async fn integration_spec_3140_c04_ops_routes_render_config_training_safety_diagnostics_panels() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build no-redirect client");
    let route_cases = [
        ("/ops/config", "id=\"tau-ops-config-panel\" data-route=\"/ops/config\" aria-hidden=\"false\" data-panel-visible=\"true\""),
        ("/ops/training", "id=\"tau-ops-training-panel\" data-route=\"/ops/training\" aria-hidden=\"false\" data-panel-visible=\"true\""),
        ("/ops/safety", "id=\"tau-ops-safety-panel\" data-route=\"/ops/safety\" aria-hidden=\"false\" data-panel-visible=\"true\""),
        ("/ops/diagnostics", "id=\"tau-ops-diagnostics-panel\" data-route=\"/ops/diagnostics\" aria-hidden=\"false\" data-panel-visible=\"true\""),
        ("/ops/harness", "id=\"tau-ops-harness-panel\" data-route=\"/ops/harness\" data-component=\"MissionHarnessWorkspace\" data-design-template=\"three-window-agent-harness\" aria-hidden=\"false\" data-panel-visible=\"true\""),
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
async fn integration_spec_3756_c04_ops_harness_route_renders_benchmark_and_apply_markers() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops/harness?theme=dark&sidebar=expanded&session=ops-harness-contract"
        ))
        .send()
        .await
        .expect("load ops harness route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops harness route body");

    for marker in [
        "id=\"tau-ops-harness-benchmark-panel\" data-benchmark-id=\"m334-tranche-one-autonomy\"",
        "id=\"tau-ops-harness-run-benchmark-form\" action=\"/ops/harness/run-benchmark?session=ops-harness-contract\" method=\"post\" data-command=\"tau_agent_harness\"",
        "id=\"tau-ops-harness-conservative-policy\" data-policy=\"conservative-self-improvement\" data-allowed-targets=\"skill,config,prompt\" data-blocked-targets=\"source-code,safety-policy\"",
        "id=\"tau-ops-harness-apply-form\" action=\"/ops/harness/proposals/PR-044/apply\" method=\"post\" data-approval-state=\"approval-required\"",
        "id=\"tau-ops-harness-action-apply\" type=\"submit\" data-action=\"apply\" data-action-tone=\"disabled\" data-disabled=\"true\" aria-disabled=\"true\" data-approval-required=\"true\" disabled",
        "id=\"tau-ops-harness-tui-companion\" data-component=\"TuiCompanion\" data-command=\"tau status\"",
    ] {
        assert!(
            body.contains(marker),
            "missing harness gateway marker `{marker}`"
        );
    }

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3756_c05_ops_harness_actions_execute_and_persist_proof() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let state_dir = state.config.state_dir.clone();
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build no-redirect client");

    let benchmark_response = client
        .post(format!("http://{addr}/ops/harness/run-benchmark"))
        .send()
        .await
        .expect("run harness benchmark");
    assert_eq!(benchmark_response.status(), StatusCode::SEE_OTHER);
    let benchmark_location = benchmark_response
        .headers()
        .get(reqwest::header::LOCATION)
        .expect("benchmark redirect location")
        .to_str()
        .expect("location header is utf8");
    assert!(benchmark_location.contains("benchmark_status=passed"));

    let proof_path = state_dir.join("ops-harness/m334/latest.json");
    let proof_json = std::fs::read_to_string(&proof_path).expect("read benchmark proof");
    let proof: serde_json::Value =
        serde_json::from_str(&proof_json).expect("benchmark proof is json");
    assert_eq!(proof["benchmark_id"], "m334-tranche-one-autonomy");
    assert_eq!(proof["passed"], true);
    assert_eq!(proof["tasks"].as_array().expect("task array").len(), 4);

    let harness_memory_records = gateway_memory_store(&state_dir, "default")
        .list_latest_records(
            Some(&tau_memory::runtime::MemoryScopeFilter {
                workspace_id: Some("default".to_string()),
                channel_id: Some("tau-agent-harness".to_string()),
                actor_id: Some("tau".to_string()),
            }),
            10,
        )
        .expect("list harness learning records");
    assert_eq!(harness_memory_records.len(), 4);
    assert!(harness_memory_records
        .iter()
        .all(|record| record.memory_type.as_str() == "decision"));
    assert!(harness_memory_records
        .iter()
        .all(|record| record.entry.tags.contains(&"mission_learning".to_string())));
    assert!(harness_memory_records.iter().any(|record| record
        .entry
        .summary
        .contains("repo_spec_to_pr_feature_delivery")));

    let graph_endpoint = expand_session_template(GATEWAY_MEMORY_GRAPH_ENDPOINT, "default");
    let graph_response = client
        .get(format!(
            "http://{addr}{graph_endpoint}?workspace_id=default&channel_id=tau-agent-harness&actor_id=tau&memory_type=decision"
        ))
        .bearer_auth("secret")
        .send()
        .await
        .expect("request harness learning graph");
    assert_eq!(graph_response.status(), StatusCode::OK);
    let graph_payload = graph_response
        .json::<Value>()
        .await
        .expect("parse harness learning graph");
    assert!(graph_payload["node_count"].as_u64().unwrap_or_default() >= 4);
    assert!(graph_payload["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .any(|node| node["label"]
            .as_str()
            .unwrap_or_default()
            .contains("Autonomy benchmark task")));

    let apply_response = client
        .post(format!("http://{addr}/ops/harness/proposals/PR-044/apply"))
        .send()
        .await
        .expect("direct apply is rejected");
    assert_eq!(apply_response.status(), StatusCode::FORBIDDEN);
    let apply_body = apply_response.text().await.expect("read apply rejection");
    assert!(apply_body.contains("id=\"tau-ops-harness-apply-blocked\""));
    assert!(apply_body.contains("data-result=\"blocked_approval_required\""));
    let audit_log =
        std::fs::read_to_string(state_dir.join("ops-harness/audit.jsonl")).expect("audit log");
    assert!(audit_log.contains("\"action\":\"apply\""));
    assert!(audit_log.contains("\"result\":\"blocked_approval_required\""));

    let approve_response = client
        .post(format!(
            "http://{addr}/ops/harness/proposals/PR-044/approve"
        ))
        .send()
        .await
        .expect("approve proposal");
    assert_eq!(approve_response.status(), StatusCode::SEE_OTHER);
    let audit_log =
        std::fs::read_to_string(state_dir.join("ops-harness/audit.jsonl")).expect("audit log");
    assert!(audit_log.contains("\"proposal_id\":\"PR-044\""));
    assert!(audit_log.contains("\"action\":\"approve\""));
    assert!(audit_log.contains("\"result\":\"recorded\""));

    let diff_response = client
        .get(format!("http://{addr}/ops/harness/proposals/PR-044/diff"))
        .send()
        .await
        .expect("load proposal diff");
    assert_eq!(diff_response.status(), StatusCode::OK);
    let diff_body = diff_response.text().await.expect("read diff body");
    assert!(diff_body.contains("id=\"tau-ops-harness-diff\""));
    assert!(diff_body.contains("data-proposal-id=\"PR-044\""));
    assert!(diff_body.contains("data-diff-view=\"operator-review\""));
    assert!(diff_body.contains("data-target-path=\"prompts/research_to_doc/system.md\""));
    assert!(diff_body.contains("data-dry-run-result=\"passed\""));
    assert!(diff_body.contains("data-safety-check=\"passed\""));
    assert!(diff_body.contains("data-policy-allowed=\"skill,config,prompt\""));
    assert!(diff_body.contains("data-policy-blocked=\"source-code,safety-policy\""));
    assert!(diff_body
        .contains("Compress system prompt by removing redundant instructions and examples."));
    assert!(diff_body.contains("class=\"tau-harness-diff-line tau-harness-diff-line-remove\""));
    assert!(diff_body.contains("class=\"tau-harness-diff-line tau-harness-diff-line-add\""));

    handle.abort();
}

#[tokio::test]
async fn integration_ops_harness_proposal_actions_delegate_dry_run_and_approved_apply() {
    let temp = tempdir().expect("tempdir");
    let state = test_state_with_client_auth_and_harness_runner(
        temp.path(),
        4_096,
        Arc::new(MockGatewayLlmClient::default()),
        Arc::new(NoopGatewayToolRegistrar),
        GatewayOpenResponsesAuthMode::Token,
        Some("secret"),
        None,
        60,
        120,
        Arc::new(FixtureHarnessSelfImprovementRunner),
    );
    let state_dir = state.config.state_dir.clone();
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build no-redirect client");

    let dry_run = client
        .post(format!(
            "http://{addr}/ops/harness/proposals/PR-044/dry-run"
        ))
        .send()
        .await
        .expect("dry-run proposal");
    assert_eq!(dry_run.status(), StatusCode::SEE_OTHER);
    assert!(dry_run
        .headers()
        .get(reqwest::header::LOCATION)
        .expect("dry-run redirect")
        .to_str()
        .expect("dry-run location")
        .contains("proposal_status=dry_run_passed"));

    let approve = client
        .post(format!(
            "http://{addr}/ops/harness/proposals/PR-044/approve"
        ))
        .send()
        .await
        .expect("approve proposal");
    assert_eq!(approve.status(), StatusCode::SEE_OTHER);

    let harness_response = client
        .get(format!("http://{addr}/ops/harness"))
        .send()
        .await
        .expect("load approved harness route");
    assert_eq!(harness_response.status(), StatusCode::OK);
    let harness_body = harness_response.text().await.expect("harness body");
    assert!(harness_body.contains(
        "id=\"tau-ops-harness-apply-form\" action=\"/ops/harness/proposals/PR-044/apply\" method=\"post\" data-approval-state=\"approved\""
    ));
    assert!(harness_body.contains(
        "id=\"tau-ops-harness-action-apply\" type=\"submit\" data-action=\"apply\" data-action-tone=\"disabled\" data-disabled=\"false\" aria-disabled=\"false\""
    ));

    let apply = client
        .post(format!("http://{addr}/ops/harness/proposals/PR-044/apply"))
        .send()
        .await
        .expect("apply proposal");
    assert_eq!(apply.status(), StatusCode::SEE_OTHER);
    assert!(apply
        .headers()
        .get(reqwest::header::LOCATION)
        .expect("apply redirect")
        .to_str()
        .expect("apply location")
        .contains("proposal_status=applied"));

    let applied_harness_response = client
        .get(format!("http://{addr}/ops/harness"))
        .send()
        .await
        .expect("load applied harness route");
    assert_eq!(applied_harness_response.status(), StatusCode::OK);
    let applied_harness_body = applied_harness_response
        .text()
        .await
        .expect("applied harness body");
    assert!(applied_harness_body.contains(
        "id=\"tau-ops-harness-apply-form\" action=\"/ops/harness/proposals/PR-044/apply\" method=\"post\" data-approval-state=\"applied\""
    ));
    assert!(applied_harness_body.contains(
        "id=\"tau-ops-harness-action-apply\" type=\"submit\" data-action=\"apply\" data-action-tone=\"disabled\" data-disabled=\"true\" aria-disabled=\"true\""
    ));

    let audit_log =
        std::fs::read_to_string(state_dir.join("ops-harness/audit.jsonl")).expect("audit log");
    assert!(audit_log.contains("\"action\":\"dry-run\""));
    assert!(audit_log.contains("\"result\":\"passed\""));
    assert!(audit_log.contains("\"action\":\"apply\""));
    assert!(audit_log.contains("\"result\":\"applied\""));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build no-redirect client");

    let benchmark_response = client
        .post(format!("http://{addr}/ops/harness/run-benchmark"))
        .send()
        .await
        .expect("run benchmark");
    assert_eq!(benchmark_response.status(), StatusCode::SEE_OTHER);

    let apply_response = client
        .post(format!("http://{addr}/ops/harness/proposals/PR-044/apply"))
        .send()
        .await
        .expect("apply is blocked and audited");
    assert_eq!(apply_response.status(), StatusCode::FORBIDDEN);

    let harness_response = client
        .get(format!("http://{addr}/ops/harness"))
        .send()
        .await
        .expect("load state-backed harness route");
    assert_eq!(harness_response.status(), StatusCode::OK);
    let body = harness_response.text().await.expect("read harness body");

    for marker in [
        "data-task-count=\"4\" data-pass-count=\"4\" data-failed-gates=\"none\" data-proof-source=\"state\"",
        "data-category=\"repo_build\" data-task-count=\"1\" data-last-run=\"1/1 pass\" data-pass-rate=\"100\"",
        "id=\"tau-ops-harness-audit-log\" data-audit-row-count=\"1\" data-audit-source=\"state\"",
        "data-action=\"apply\" data-result=\"blocked_approval_required\"",
        "Blocked Approval Required",
    ] {
        assert!(
            body.contains(marker),
            "missing state-backed harness marker `{marker}`"
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
