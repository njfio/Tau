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
async fn integration_ops_harness_proposal_registry_renders_selected_proposal() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let state_dir = state.config.state_dir.clone();
    let mission_dir = state_dir
        .join("ops-harness")
        .join("self-improvement")
        .join("PR-045");
    std::fs::create_dir_all(&mission_dir).expect("mission dir");
    std::fs::write(
        mission_dir.join("mission.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "mission_id": "ops-harness-self-improve-pr-045",
            "status": "completed",
            "plan_dag": [
                {"id": "observe-failure", "description": "Record failure learning from the harness observation.", "status": "completed"},
                {"id": "dry-run", "description": "Dry-run the conservative proposal against the safety policy.", "status": "completed"},
                {"id": "operator-approval", "description": "Record operator approval for the proposed change.", "status": "completed"},
                {"id": "apply-update", "description": "Apply the approved target update with rollback metadata.", "status": "completed"},
                {"id": "curate-learning", "description": "Update the curator record after successful apply.", "status": "completed"}
            ],
            "verification_gates": [
                {"id": "VG-DRY-RUN", "description": "Self-modification dry-run passes safety policy.", "status": "passed"},
                {"id": "VG-APPROVAL", "description": "Operator approval is present before apply.", "status": "passed"},
                {"id": "VG-APPLY", "description": "Target update is applied and curator state is updated.", "status": "passed"}
            ],
            "memory_hits": [
                {"key": "LR-045", "summary": "Benchmark artifacts were hard to correlate with missions."}
            ],
            "artifacts": [
                {"artifact_id": "dry-run-result", "kind": "self-improvement-dry-run", "path": "ops-harness/self-improvement/PR-045/dry-run-result.json"},
                {"artifact_id": "apply-result", "kind": "self-improvement-apply", "path": "ops-harness/self-improvement/PR-045/apply-result.json"},
                {"artifact_id": "target:skills/benchmark_artifacts/SKILL.md", "kind": "skill", "path": "skills/benchmark_artifacts/SKILL.md"}
            ],
            "final_learning_output": {
                "summary": "Applied PR-045 and updated curator state for LR-045.",
                "records": ["LR-045"]
            }
        }))
        .expect("mission json"),
    )
    .expect("write mission");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let harness_response = client
        .get(format!(
            "http://{addr}/ops/harness?proposal_id=PR-045&theme=dark&sidebar=expanded"
        ))
        .send()
        .await
        .expect("load selected harness proposal");
    assert_eq!(harness_response.status(), StatusCode::OK);
    let harness_body = harness_response.text().await.expect("harness body");
    for marker in [
        "id=\"tau-ops-harness-self-improvement-window\" data-window=\"self-improvement-review-apply-flow\" data-window-order=\"3\" data-selected-proposal=\"PR-045\"",
        "id=\"tau-ops-harness-approve-form\" action=\"/ops/harness/proposals/PR-045/approve\" method=\"post\"",
        "id=\"tau-ops-harness-proposal-detail\" data-proposal-id=\"PR-045\" data-learning-record=\"LR-045\" data-target-type=\"Skill\" data-target-path=\"skills/benchmark_artifacts/SKILL.md\"",
        "PR-045 Skill patch for benchmark artifact naming",
        "Add a skill rule for deterministic benchmark artifact naming.",
        "id=\"tau-ops-harness-learning-queue\" data-queue-count=\"1\"",
        "data-queue-source=\"state\"",
        "data-learning-id=\"PR-045\" data-status=\"completed\" data-selected=\"true\" data-actionable=\"true\"",
        "href=\"/ops/harness?theme=dark&amp;sidebar=expanded&amp;session=default&amp;proposal_id=PR-045\" data-proposal-link=\"PR-045\" aria-current=\"page\"",
        "<span class=\"tau-harness-queue-label\">Skill patch for benchmark artifact naming</span>",
        "<span class=\"tau-harness-queue-status\">Completed</span>",
        "id=\"tau-ops-harness-self-improvement-proof\" data-proof-source=\"state\" data-mission-id=\"ops-harness-self-improve-pr-045\" data-mission-status=\"completed\" data-plan-completed=\"5\" data-plan-total=\"5\" data-gates-passed=\"3\" data-gates-total=\"3\" data-memory-hits=\"1\" data-artifact-count=\"3\" data-final-learning-records=\"LR-045\"",
        "Applied PR-045 and updated curator state for LR-045.",
        "data-proof-row=\"artifact\" data-proof-id=\"target:skills/benchmark_artifacts/SKILL.md\" data-proof-status=\"skill\">skills/benchmark_artifacts/SKILL.md</li>",
    ] {
        assert!(
            harness_body.contains(marker),
            "missing registry-backed harness marker `{marker}`"
        );
    }
    for stale_marker in [
        "data-learning-id=\"LR-219\"",
        "data-learning-id=\"LR-220\"",
        "data-learning-id=\"PR-044\"",
    ] {
        assert!(
            !harness_body.contains(stale_marker),
            "state-backed selected proposal queue rendered stale row `{stale_marker}`"
        );
    }

    let diff_response = client
        .get(format!("http://{addr}/ops/harness/proposals/PR-045/diff"))
        .send()
        .await
        .expect("load selected proposal diff");
    assert_eq!(diff_response.status(), StatusCode::OK);
    let diff_body = diff_response.text().await.expect("diff body");
    for marker in [
        "data-proposal-id=\"PR-045\"",
        "data-target-path=\"skills/benchmark_artifacts/SKILL.md\"",
        "PR-045 Skill patch for benchmark artifact naming",
        "Name benchmark artifacts with mission id, benchmark id, run id, and proof type.",
    ] {
        assert!(
            diff_body.contains(marker),
            "missing registry-backed diff marker `{marker}`"
        );
    }

    let missing_response = client
        .get(format!("http://{addr}/ops/harness/proposals/PR-999/diff"))
        .send()
        .await
        .expect("load missing proposal diff");
    assert_eq!(missing_response.status(), StatusCode::NOT_FOUND);

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
        "id=\"tau-ops-harness-proof-window\" data-window=\"mission-detail-proof-view\" data-window-order=\"2\" data-run-id=\"gateway-harness-",
        "data-mission-status=\"completed\" data-tool-budget=\"20/32\"",
        "Canonical M334 Tranche One Autonomy benchmark proof run",
        "id=\"tau-ops-harness-kpi-active\" data-harness-kpi-card=\"active-missions\" data-kpi-value=\"1\"",
        "<h4>Benchmark Runs</h4><p>1</p><small>completed</small>",
        "id=\"tau-ops-harness-kpi-verifications\" data-harness-kpi-card=\"pending-verifications\" data-kpi-value=\"0\"",
        "<small>none failed</small>",
        "id=\"tau-ops-harness-kpi-memory\" data-harness-kpi-card=\"memory-writes\" data-kpi-value=\"4\"",
        "<h4>Memory Writes</h4><p>4</p><small>learning records</small>",
        "id=\"tau-ops-harness-kpi-cost\" data-harness-kpi-card=\"runtime-cost-today\" data-kpi-value=\"$0.00\"",
        "<h4>Runtime Cost Today</h4><p>$0.00</p><small>Across 1 run</small>",
        "id=\"tau-ops-harness-active-missions\" data-active-count=\"1\" data-running-count=\"0\" data-blocked-count=\"0\"",
        "<h4>Benchmark Runs</h4>",
        "id=\"tau-ops-harness-learning-queue\" data-queue-count=\"1\"",
        "data-queue-source=\"state\"",
        "data-learning-id=\"PR-044\" data-status=\"proposal\" data-selected=\"true\" data-actionable=\"true\"",
        "href=\"/ops/harness?theme=dark&amp;sidebar=expanded&amp;session=default&amp;proposal_id=PR-044\" data-proposal-link=\"PR-044\" aria-current=\"page\"",
        "<span class=\"tau-harness-queue-label\">Prompt compression for research tasks</span>",
        "<span class=\"tau-harness-queue-status\">Proposal</span>",
        "Benchmark tasks passed 4/4",
        "data-gate-id=\"memory_write_proof\" data-gate-status=\"passed\"",
        "state proof loaded:",
        "id=\"tau-ops-harness-audit-log\" data-audit-row-count=\"1\" data-audit-source=\"state\"",
        "data-action=\"apply\" data-result=\"blocked_approval_required\" data-timestamp-unix-ms=\"",
        "Blocked Approval Required",
    ] {
        assert!(
            body.contains(marker),
            "missing state-backed harness marker `{marker}`"
        );
    }
    for stale_marker in [
        "data-mission-id=\"run_linux_ci\"",
        "data-mission-id=\"run_m334_flaky\"",
        "data-mission-id=\"run_receipts\"",
        "data-learning-id=\"LR-219\"",
        "data-learning-id=\"LR-220\"",
    ] {
        assert!(
            !body.contains(stale_marker),
            "state-backed harness route still rendered stale demo mission `{stale_marker}`"
        );
    }
    assert!(
        !body.contains(">ts:"),
        "state-backed harness audit rows should not expose raw timestamp labels"
    );

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
