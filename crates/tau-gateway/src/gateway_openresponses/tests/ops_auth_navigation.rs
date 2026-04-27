use super::*;

#[tokio::test]
async fn functional_dashboard_shell_endpoint_returns_html_shell() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");

    let client = Client::new();
    let response = client
        .get(format!("http://{addr}{DASHBOARD_SHELL_ENDPOINT}"))
        .send()
        .await
        .expect("dashboard shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    assert!(content_type.contains("text/html"));
    let body = response.text().await.expect("read dashboard shell body");
    assert!(body.contains("Tau Ops Dashboard"));
    assert!(body.contains("id=\"dashboard-shell-view-overview\""));
    assert!(body.contains("id=\"dashboard-shell-view-sessions\""));
    assert!(body.contains("id=\"dashboard-shell-view-memory\""));
    assert!(body.contains("id=\"dashboard-shell-view-configuration\""));
    assert!(body.contains("id=\"dashboardShellToken\""));
    assert!(body.contains("id=\"dashboardOverviewRefresh\""));
    assert!(body.contains("id=\"dashboardSessionsRefresh\""));
    assert!(body.contains("id=\"dashboardMemoryRefresh\""));
    assert!(body.contains("id=\"dashboardConfigurationRefresh\""));
    assert!(body.contains("id=\"dashboardOverviewOutput\""));
    assert!(body.contains("id=\"dashboardSessionsOutput\""));
    assert!(body.contains("id=\"dashboardMemoryOutput\""));
    assert!(body.contains("id=\"dashboardConfigurationOutput\""));
    assert!(body.contains("async function refreshOverviewView()"));
    assert!(body.contains("async function refreshSessionsView()"));
    assert!(body.contains("async function refreshMemoryView()"));
    assert!(body.contains("async function refreshConfigurationView()"));
    handle.abort();
}

#[tokio::test]
async fn functional_ops_dashboard_shell_endpoint_returns_leptos_foundation_shell() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");

    let client = Client::new();
    let response = client
        .get(format!("http://{addr}{OPS_DASHBOARD_ENDPOINT}"))
        .send()
        .await
        .expect("ops dashboard shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    assert!(content_type.contains("text/html"));
    let body = response
        .text()
        .await
        .expect("read ops dashboard shell body");
    assert!(body.contains("Tau Ops Dashboard"));
    assert!(body.contains("id=\"tau-ops-shell\""));
    assert!(body.contains("id=\"tau-ops-header\""));
    assert!(body.contains("id=\"tau-ops-sidebar\""));
    assert!(body.contains("id=\"tau-ops-command-center\""));
    assert!(body.contains("id=\"tau-ops-auth-shell\""));
    assert!(body.contains("data-active-route=\"ops\""));
    assert!(body.contains("data-component=\"HealthBadge\""));
    assert!(body.contains("data-component=\"StatCard\""));
    assert!(body.contains("data-component=\"AlertFeed\""));
    assert!(body.contains("data-component=\"DataTable\""));
    handle.abort();
}

#[tokio::test]
async fn functional_spec_2786_c01_gateway_auth_bootstrap_endpoint_reports_token_mode_contract() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");

    let client = Client::new();
    let response = client
        .get(format!("http://{addr}/gateway/auth/bootstrap"))
        .send()
        .await
        .expect("auth bootstrap request");
    assert_eq!(response.status(), StatusCode::OK);
    let payload = response
        .json::<Value>()
        .await
        .expect("parse auth bootstrap payload");
    assert_eq!(payload["auth_mode"], Value::String("token".to_string()));
    assert_eq!(payload["ui_auth_mode"], Value::String("token".to_string()));
    assert_eq!(payload["requires_authentication"], Value::Bool(true));
    assert_eq!(payload["ops_endpoint"], Value::String("/ops".to_string()));
    assert_eq!(
        payload["ops_login_endpoint"],
        Value::String("/ops/login".to_string())
    );
    assert_eq!(
        payload["auth_session_endpoint"],
        Value::String("/gateway/auth/session".to_string())
    );
    handle.abort();
}

#[tokio::test]
async fn functional_spec_2786_c02_gateway_auth_bootstrap_maps_localhost_dev_to_none_mode() {
    let temp = tempdir().expect("tempdir");
    let state = test_state_with_auth(
        temp.path(),
        4_096,
        GatewayOpenResponsesAuthMode::LocalhostDev,
        None,
        None,
        60,
        120,
    );
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");

    let client = Client::new();
    let response = client
        .get(format!("http://{addr}/gateway/auth/bootstrap"))
        .send()
        .await
        .expect("auth bootstrap request");
    assert_eq!(response.status(), StatusCode::OK);
    let payload = response
        .json::<Value>()
        .await
        .expect("parse auth bootstrap payload");
    assert_eq!(
        payload["auth_mode"],
        Value::String("localhost-dev".to_string())
    );
    assert_eq!(payload["ui_auth_mode"], Value::String("none".to_string()));
    assert_eq!(payload["requires_authentication"], Value::Bool(false));
    handle.abort();
}

#[tokio::test]
async fn functional_spec_3426_c02_gateway_auth_bootstrap_reports_password_session_mode_contract() {
    let temp = tempdir().expect("tempdir");
    let state = test_state_with_auth(
        temp.path(),
        4_096,
        GatewayOpenResponsesAuthMode::PasswordSession,
        None,
        Some("pw-secret"),
        60,
        120,
    );
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");

    let client = Client::new();
    let response = client
        .get(format!("http://{addr}{GATEWAY_AUTH_BOOTSTRAP_ENDPOINT}"))
        .send()
        .await
        .expect("auth bootstrap request");
    assert_eq!(response.status(), StatusCode::OK);
    let payload = response
        .json::<Value>()
        .await
        .expect("parse auth bootstrap payload");
    assert_eq!(
        payload["auth_mode"],
        Value::String("password-session".to_string())
    );
    assert_eq!(
        payload["ui_auth_mode"],
        Value::String("password-session".to_string())
    );
    assert_eq!(payload["requires_authentication"], Value::Bool(true));
    assert_eq!(
        payload["auth_session_endpoint"],
        Value::String(GATEWAY_AUTH_SESSION_ENDPOINT.to_string())
    );

    handle.abort();
}

#[tokio::test]
async fn functional_spec_2786_c04_ops_login_shell_endpoint_returns_login_route_markers() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");

    let client = Client::new();
    let response = client
        .get(format!("http://{addr}/ops/login"))
        .send()
        .await
        .expect("ops login shell request");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .text()
        .await
        .expect("read ops login dashboard shell body");
    assert!(body.contains("id=\"tau-ops-auth-shell\""));
    assert!(body.contains("data-active-route=\"login\""));
    assert!(body.contains("id=\"tau-ops-login-shell\""));
    assert!(body.contains("data-route=\"/ops/login\""));
    assert!(body.contains("id=\"tau-ops-protected-shell\""));
    handle.abort();
}

#[tokio::test]
async fn functional_spec_2790_c05_ops_routes_include_navigation_and_breadcrumb_markers() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let cases = [("/ops", "command-center"), ("/ops/login", "login")];

    for (route, breadcrumb_current) in cases {
        let response = client
            .get(format!("http://{addr}{route}"))
            .send()
            .await
            .expect("ops route request");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.text().await.expect("read ops route body");
        assert_eq!(body.matches("data-nav-item=").count(), 14);
        assert!(body.contains("id=\"tau-ops-breadcrumbs\""));
        assert!(body.contains("id=\"tau-ops-breadcrumb-current\""));
        assert!(body.contains(&format!("data-breadcrumb-current=\"{breadcrumb_current}\"")));
    }

    handle.abort();
}

#[tokio::test]
async fn functional_spec_2794_c01_c02_c03_all_sidebar_ops_routes_return_shell_with_route_markers() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let route_cases = [
        ("/ops", "ops", "command-center"),
        ("/ops/agents", "agents", "agent-fleet"),
        ("/ops/agents/default", "agent-detail", "agent-detail"),
        ("/ops/chat", "chat", "chat"),
        ("/ops/sessions", "sessions", "sessions"),
        ("/ops/memory", "memory", "memory"),
        ("/ops/memory-graph", "memory-graph", "memory-graph"),
        ("/ops/tools-jobs", "tools-jobs", "tools-jobs"),
        ("/ops/channels", "channels", "channels"),
        ("/ops/config", "config", "config"),
        ("/ops/training", "training", "training"),
        ("/ops/safety", "safety", "safety"),
        ("/ops/diagnostics", "diagnostics", "diagnostics"),
        ("/ops/deploy", "deploy", "deploy"),
    ];

    for (route, active_route, breadcrumb_current) in route_cases {
        let response = client
            .get(format!("http://{addr}{route}"))
            .send()
            .await
            .expect("ops route request");
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "route {route} should resolve"
        );
        let body = response.text().await.expect("read ops route body");
        assert!(body.contains("id=\"tau-ops-shell\""));
        assert!(body.contains(&format!("data-active-route=\"{active_route}\"")));
        assert!(body.contains("id=\"tau-ops-breadcrumbs\""));
        assert!(body.contains(&format!("data-breadcrumb-current=\"{breadcrumb_current}\"")));
        assert_eq!(body.matches("data-nav-item=").count(), 14);
    }

    handle.abort();
}
