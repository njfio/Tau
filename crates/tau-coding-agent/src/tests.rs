//! Crate-level regression/integration test harness for coding-agent runtime.
//!
//! These tests validate command dispatch, startup modes, and transport/routing
//! behaviors against deterministic fixtures and failure contracts.

use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    future::{pending, ready},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use clap::Parser;
use httpmock::prelude::*;
use sha2::{Digest, Sha256};
use tau_agent_core::{Agent, AgentConfig, AgentEvent, ToolExecutionResult};
use tau_ai::{
    ChatRequest, ChatResponse, ChatUsage, ContentBlock, LlmClient, Message, MessageRole, ModelRef,
    Provider, TauAiError,
};
use tau_cli::cli_args::{CliExecutionDomainFlags, CliGatewayDaemonFlags, CliRuntimeTailFlags};
use tau_cli::CliPromptSanitizerMode;
use tempfile::tempdir;
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::sleep;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::{
    apply_trust_root_mutations, branch_alias_path_for_session, build_auth_command_config,
    build_doctor_command_config, build_multi_channel_incident_timeline_report,
    build_multi_channel_route_inspect_report, build_profile_defaults, build_provider_client,
    build_tool_policy, command_file_error_mode_label, compose_startup_system_prompt,
    compute_session_entry_depths, compute_session_stats, current_unix_timestamp,
    decrypt_credential_store_secret, default_macro_config_path, default_profile_store_path,
    default_skills_lock_path, derive_skills_prune_candidates, encrypt_credential_store_secret,
    ensure_non_empty_text, escape_graph_label, evaluate_multi_channel_live_readiness,
    execute_auth_command, execute_branch_alias_command, execute_channel_store_admin_command,
    execute_command_file, execute_doctor_cli_command, execute_doctor_command,
    execute_doctor_command_with_options, execute_integration_auth_command, execute_macro_command,
    execute_package_activate_command, execute_package_activate_on_startup,
    execute_package_conflicts_command, execute_package_install_command,
    execute_package_list_command, execute_package_remove_command, execute_package_rollback_command,
    execute_package_show_command, execute_package_update_command, execute_package_validate_command,
    execute_profile_command, execute_rpc_capabilities_command, execute_rpc_dispatch_frame_command,
    execute_rpc_dispatch_ndjson_command, execute_rpc_serve_ndjson_command,
    execute_rpc_validate_frame_command, execute_session_bookmark_command,
    execute_session_diff_command, execute_session_graph_export_command,
    execute_session_search_command, execute_session_stats_command, execute_skills_list_command,
    execute_skills_lock_diff_command, execute_skills_lock_write_command,
    execute_skills_prune_command, execute_skills_search_command, execute_skills_show_command,
    execute_skills_sync_command, execute_skills_trust_add_command,
    execute_skills_trust_list_command, execute_skills_trust_revoke_command,
    execute_skills_trust_rotate_command, execute_skills_verify_command, execute_startup_preflight,
    format_id_list, format_remap_ids, handle_command, handle_command_with_session_import_mode,
    initialize_session, is_retryable_provider_error, load_branch_aliases, load_credential_store,
    load_macro_file, load_multi_agent_route_table, load_profile_store, load_session_bookmarks,
    load_trust_root_records, normalize_daemon_subcommand_args, parse_auth_command,
    parse_branch_alias_command, parse_command, parse_command_file, parse_doctor_command_args,
    parse_integration_auth_command, parse_macro_command, parse_numbered_plan_steps,
    parse_profile_command, parse_sandbox_command_tokens, parse_session_bookmark_command,
    parse_session_diff_args, parse_session_search_args, parse_session_stats_args,
    parse_skills_lock_diff_args, parse_skills_prune_args, parse_skills_search_args,
    parse_skills_trust_list_args, parse_skills_trust_mutation_args, parse_skills_verify_args,
    parse_trust_rotation_spec, parse_trusted_root_spec, percentile_duration_ms,
    provider_auth_capability, refresh_provider_access_token,
    register_runtime_extension_tool_hook_subscriber, render_audit_summary, render_command_help,
    render_doctor_report, render_doctor_report_json, render_help_overview, render_macro_list,
    render_macro_show, render_profile_diffs, render_profile_list, render_profile_show,
    render_session_diff, render_session_graph_dot, render_session_graph_mermaid,
    render_session_stats, render_session_stats_json, render_skills_list,
    render_skills_lock_diff_drift, render_skills_lock_diff_in_sync,
    render_skills_lock_write_success, render_skills_search, render_skills_show,
    render_skills_sync_drift_details, render_skills_trust_list, render_skills_verify_report,
    resolve_credential_store_encryption_mode, resolve_fallback_models, resolve_prompt_input,
    resolve_prunable_skill_file_name, resolve_secret_from_cli_or_store_id,
    resolve_session_graph_format, resolve_skill_trust_roots, resolve_skills_lock_path,
    resolve_store_backed_provider_credential, resolve_system_prompt, rpc_capabilities_payload,
    run_doctor_checks, run_doctor_checks_with_lookup, run_plan_first_prompt,
    run_plan_first_prompt_with_policy_context,
    run_plan_first_prompt_with_policy_context_and_routing, run_prompt_with_cancellation,
    save_branch_aliases, save_credential_store, save_macro_file, save_profile_store,
    save_session_bookmarks, search_session_entries, session_bookmark_path_for_session,
    session_lineage_messages, session_message_preview, shared_lineage_prefix_depth,
    stream_text_chunks, summarize_audit_file, tool_audit_event_json, tool_policy_to_json,
    trust_record_status, unknown_command_message, validate_branch_alias_name,
    validate_custom_command_contract_runner_cli, validate_daemon_cli,
    validate_dashboard_contract_runner_cli, validate_deployment_contract_runner_cli,
    validate_deployment_wasm_inspect_cli, validate_deployment_wasm_package_cli,
    validate_event_webhook_ingest_cli, validate_events_runner_cli,
    validate_gateway_contract_runner_cli, validate_gateway_openresponses_server_cli,
    validate_gateway_remote_plan_cli, validate_gateway_remote_profile_inspect_cli,
    validate_gateway_service_cli, validate_github_issues_bridge_cli, validate_macro_command_entry,
    validate_macro_name, validate_memory_contract_runner_cli,
    validate_multi_agent_contract_runner_cli, validate_multi_channel_channel_lifecycle_cli,
    validate_multi_channel_contract_runner_cli, validate_multi_channel_incident_timeline_cli,
    validate_multi_channel_live_connectors_runner_cli, validate_multi_channel_live_ingest_cli,
    validate_multi_channel_live_runner_cli, validate_multi_channel_send_cli, validate_profile_name,
    validate_project_index_cli, validate_rpc_frame_file, validate_session_file,
    validate_skills_prune_file_name, validate_slack_bridge_cli, validate_voice_contract_runner_cli,
    validate_voice_live_runner_cli, AuthCommand, AuthCommandConfig, BranchAliasCommand,
    BranchAliasFile, Cli, CliBashProfile, CliCommandFileErrorMode,
    CliCredentialStoreEncryptionMode, CliDaemonProfile, CliDeploymentWasmBrowserDidMethod,
    CliDeploymentWasmRuntimeProfile, CliEventTemplateSchedule, CliGatewayOpenResponsesAuthMode,
    CliGatewayRemoteProfile, CliMultiChannelLiveConnectorMode, CliMultiChannelOutboundMode,
    CliMultiChannelTransport, CliOrchestratorMode, CliOsSandboxMode, CliProviderAuthMode,
    CliSessionImportMode, CliToolPolicyPreset, CliWebhookSignatureAlgorithm, ClientRoute,
    CommandAction, CommandExecutionContext, CommandFileEntry, CommandFileReport,
    CredentialStoreData, CredentialStoreEncryptionMode, DoctorCheckOptions, DoctorCheckResult,
    DoctorCommandArgs, DoctorCommandConfig, DoctorCommandOutputFormat,
    DoctorMultiChannelReadinessConfig, DoctorProviderKeyStatus, DoctorStatus,
    FallbackRoutingClient, IntegrationAuthCommand, IntegrationCredentialStoreRecord, MacroCommand,
    MacroFile, MultiAgentRouteTable, PlanFirstPromptPolicyRequest, PlanFirstPromptRequest,
    PlanFirstPromptRoutingRequest, ProfileCommand, ProfileDefaults, ProfileStoreFile,
    PromptRunStatus, PromptTelemetryLogger, ProviderAuthMethod, ProviderCredentialStoreRecord,
    RenderOptions, RuntimeExtensionHooksConfig, SessionBookmarkCommand, SessionBookmarkFile,
    SessionDiffEntry, SessionDiffReport, SessionGraphFormat, SessionRuntime, SessionSearchArgs,
    SessionStats, SessionStatsOutputFormat, SkillsPruneMode, SkillsSyncCommandConfig,
    SkillsVerifyEntry, SkillsVerifyReport, SkillsVerifyStatus, SkillsVerifySummary,
    SkillsVerifyTrustSummary, ToolAuditLogger, TrustedRootRecord, BRANCH_ALIAS_SCHEMA_VERSION,
    BRANCH_ALIAS_USAGE, MACRO_SCHEMA_VERSION, MACRO_USAGE, PROFILE_SCHEMA_VERSION, PROFILE_USAGE,
    SESSION_BOOKMARK_SCHEMA_VERSION, SESSION_BOOKMARK_USAGE, SESSION_SEARCH_DEFAULT_RESULTS,
    SESSION_SEARCH_PREVIEW_CHARS, SKILLS_PRUNE_USAGE, SKILLS_TRUST_ADD_USAGE,
    SKILLS_TRUST_LIST_USAGE, SKILLS_VERIFY_USAGE,
};
use crate::auth_commands::{
    auth_availability_counts, auth_mode_counts, auth_provider_counts, auth_revoked_counts,
    auth_source_kind, auth_source_kind_counts, auth_state_counts, auth_status_row_for_provider,
    format_auth_state_counts, AuthMatrixAvailabilityFilter, AuthMatrixModeSupportFilter,
    AuthRevokedFilter, AuthSourceKindFilter,
};
use crate::extension_manifest::discover_extension_runtime_registrations;
use crate::provider_api_key_candidates_with_inputs;
use crate::provider_auth_snapshot_for_status;
use crate::resolve_api_key;
use crate::tools::{register_extension_tools, BashCommandProfile, OsSandboxMode, ToolPolicyPreset};
use crate::{default_model_catalog_cache_path, ModelCatalog, MODELS_LIST_USAGE, MODEL_SHOW_USAGE};
use crate::{
    execute_extension_exec_command, execute_extension_list_command, execute_extension_show_command,
    execute_extension_validate_command,
};
use tau_session::{SessionImportMode, SessionStore};

static AUTH_ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

fn snapshot_env_vars(keys: &[&str]) -> Vec<(String, Option<String>)> {
    keys.iter()
        .map(|key| ((*key).to_string(), std::env::var(key).ok()))
        .collect()
}

fn restore_env_vars(snapshot: Vec<(String, Option<String>)>) {
    for (key, value) in snapshot {
        if let Some(value) = value {
            std::env::set_var(key, value);
        } else {
            std::env::remove_var(key);
        }
    }
}

#[cfg(unix)]
fn write_mock_codex_script(dir: &Path, body: &str) -> PathBuf {
    let script = dir.join("mock-codex.sh");
    let content = format!("#!/bin/sh\nset -eu\n{body}\n");
    std::fs::write(&script, content).expect("write mock codex script");
    let mut perms = std::fs::metadata(&script)
        .expect("mock codex metadata")
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script, perms).expect("chmod mock codex script");
    script
}

#[cfg(unix)]
fn write_mock_gemini_script(dir: &Path, body: &str) -> PathBuf {
    let script = dir.join("mock-gemini.sh");
    let content = format!("#!/bin/sh\nset -eu\n{body}\n");
    std::fs::write(&script, content).expect("write mock gemini script");
    let mut perms = std::fs::metadata(&script)
        .expect("mock gemini metadata")
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script, perms).expect("chmod mock gemini script");
    script
}

#[cfg(unix)]
fn write_mock_claude_script(dir: &Path, body: &str) -> PathBuf {
    let script = dir.join("mock-claude.sh");
    let content = format!("#!/bin/sh\nset -eu\n{body}\n");
    std::fs::write(&script, content).expect("write mock claude script");
    let mut perms = std::fs::metadata(&script)
        .expect("mock claude metadata")
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script, perms).expect("chmod mock claude script");
    script
}

#[cfg(unix)]
fn write_mock_gcloud_script(dir: &Path, body: &str) -> PathBuf {
    let script = dir.join("mock-gcloud.sh");
    let content = format!("#!/bin/sh\nset -eu\n{body}\n");
    std::fs::write(&script, content).expect("write mock gcloud script");
    let mut perms = std::fs::metadata(&script)
        .expect("mock gcloud metadata")
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script, perms).expect("chmod mock gcloud script");
    script
}

struct NoopClient;

#[async_trait]
impl LlmClient for NoopClient {
    async fn complete(&self, _request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        Err(TauAiError::InvalidResponse(
            "noop client should not be called".to_string(),
        ))
    }
}

struct SuccessClient;

#[async_trait]
impl LlmClient for SuccessClient {
    async fn complete(&self, _request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        Ok(ChatResponse {
            message: tau_ai::Message::assistant_text("done"),
            finish_reason: Some("stop".to_string()),
            usage: ChatUsage::default(),
        })
    }
}

struct SlowClient;

#[async_trait]
impl LlmClient for SlowClient {
    async fn complete(&self, _request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        sleep(Duration::from_secs(5)).await;
        Ok(ChatResponse {
            message: tau_ai::Message::assistant_text("slow"),
            finish_reason: Some("stop".to_string()),
            usage: ChatUsage::default(),
        })
    }
}

struct QueueClient {
    responses: AsyncMutex<VecDeque<ChatResponse>>,
}

#[async_trait]
impl LlmClient for QueueClient {
    async fn complete(&self, _request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        let mut responses = self.responses.lock().await;
        responses
            .pop_front()
            .ok_or_else(|| TauAiError::InvalidResponse("mock response queue is empty".to_string()))
    }
}

struct SequenceClient {
    outcomes: AsyncMutex<VecDeque<Result<ChatResponse, TauAiError>>>,
}

#[async_trait]
impl LlmClient for SequenceClient {
    async fn complete(&self, _request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        let mut outcomes = self.outcomes.lock().await;
        outcomes.pop_front().unwrap_or_else(|| {
            Err(TauAiError::InvalidResponse(
                "mock outcome queue is empty".to_string(),
            ))
        })
    }
}

struct RecordingSequenceClient {
    outcomes: AsyncMutex<VecDeque<Result<ChatResponse, TauAiError>>>,
    recorded_models: Arc<AsyncMutex<Vec<String>>>,
}

#[async_trait]
impl LlmClient for RecordingSequenceClient {
    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        self.recorded_models.lock().await.push(request.model);
        let mut outcomes = self.outcomes.lock().await;
        outcomes.pop_front().unwrap_or_else(|| {
            Err(TauAiError::InvalidResponse(
                "mock outcome queue is empty".to_string(),
            ))
        })
    }
}

fn test_render_options() -> RenderOptions {
    RenderOptions {
        stream_output: false,
        stream_delay_ms: 0,
    }
}

fn test_tool_policy_json() -> serde_json::Value {
    serde_json::json!({
        "schema_version": 6,
        "preset": "balanced",
        "allowed_roots": [],
        "allowed_commands": [],
        "bash_profile": "balanced",
        "os_sandbox_mode": "off",
        "os_sandbox_policy_mode": "best-effort",
        "bash_dry_run": false,
        "enforce_regular_files": true,
        "max_command_length": 4096,
        "max_command_output_bytes": 16000,
        "http_timeout_ms": 20000,
        "http_max_response_bytes": 256000,
        "http_max_redirects": 5,
        "http_allow_http": false,
        "http_allow_private_network": false,
    })
}

fn test_chat_request() -> ChatRequest {
    ChatRequest {
        model: "placeholder-model".to_string(),
        messages: vec![Message::user("hello")],
        tools: vec![],
        tool_choice: None,
        json_mode: false,
        max_tokens: None,
        temperature: None,
        prompt_cache: Default::default(),
    }
}

pub(crate) fn test_cli() -> Cli {
    let mut cli = parse_cli_with_stack(["tau-rs"]);
    cli.system_prompt = "sys".to_string();
    cli
}

fn parse_cli_with_stack<I, T>(args: I) -> Cli
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString>,
{
    let (owned_args, _) = normalize_startup_cli_args(
        args.into_iter()
            .map(Into::into)
            .map(|value: std::ffi::OsString| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
    );
    thread::Builder::new()
        .name("tau-cli-parse".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || Cli::parse_from(owned_args))
        .expect("spawn cli parse thread")
        .join()
        .expect("join cli parse thread")
}

fn try_parse_cli_with_stack<I, T>(args: I) -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString>,
{
    let (owned_args, _) = normalize_startup_cli_args(
        args.into_iter()
            .map(Into::into)
            .map(|value: std::ffi::OsString| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
    );
    thread::Builder::new()
        .name("tau-cli-try-parse".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || Cli::try_parse_from(owned_args))
        .expect("spawn cli try-parse thread")
        .join()
        .expect("join cli try-parse thread")
}

fn normalize_startup_cli_args(args: Vec<String>) -> (Vec<String>, Vec<String>) {
    crate::normalize_startup_cli_args(args)
}

fn set_workspace_tau_paths(cli: &mut Cli, workspace: &Path) {
    let tau_root = workspace.join(".tau");
    cli.session = tau_root.join("sessions/default.sqlite");
    cli.credential_store = tau_root.join("credentials.json");
    cli.skills_dir = tau_root.join("skills");
    cli.model_catalog_cache = tau_root.join("models/catalog.json");
    cli.channel_store_root = tau_root.join("channel-store");
    cli.events_dir = tau_root.join("events");
    cli.events_state_path = tau_root.join("events/state.json");
    cli.runtime_heartbeat_state_path = tau_root.join("runtime-heartbeat/state.json");
    cli.multi_channel_state_dir = tau_root.join("multi-channel");
    cli.multi_agent_state_dir = tau_root.join("multi-agent");
    cli.browser_automation_state_dir = tau_root.join("browser-automation");
    cli.memory_state_dir = tau_root.join("memory");
    cli.dashboard_state_dir = tau_root.join("dashboard");
    cli.gateway_state_dir = tau_root.join("gateway");
    cli.daemon_state_dir = tau_root.join("daemon");
    cli.deployment_state_dir = tau_root.join("deployment");
    cli.deployment_wasm_package_output_dir = tau_root.join("deployment/wasm-artifacts");
    cli.custom_command_state_dir = tau_root.join("custom-command");
    cli.voice_state_dir = tau_root.join("voice");
    cli.github_state_dir = tau_root.join("github-issues");
    cli.slack_state_dir = tau_root.join("slack");
    cli.package_install_root = tau_root.join("packages");
    cli.package_update_root = tau_root.join("packages");
    cli.package_list_root = tau_root.join("packages");
    cli.package_remove_root = tau_root.join("packages");
    cli.package_rollback_root = tau_root.join("packages");
    cli.package_conflicts_root = tau_root.join("packages");
    cli.package_activate_root = tau_root.join("packages");
    cli.package_activate_destination = tau_root.join("packages-active");
    cli.extension_list_root = tau_root.join("extensions");
    cli.extension_runtime_root = tau_root.join("extensions");
}

fn write_test_provider_credential(
    path: &Path,
    encryption: CredentialStoreEncryptionMode,
    key: Option<&str>,
    provider: Provider,
    record: ProviderCredentialStoreRecord,
) {
    let mut store = CredentialStoreData {
        encryption,
        providers: BTreeMap::new(),
        integrations: BTreeMap::new(),
    };
    store
        .providers
        .insert(provider.as_str().to_string(), record);
    save_credential_store(path, &store, key).expect("save credential store");
}

fn write_test_integration_credential(
    path: &Path,
    encryption: CredentialStoreEncryptionMode,
    key: Option<&str>,
    integration_id: &str,
    record: IntegrationCredentialStoreRecord,
) {
    let mut store = CredentialStoreData {
        encryption,
        providers: BTreeMap::new(),
        integrations: BTreeMap::new(),
    };
    store
        .integrations
        .insert(integration_id.to_string(), record);
    save_credential_store(path, &store, key).expect("save credential store");
}

fn skills_command_config(
    skills_dir: &Path,
    lock_path: &Path,
    trust_root_path: Option<&Path>,
) -> SkillsSyncCommandConfig {
    SkillsSyncCommandConfig {
        skills_dir: skills_dir.to_path_buf(),
        default_lock_path: lock_path.to_path_buf(),
        default_trust_root_path: trust_root_path.map(Path::to_path_buf),
        doctor_config: DoctorCommandConfig {
            model: "openai/gpt-4o-mini".to_string(),
            provider_keys: vec![DoctorProviderKeyStatus {
                provider_kind: Provider::OpenAi,
                provider: "openai".to_string(),
                key_env_var: "OPENAI_API_KEY".to_string(),
                present: true,
                auth_mode: ProviderAuthMethod::ApiKey,
                mode_supported: true,
                login_backend_enabled: false,
                login_backend_executable: None,
                login_backend_available: false,
            }],
            release_channel_path: PathBuf::from(".tau/release-channel.json"),
            release_lookup_cache_path: PathBuf::from(".tau/release-lookup-cache.json"),
            release_lookup_cache_ttl_ms: 900_000,
            browser_automation_playwright_cli: "playwright-cli".to_string(),
            session_enabled: true,
            session_path: PathBuf::from(".tau/sessions/default.sqlite"),
            skills_dir: skills_dir.to_path_buf(),
            skills_lock_path: lock_path.to_path_buf(),
            trust_root_path: trust_root_path.map(Path::to_path_buf),
            multi_channel_live_readiness: DoctorMultiChannelReadinessConfig::default(),
        },
    }
}

fn test_profile_defaults() -> ProfileDefaults {
    build_profile_defaults(&test_cli())
}

fn test_auth_command_config() -> AuthCommandConfig {
    let mut config = build_auth_command_config(&test_cli());
    if let Ok(current_exe) = std::env::current_exe() {
        config.openai_codex_cli = current_exe.display().to_string();
        config.anthropic_claude_cli = current_exe.display().to_string();
        config.google_gemini_cli = current_exe.display().to_string();
        config.google_gcloud_cli = current_exe.display().to_string();
    }
    config
}

fn test_command_context<'a>(
    tool_policy_json: &'a serde_json::Value,
    profile_defaults: &'a ProfileDefaults,
    skills_command_config: &'a SkillsSyncCommandConfig,
    auth_command_config: &'a AuthCommandConfig,
    model_catalog: &'a ModelCatalog,
) -> CommandExecutionContext<'a> {
    CommandExecutionContext {
        tool_policy_json,
        session_import_mode: SessionImportMode::Merge,
        profile_defaults,
        skills_command_config,
        auth_command_config,
        model_catalog,
        extension_commands: &[],
    }
}

fn set_provider_auth_mode(
    config: &mut AuthCommandConfig,
    provider: Provider,
    mode: ProviderAuthMethod,
) {
    match provider {
        Provider::OpenAi | Provider::OpenRouter => config.openai_auth_mode = mode,
        Provider::Anthropic => config.anthropic_auth_mode = mode,
        Provider::Google => config.google_auth_mode = mode,
    }
}

fn set_provider_api_key(config: &mut AuthCommandConfig, provider: Provider, value: &str) {
    match provider {
        Provider::OpenAi | Provider::OpenRouter => config.openai_api_key = Some(value.to_string()),
        Provider::Anthropic => config.anthropic_api_key = Some(value.to_string()),
        Provider::Google => config.google_api_key = Some(value.to_string()),
    }
}

#[path = "tests/auth_provider/mod.rs"]
mod auth_provider;
#[path = "tests/cli_validation.rs"]
mod cli_validation;
#[path = "tests/extensions_rpc.rs"]
mod extensions_rpc;
#[path = "tests/misc.rs"]
mod misc;
#[path = "tests/runtime_agent.rs"]
mod runtime_agent;
