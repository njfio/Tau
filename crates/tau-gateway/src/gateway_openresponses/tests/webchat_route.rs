use super::*;

#[tokio::test]
async fn functional_webchat_endpoint_returns_html_shell() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");

    let client = Client::new();
    let response = client
        .get(format!("http://{addr}{WEBCHAT_ENDPOINT}"))
        .send()
        .await
        .expect("send request");

    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(content_type.contains("text/html"));
    let body = response.text().await.expect("read webchat body");
    assert!(body.contains("Tau Gateway Webchat"));
    assert!(body.contains(OPENRESPONSES_ENDPOINT));
    assert!(body.contains(GATEWAY_STATUS_ENDPOINT));
    assert!(body.contains(GATEWAY_WS_ENDPOINT));
    assert!(body.contains("Connector Channels"));
    assert!(body.contains("Reason Code Counts"));
    assert!(body.contains("Dashboard"));
    assert!(body.contains("Live Dashboard"));
    assert!(body.contains("Dashboard Alerts"));
    assert!(body.contains("Dashboard Queue Timeline"));
    assert!(body.contains("Dashboard Widgets"));
    assert!(body.contains("Cortex"));
    assert!(body.contains("id=\"view-cortex\""));
    assert!(body.contains("id=\"cortexPrompt\""));
    assert!(body.contains("id=\"cortexOutput\""));
    assert!(body.contains("id=\"cortexStatus\""));
    assert!(body.contains("Routines"));
    assert!(body.contains("id=\"view-routines\""));
    assert!(body.contains("id=\"routinesStatus\""));
    assert!(body.contains("id=\"routinesDiagnostics\""));
    assert!(body.contains("id=\"routinesJobsTableBody\""));
    assert!(body.contains("id=\"dashboardStatus\""));
    assert!(body.contains("id=\"dashboardLive\""));
    assert!(body.contains("id=\"dashboardActionReason\""));
    assert!(body.contains("Sessions"));
    assert!(body.contains("Memory"));
    assert!(body.contains("Configuration"));
    assert!(body.contains("Memory Graph"));
    assert!(body.contains("id=\"memoryGraphCanvas\""));
    assert!(body.contains("id=\"healthStateValue\""));
    assert!(body.contains("multi-channel lifecycle summary"));
    assert!(body.contains("connector counters"));
    assert!(body.contains("recent reason codes"));

    handle.abort();
}
