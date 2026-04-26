use super::*;

#[tokio::test]
async fn functional_spec_2798_c04_ops_shell_exposes_responsive_and_theme_contract_markers() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!("http://{addr}/ops"))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");
    assert!(body.contains("id=\"tau-ops-shell-controls\""));
    assert!(body.contains("id=\"tau-ops-sidebar-toggle\""));
    assert!(body.contains("id=\"tau-ops-sidebar-hamburger\""));
    assert!(body.contains("data-sidebar-mobile-default=\"collapsed\""));
    assert!(body.contains("data-sidebar-state=\"expanded\""));
    assert!(body.contains("data-theme=\"dark\""));
    assert!(body.contains("id=\"tau-ops-theme-toggle-dark\""));
    assert!(body.contains("id=\"tau-ops-theme-toggle-light\""));

    handle.abort();
}

#[tokio::test]
async fn functional_spec_2802_c01_c02_ops_routes_apply_query_control_state_markers() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let route_cases = [
        ("/ops?theme=light&sidebar=collapsed", "ops"),
        ("/ops/chat?theme=light&sidebar=collapsed", "chat"),
        (
            "/ops/agents/default?theme=light&sidebar=collapsed",
            "agent-detail",
        ),
    ];

    for (route, active_route) in route_cases {
        let response = client
            .get(format!("http://{addr}{route}"))
            .send()
            .await
            .expect("ops shell request");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.text().await.expect("read ops shell body");
        assert!(body.contains(&format!("data-active-route=\"{active_route}\"")));
        assert!(body.contains("data-theme=\"light\""));
        assert!(body.contains("data-sidebar-state=\"collapsed\""));
        assert!(body.contains("aria-expanded=\"false\""));
    }

    handle.abort();
}

#[tokio::test]
async fn functional_spec_2802_c03_invalid_query_control_values_fall_back_to_defaults() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!("http://{addr}/ops?theme=banana&sidebar=sideways"))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops shell body");
    assert!(body.contains("data-theme=\"dark\""));
    assert!(body.contains("data-sidebar-state=\"expanded\""));
    assert!(body.contains("aria-expanded=\"true\""));

    handle.abort();
}
