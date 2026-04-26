use super::*;

#[tokio::test]
async fn functional_gateway_memory_endpoint_supports_read_and_policy_gated_write() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state.clone())
        .await
        .expect("spawn server");
    let session_key = "memory-session";
    let endpoint = expand_session_template(GATEWAY_MEMORY_ENDPOINT, session_key);

    let client = Client::new();
    let read_empty = client
        .get(format!("http://{addr}{endpoint}"))
        .bearer_auth("secret")
        .send()
        .await
        .expect("read empty memory");
    assert_eq!(read_empty.status(), StatusCode::OK);
    let read_empty_payload = read_empty
        .json::<Value>()
        .await
        .expect("parse empty memory payload");
    assert_eq!(read_empty_payload["exists"], Value::Bool(false));

    let write_forbidden = client
        .put(format!("http://{addr}{endpoint}"))
        .bearer_auth("secret")
        .json(&json!({"content":"memory text"}))
        .send()
        .await
        .expect("write memory without policy gate");
    assert_eq!(write_forbidden.status(), StatusCode::FORBIDDEN);

    let write_ok = client
        .put(format!("http://{addr}{endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "content": "memory text",
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("write memory");
    assert_eq!(write_ok.status(), StatusCode::OK);

    let read_written = client
        .get(format!("http://{addr}{endpoint}"))
        .bearer_auth("secret")
        .send()
        .await
        .expect("read written memory");
    assert_eq!(read_written.status(), StatusCode::OK);
    let read_written_payload = read_written
        .json::<Value>()
        .await
        .expect("parse written memory payload");
    assert_eq!(read_written_payload["exists"], Value::Bool(true));
    assert!(read_written_payload["content"]
        .as_str()
        .unwrap_or_default()
        .contains("memory text"));

    let memory_path = state
        .config
        .state_dir
        .join("openresponses")
        .join("memory")
        .join(format!("{session_key}.md"));
    assert!(memory_path.exists());
    handle.abort();
}
