use super::*;

#[tokio::test]
async fn integration_spec_2670_c01_channel_lifecycle_endpoint_supports_logout_and_status_contract()
{
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");

    let client = Client::new();
    let telegram_endpoint = expand_channel_template(GATEWAY_CHANNEL_LIFECYCLE_ENDPOINT, "telegram");
    let discord_endpoint = expand_channel_template(GATEWAY_CHANNEL_LIFECYCLE_ENDPOINT, "discord");

    let logout_response = client
        .post(format!("http://{addr}{telegram_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({"action":"logout"}))
        .send()
        .await
        .expect("telegram logout action");
    assert_eq!(logout_response.status(), StatusCode::OK);
    let logout_payload = logout_response
        .json::<Value>()
        .await
        .expect("parse logout payload");
    assert_eq!(logout_payload["report"]["action"], "logout");
    assert_eq!(logout_payload["report"]["channel"], "telegram");
    assert_eq!(logout_payload["report"]["lifecycle_status"], "logged_out");
    assert_eq!(
        logout_payload["report"]["state_persisted"],
        Value::Bool(true)
    );

    let lifecycle_state_path = temp
        .path()
        .join(".tau")
        .join("multi-channel")
        .join("security")
        .join("channel-lifecycle.json");
    assert!(lifecycle_state_path.exists());

    let status_response = client
        .post(format!("http://{addr}{discord_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({"action":"status"}))
        .send()
        .await
        .expect("discord status action");
    assert_eq!(status_response.status(), StatusCode::OK);
    let status_payload = status_response
        .json::<Value>()
        .await
        .expect("parse lifecycle status payload");
    assert_eq!(status_payload["report"]["action"], "status");
    assert_eq!(status_payload["report"]["channel"], "discord");

    let gateway_status = client
        .get(format!("http://{addr}{GATEWAY_STATUS_ENDPOINT}"))
        .bearer_auth("secret")
        .send()
        .await
        .expect("gateway status");
    assert_eq!(gateway_status.status(), StatusCode::OK);
    let gateway_status_payload = gateway_status
        .json::<Value>()
        .await
        .expect("parse gateway status payload");
    assert_eq!(
        gateway_status_payload["gateway"]["web_ui"]["channel_lifecycle_endpoint"],
        GATEWAY_CHANNEL_LIFECYCLE_ENDPOINT
    );

    handle.abort();
}

#[tokio::test]
async fn regression_spec_2670_c04_channel_lifecycle_endpoint_rejects_invalid_channel_action_and_auth(
) {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let endpoint = expand_channel_template(GATEWAY_CHANNEL_LIFECYCLE_ENDPOINT, "telegram");

    let client = Client::new();
    let unauthorized = client
        .post(format!("http://{addr}{endpoint}"))
        .json(&json!({"action":"logout"}))
        .send()
        .await
        .expect("unauthorized lifecycle action");
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

    let invalid_channel = client
        .post(format!(
            "http://{addr}{}",
            expand_channel_template(GATEWAY_CHANNEL_LIFECYCLE_ENDPOINT, "unknown")
        ))
        .bearer_auth("secret")
        .json(&json!({"action":"status"}))
        .send()
        .await
        .expect("invalid channel request");
    assert_eq!(invalid_channel.status(), StatusCode::BAD_REQUEST);
    let invalid_channel_payload = invalid_channel
        .json::<Value>()
        .await
        .expect("parse invalid channel payload");
    assert_eq!(invalid_channel_payload["error"]["code"], "invalid_channel");

    let invalid_action = client
        .post(format!("http://{addr}{endpoint}"))
        .bearer_auth("secret")
        .json(&json!({"action":"warp-speed"}))
        .send()
        .await
        .expect("invalid action request");
    assert_eq!(invalid_action.status(), StatusCode::BAD_REQUEST);
    let invalid_action_payload = invalid_action
        .json::<Value>()
        .await
        .expect("parse invalid action payload");
    assert_eq!(
        invalid_action_payload["error"]["code"],
        "invalid_lifecycle_action"
    );

    handle.abort();
}
