use super::*;

#[derive(Debug, Default)]
struct FixtureHarnessSelfImprovementRunner;

impl GatewayOpsHarnessSelfImprovementRunner for FixtureHarnessSelfImprovementRunner {
    fn dry_run(
        &self,
        request: GatewayOpsHarnessSelfImprovementRequest,
    ) -> Result<GatewayOpsHarnessSelfImprovementResult> {
        let artifact_path = request
            .state_dir
            .join("ops-harness")
            .join("self-improvement")
            .join(&request.proposal_id)
            .join("dry-run-result.json");
        if let Some(parent) = artifact_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(
            &artifact_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "proposal_id": request.proposal_id.as_str(),
                "result": "passed",
                "summary": "fixture dry-run passed"
            }))?,
        )?;
        Ok(GatewayOpsHarnessSelfImprovementResult {
            proposal_id: request.proposal_id.clone(),
            mission_id: format!("mission-{}", request.proposal_id),
            target_path: "prompts/research_to_doc/system.md".to_string(),
            result_key: "passed".to_string(),
            summary: "fixture dry-run passed".to_string(),
            artifact_path: Some(artifact_path),
            applied: false,
        })
    }

    fn apply(
        &self,
        request: GatewayOpsHarnessSelfImprovementRequest,
    ) -> Result<GatewayOpsHarnessSelfImprovementResult> {
        let artifact_path = request
            .state_dir
            .join("ops-harness")
            .join("self-improvement")
            .join(&request.proposal_id)
            .join("apply-result.json");
        if let Some(parent) = artifact_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(
            &artifact_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "proposal_id": request.proposal_id.as_str(),
                "result": "applied",
                "summary": "fixture apply completed"
            }))?,
        )?;
        Ok(GatewayOpsHarnessSelfImprovementResult {
            proposal_id: request.proposal_id.clone(),
            mission_id: format!("mission-{}", request.proposal_id),
            target_path: "prompts/research_to_doc/system.md".to_string(),
            result_key: "applied".to_string(),
            summary: "fixture apply completed".to_string(),
            artifact_path: Some(artifact_path),
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
    let state_dir = state.config.state_dir.display().to_string();
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
        "id=\"tau-ops-harness-topbar\"",
        "data-model=\"openai/gpt-5.2\"",
        "data-transport=\"gateway\"",
        "data-health=\"unknown\"",
        "<span data-topbar-field=\"model\">openai/gpt-5.2</span>",
        "<span data-topbar-field=\"transport\">gateway</span>",
        "<span data-topbar-field=\"health\">Unknown</span>",
        "id=\"tau-ops-harness-new-mission-form\" action=\"/ops/harness/missions/draft?theme=dark&amp;sidebar=expanded&amp;session=ops-harness-contract&amp;proposal_id=PR-044\" method=\"post\" data-action-contract=\"durable-mission-draft\" data-preserves-shell-context=\"true\"",
        "id=\"tau-ops-harness-new-mission\" data-action=\"new-mission\" data-action-contract=\"durable-mission-draft\" data-preserves-session=\"true\" data-preserves-proposal=\"true\" type=\"submit\"",
        "id=\"tau-ops-harness-history\" data-action=\"history\" data-action-contract=\"context-preserving\" data-preserves-session=\"true\" data-preserves-proposal=\"true\" href=\"/ops/harness?theme=dark&amp;sidebar=expanded&amp;session=ops-harness-contract&amp;proposal_id=PR-044&amp;view=history\"",
        "id=\"tau-ops-harness-run-benchmark-form\" action=\"/ops/harness/run-benchmark?theme=dark&amp;sidebar=expanded&amp;session=ops-harness-contract&amp;proposal_id=PR-044\" method=\"post\" data-command=\"tau_agent_harness\" data-preserves-shell-context=\"true\"",
        "id=\"tau-ops-harness-conservative-policy\" data-policy=\"conservative-self-improvement\" data-allowed-targets=\"skill,config,prompt\" data-blocked-targets=\"source-code,safety-policy\"",
        "id=\"tau-ops-harness-apply-form\" action=\"/ops/harness/proposals/PR-044/apply?theme=dark&amp;sidebar=expanded&amp;session=ops-harness-contract\" method=\"post\" data-approval-state=\"approval-required\"",
        "id=\"tau-ops-harness-action-apply\" type=\"submit\" data-action=\"apply\" data-action-tone=\"disabled\" data-disabled=\"true\" aria-disabled=\"true\" data-approval-required=\"true\" disabled",
        "id=\"tau-ops-harness-tui-companion\" data-component=\"TuiCompanion\" data-command=\"tau status\"",
    ] {
        assert!(
            body.contains(marker),
            "missing harness gateway marker `{marker}`"
        );
    }
    assert!(
        body.contains(&format!("data-workspace=\"{state_dir}\"")),
        "harness topbar should use gateway state dir instead of demo workspace"
    );
    assert!(
        body.contains(&format!(
            "<span data-topbar-field=\"workspace\">{state_dir}</span>"
        )),
        "harness topbar should visibly expose gateway state dir"
    );

    let history_response = client
        .get(format!(
            "http://{addr}/ops/harness?theme=dark&sidebar=expanded&session=ops-harness-contract&proposal_id=PR-044&view=history"
        ))
        .send()
        .await
        .expect("load ops harness history route");
    assert_eq!(history_response.status(), StatusCode::OK);
    let history_body = history_response
        .text()
        .await
        .expect("read ops harness history body");
    assert!(history_body.contains(
        "id=\"tau-ops-harness-route-action\" data-route-action-key=\"history\" data-route-action-label=\"Applied History\""
    ));
    assert!(history_body.contains("data-route-action-visible=\"true\""));
    assert!(history_body.contains(
        "id=\"tau-ops-harness-history-view\" data-history-view=\"true\" data-history-source=\"fallback\" data-history-row-count=\"4\" data-history-total-count=\"4\" data-history-proof-count=\"0\" data-history-action-filter=\"all\" data-history-selected-proposal=\"PR-044\""
    ));
    assert!(history_body.contains("data-history-filter-action=\"dry-run\""));
    assert!(history_body.contains("data-history-audit-anchor=\"true\""));

    let draft_response = client
        .get(format!(
            "http://{addr}/ops/harness?theme=dark&sidebar=expanded&session=ops-harness-contract&proposal_id=PR-044&intent=new-mission"
        ))
        .send()
        .await
        .expect("load ops harness new mission route");
    assert_eq!(draft_response.status(), StatusCode::OK);
    let draft_body = draft_response
        .text()
        .await
        .expect("read ops harness new mission body");
    assert!(draft_body.contains(
        "id=\"tau-ops-harness-route-action\" data-route-action-key=\"new-mission\" data-route-action-label=\"New Mission Draft\""
    ));
    assert!(draft_body.contains("data-route-action-visible=\"true\""));

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

    let draft_response = client
        .post(format!("http://{addr}/ops/harness/missions/draft"))
        .query(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", "ops-harness-context"),
            ("proposal_id", "PR-045"),
        ])
        .send()
        .await
        .expect("create harness mission draft");
    assert_eq!(draft_response.status(), StatusCode::SEE_OTHER);
    let draft_location = draft_response
        .headers()
        .get(reqwest::header::LOCATION)
        .expect("draft redirect location")
        .to_str()
        .expect("draft location is utf8");
    assert!(draft_location.contains("mission_status=draft_created"));
    assert!(draft_location.contains("theme=light"));
    assert!(draft_location.contains("sidebar=collapsed"));
    assert!(draft_location.contains("session=ops-harness-context"));
    assert!(draft_location.contains("proposal_id=PR-045"));
    let mission_id = draft_location
        .split("mission_id=")
        .nth(1)
        .and_then(|value| value.split('&').next())
        .expect("mission_id query param");
    let mission_path = state_dir
        .join("ops-harness")
        .join("missions")
        .join(mission_id)
        .join("mission.json");
    let mission_json = std::fs::read_to_string(&mission_path).expect("read mission draft");
    let mission: serde_json::Value = serde_json::from_str(&mission_json).expect("mission json");
    assert_eq!(mission["mission_id"], mission_id);
    assert_eq!(mission["session_key"], "ops-harness-context");
    assert_eq!(mission["status"], "draft");
    assert!(mission["goal"]
        .as_str()
        .expect("mission goal")
        .contains("PR-045"));
    assert_eq!(
        mission["acceptance_criteria"]
            .as_array()
            .expect("acceptance criteria")
            .len(),
        3
    );
    assert_eq!(mission["plan_dag"].as_array().expect("plan dag").len(), 5);
    assert!(
        mission["tool_budget"]["max_tool_calls"]
            .as_u64()
            .unwrap_or_default()
            > 0
    );
    assert_eq!(
        mission["verification_gates"]
            .as_array()
            .expect("verification gates")
            .len(),
        3
    );
    assert_eq!(
        mission["checkpoints"]
            .as_array()
            .expect("checkpoints")
            .len(),
        1
    );
    assert!(mission["memory_hits"]
        .as_array()
        .expect("memory hits")
        .is_empty());
    assert!(mission["artifacts"]
        .as_array()
        .expect("artifacts")
        .iter()
        .any(|artifact| artifact["artifact_id"] == "mission-json"));
    assert!(mission["final_learning_output"].is_null());

    let draft_harness_response = client
        .get(format!("http://{addr}{draft_location}"))
        .send()
        .await
        .expect("load draft harness redirect");
    assert_eq!(draft_harness_response.status(), StatusCode::OK);
    let draft_harness_body = draft_harness_response
        .text()
        .await
        .expect("draft harness body");
    assert!(draft_harness_body.contains(
        "id=\"tau-ops-harness-route-action\" data-route-action-key=\"mission-draft\" data-route-action-label=\"Mission Draft Created\""
    ));
    assert!(draft_harness_body.contains(&format!("data-mission-id=\"{mission_id}\"")));
    assert!(draft_harness_body.contains("data-status=\"draft\""));

    let benchmark_response = client
        .post(format!("http://{addr}/ops/harness/run-benchmark"))
        .query(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", "ops-harness-context"),
            ("proposal_id", "PR-045"),
        ])
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
    assert!(benchmark_location.contains("theme=light"));
    assert!(benchmark_location.contains("sidebar=collapsed"));
    assert!(benchmark_location.contains("session=ops-harness-context"));
    assert!(benchmark_location.contains("proposal_id=PR-045"));

    let proof_path = state_dir.join("ops-harness/m334/latest.json");
    let proof_json = std::fs::read_to_string(&proof_path).expect("read benchmark proof");
    let proof: serde_json::Value =
        serde_json::from_str(&proof_json).expect("benchmark proof is json");
    assert_eq!(proof["benchmark_id"], "m334-tranche-one-autonomy");
    assert_eq!(proof["passed"], true);
    assert_eq!(proof["tasks"].as_array().expect("task array").len(), 4);
    let artifact_response = client
        .get(format!(
            "http://{addr}/ops/harness/artifacts/ops-harness/m334/latest.json"
        ))
        .send()
        .await
        .expect("read harness benchmark artifact");
    assert_eq!(artifact_response.status(), StatusCode::OK);
    assert_eq!(
        artifact_response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/json; charset=utf-8")
    );
    let artifact_payload: serde_json::Value = serde_json::from_str(
        artifact_response
            .text()
            .await
            .expect("read benchmark artifact body")
            .as_str(),
    )
    .expect("benchmark artifact endpoint returns json");
    assert_eq!(
        artifact_payload["benchmark_id"],
        "m334-tranche-one-autonomy"
    );
    let artifact_view_response = client
        .get(format!(
            "http://{addr}/ops/harness/artifacts/view/ops-harness/m334/latest.json"
        ))
        .send()
        .await
        .expect("read harness benchmark artifact view");
    assert_eq!(artifact_view_response.status(), StatusCode::OK);
    let artifact_view_body = artifact_view_response
        .text()
        .await
        .expect("read benchmark artifact view body");
    assert!(artifact_view_body.contains("id=\"tau-ops-harness-artifact-view\""));
    assert!(artifact_view_body.contains("data-artifact-path=\"ops-harness/m334/latest.json\""));
    assert!(artifact_view_body.contains("m334-tranche-one-autonomy"));

    let harness_memory_records = gateway_memory_store(&state_dir, "ops-harness-context")
        .list_latest_records(
            Some(&tau_memory::runtime::MemoryScopeFilter {
                workspace_id: Some("ops-harness-context".to_string()),
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

    let graph_endpoint =
        expand_session_template(GATEWAY_MEMORY_GRAPH_ENDPOINT, "ops-harness-context");
    let graph_response = client
        .get(format!(
            "http://{addr}{graph_endpoint}?workspace_id=ops-harness-context&channel_id=tau-agent-harness&actor_id=tau&memory_type=decision"
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
    assert!(diff_body.contains(
        "href=\"/ops/harness?theme=dark&sidebar=expanded&session=default&proposal_id=PR-044\""
    ));
    assert!(diff_body
        .contains("Compress system prompt by removing redundant instructions and examples."));
    assert!(diff_body.contains("class=\"tau-harness-diff-line tau-harness-diff-line-remove\""));
    assert!(diff_body.contains("class=\"tau-harness-diff-line tau-harness-diff-line-add\""));

    handle.abort();
}

#[tokio::test]
async fn integration_ops_harness_active_missions_keep_latest_mission_per_proposal() {
    fn write_mission(
        state_dir: &std::path::Path,
        mission_id: &str,
        proposal_id: &str,
        status: &str,
        updated_unix_ms: u64,
        gate_statuses: &[&str],
    ) {
        let mission_dir = state_dir
            .join("ops-harness")
            .join("missions")
            .join(mission_id);
        std::fs::create_dir_all(&mission_dir).expect("mission dir");
        let verification_gates = gate_statuses
            .iter()
            .enumerate()
            .map(|(index, status)| {
                serde_json::json!({
                    "id": format!("VG-{}", index + 1),
                    "description": format!("Gate {}", index + 1),
                    "status": status
                })
            })
            .collect::<Vec<_>>();
        std::fs::write(
            mission_dir.join("mission.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "mission_id": mission_id,
                "proposal_id": proposal_id,
                "status": status,
                "goal": format!("{proposal_id} harness mission"),
                "updated_unix_ms": updated_unix_ms,
                "acceptance_criteria": [
                    {"id": "AC-1", "verification_gate_ids": ["VG-1"]},
                    {"id": "AC-2", "verification_gate_ids": ["VG-2"]}
                ],
                "plan_dag": [
                    {"id": "plan", "status": "completed"},
                    {"id": "execute", "status": if status == "completed" { "completed" } else { "pending" }}
                ],
                "verification_gates": verification_gates,
                "tool_budget": {
                    "consumed_tool_calls": 2,
                    "max_tool_calls": 40
                },
                "memory_hits": [{"key": "learning", "summary": "mission memory"}],
                "artifacts": [
                    {
                        "artifact_id": "mission-json",
                        "kind": "mission-state",
                        "path": format!("ops-harness/missions/{mission_id}/mission.json")
                    }
                ],
                "checkpoints": [
                    {"summary": format!("{mission_id} checkpoint")}
                ]
            }))
            .expect("mission json"),
        )
        .expect("write mission");
    }

    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let state_dir = state.config.state_dir.clone();
    write_mission(
        &state_dir,
        "mission-stale-awaiting",
        "PR-045",
        "awaiting_approval",
        1_000,
        &["passed", "pending"],
    );
    write_mission(
        &state_dir,
        "mission-latest-completed",
        "PR-045",
        "completed",
        2_000,
        &["passed", "passed"],
    );
    write_mission(
        &state_dir,
        "mission-other-awaiting",
        "PR-044",
        "awaiting_approval",
        1_500,
        &["passed", "pending"],
    );

    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();
    let harness_response = client
        .get(format!(
            "http://{addr}/ops/harness?proposal_id=PR-045&theme=dark&sidebar=expanded"
        ))
        .send()
        .await
        .expect("load harness with duplicate proposal missions");
    assert_eq!(harness_response.status(), StatusCode::OK);
    let harness_body = harness_response.text().await.expect("harness body");

    assert_eq!(
        harness_body
            .matches("id=\"tau-ops-harness-mission-row-")
            .count(),
        2,
        "active missions should keep one latest row per proposal"
    );
    assert!(harness_body
        .contains("data-mission-id=\"mission-latest-completed\" data-status=\"completed\""));
    assert!(harness_body
        .contains("data-mission-id=\"mission-other-awaiting\" data-status=\"awaiting_approval\""));
    assert!(
        !harness_body.contains("mission-stale-awaiting"),
        "stale mission attempts for the same proposal should not remain active"
    );

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

    let draft = client
        .post(format!("http://{addr}/ops/harness/missions/draft"))
        .query(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", "ops-harness-context"),
            ("proposal_id", "PR-044"),
        ])
        .send()
        .await
        .expect("create mission draft");
    assert_eq!(draft.status(), StatusCode::SEE_OTHER);
    let draft_location = draft
        .headers()
        .get(reqwest::header::LOCATION)
        .expect("draft redirect")
        .to_str()
        .expect("draft location");
    let mission_id = draft_location
        .split("mission_id=")
        .nth(1)
        .and_then(|value| value.split('&').next())
        .expect("mission id from draft redirect")
        .to_string();
    let start = client
        .post(format!(
            "http://{addr}/ops/harness/missions/{mission_id}/start"
        ))
        .query(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", "ops-harness-context"),
            ("proposal_id", "PR-044"),
        ])
        .send()
        .await
        .expect("start mission draft");
    assert_eq!(start.status(), StatusCode::SEE_OTHER);
    let start_location = start
        .headers()
        .get(reqwest::header::LOCATION)
        .expect("start redirect")
        .to_str()
        .expect("start location");
    assert!(start_location.contains("mission_status=mission_started"));
    assert!(start_location.contains("theme=light"));
    assert!(start_location.contains("sidebar=collapsed"));
    assert!(start_location.contains("session=ops-harness-context"));
    assert!(start_location.contains("proposal_id=PR-044"));
    assert!(start_location.contains(&format!("mission_id={mission_id}")));
    let mission_path = state_dir
        .join("ops-harness")
        .join("missions")
        .join(&mission_id)
        .join("mission.json");
    let mission: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&mission_path).expect("read started mission"),
    )
    .expect("started mission json");
    assert_eq!(mission["status"], "awaiting_approval");
    assert_eq!(mission["proposal_id"], "PR-044");
    assert_eq!(
        mission["linked_self_improvement_mission_id"],
        "mission-PR-044"
    );
    assert_eq!(mission["verification_gates"][0]["status"], "passed");
    assert_eq!(mission["verification_gates"][1]["status"], "passed");
    assert!(mission["final_learning_output"].is_null());
    assert!(mission["tool_evidence"]
        .as_array()
        .expect("tool evidence")
        .iter()
        .any(|tool| tool["tool_name"] == "self_modification.dry_run"));
    assert!(mission["recovery_state"]["next_action"]
        .as_str()
        .expect("next action")
        .contains("approve"));
    let started_harness = client
        .get(format!("http://{addr}{start_location}"))
        .send()
        .await
        .expect("load started harness");
    assert_eq!(started_harness.status(), StatusCode::OK);
    let started_harness_body = started_harness.text().await.expect("started harness body");
    assert!(started_harness_body.contains(
        "id=\"tau-ops-harness-route-action\" data-route-action-key=\"mission-start\" data-route-action-label=\"Mission Started\""
    ));
    assert!(started_harness_body.contains(&format!("data-mission-id=\"{mission_id}\"")));
    assert!(started_harness_body.contains("data-status=\"awaiting_approval\""));
    assert!(started_harness_body.contains("data-mission-state-chip=\"awaiting_approval\""));

    let dry_run = client
        .post(format!(
            "http://{addr}/ops/harness/proposals/PR-044/dry-run"
        ))
        .query(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", "ops-harness-context"),
        ])
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
    let dry_run_location = dry_run
        .headers()
        .get(reqwest::header::LOCATION)
        .expect("dry-run redirect")
        .to_str()
        .expect("dry-run location");
    assert!(dry_run_location.contains("theme=light"));
    assert!(dry_run_location.contains("sidebar=collapsed"));
    assert!(dry_run_location.contains("session=ops-harness-context"));
    assert!(dry_run_location.contains("proposal_id=PR-044"));
    let dry_run_harness = client
        .get(format!("http://{addr}{dry_run_location}"))
        .send()
        .await
        .expect("load dry-run status harness route");
    assert_eq!(dry_run_harness.status(), StatusCode::OK);
    let dry_run_harness_body = dry_run_harness.text().await.expect("dry-run harness body");
    for marker in [
        "id=\"tau-ops-harness-route-action\" data-route-action-key=\"proposal-dry-run\" data-route-action-label=\"Dry Run Passed\"",
        "data-route-action-visible=\"true\"",
        "Proposal PR-044 | session ops-harness-context | selected PR-044",
        "data-action=\"dry-run\" data-result=\"passed\"",
        "data-audit-proof-artifact=\"ops-harness/self-improvement/PR-044/dry-run-result.json\"",
        "href=\"/ops/harness/artifacts/view/ops-harness/self-improvement/PR-044/dry-run-result.json\"",
        "Proof ops-harness/self-improvement/PR-044/dry-run-result.json",
    ] {
        assert!(
            dry_run_harness_body.contains(marker),
            "dry-run redirect should render visible proposal action marker `{marker}`"
        );
    }

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
        "id=\"tau-ops-harness-apply-form\" action=\"/ops/harness/proposals/PR-044/apply?theme=dark&amp;sidebar=expanded&amp;session=default\" method=\"post\" data-approval-state=\"approved\""
    ));
    assert!(harness_body.contains(
        "id=\"tau-ops-harness-action-apply\" type=\"submit\" data-action=\"apply\" data-action-tone=\"disabled\" data-disabled=\"false\" aria-disabled=\"false\""
    ));

    let apply = client
        .post(format!("http://{addr}/ops/harness/proposals/PR-044/apply"))
        .query(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", "ops-harness-context"),
        ])
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
    let apply_location = apply
        .headers()
        .get(reqwest::header::LOCATION)
        .expect("apply redirect")
        .to_str()
        .expect("apply location");
    assert!(apply_location.contains("theme=light"));
    assert!(apply_location.contains("sidebar=collapsed"));
    assert!(apply_location.contains("session=ops-harness-context"));
    assert!(apply_location.contains("proposal_id=PR-044"));

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
        "id=\"tau-ops-harness-apply-form\" action=\"/ops/harness/proposals/PR-044/apply?theme=dark&amp;sidebar=expanded&amp;session=default\" method=\"post\" data-approval-state=\"applied\""
    ));
    assert!(applied_harness_body.contains(
        "id=\"tau-ops-harness-action-apply\" type=\"submit\" data-action=\"apply\" data-action-tone=\"disabled\" data-disabled=\"true\" aria-disabled=\"true\""
    ));
    assert!(applied_harness_body.contains("data-action=\"apply\" data-result=\"applied\""));
    assert!(applied_harness_body.contains(
        "data-audit-proof-artifact=\"ops-harness/self-improvement/PR-044/apply-result.json\""
    ));
    assert!(applied_harness_body.contains(
        "href=\"/ops/harness/artifacts/view/ops-harness/self-improvement/PR-044/apply-result.json\""
    ));
    assert!(applied_harness_body
        .contains("Proof ops-harness/self-improvement/PR-044/apply-result.json"));

    let audit_log =
        std::fs::read_to_string(state_dir.join("ops-harness/audit.jsonl")).expect("audit log");
    assert!(audit_log.contains("\"action\":\"start-mission\""));
    assert!(audit_log.contains(&format!("\"mission_id\":\"{mission_id}\"")));
    assert!(audit_log.contains(&format!(
        "\"proof_artifact\":\"ops-harness/missions/{mission_id}/mission.json\""
    )));
    assert!(audit_log.contains("\"action\":\"dry-run\""));
    assert!(audit_log.contains("\"result\":\"passed\""));
    assert!(audit_log.contains(
        "\"proof_artifact\":\"ops-harness/self-improvement/PR-044/dry-run-result.json\""
    ));
    assert!(audit_log.contains("\"action\":\"apply\""));
    assert!(audit_log.contains("\"result\":\"applied\""));
    assert!(audit_log
        .contains("\"proof_artifact\":\"ops-harness/self-improvement/PR-044/apply-result.json\""));

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
    std::fs::write(
        mission_dir.join("dry-run-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "proposal_id": "PR-045",
            "result": "passed",
            "summary": "Dry-run passed for deterministic benchmark artifact naming."
        }))
        .expect("dry-run proof json"),
    )
    .expect("write dry-run proof");
    std::fs::write(
        state_dir.join("ops-harness").join("audit.jsonl"),
        concat!(
            "{\"action\":\"dry-run\",\"proposal_id\":\"PR-045\",\"result\":\"passed\",",
            "\"proof_artifact\":\"ops-harness/self-improvement/PR-045/dry-run-result.json\",",
            "\"timestamp_unix_ms\":1778419581966}\n"
        ),
    )
    .expect("write latest dry-run audit");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();
    let no_redirect_client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build no-redirect client");

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
        "id=\"tau-ops-harness-approve-form\" action=\"/ops/harness/proposals/PR-045/approve?theme=dark&amp;sidebar=expanded&amp;session=default\" method=\"post\"",
        "id=\"tau-ops-harness-proposal-detail\" data-proposal-id=\"PR-045\" data-learning-record=\"LR-045\" data-target-type=\"Skill\" data-target-path=\"skills/benchmark_artifacts/SKILL.md\"",
        "PR-045 Skill patch for benchmark artifact naming",
        "Add a skill rule for deterministic benchmark artifact naming.",
        "id=\"tau-ops-harness-learning-queue\" data-queue-count=\"1\"",
        "data-queue-source=\"state\"",
        "data-learning-id=\"PR-045\" data-status=\"completed\" data-selected=\"true\" data-actionable=\"true\"",
        "href=\"/ops/harness?theme=dark&amp;sidebar=expanded&amp;session=default&amp;proposal_id=PR-045\" data-proposal-link=\"PR-045\" aria-current=\"page\"",
        "<span class=\"tau-harness-queue-label\">Skill patch for benchmark artifact naming</span>",
        "<span class=\"tau-harness-queue-status\">Completed</span>",
        "id=\"tau-ops-harness-operator-actions\" data-apply-requires-approval=\"true\" data-action-row-priority=\"approval-flow\" data-action-grid=\"two-column-priority\" data-action-first-viewport=\"all-controls\" data-review-state=\"applied\" data-terminal-state=\"true\" data-selected-proposal=\"PR-045\"",
        "id=\"tau-ops-harness-action-approve\" type=\"submit\" data-action=\"approve\" data-action-tone=\"approve\" data-action-state=\"applied\" data-disabled=\"true\" aria-disabled=\"true\" disabled",
        "id=\"tau-ops-harness-action-reject\" type=\"submit\" data-action=\"reject\" data-action-tone=\"reject\" data-action-state=\"applied\" data-disabled=\"true\" aria-disabled=\"true\" disabled",
        "id=\"tau-ops-harness-action-apply\" type=\"submit\" data-action=\"apply\" data-action-tone=\"disabled\" data-disabled=\"true\" aria-disabled=\"true\" data-approval-required=\"true\" disabled",
        "id=\"tau-ops-harness-self-improvement-proof\" data-proof-source=\"state\" data-mission-id=\"ops-harness-self-improve-pr-045\" data-mission-status=\"completed\" data-plan-completed=\"5\" data-plan-total=\"5\" data-gates-passed=\"3\" data-gates-total=\"3\" data-memory-hits=\"1\" data-artifact-count=\"3\" data-final-learning-records=\"LR-045\"",
        "Applied PR-045 and updated curator state for LR-045.",
        "<dt>Test Evidence</dt><dd><a href=\"/ops/harness/artifacts/view/ops-harness/self-improvement/PR-045/dry-run-result.json\">ops-harness/self-improvement/PR-045/dry-run-result.json</a></dd>",
        "data-action=\"dry-run\" data-result=\"passed\" data-timestamp-unix-ms=\"1778419581966\" data-audit-detail-label=\"\" data-audit-detail-value=\"\" data-audit-proof-artifact=\"ops-harness/self-improvement/PR-045/dry-run-result.json\"",
        "<td>Skill</td><td data-audit-item-cell=\"item-proof\"><span>PR-045</span>",
        "href=\"/ops/harness/artifacts/view/ops-harness/self-improvement/PR-045/dry-run-result.json\"",
        "data-proof-row=\"artifact\" data-proof-id=\"dry-run-result\" data-proof-status=\"self-improvement-dry-run\"><a href=\"/ops/harness/artifacts/view/ops-harness/self-improvement/PR-045/dry-run-result.json\" data-proof-artifact-href=\"true\" data-proof-artifact-path=\"ops-harness/self-improvement/PR-045/dry-run-result.json\">ops-harness/self-improvement/PR-045/dry-run-result.json</a></li>",
        "data-proof-row=\"artifact\" data-proof-id=\"apply-result\" data-proof-status=\"self-improvement-apply\"><a href=\"/ops/harness/artifacts/view/ops-harness/self-improvement/PR-045/apply-result.json\" data-proof-artifact-href=\"true\" data-proof-artifact-path=\"ops-harness/self-improvement/PR-045/apply-result.json\">ops-harness/self-improvement/PR-045/apply-result.json</a></li>",
        "data-proof-row=\"artifact\" data-proof-id=\"target:skills/benchmark_artifacts/SKILL.md\" data-proof-status=\"skill\"><span>skills/benchmark_artifacts/SKILL.md</span></li>",
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
        "<td>Prompt</td><td data-audit-item-cell=\"item-proof\"><span>PR-045</span>",
    ] {
        assert!(
            !harness_body.contains(stale_marker),
            "state-backed selected proposal queue rendered stale row `{stale_marker}`"
        );
    }

    let evidence_response = client
        .get(format!(
            "http://{addr}/ops/harness/artifacts/view/ops-harness/self-improvement/PR-045/dry-run-result.json"
        ))
        .send()
        .await
        .expect("load selected proposal evidence artifact");
    assert_eq!(evidence_response.status(), StatusCode::OK);
    let evidence_body = evidence_response.text().await.expect("evidence body");
    for marker in [
        "id=\"tau-ops-harness-artifact-view\"",
        "ops-harness/self-improvement/PR-045/dry-run-result.json",
        "Dry-run passed for deterministic benchmark artifact naming.",
    ] {
        assert!(
            evidence_body.contains(marker),
            "missing selected proposal evidence marker `{marker}`"
        );
    }

    let draft = no_redirect_client
        .post(format!("http://{addr}/ops/harness/missions/draft"))
        .query(&[
            ("theme", "dark"),
            ("sidebar", "expanded"),
            ("session", "default"),
            ("proposal_id", "PR-045"),
        ])
        .send()
        .await
        .expect("create draft for completed proposal");
    assert_eq!(draft.status(), StatusCode::SEE_OTHER);
    let draft_location = draft
        .headers()
        .get(reqwest::header::LOCATION)
        .expect("draft redirect")
        .to_str()
        .expect("draft location");
    let mission_id = draft_location
        .split("mission_id=")
        .nth(1)
        .and_then(|value| value.split('&').next())
        .expect("draft mission id");
    let start = no_redirect_client
        .post(format!(
            "http://{addr}/ops/harness/missions/{mission_id}/start"
        ))
        .query(&[
            ("theme", "dark"),
            ("sidebar", "expanded"),
            ("session", "default"),
            ("proposal_id", "PR-045"),
        ])
        .send()
        .await
        .expect("start completed proposal draft");
    assert_eq!(start.status(), StatusCode::SEE_OTHER);
    let start_location = start
        .headers()
        .get(reqwest::header::LOCATION)
        .expect("start redirect")
        .to_str()
        .expect("start location");
    assert!(start_location.contains("mission_status=mission_completed"));
    assert!(start_location.contains(&format!("mission_id={mission_id}")));
    let draft_mission_path = state_dir
        .join("ops-harness")
        .join("missions")
        .join(mission_id)
        .join("mission.json");
    let draft_mission: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&draft_mission_path).expect("read completed draft mission"),
    )
    .expect("completed draft mission json");
    assert_eq!(draft_mission["status"], "completed");
    assert_eq!(
        draft_mission["linked_self_improvement_mission_id"],
        "ops-harness-self-improve-pr-045"
    );
    assert_eq!(draft_mission["verification_gates"][2]["status"], "passed");
    assert!(draft_mission["recovery_state"].is_null());
    assert_eq!(
        draft_mission["final_learning_output"]["summary"],
        "Applied PR-045 and updated curator state for LR-045."
    );
    let selected_mission_response = client
        .get(format!("http://{addr}{start_location}"))
        .send()
        .await
        .expect("load selected completed mission harness");
    assert_eq!(selected_mission_response.status(), StatusCode::OK);
    let selected_mission_body = selected_mission_response
        .text()
        .await
        .expect("selected mission harness body");
    for marker in [
        format!(
            "id=\"tau-ops-harness-proof-window\" data-window=\"mission-detail-proof-view\" data-window-order=\"2\" data-run-id=\"{mission_id}\" data-mission-status=\"completed\" data-tool-budget=\"1/40\""
        ),
        format!("ops-harness/missions/{mission_id}/mission.json"),
        "PR-045 Skill patch for benchmark artifact naming".to_string(),
        "data-tool=\"self_improvement.completed_proof\" data-status=\"passed\"".to_string(),
        "data-gate-id=\"VG-LEARN\" data-gate-status=\"passed\"".to_string(),
        "mission state loaded:".to_string(),
        format!("mission={mission_id}"),
        "Mission Proof".to_string(),
        "Acceptance: 3/3".to_string(),
        "Gates: 3/3 passed".to_string(),
        "Proof:".to_string(),
    ] {
        assert!(
            selected_mission_body.contains(&marker),
            "selected mission route should render durable mission detail marker `{marker}`"
        );
    }

    let diff_response = client
        .get(format!(
            "http://{addr}/ops/harness/proposals/PR-045/diff?theme=light&sidebar=collapsed&session=ops-harness-context"
        ))
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
        "href=\"/ops/harness?theme=light&sidebar=collapsed&session=ops-harness-context&proposal_id=PR-045\"",
    ] {
        assert!(
            diff_body.contains(marker),
            "missing registry-backed diff marker `{marker}`"
        );
    }

    let missing_response = client
        .get(format!(
            "http://{addr}/ops/harness/proposals/PR-999/diff?theme=light&sidebar=collapsed&session=ops-harness-context"
        ))
        .send()
        .await
        .expect("load missing proposal diff");
    assert_eq!(missing_response.status(), StatusCode::NOT_FOUND);
    let missing_body = missing_response.text().await.expect("missing diff body");
    assert!(missing_body.contains("id=\"tau-ops-harness-diff-missing\""));
    assert!(missing_body.contains(
        "href=\"/ops/harness?theme=light&sidebar=collapsed&session=ops-harness-context&proposal_id=PR-999\""
    ));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit() {
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
        .expect("run benchmark");
    assert_eq!(benchmark_response.status(), StatusCode::SEE_OTHER);
    let benchmark_audit_log = std::fs::read_to_string(state_dir.join("ops-harness/audit.jsonl"))
        .expect("benchmark audit log");
    for marker in [
        "\"action\":\"run-benchmark\"",
        "\"result\":\"passed\"",
        "\"benchmark_id\":\"m334-tranche-one-autonomy\"",
        "\"run_id\":\"gateway-harness-",
        "\"proof_artifact\":\"ops-harness/m334/latest.json\"",
    ] {
        assert!(
            benchmark_audit_log.contains(marker),
            "benchmark run should be audited with marker `{marker}`"
        );
    }
    let benchmark_audit_ref = benchmark_audit_log
        .lines()
        .find_map(|line| {
            let record = serde_json::from_str::<serde_json::Value>(line).ok()?;
            (record.get("action").and_then(serde_json::Value::as_str) == Some("run-benchmark"))
                .then(|| {
                    record
                        .get("timestamp_unix_ms")
                        .and_then(serde_json::Value::as_u64)
                        .map(|timestamp| timestamp.to_string())
                })
                .flatten()
        })
        .expect("benchmark audit timestamp");

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
        "href=\"/ops/harness/artifacts/view/ops-harness/m334/latest.json\"",
        "Benchmark proof artifact",
        "data-tool-artifact-href=\"/ops/harness/artifacts/view/ops-harness/m334/latest.json\"",
        "<a href=\"/ops/harness/artifacts/view/ops-harness/m334/latest.json\" data-tool-proof-artifact-href=\"true\">proof</a>",
        "data-gate-id=\"memory_write_proof\" data-gate-status=\"passed\"",
        "state proof loaded:",
        "id=\"tau-ops-harness-audit-log\" data-audit-row-count=\"2\" data-audit-source=\"state\"",
        "data-action=\"apply\" data-result=\"blocked_approval_required\" data-timestamp-unix-ms=\"",
        "data-action=\"run-benchmark\" data-result=\"passed\" data-timestamp-unix-ms=\"",
        "data-audit-detail-label=\"Run\" data-audit-detail-value=\"gateway-harness-",
        "data-audit-proof-artifact=\"ops-harness/m334/latest.json\"",
        "href=\"/ops/harness/artifacts/view/ops-harness/m334/latest.json\"",
        "Run gateway-harness-",
        "Proof ops-harness/m334/latest.json",
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

    let history_response = client
        .get(format!(
            "http://{addr}/ops/harness?theme=dark&sidebar=expanded&session=default&proposal_id=PR-044&view=history"
        ))
        .send()
        .await
        .expect("load state-backed harness history route");
    assert_eq!(history_response.status(), StatusCode::OK);
    let history_body = history_response.text().await.expect("read history body");
    for marker in [
        "id=\"tau-ops-harness-history-view\" data-history-view=\"true\" data-history-source=\"state\" data-history-row-count=\"2\" data-history-total-count=\"2\" data-history-proof-count=\"1\" data-history-action-filter=\"all\" data-history-selected-proposal=\"PR-044\"",
        "data-history-latest-action=\"Apply PR-044 Blocked Approval Required\"",
        "data-history-filter-action=\"all\" aria-current=\"page\"",
        "id=\"tau-ops-harness-history-detail\" data-history-selected-audit=\"true\"",
        "data-selected-action=\"apply\" data-selected-result=\"blocked_approval_required\"",
        "data-audit-inspect-link=\"true\"",
        "data-history-audit-anchor=\"true\"",
        "data-audit-row=\"true\"",
    ] {
        assert!(
            history_body.contains(marker),
            "missing state-backed history marker `{marker}`"
        );
    }

    let filtered_history_response = client
        .get(format!(
            "http://{addr}/ops/harness?theme=dark&sidebar=expanded&session=default&proposal_id=PR-044&view=history&audit_action=run-benchmark&audit_ref={benchmark_audit_ref}"
        ))
        .send()
        .await
        .expect("load filtered state-backed harness history route");
    assert_eq!(filtered_history_response.status(), StatusCode::OK);
    let filtered_history_body = filtered_history_response
        .text()
        .await
        .expect("read filtered history body");
    for marker in [
        "id=\"tau-ops-harness-history-view\" data-history-view=\"true\" data-history-source=\"state\" data-history-row-count=\"1\" data-history-total-count=\"2\" data-history-proof-count=\"1\" data-history-action-filter=\"run-benchmark\" data-history-selected-proposal=\"PR-044\"",
        "data-history-filter-action=\"run-benchmark\" aria-current=\"page\"",
        "data-action=\"run-benchmark\" data-result=\"passed\"",
        "data-audit-detail-label=\"Run\" data-audit-detail-value=\"gateway-harness-",
        "data-history-selected-proof-link=\"true\" data-history-selected-proof-artifact=\"ops-harness/m334/latest.json\"",
    ] {
        assert!(
            filtered_history_body.contains(marker),
            "missing filtered state-backed history marker `{marker}`"
        );
    }
    for marker in [
        format!(
            "data-history-selected-audit-ref=\"{benchmark_audit_ref}\" data-selected-action=\"run-benchmark\" data-selected-result=\"passed\" data-selected-proof-artifact=\"ops-harness/m334/latest.json\""
        ),
        format!(
            "data-audit-ref=\"{benchmark_audit_ref}\" data-audit-selected=\"true\""
        ),
        format!("audit_ref={benchmark_audit_ref}"),
    ] {
        assert!(
            filtered_history_body.contains(&marker),
            "missing selected audit detail marker `{marker}`"
        );
    }
    assert!(
        !filtered_history_body.contains("<tr data-action=\"apply\""),
        "filtered history should not include apply rows when audit_action=run-benchmark"
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
