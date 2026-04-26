use super::*;

#[tokio::test]
async fn functional_gateway_sessions_endpoints_support_list_detail_append_and_reset() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state.clone())
        .await
        .expect("spawn server");
    let session_key = "functional-session";

    let client = Client::new();
    let empty_list = client
        .get(format!("http://{addr}{GATEWAY_SESSIONS_ENDPOINT}"))
        .bearer_auth("secret")
        .send()
        .await
        .expect("request empty session list");
    assert_eq!(empty_list.status(), StatusCode::OK);
    let empty_payload = empty_list
        .json::<Value>()
        .await
        .expect("parse empty list payload");
    assert!(empty_payload["sessions"]
        .as_array()
        .expect("sessions array")
        .is_empty());

    let append_without_gate = client
        .post(format!(
            "http://{addr}{}",
            expand_session_template(GATEWAY_SESSION_APPEND_ENDPOINT, session_key)
        ))
        .bearer_auth("secret")
        .json(&json!({"role":"user","content":"hello"}))
        .send()
        .await
        .expect("append without policy gate");
    assert_eq!(append_without_gate.status(), StatusCode::FORBIDDEN);

    let append_response = client
        .post(format!(
            "http://{addr}{}",
            expand_session_template(GATEWAY_SESSION_APPEND_ENDPOINT, session_key)
        ))
        .bearer_auth("secret")
        .json(&json!({
            "role": "user",
            "content": "hello from session admin",
            "policy_gate": SESSION_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("append with policy gate");
    assert_eq!(append_response.status(), StatusCode::OK);

    let detail_response = client
        .get(format!(
            "http://{addr}{}",
            expand_session_template(GATEWAY_SESSION_DETAIL_ENDPOINT, session_key)
        ))
        .bearer_auth("secret")
        .send()
        .await
        .expect("fetch session detail");
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_payload = detail_response
        .json::<Value>()
        .await
        .expect("parse detail payload");
    assert_eq!(detail_payload["session_key"].as_str(), Some(session_key));
    assert!(detail_payload["entry_count"].as_u64().unwrap_or_default() >= 2);

    let list_response = client
        .get(format!("http://{addr}{GATEWAY_SESSIONS_ENDPOINT}"))
        .bearer_auth("secret")
        .send()
        .await
        .expect("request populated session list");
    let list_payload = list_response
        .json::<Value>()
        .await
        .expect("parse list payload");
    assert!(list_payload["sessions"]
        .as_array()
        .expect("sessions array")
        .iter()
        .any(|entry| entry["session_key"] == session_key));

    let reset_response = client
        .post(format!(
            "http://{addr}{}",
            expand_session_template(GATEWAY_SESSION_RESET_ENDPOINT, session_key)
        ))
        .bearer_auth("secret")
        .json(&json!({"policy_gate": SESSION_WRITE_POLICY_GATE}))
        .send()
        .await
        .expect("reset session");
    assert_eq!(reset_response.status(), StatusCode::OK);
    let reset_payload = reset_response
        .json::<Value>()
        .await
        .expect("parse reset payload");
    assert_eq!(reset_payload["reset"], Value::Bool(true));

    let detail_after_reset = client
        .get(format!(
            "http://{addr}{}",
            expand_session_template(GATEWAY_SESSION_DETAIL_ENDPOINT, session_key)
        ))
        .bearer_auth("secret")
        .send()
        .await
        .expect("fetch detail after reset");
    assert_eq!(detail_after_reset.status(), StatusCode::NOT_FOUND);

    let session_path = gateway_session_path(&state.config.state_dir, session_key);
    assert!(!session_path.exists());
    handle.abort();
}
