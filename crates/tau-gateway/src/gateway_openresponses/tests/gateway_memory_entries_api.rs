use super::*;

#[tokio::test]
async fn integration_spec_2667_c01_memory_entry_endpoints_support_crud_search_and_legacy_compatibility(
) {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state.clone())
        .await
        .expect("spawn server");
    let session_key = "memory-entry-session";
    let legacy_endpoint = expand_session_template(GATEWAY_MEMORY_ENDPOINT, session_key);
    let entry_endpoint =
        expand_memory_entry_template(GATEWAY_MEMORY_ENTRY_ENDPOINT, session_key, "mem-entry-1");
    let second_entry_endpoint =
        expand_memory_entry_template(GATEWAY_MEMORY_ENTRY_ENDPOINT, session_key, "mem-entry-2");

    let client = Client::new();

    let create_without_gate = client
        .put(format!("http://{addr}{entry_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "summary": "Tau uses ArcSwap for lock-free hot reload.",
            "memory_type": "fact"
        }))
        .send()
        .await
        .expect("create memory entry without policy gate");
    assert_eq!(create_without_gate.status(), StatusCode::FORBIDDEN);

    let create_first = client
        .put(format!("http://{addr}{entry_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "summary": "Tau uses ArcSwap for lock-free hot reload.",
            "tags": ["rust", "arcswap"],
            "facts": ["hot reload"],
            "source_event_key": "evt-memory-1",
            "workspace_id": "workspace-a",
            "channel_id": "gateway",
            "actor_id": "operator",
            "memory_type": "fact",
            "importance": 0.91,
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("create first memory entry");
    assert_eq!(create_first.status(), StatusCode::CREATED);

    let create_second = client
        .put(format!("http://{addr}{second_entry_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "summary": "Ship the dashboard migration safely.",
            "tags": ["ops", "migration"],
            "facts": ["phase-1 foundation"],
            "source_event_key": "evt-memory-2",
            "workspace_id": "workspace-a",
            "channel_id": "gateway",
            "actor_id": "operator",
            "memory_type": "goal",
            "importance": 0.82,
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("create second memory entry");
    assert_eq!(create_second.status(), StatusCode::CREATED);

    let read_first = client
        .get(format!("http://{addr}{entry_endpoint}"))
        .bearer_auth("secret")
        .send()
        .await
        .expect("read first memory entry");
    assert_eq!(read_first.status(), StatusCode::OK);
    let read_first_payload = read_first
        .json::<Value>()
        .await
        .expect("parse first memory entry response");
    assert_eq!(
        read_first_payload["entry"]["memory_id"].as_str(),
        Some("mem-entry-1")
    );
    assert_eq!(
        read_first_payload["entry"]["memory_type"].as_str(),
        Some("fact")
    );

    let search_fact = client
        .get(format!(
            "http://{addr}{legacy_endpoint}?query=ArcSwap&workspace_id=workspace-a&memory_type=fact&limit=25"
        ))
        .bearer_auth("secret")
        .send()
        .await
        .expect("search fact entries");
    assert_eq!(search_fact.status(), StatusCode::OK);
    let search_fact_payload = search_fact
        .json::<Value>()
        .await
        .expect("parse search payload");
    assert_eq!(search_fact_payload["mode"].as_str(), Some("search"));
    let matches = search_fact_payload["matches"]
        .as_array()
        .expect("search matches array");
    assert!(!matches.is_empty(), "expected at least one filtered match");
    assert!(
        matches
            .iter()
            .all(|item| item["memory_type"].as_str() == Some("fact")),
        "memory_type filter should keep only fact entries"
    );

    let delete_without_gate = client
        .delete(format!("http://{addr}{entry_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({}))
        .send()
        .await
        .expect("delete entry without policy gate");
    assert_eq!(delete_without_gate.status(), StatusCode::FORBIDDEN);

    let delete_ok = client
        .delete(format!("http://{addr}{entry_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("delete entry with policy gate");
    assert_eq!(delete_ok.status(), StatusCode::OK);
    let delete_payload = delete_ok
        .json::<Value>()
        .await
        .expect("parse delete response");
    assert_eq!(delete_payload["deleted"], Value::Bool(true));

    let read_deleted = client
        .get(format!("http://{addr}{entry_endpoint}"))
        .bearer_auth("secret")
        .send()
        .await
        .expect("read deleted entry");
    assert_eq!(read_deleted.status(), StatusCode::NOT_FOUND);

    let legacy_write = client
        .put(format!("http://{addr}{legacy_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "content": "legacy memory payload",
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("legacy memory write");
    assert_eq!(legacy_write.status(), StatusCode::OK);
    let legacy_read = client
        .get(format!("http://{addr}{legacy_endpoint}"))
        .bearer_auth("secret")
        .send()
        .await
        .expect("legacy memory read");
    assert_eq!(legacy_read.status(), StatusCode::OK);
    let legacy_payload = legacy_read
        .json::<Value>()
        .await
        .expect("parse legacy memory payload");
    assert!(legacy_payload["content"]
        .as_str()
        .unwrap_or_default()
        .contains("legacy memory payload"));

    handle.abort();
}

#[tokio::test]
async fn regression_spec_2667_c05_memory_entry_endpoints_reject_unauthorized_requests() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let endpoint =
        expand_memory_entry_template(GATEWAY_MEMORY_ENTRY_ENDPOINT, "unauthorized-session", "e1");

    let client = Client::new();
    let get_response = client
        .get(format!("http://{addr}{endpoint}"))
        .send()
        .await
        .expect("unauthorized get");
    assert_eq!(get_response.status(), StatusCode::UNAUTHORIZED);

    let put_response = client
        .put(format!("http://{addr}{endpoint}"))
        .json(&json!({
            "summary": "unauthorized write",
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("unauthorized put");
    assert_eq!(put_response.status(), StatusCode::UNAUTHORIZED);

    let delete_response = client
        .delete(format!("http://{addr}{endpoint}"))
        .json(&json!({
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("unauthorized delete");
    assert_eq!(delete_response.status(), StatusCode::UNAUTHORIZED);

    handle.abort();
}
