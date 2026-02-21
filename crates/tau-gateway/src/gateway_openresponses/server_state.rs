//! Gateway server config/state types and core state helpers.

use super::*;

#[derive(Clone)]
/// Public struct `GatewayOpenResponsesServerConfig` used across Tau components.
pub struct GatewayOpenResponsesServerConfig {
    pub client: Arc<dyn LlmClient>,
    pub model: String,
    pub model_input_cost_per_million: Option<f64>,
    pub model_cached_input_cost_per_million: Option<f64>,
    pub model_output_cost_per_million: Option<f64>,
    pub system_prompt: String,
    pub max_turns: usize,
    pub tool_registrar: Arc<dyn GatewayToolRegistrar>,
    pub turn_timeout_ms: u64,
    pub session_lock_wait_ms: u64,
    pub session_lock_stale_ms: u64,
    pub state_dir: PathBuf,
    pub bind: String,
    pub auth_mode: GatewayOpenResponsesAuthMode,
    pub auth_token: Option<String>,
    pub auth_password: Option<String>,
    pub session_ttl_seconds: u64,
    pub rate_limit_window_seconds: u64,
    pub rate_limit_max_requests: usize,
    pub max_input_chars: usize,
    pub runtime_heartbeat: RuntimeHeartbeatSchedulerConfig,
    pub external_coding_agent_bridge: ExternalCodingAgentBridgeConfig,
}

#[derive(Clone)]
pub(super) struct GatewayOpenResponsesServerState {
    pub(super) config: GatewayOpenResponsesServerConfig,
    pub(super) response_sequence: Arc<AtomicU64>,
    pub(super) auth_runtime: Arc<Mutex<GatewayAuthRuntimeState>>,
    pub(super) compat_runtime: Arc<Mutex<GatewayOpenAiCompatRuntimeState>>,
    pub(super) ui_telemetry_runtime: Arc<Mutex<GatewayUiTelemetryRuntimeState>>,
    pub(super) external_coding_agent_bridge: Arc<ExternalCodingAgentBridge>,
    pub(super) cortex: Arc<Cortex>,
}

impl GatewayOpenResponsesServerState {
    pub(super) fn new(config: GatewayOpenResponsesServerConfig) -> Self {
        let external_coding_agent_bridge = Arc::new(ExternalCodingAgentBridge::new(
            config.external_coding_agent_bridge.clone(),
        ));
        let cortex = Arc::new(Cortex::new(CortexConfig::new(gateway_memory_stores_root(
            &config.state_dir,
        ))));
        Self {
            config,
            response_sequence: Arc::new(AtomicU64::new(0)),
            auth_runtime: Arc::new(Mutex::new(GatewayAuthRuntimeState::default())),
            compat_runtime: Arc::new(Mutex::new(GatewayOpenAiCompatRuntimeState::default())),
            ui_telemetry_runtime: Arc::new(Mutex::new(GatewayUiTelemetryRuntimeState::default())),
            external_coding_agent_bridge,
            cortex,
        }
    }

    pub(super) fn next_sequence(&self) -> u64 {
        self.response_sequence.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub(super) fn next_response_id(&self) -> String {
        format!("resp_{:016x}", self.next_sequence())
    }

    pub(super) fn next_output_message_id(&self) -> String {
        format!("msg_{:016x}", self.next_sequence())
    }

    pub(super) fn resolved_system_prompt(&self) -> String {
        self.cortex
            .compose_system_prompt(self.config.system_prompt.as_str())
    }
}
