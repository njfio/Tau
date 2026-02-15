# M23 Wave2 Undocumented Public API Hotspots (Baseline)

Total undocumented public items: **257**

## Crate Summary

| Crate | Undocumented public items |
| --- | ---: |
| tau-provider | 92 |
| tau-multi-channel | 79 |
| tau-gateway | 47 |
| tau-ops | 39 |

## tau-provider

Undocumented items: **92**

| File | Line | Kind | Signature |
| --- | ---: | --- | --- |
| `crates/tau-provider/src/auth.rs` | 98 | `fn` | `pub fn provider_auth_capability(` |
| `crates/tau-provider/src/auth.rs` | 113 | `fn` | `pub fn provider_supported_auth_modes(provider: Provider) -> Vec<ProviderAuthMethod> {` |
| `crates/tau-provider/src/auth.rs` | 121 | `fn` | `pub fn configured_provider_auth_method(cli: &Cli, provider: Provider) -> ProviderAuthMethod {` |
| `crates/tau-provider/src/auth.rs` | 129 | `fn` | `pub fn configured_provider_auth_method_from_config(` |
| `crates/tau-provider/src/auth.rs` | 140 | `fn` | `pub fn provider_auth_mode_flag(provider: Provider) -> &'static str {` |
| `crates/tau-provider/src/auth.rs` | 148 | `fn` | `pub fn missing_provider_api_key_message(provider: Provider) -> &'static str {` |
| `crates/tau-provider/src/auth.rs` | 162 | `fn` | `pub fn provider_api_key_candidates_with_inputs(` |
| `crates/tau-provider/src/auth.rs` | 203 | `fn` | `pub fn provider_api_key_candidates(` |
| `crates/tau-provider/src/auth.rs` | 216 | `fn` | `pub fn resolve_api_key(candidates: Vec<Option<String>>) -> Option<String> {` |
| `crates/tau-provider/src/auth.rs` | 223 | `fn` | `pub fn provider_api_key_candidates_from_auth_config(` |
| `crates/tau-provider/src/auth.rs` | 236 | `fn` | `pub fn provider_login_access_token_candidates(` |
| `crates/tau-provider/src/auth.rs` | 273 | `fn` | `pub fn provider_login_refresh_token_candidates(` |
| `crates/tau-provider/src/auth.rs` | 310 | `fn` | `pub fn provider_login_expires_candidates(` |
| `crates/tau-provider/src/auth.rs` | 347 | `fn` | `pub fn resolve_auth_login_expires_unix(provider: Provider) -> Result<Option<u64>> {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 18 | `const` | `pub const AUTH_USAGE: &str = "usage: /auth <login\|reauth\|status\|logout\|matrix> ...";` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 19 | `const` | `pub const AUTH_LOGIN_USAGE: &str =` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 21 | `const` | `pub const AUTH_REAUTH_USAGE: &str =` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 23 | `const` | `pub const AUTH_STATUS_USAGE: &str =` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 25 | `const` | `pub const AUTH_LOGOUT_USAGE: &str = "usage: /auth logout <provider> [--json]";` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 26 | `const` | `pub const AUTH_MATRIX_USAGE: &str =` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 38 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 56 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 76 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 96 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 217 | `fn` | `pub fn parse_auth_provider(token: &str) -> Result<Provider> {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 231 | `fn` | `pub fn parse_provider_auth_method_token(token: &str) -> Result<ProviderAuthMethod> {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 244 | `fn` | `pub fn parse_auth_command(command_args: &str) -> Result<AuthCommand> {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 578 | `fn` | `pub fn parse_auth_matrix_availability_filter(token: &str) -> Result<AuthMatrixAvailabilityFilter> {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 590 | `fn` | `pub fn parse_auth_matrix_mode_support_filter(token: &str) -> Result<AuthMatrixModeSupportFilter> {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 602 | `fn` | `pub fn parse_auth_matrix_state_filter(token: &str) -> Result<String> {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 610 | `fn` | `pub fn parse_auth_source_kind_filter(token: &str) -> Result<AuthSourceKindFilter> {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 624 | `fn` | `pub fn parse_auth_revoked_filter(token: &str) -> Result<AuthRevokedFilter> {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 768 | `fn` | `pub fn execute_auth_login_command(` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 1602 | `fn` | `pub fn execute_auth_reauth_command(` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 1723 | `fn` | `pub fn auth_status_row_for_provider(` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 2040 | `fn` | `pub fn auth_state_counts(rows: &[AuthStatusRow]) -> std::collections::BTreeMap<String, usize> {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 2048 | `fn` | `pub fn auth_mode_counts(rows: &[AuthStatusRow]) -> std::collections::BTreeMap<String, usize> {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 2056 | `fn` | `pub fn auth_provider_counts(rows: &[AuthStatusRow]) -> std::collections::BTreeMap<String, usize> {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 2064 | `fn` | `pub fn auth_availability_counts(` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 2079 | `fn` | `pub fn auth_source_kind(source: &str) -> &'static str {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 2103 | `fn` | `pub fn auth_source_kind_counts(` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 2115 | `fn` | `pub fn auth_revoked_counts(rows: &[AuthStatusRow]) -> std::collections::BTreeMap<String, usize> {` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 2128 | `fn` | `pub fn format_auth_state_counts(` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 2141 | `fn` | `pub fn execute_auth_status_command(` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 2370 | `fn` | `pub fn execute_auth_matrix_command(` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 2579 | `fn` | `pub fn execute_auth_logout_command(` |
| `crates/tau-provider/src/auth_commands_runtime.rs` | 2657 | `fn` | `pub fn execute_auth_command(config: &AuthCommandConfig, command_args: &str) -> String {` |
| `crates/tau-provider/src/claude_cli_client.rs` | 34 | `fn` | `pub fn new(config: ClaudeCliConfig) -> Result<Self, TauAiError> {` |
| `crates/tau-provider/src/cli_executable.rs` | 29 | `fn` | `pub fn is_executable_available(executable: &str) -> bool {` |
| `crates/tau-provider/src/client.rs` | 269 | `fn` | `pub fn build_provider_client(cli: &Cli, provider: Provider) -> Result<Arc<dyn LlmClient>> {` |
| `crates/tau-provider/src/codex_cli_client.rs` | 46 | `fn` | `pub fn new(config: CodexCliConfig) -> Result<Self, TauAiError> {` |
| `crates/tau-provider/src/credential_store.rs` | 87 | `fn` | `pub fn resolve_credential_store_encryption_mode(cli: &Cli) -> CredentialStoreEncryptionMode {` |
| `crates/tau-provider/src/credential_store.rs` | 182 | `fn` | `pub fn encrypt_credential_store_secret(` |
| `crates/tau-provider/src/credential_store.rs` | 215 | `fn` | `pub fn decrypt_credential_store_secret(` |
| `crates/tau-provider/src/credential_store.rs` | 262 | `fn` | `pub fn load_credential_store(` |
| `crates/tau-provider/src/credential_store.rs` | 354 | `fn` | `pub fn save_credential_store(` |
| `crates/tau-provider/src/credential_store.rs` | 433 | `fn` | `pub fn refresh_provider_access_token(` |
| `crates/tau-provider/src/credential_store.rs` | 458 | `fn` | `pub fn reauth_required_error(provider: Provider, reason: &str) -> anyhow::Error {` |
| `crates/tau-provider/src/credentials.rs` | 26 | `fn` | `pub fn resolve_store_backed_provider_credential(` |
| `crates/tau-provider/src/credentials.rs` | 142 | `fn` | `pub fn resolve_non_empty_secret_with_source(` |
| `crates/tau-provider/src/credentials.rs` | 392 | `fn` | `pub fn provider_auth_snapshot_for_status(` |
| `crates/tau-provider/src/fallback.rs` | 70 | `fn` | `pub fn new(routes: Vec<ClientRoute>, event_sink: Option<FallbackEventSink>) -> Self {` |
| `crates/tau-provider/src/fallback.rs` | 74 | `fn` | `pub fn with_circuit_breaker(` |
| `crates/tau-provider/src/fallback.rs` | 281 | `fn` | `pub fn is_retryable_provider_error(error: &TauAiError) -> bool {` |
| `crates/tau-provider/src/fallback.rs` | 312 | `fn` | `pub fn resolve_fallback_models(cli: &Cli, primary: &ModelRef) -> Result<Vec<ModelRef>> {` |
| `crates/tau-provider/src/fallback.rs` | 333 | `fn` | `pub fn build_client_with_fallbacks(` |
| `crates/tau-provider/src/gemini_cli_client.rs` | 34 | `fn` | `pub fn new(config: GeminiCliConfig) -> Result<Self, TauAiError> {` |
| `crates/tau-provider/src/integration_auth.rs` | 19 | `fn` | `pub fn resolve_non_empty_cli_value(value: Option<&str>) -> Option<String> {` |
| `crates/tau-provider/src/integration_auth.rs` | 26 | `fn` | `pub fn resolve_secret_from_cli_or_store_id(` |
| `crates/tau-provider/src/integration_auth.rs` | 130 | `fn` | `pub fn normalize_integration_credential_id(raw: &str) -> Result<String> {` |
| `crates/tau-provider/src/integration_auth.rs` | 151 | `fn` | `pub fn parse_integration_auth_command(command_args: &str) -> Result<IntegrationAuthCommand> {` |
| `crates/tau-provider/src/integration_auth.rs` | 652 | `fn` | `pub fn execute_integration_auth_command(config: &AuthCommandConfig, command_args: &str) -> String {` |
| `crates/tau-provider/src/model_catalog.rs` | 19 | `const` | `pub const MODEL_CATALOG_SCHEMA_VERSION: u32 = 1;` |
| `crates/tau-provider/src/model_catalog.rs` | 20 | `const` | `pub const MODELS_LIST_USAGE: &str = "/models-list [query] [--provider <name>] [--tools <true\|false>] [--multimodal <true\|false>] [--reasoning <true\|false>] [--limit <n>]";` |
| `crates/tau-provider/src/model_catalog.rs` | 21 | `const` | `pub const MODEL_SHOW_USAGE: &str = "/model-show <provider/model>";` |
| `crates/tau-provider/src/model_catalog.rs` | 99 | `fn` | `pub fn built_in() -> Self {` |
| `crates/tau-provider/src/model_catalog.rs` | 118 | `fn` | `pub fn entries(&self) -> &[ModelCatalogEntry] {` |
| `crates/tau-provider/src/model_catalog.rs` | 122 | `fn` | `pub fn source(&self) -> &ModelCatalogSource {` |
| `crates/tau-provider/src/model_catalog.rs` | 126 | `fn` | `pub fn find(&self, provider: &str, model: &str) -> Option<&ModelCatalogEntry> {` |
| `crates/tau-provider/src/model_catalog.rs` | 133 | `fn` | `pub fn find_model_ref(&self, model_ref: &ModelRef) -> Option<&ModelCatalogEntry> {` |
| `crates/tau-provider/src/model_catalog.rs` | 137 | `fn` | `pub fn is_stale(&self, stale_after_hours: u64) -> bool {` |
| `crates/tau-provider/src/model_catalog.rs` | 142 | `fn` | `pub fn diagnostics_line(&self, stale_after_hours: u64) -> String {` |
| `crates/tau-provider/src/model_catalog.rs` | 170 | `fn` | `pub fn from_file(` |
| `crates/tau-provider/src/model_catalog.rs` | 198 | `fn` | `pub fn default_model_catalog_cache_path() -> PathBuf {` |
| `crates/tau-provider/src/model_catalog.rs` | 202 | `fn` | `pub fn parse_model_catalog_payload(payload: &str) -> Result<ModelCatalogFile> {` |
| `crates/tau-provider/src/model_catalog.rs` | 215 | `fn` | `pub fn validate_model_catalog_file(file: &ModelCatalogFile) -> Result<()> {` |
| `crates/tau-provider/src/model_catalog.rs` | 272 | `fn` | `pub async fn load_model_catalog_with_cache(` |
| `crates/tau-provider/src/model_catalog.rs` | 322 | `fn` | `pub fn ensure_model_supports_tools(catalog: &ModelCatalog, model_ref: &ModelRef) -> Result<()> {` |
| `crates/tau-provider/src/model_catalog.rs` | 342 | `fn` | `pub fn parse_models_list_args(input: &str) -> Result<ModelListArgs> {` |
| `crates/tau-provider/src/model_catalog.rs` | 408 | `fn` | `pub fn render_models_list(catalog: &ModelCatalog, args: &ModelListArgs) -> String {` |
| `crates/tau-provider/src/model_catalog.rs` | 457 | `fn` | `pub fn render_model_show(catalog: &ModelCatalog, raw_model: &str) -> Result<String> {` |
| `crates/tau-provider/src/types.rs` | 30 | `fn` | `pub fn as_str(self) -> &'static str {` |

## tau-multi-channel

Undocumented items: **79**

| File | Line | Kind | Signature |
| --- | ---: | --- | --- |
| `crates/tau-multi-channel/src/lib.rs` | 36 | `mod` | `pub mod multi_channel_contract;` |
| `crates/tau-multi-channel/src/lib.rs` | 37 | `mod` | `pub mod multi_channel_credentials;` |
| `crates/tau-multi-channel/src/lib.rs` | 38 | `mod` | `pub mod multi_channel_incident;` |
| `crates/tau-multi-channel/src/lib.rs` | 39 | `mod` | `pub mod multi_channel_lifecycle;` |
| `crates/tau-multi-channel/src/lib.rs` | 40 | `mod` | `pub mod multi_channel_live_connectors;` |
| `crates/tau-multi-channel/src/lib.rs` | 41 | `mod` | `pub mod multi_channel_live_ingress;` |
| `crates/tau-multi-channel/src/lib.rs` | 42 | `mod` | `pub mod multi_channel_media;` |
| `crates/tau-multi-channel/src/lib.rs` | 43 | `mod` | `pub mod multi_channel_outbound;` |
| `crates/tau-multi-channel/src/lib.rs` | 44 | `mod` | `pub mod multi_channel_policy;` |
| `crates/tau-multi-channel/src/lib.rs` | 45 | `mod` | `pub mod multi_channel_route_inspect;` |
| `crates/tau-multi-channel/src/lib.rs` | 46 | `mod` | `pub mod multi_channel_routing;` |
| `crates/tau-multi-channel/src/lib.rs` | 47 | `mod` | `pub mod multi_channel_runtime;` |
| `crates/tau-multi-channel/src/lib.rs` | 48 | `mod` | `pub mod multi_channel_send;` |
| `crates/tau-multi-channel/src/multi_channel_contract.rs` | 18 | `const` | `pub const MULTI_CHANNEL_CONTRACT_SCHEMA_VERSION: u32 = 1;` |
| `crates/tau-multi-channel/src/multi_channel_contract.rs` | 34 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-multi-channel/src/multi_channel_contract.rs` | 99 | `fn` | `pub fn parse_multi_channel_contract_fixture(raw: &str) -> Result<MultiChannelContractFixture> {` |
| `crates/tau-multi-channel/src/multi_channel_contract.rs` | 107 | `fn` | `pub fn load_multi_channel_contract_fixture(path: &Path) -> Result<MultiChannelContractFixture> {` |
| `crates/tau-multi-channel/src/multi_channel_contract.rs` | 111 | `fn` | `pub fn validate_multi_channel_contract_fixture(` |
| `crates/tau-multi-channel/src/multi_channel_contract.rs` | 135 | `fn` | `pub fn validate_multi_channel_inbound_event(event: &MultiChannelInboundEvent) -> Result<()> {` |
| `crates/tau-multi-channel/src/multi_channel_contract.rs` | 198 | `fn` | `pub fn event_contract_key(event: &MultiChannelInboundEvent) -> String {` |
| `crates/tau-multi-channel/src/multi_channel_credentials.rs` | 36 | `fn` | `pub fn resolve_secret(` |
| `crates/tau-multi-channel/src/multi_channel_incident.rs` | 142 | `fn` | `pub fn build_multi_channel_incident_timeline_report(` |
| `crates/tau-multi-channel/src/multi_channel_incident.rs` | 636 | `fn` | `pub fn render_multi_channel_incident_timeline_report(` |
| `crates/tau-multi-channel/src/multi_channel_lifecycle.rs` | 24 | `const` | `pub const MULTI_CHANNEL_LIFECYCLE_STATE_FILE_NAME: &str = "channel-lifecycle.json";` |
| `crates/tau-multi-channel/src/multi_channel_lifecycle.rs` | 171 | `fn` | `pub fn default_probe_timeout_ms() -> u64 {` |
| `crates/tau-multi-channel/src/multi_channel_lifecycle.rs` | 175 | `fn` | `pub fn default_probe_max_attempts() -> usize {` |
| `crates/tau-multi-channel/src/multi_channel_lifecycle.rs` | 179 | `fn` | `pub fn default_probe_retry_delay_ms() -> u64 {` |
| `crates/tau-multi-channel/src/multi_channel_lifecycle.rs` | 183 | `fn` | `pub fn execute_multi_channel_lifecycle_action(` |
| `crates/tau-multi-channel/src/multi_channel_lifecycle.rs` | 912 | `fn` | `pub fn render_multi_channel_lifecycle_report(report: &MultiChannelLifecycleReport) -> String {` |
| `crates/tau-multi-channel/src/multi_channel_live_connectors.rs` | 66 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-multi-channel/src/multi_channel_live_connectors.rs` | 74 | `fn` | `pub fn is_disabled(self) -> bool {` |
| `crates/tau-multi-channel/src/multi_channel_live_connectors.rs` | 78 | `fn` | `pub fn is_polling(self) -> bool {` |
| `crates/tau-multi-channel/src/multi_channel_live_connectors.rs` | 82 | `fn` | `pub fn is_webhook(self) -> bool {` |
| `crates/tau-multi-channel/src/multi_channel_live_connectors.rs` | 88 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-multi-channel/src/multi_channel_live_connectors.rs` | 255 | `fn` | `pub fn load_multi_channel_live_connectors_status_report(` |
| `crates/tau-multi-channel/src/multi_channel_live_connectors.rs` | 277 | `fn` | `pub async fn run_multi_channel_live_connectors_runner(` |
| `crates/tau-multi-channel/src/multi_channel_live_ingress.rs` | 42 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-multi-channel/src/multi_channel_live_ingress.rs` | 109 | `fn` | `pub fn default_multi_channel_live_provider_label(transport: MultiChannelTransport) -> &'static str {` |
| `crates/tau-multi-channel/src/multi_channel_live_ingress.rs` | 117 | `fn` | `pub fn build_multi_channel_live_envelope_from_raw_payload(` |
| `crates/tau-multi-channel/src/multi_channel_live_ingress.rs` | 152 | `fn` | `pub fn ingest_multi_channel_live_raw_payload(` |
| `crates/tau-multi-channel/src/multi_channel_live_ingress.rs` | 211 | `fn` | `pub fn parse_multi_channel_live_inbound_envelope(` |
| `crates/tau-multi-channel/src/multi_channel_live_ingress.rs` | 224 | `fn` | `pub fn parse_multi_channel_live_inbound_envelope_value(` |
| `crates/tau-multi-channel/src/multi_channel_live_ingress.rs` | 261 | `fn` | `pub fn load_multi_channel_live_inbound_envelope_fixture(` |
| `crates/tau-multi-channel/src/multi_channel_media.rs` | 211 | `fn` | `pub fn process_media_attachments(` |
| `crates/tau-multi-channel/src/multi_channel_media.rs` | 218 | `fn` | `pub fn process_media_attachments_with_provider<P: MediaUnderstandingProvider>(` |
| `crates/tau-multi-channel/src/multi_channel_media.rs` | 303 | `fn` | `pub fn render_media_prompt_context(report: &MediaUnderstandingReport) -> Option<String> {` |
| `crates/tau-multi-channel/src/multi_channel_outbound.rs` | 30 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-multi-channel/src/multi_channel_outbound.rs` | 153 | `fn` | `pub fn new(config: MultiChannelOutboundConfig) -> Result<Self> {` |
| `crates/tau-multi-channel/src/multi_channel_outbound.rs` | 187 | `fn` | `pub fn mode(&self) -> MultiChannelOutboundMode {` |
| `crates/tau-multi-channel/src/multi_channel_outbound.rs` | 191 | `fn` | `pub async fn deliver(` |
| `crates/tau-multi-channel/src/multi_channel_policy.rs` | 16 | `const` | `pub const MULTI_CHANNEL_POLICY_SCHEMA_VERSION: u32 = 1;` |
| `crates/tau-multi-channel/src/multi_channel_policy.rs` | 17 | `const` | `pub const MULTI_CHANNEL_POLICY_FILE_NAME: &str = "channel-policy.json";` |
| `crates/tau-multi-channel/src/multi_channel_policy.rs` | 29 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-multi-channel/src/multi_channel_policy.rs` | 48 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-multi-channel/src/multi_channel_policy.rs` | 67 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-multi-channel/src/multi_channel_policy.rs` | 86 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-multi-channel/src/multi_channel_policy.rs` | 185 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-multi-channel/src/multi_channel_policy.rs` | 201 | `fn` | `pub fn reason_code(&self) -> &str {` |
| `crates/tau-multi-channel/src/multi_channel_policy.rs` | 207 | `fn` | `pub fn as_str(&self) -> &'static str {` |
| `crates/tau-multi-channel/src/multi_channel_policy.rs` | 227 | `fn` | `pub fn channel_policy_path_for_state_dir(state_dir: &Path) -> PathBuf {` |
| `crates/tau-multi-channel/src/multi_channel_policy.rs` | 245 | `fn` | `pub fn load_multi_channel_policy_for_state_dir(state_dir: &Path) -> Result<MultiChannelPolicyFile> {` |
| `crates/tau-multi-channel/src/multi_channel_policy.rs` | 251 | `fn` | `pub fn load_multi_channel_policy_file(path: &Path) -> Result<MultiChannelPolicyFile> {` |
| `crates/tau-multi-channel/src/multi_channel_policy.rs` | 279 | `fn` | `pub fn evaluate_multi_channel_channel_policy(` |
| `crates/tau-multi-channel/src/multi_channel_policy.rs` | 338 | `fn` | `pub fn collect_open_dm_risk_channels(policy_file: &MultiChannelPolicyFile) -> Vec<String> {` |
| `crates/tau-multi-channel/src/multi_channel_route_inspect.rs` | 44 | `fn` | `pub fn build_multi_channel_route_inspect_report(` |
| `crates/tau-multi-channel/src/multi_channel_route_inspect.rs` | 74 | `fn` | `pub fn render_multi_channel_route_inspect_report(` |
| `crates/tau-multi-channel/src/multi_channel_routing.rs` | 20 | `const` | `pub const MULTI_CHANNEL_ROUTE_BINDINGS_FILE_NAME: &str = "multi-channel-route-bindings.json";` |
| `crates/tau-multi-channel/src/multi_channel_routing.rs` | 105 | `fn` | `pub fn load_multi_channel_route_bindings_for_state_dir(` |
| `crates/tau-multi-channel/src/multi_channel_routing.rs` | 115 | `fn` | `pub fn load_multi_channel_route_bindings(path: &Path) -> Result<MultiChannelRouteBindingFile> {` |
| `crates/tau-multi-channel/src/multi_channel_routing.rs` | 129 | `fn` | `pub fn parse_multi_channel_route_bindings(raw: &str) -> Result<MultiChannelRouteBindingFile> {` |
| `crates/tau-multi-channel/src/multi_channel_routing.rs` | 136 | `fn` | `pub fn resolve_multi_channel_event_route(` |
| `crates/tau-multi-channel/src/multi_channel_routing.rs` | 236 | `fn` | `pub fn resolve_multi_channel_account_id(event: &MultiChannelInboundEvent) -> String {` |
| `crates/tau-multi-channel/src/multi_channel_routing.rs` | 254 | `fn` | `pub fn route_decision_trace_payload(` |
| `crates/tau-multi-channel/src/multi_channel_runtime.rs` | 92 | `fn` | `pub fn reason_code(&self) -> &str {` |
| `crates/tau-multi-channel/src/multi_channel_runtime.rs` | 401 | `fn` | `pub async fn run_multi_channel_contract_runner(config: MultiChannelRuntimeConfig) -> Result<()> {` |
| `crates/tau-multi-channel/src/multi_channel_runtime.rs` | 437 | `fn` | `pub async fn run_multi_channel_live_runner(config: MultiChannelLiveRuntimeConfig) -> Result<()> {` |
| `crates/tau-multi-channel/src/multi_channel_send.rs` | 91 | `fn` | `pub fn execute_multi_channel_send_action(` |
| `crates/tau-multi-channel/src/multi_channel_send.rs` | 215 | `fn` | `pub fn resolve_multi_channel_send_text(` |
| `crates/tau-multi-channel/src/multi_channel_send.rs` | 490 | `fn` | `pub fn render_multi_channel_send_report(report: &MultiChannelSendReport) -> String {` |

## tau-gateway

Undocumented items: **47**

| File | Line | Kind | Signature |
| --- | ---: | --- | --- |
| `crates/tau-gateway/src/gateway_contract.rs` | 18 | `const` | `pub const GATEWAY_CONTRACT_SCHEMA_VERSION: u32 = 1;` |
| `crates/tau-gateway/src/gateway_contract.rs` | 20 | `const` | `pub const GATEWAY_ERROR_INVALID_REQUEST: &str = "gateway_invalid_request";` |
| `crates/tau-gateway/src/gateway_contract.rs` | 21 | `const` | `pub const GATEWAY_ERROR_UNSUPPORTED_METHOD: &str = "gateway_unsupported_method";` |
| `crates/tau-gateway/src/gateway_contract.rs` | 22 | `const` | `pub const GATEWAY_ERROR_BACKEND_UNAVAILABLE: &str = "gateway_backend_unavailable";` |
| `crates/tau-gateway/src/gateway_contract.rs` | 116 | `fn` | `pub fn parse_gateway_contract_fixture(raw: &str) -> Result<GatewayContractFixture> {` |
| `crates/tau-gateway/src/gateway_contract.rs` | 124 | `fn` | `pub fn load_gateway_contract_fixture(path: &Path) -> Result<GatewayContractFixture> {` |
| `crates/tau-gateway/src/gateway_contract.rs` | 128 | `fn` | `pub fn gateway_contract_capabilities() -> GatewayContractCapabilities {` |
| `crates/tau-gateway/src/gateway_contract.rs` | 149 | `fn` | `pub fn validate_gateway_contract_compatibility(fixture: &GatewayContractFixture) -> Result<()> {` |
| `crates/tau-gateway/src/gateway_contract.rs` | 192 | `fn` | `pub fn validate_gateway_contract_fixture(fixture: &GatewayContractFixture) -> Result<()> {` |
| `crates/tau-gateway/src/gateway_contract.rs` | 209 | `fn` | `pub fn evaluate_gateway_case(case: &GatewayContractCase) -> GatewayReplayResult {` |
| `crates/tau-gateway/src/gateway_contract.rs` | 258 | `fn` | `pub fn validate_gateway_case_result_against_contract(` |
| `crates/tau-gateway/src/gateway_contract.rs` | 321 | `fn` | `pub fn run_gateway_contract_replay<D: GatewayContractDriver>(` |
| `crates/tau-gateway/src/gateway_openresponses.rs` | 133 | `fn` | `pub fn new<F>(handler: F) -> Self` |
| `crates/tau-gateway/src/gateway_openresponses.rs` | 611 | `fn` | `pub async fn run_gateway_openresponses_server(` |
| `crates/tau-gateway/src/gateway_runtime.rs` | 203 | `fn` | `pub async fn run_gateway_contract_runner(config: GatewayRuntimeConfig) -> Result<()> {` |
| `crates/tau-gateway/src/gateway_runtime.rs` | 233 | `fn` | `pub fn start_gateway_service_mode(state_dir: &Path) -> Result<GatewayServiceStatusReport> {` |
| `crates/tau-gateway/src/gateway_runtime.rs` | 252 | `fn` | `pub fn stop_gateway_service_mode(` |
| `crates/tau-gateway/src/gateway_runtime.rs` | 272 | `fn` | `pub fn inspect_gateway_service_mode(state_dir: &Path) -> Result<GatewayServiceStatusReport> {` |
| `crates/tau-gateway/src/gateway_runtime.rs` | 281 | `fn` | `pub fn render_gateway_service_status_report(report: &GatewayServiceStatusReport) -> String {` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 13 | `const` | `pub const GATEWAY_WS_REQUEST_SCHEMA_VERSION: u32 = 1;` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 14 | `const` | `pub const GATEWAY_WS_RESPONSE_SCHEMA_VERSION: u32 = 1;` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 15 | `const` | `pub const GATEWAY_WS_PROTOCOL_VERSION: &str = "0.1.0";` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 16 | `const` | `pub const GATEWAY_WS_HEARTBEAT_INTERVAL_SECONDS: u64 = 15;` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 21 | `const` | `pub const GATEWAY_WS_ERROR_CODE_INVALID_JSON: &str = "invalid_json";` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 22 | `const` | `pub const GATEWAY_WS_ERROR_CODE_UNSUPPORTED_SCHEMA: &str = "unsupported_schema";` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 23 | `const` | `pub const GATEWAY_WS_ERROR_CODE_UNSUPPORTED_KIND: &str = "unsupported_kind";` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 24 | `const` | `pub const GATEWAY_WS_ERROR_CODE_INVALID_REQUEST_ID: &str = "invalid_request_id";` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 25 | `const` | `pub const GATEWAY_WS_ERROR_CODE_INVALID_PAYLOAD: &str = "invalid_payload";` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 26 | `const` | `pub const GATEWAY_WS_ERROR_CODE_UNAUTHORIZED: &str = "unauthorized";` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 27 | `const` | `pub const GATEWAY_WS_ERROR_CODE_RATE_LIMITED: &str = "rate_limited";` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 28 | `const` | `pub const GATEWAY_WS_ERROR_CODE_INTERNAL_ERROR: &str = "internal_error";` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 110 | `fn` | `pub fn parse_gateway_ws_request_frame(raw: &str) -> Result<GatewayWsRequestFrame> {` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 143 | `fn` | `pub fn best_effort_gateway_ws_request_id(raw: &str) -> Option<String> {` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 154 | `fn` | `pub fn classify_gateway_ws_parse_error(message: &str) -> &'static str {` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 172 | `fn` | `pub fn build_gateway_ws_response_frame(` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 185 | `fn` | `pub fn build_gateway_ws_error_frame(` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 200 | `fn` | `pub fn parse_optional_session_key(` |
| `crates/tau-gateway/src/gateway_ws_protocol.rs` | 217 | `fn` | `pub fn gateway_ws_capabilities_payload() -> Value {` |
| `crates/tau-gateway/src/lib.rs` | 6 | `mod` | `pub mod gateway_contract;` |
| `crates/tau-gateway/src/lib.rs` | 7 | `mod` | `pub mod gateway_openresponses;` |
| `crates/tau-gateway/src/lib.rs` | 8 | `mod` | `pub mod gateway_runtime;` |
| `crates/tau-gateway/src/lib.rs` | 9 | `mod` | `pub mod gateway_ws_protocol;` |
| `crates/tau-gateway/src/lib.rs` | 10 | `mod` | `pub mod remote_profile;` |
| `crates/tau-gateway/src/remote_profile.rs` | 21 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-gateway/src/remote_profile.rs` | 41 | `fn` | `pub fn as_str(self) -> &'static str {` |
| `crates/tau-gateway/src/remote_profile.rs` | 101 | `fn` | `pub fn validate_gateway_openresponses_bind(bind: &str) -> Result<SocketAddr> {` |
| `crates/tau-gateway/src/remote_profile.rs` | 106 | `fn` | `pub fn evaluate_gateway_remote_profile_config(` |

## tau-ops

Undocumented items: **39**

| File | Line | Kind | Signature |
| --- | ---: | --- | --- |
| `crates/tau-ops/src/canvas_commands.rs` | 30 | `const` | `pub const CANVAS_USAGE: &str =` |
| `crates/tau-ops/src/canvas_commands.rs` | 234 | `fn` | `pub fn execute_canvas_command(command_args: &str, config: &CanvasCommandConfig) -> String {` |
| `crates/tau-ops/src/channel_store_admin.rs` | 912 | `fn` | `pub fn execute_channel_store_admin_command(cli: &Cli) -> Result<()> {` |
| `crates/tau-ops/src/command_catalog.rs` | 14 | `const` | `pub const MODELS_LIST_USAGE: &str = "/models-list [query] [--provider <name>] [--tools <true\|false>] [--multimodal <true\|false>] [--reasoning <true\|false>] [--limit <n>]";` |
| `crates/tau-ops/src/command_catalog.rs` | 15 | `const` | `pub const MODEL_SHOW_USAGE: &str = "/model-show <provider/model>";` |
| `crates/tau-ops/src/command_catalog.rs` | 17 | `const` | `pub const COMMAND_SPECS: &[CommandSpec] = &[` |
| `crates/tau-ops/src/command_catalog.rs` | 382 | `const` | `pub const COMMAND_NAMES: &[&str] = &[` |
| `crates/tau-ops/src/command_catalog.rs` | 433 | `fn` | `pub fn render_help_overview() -> String {` |
| `crates/tau-ops/src/command_catalog.rs` | 437 | `fn` | `pub fn render_command_help(topic: &str) -> Option<String> {` |
| `crates/tau-ops/src/command_catalog.rs` | 441 | `fn` | `pub fn unknown_help_topic_message(topic: &str) -> String {` |
| `crates/tau-ops/src/command_catalog.rs` | 445 | `fn` | `pub fn unknown_command_message(command: &str) -> String {` |
| `crates/tau-ops/src/daemon_runtime.rs` | 93 | `fn` | `pub fn resolve_tau_daemon_profile(profile: CliDaemonProfile) -> CliDaemonProfile {` |
| `crates/tau-ops/src/daemon_runtime.rs` | 104 | `fn` | `pub fn tau_daemon_mode_requested(cli: &Cli) -> bool {` |
| `crates/tau-ops/src/daemon_runtime.rs` | 112 | `fn` | `pub fn install_tau_daemon(config: &TauDaemonConfig) -> Result<TauDaemonStatusReport> {` |
| `crates/tau-ops/src/daemon_runtime.rs` | 158 | `fn` | `pub fn uninstall_tau_daemon(config: &TauDaemonConfig) -> Result<TauDaemonStatusReport> {` |
| `crates/tau-ops/src/daemon_runtime.rs` | 189 | `fn` | `pub fn start_tau_daemon(config: &TauDaemonConfig) -> Result<TauDaemonStatusReport> {` |
| `crates/tau-ops/src/daemon_runtime.rs` | 215 | `fn` | `pub fn stop_tau_daemon(` |
| `crates/tau-ops/src/daemon_runtime.rs` | 243 | `fn` | `pub fn inspect_tau_daemon(config: &TauDaemonConfig) -> Result<TauDaemonStatusReport> {` |
| `crates/tau-ops/src/daemon_runtime.rs` | 301 | `fn` | `pub fn render_tau_daemon_status_report(report: &TauDaemonStatusReport) -> String {` |
| `crates/tau-ops/src/daemon_runtime.rs` | 349 | `fn` | `pub fn render_launchd_plist(label: &str, executable: &Path, state_dir: &Path) -> String {` |
| `crates/tau-ops/src/daemon_runtime.rs` | 387 | `fn` | `pub fn render_systemd_user_unit(label: &str, executable: &Path, state_dir: &Path) -> String {` |
| `crates/tau-ops/src/macro_commands.rs` | 15 | `const` | `pub const MACRO_SCHEMA_VERSION: u32 = 1;` |
| `crates/tau-ops/src/macro_commands.rs` | 16 | `const` | `pub const MACRO_USAGE: &str = "usage: /macro <save\|run\|list\|show\|delete> ...";` |
| `crates/tau-ops/src/macro_commands.rs` | 52 | `fn` | `pub fn default_macro_config_path() -> Result<PathBuf> {` |
| `crates/tau-ops/src/macro_commands.rs` | 59 | `fn` | `pub fn validate_macro_name(name: &str) -> Result<()> {` |
| `crates/tau-ops/src/macro_commands.rs` | 76 | `fn` | `pub fn parse_macro_command(command_args: &str) -> Result<MacroCommand> {` |
| `crates/tau-ops/src/macro_commands.rs` | 148 | `fn` | `pub fn load_macro_file(path: &Path) -> Result<BTreeMap<String, Vec<String>>> {` |
| `crates/tau-ops/src/macro_commands.rs` | 168 | `fn` | `pub fn save_macro_file(path: &Path, macros: &BTreeMap<String, Vec<String>>) -> Result<()> {` |
| `crates/tau-ops/src/macro_commands.rs` | 190 | `fn` | `pub fn load_macro_commands(commands_file: &Path) -> Result<Vec<String>> {` |
| `crates/tau-ops/src/macro_commands.rs` | 209 | `fn` | `pub fn validate_macro_command_entry(command: &str, command_names: &[&str]) -> Result<()> {` |
| `crates/tau-ops/src/macro_commands.rs` | 225 | `fn` | `pub fn validate_macro_commands(commands: &[String], command_names: &[&str]) -> Result<()> {` |
| `crates/tau-ops/src/macro_commands.rs` | 233 | `fn` | `pub fn render_macro_list(path: &Path, macros: &BTreeMap<String, Vec<String>>) -> String {` |
| `crates/tau-ops/src/macro_commands.rs` | 249 | `fn` | `pub fn render_macro_show(path: &Path, name: &str, commands: &[String]) -> String {` |
| `crates/tau-ops/src/macro_commands.rs` | 262 | `fn` | `pub fn execute_macro_command_with_runner<F>(` |
| `crates/tau-ops/src/project_index.rs` | 95 | `fn` | `pub fn execute_project_index_command(cli: &Cli) -> Result<()> {` |
| `crates/tau-ops/src/project_index.rs` | 708 | `fn` | `pub fn build_index() {}` |
| `crates/tau-ops/src/qa_loop_commands.rs` | 21 | `const` | `pub const QA_LOOP_USAGE: &str = "usage: /qa-loop [--json] [--config <path>] [--stage-timeout-ms <ms>] [--retry-failures <n>] [--max-output-bytes <bytes>] [--changed-file-limit <n>]";` |
| `crates/tau-ops/src/qa_loop_commands.rs` | 308 | `fn` | `pub fn execute_qa_loop_cli_command(command_args: &str) -> String {` |
| `crates/tau-ops/src/qa_loop_commands.rs` | 320 | `fn` | `pub fn execute_qa_loop_preflight_command(cli: &Cli) -> Result<()> {` |

