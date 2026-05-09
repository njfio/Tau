use super::*;

#[tokio::test]
async fn functional_gateway_memory_graph_endpoint_returns_filtered_relations() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state.clone())
        .await
        .expect("spawn server");
    let session_key = "memory-graph-session";
    let memory_endpoint = expand_session_template(GATEWAY_MEMORY_ENDPOINT, session_key);
    let graph_endpoint = expand_session_template(GATEWAY_MEMORY_GRAPH_ENDPOINT, session_key);

    let client = Client::new();
    let write_ok = client
        .put(format!("http://{addr}{memory_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "content": "release checklist alpha\nrelease notes alpha\nincident runbook beta\n",
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("write memory");
    assert_eq!(write_ok.status(), StatusCode::OK);

    let graph_response = client
        .get(format!(
            "http://{addr}{graph_endpoint}?max_nodes=4&min_edge_weight=1&relation_types=contains,keyword_overlap"
        ))
        .bearer_auth("secret")
        .send()
        .await
        .expect("request graph");
    assert_eq!(graph_response.status(), StatusCode::OK);
    let payload = graph_response
        .json::<Value>()
        .await
        .expect("parse graph payload");

    assert_eq!(payload["session_key"], session_key);
    assert_eq!(payload["exists"], Value::Bool(true));
    assert!(payload["node_count"].as_u64().unwrap_or_default() >= 1);
    let edges = payload["edges"].as_array().expect("edges array");
    assert!(!edges.is_empty(), "expected at least one graph edge");
    for edge in edges {
        let relation = edge["relation_type"].as_str().unwrap_or_default();
        assert!(
            relation == "contains" || relation == "keyword_overlap",
            "unexpected relation type: {relation}"
        );
    }

    handle.abort();
}

#[tokio::test]
async fn regression_gateway_memory_graph_endpoint_reads_durable_memory_records() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state.clone())
        .await
        .expect("spawn server");
    let session_key = "memory-graph-records";
    let target_endpoint =
        expand_memory_entry_template(GATEWAY_MEMORY_ENTRY_ENDPOINT, session_key, "mem-target");
    let source_endpoint =
        expand_memory_entry_template(GATEWAY_MEMORY_ENTRY_ENDPOINT, session_key, "mem-source");
    let graph_endpoint = expand_session_template(GATEWAY_MEMORY_GRAPH_ENDPOINT, session_key);

    let client = Client::new();
    let target_create = client
        .put(format!("http://{addr}{target_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "summary": "Target memory record",
            "workspace_id": "workspace-graph-api",
            "channel_id": "channel-graph-api",
            "actor_id": "operator",
            "memory_type": "fact",
            "importance": 0.66,
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("create target memory record");
    assert_eq!(target_create.status(), StatusCode::CREATED);

    let source_create = client
        .put(format!("http://{addr}{source_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "summary": "Source memory record",
            "workspace_id": "workspace-graph-api",
            "channel_id": "channel-graph-api",
            "actor_id": "operator",
            "memory_type": "goal",
            "importance": 0.88,
            "relations": [{
                "target_id": "mem-target",
                "relation_type": "related_to",
                "weight": 0.77
            }],
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("create source memory record");
    assert_eq!(source_create.status(), StatusCode::CREATED);

    let graph_response = client
        .get(format!(
            "http://{addr}{graph_endpoint}?workspace_id=workspace-graph-api&channel_id=channel-graph-api&actor_id=operator&relation_types=related_to"
        ))
        .bearer_auth("secret")
        .send()
        .await
        .expect("request memory record graph");
    assert_eq!(graph_response.status(), StatusCode::OK);
    let payload = graph_response
        .json::<Value>()
        .await
        .expect("parse memory record graph payload");

    assert_eq!(payload["session_key"], session_key);
    assert_eq!(payload["exists"], Value::Bool(true));
    assert_eq!(payload["node_count"], Value::from(2));
    assert_eq!(payload["edge_count"], Value::from(1));
    assert_eq!(payload["nodes"][0]["id"], Value::from("mem-source"));
    assert_eq!(payload["nodes"][1]["id"], Value::from("mem-target"));
    assert_eq!(payload["edges"][0]["source"], Value::from("mem-source"));
    assert_eq!(payload["edges"][0]["target"], Value::from("mem-target"));
    assert_eq!(
        payload["edges"][0]["relation_type"],
        Value::from("related_to")
    );

    handle.abort();
}

#[tokio::test]
async fn integration_gateway_memory_graph_api_includes_harness_self_improvement_lineage() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let mission_dir = state
        .config
        .state_dir
        .clone()
        .join("ops-harness")
        .join("self-improvement")
        .join("PR-045");
    std::fs::create_dir_all(&mission_dir).expect("create harness mission dir");
    std::fs::write(
        mission_dir.join("mission.json"),
        serde_json::to_vec_pretty(&json!({
            "mission_id": "ops-harness-self-improve-pr-045",
            "goal": "Standardize benchmark artifact names through a skill update",
            "learning_records": [{
                "record_id": "LR-045",
                "summary": "Benchmark artifacts were hard to correlate with missions."
            }],
            "improvement_proposals": [{
                "proposal_id": "PR-045",
                "source_learning_record_id": "LR-045",
                "target_path": "skills/benchmark_artifacts/SKILL.md",
                "patch_summary": "Add a skill rule for deterministic benchmark artifact naming.",
                "dry_run": {
                    "passed": true
                },
                "applied_unix_ms": 1_700
            }]
        }))
        .expect("serialize harness mission"),
    )
    .expect("write harness mission");
    let (addr, handle) = spawn_test_server(state.clone())
        .await
        .expect("spawn server");
    let client = Client::new();
    let graph_endpoint = expand_session_template(GATEWAY_MEMORY_GRAPH_ENDPOINT, "default");

    for endpoint in [
        graph_endpoint.as_str(),
        "/api/memories/graph?session_key=default",
    ] {
        let graph_response = client
            .get(format!("http://{addr}{endpoint}"))
            .bearer_auth("secret")
            .send()
            .await
            .expect("request harness lineage graph");
        assert_eq!(graph_response.status(), StatusCode::OK);
        let payload = graph_response
            .json::<Value>()
            .await
            .expect("parse harness lineage graph payload");

        assert_eq!(payload["session_key"], Value::from("default"));
        assert_eq!(payload["exists"], Value::Bool(true));
        assert!(payload["node_count"].as_u64().unwrap_or_default() >= 6);
        assert!(payload["edge_count"].as_u64().unwrap_or_default() >= 5);
        let nodes = payload["nodes"].as_array().expect("nodes array");
        for node_id in [
            "ops-harness-self-improve-pr-045",
            "LR-045",
            "PR-045",
            "dry-run:PR-045",
            "apply:PR-045",
            "target:skills/benchmark_artifacts/SKILL.md",
        ] {
            assert!(
                nodes
                    .iter()
                    .any(|node| node["id"].as_str() == Some(node_id)),
                "missing harness node {node_id} from {endpoint}"
            );
        }
        let edges = payload["edges"].as_array().expect("edges array");
        assert!(edges.iter().any(|edge| {
            edge["source"].as_str() == Some("LR-045")
                && edge["target"].as_str() == Some("PR-045")
                && edge["relation_type"].as_str() == Some("supports")
        }));
        assert!(edges.iter().any(|edge| {
            edge["source"].as_str() == Some("PR-045")
                && edge["target"].as_str() == Some("target:skills/benchmark_artifacts/SKILL.md")
                && edge["relation_type"].as_str() == Some("updates")
        }));
    }

    handle.abort();
}
