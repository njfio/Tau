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
