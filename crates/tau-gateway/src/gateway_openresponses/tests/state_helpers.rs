use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn test_state_with_client_and_auth(
    root: &Path,
    max_input_chars: usize,
    client: Arc<dyn LlmClient>,
    tool_registrar: Arc<dyn GatewayToolRegistrar>,
    auth_mode: GatewayOpenResponsesAuthMode,
    token: Option<&str>,
    password: Option<&str>,
    rate_limit_window_seconds: u64,
    rate_limit_max_requests: usize,
) -> Arc<GatewayOpenResponsesServerState> {
    Arc::new(GatewayOpenResponsesServerState::new(
        GatewayOpenResponsesServerConfig {
            client,
            model: "openai/gpt-5.2".to_string(),
            model_input_cost_per_million: Some(10.0),
            model_cached_input_cost_per_million: None,
            model_output_cost_per_million: Some(20.0),
            system_prompt: "You are Tau.".to_string(),
            available_skills: Vec::new(),
            explicit_skill_names: Vec::new(),
            max_turns: 4,
            tool_registrar,
            turn_timeout_ms: 0,
            session_lock_wait_ms: 500,
            session_lock_stale_ms: 10_000,
            state_dir: root.join(".tau/gateway"),
            bind: "127.0.0.1:0".to_string(),
            auth_mode,
            auth_token: token.map(str::to_string),
            auth_password: password.map(str::to_string),
            session_ttl_seconds: 3_600,
            rate_limit_window_seconds,
            rate_limit_max_requests,
            max_input_chars,
            runtime_heartbeat: RuntimeHeartbeatSchedulerConfig {
                enabled: false,
                interval: std::time::Duration::from_secs(5),
                state_path: root.join(".tau/runtime-heartbeat/state.json"),
                ..RuntimeHeartbeatSchedulerConfig::default()
            },
            external_coding_agent_bridge: tau_runtime::ExternalCodingAgentBridgeConfig::default(),
            delegated_tool_execution: false,
        },
    ))
}

pub(super) fn test_state_with_auth(
    root: &Path,
    max_input_chars: usize,
    auth_mode: GatewayOpenResponsesAuthMode,
    token: Option<&str>,
    password: Option<&str>,
    rate_limit_window_seconds: u64,
    rate_limit_max_requests: usize,
) -> Arc<GatewayOpenResponsesServerState> {
    test_state_with_client_and_auth(
        root,
        max_input_chars,
        Arc::new(MockGatewayLlmClient::default()),
        Arc::new(NoopGatewayToolRegistrar),
        auth_mode,
        token,
        password,
        rate_limit_window_seconds,
        rate_limit_max_requests,
    )
}

pub(super) fn test_state_with_fixture_tools(
    root: &Path,
    max_input_chars: usize,
    token: &str,
) -> Arc<GatewayOpenResponsesServerState> {
    test_state_with_client_and_auth(
        root,
        max_input_chars,
        Arc::new(MockGatewayLlmClient::default()),
        Arc::new(FixtureGatewayToolRegistrar),
        GatewayOpenResponsesAuthMode::Token,
        Some(token),
        None,
        60,
        120,
    )
}

pub(super) fn test_state_with_fixture_mcp_tools(
    root: &Path,
    max_input_chars: usize,
    token: &str,
) -> Arc<GatewayOpenResponsesServerState> {
    test_state_with_client_and_auth(
        root,
        max_input_chars,
        Arc::new(MockGatewayLlmClient::default()),
        Arc::new(FixtureGatewayMcpToolRegistrar),
        GatewayOpenResponsesAuthMode::Token,
        Some(token),
        None,
        60,
        120,
    )
}

pub(super) fn test_state(
    root: &Path,
    max_input_chars: usize,
    token: &str,
) -> Arc<GatewayOpenResponsesServerState> {
    test_state_with_auth(
        root,
        max_input_chars,
        GatewayOpenResponsesAuthMode::Token,
        Some(token),
        None,
        60,
        120,
    )
}

pub(super) async fn spawn_test_server(
    state: Arc<GatewayOpenResponsesServerState>,
) -> Result<(SocketAddr, tokio::task::JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind ephemeral listener")?;
    let addr = listener.local_addr().context("resolve listener addr")?;
    let app = build_gateway_openresponses_router(state);
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    tokio::time::sleep(Duration::from_millis(20)).await;
    Ok((addr, handle))
}
