use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    ffi::OsString,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, OnceLock,
    },
    time::{Duration, Instant},
};

use async_trait::async_trait;
use reqwest::{
    header::{HeaderName, HeaderValue},
    redirect::Policy as RedirectPolicy,
    Method, StatusCode,
};
use serde::Serialize;
use serde_json::{json, Value};
use tau_agent_core::{Agent, AgentTool, DefaultLeakDetector, LeakDetector, ToolExecutionResult};
use tau_ai::{Message, ToolDefinition};
#[allow(deprecated)]
use tau_extensions::{execute_extension_registered_tool, ExtensionRegisteredTool};
use tau_runtime::{
    build_generated_wasm_tool, BackgroundJobCreateRequest, BackgroundJobRuntime,
    BackgroundJobRuntimeConfig, BackgroundJobStatusFilter, BackgroundJobTraceContext,
    GeneratedToolBuildRequest, SsrfGuard, SsrfProtectionConfig, SsrfViolation,
    WasmSandboxCapabilityProfile, WasmSandboxFilesystemMode, WasmSandboxLimits,
    WasmSandboxNetworkMode, WASM_SANDBOX_FUEL_LIMIT_DEFAULT,
    WASM_SANDBOX_MAX_RESPONSE_BYTES_DEFAULT, WASM_SANDBOX_MEMORY_LIMIT_BYTES_DEFAULT,
    WASM_SANDBOX_TIMEOUT_MS_DEFAULT,
};

use tau_access::ApprovalAction;
use tau_memory::memory_contract::{MemoryEntry, MemoryScope};
use tau_memory::runtime::{
    FileMemoryStore, MemoryEmbeddingProviderConfig, MemoryRelationInput, MemoryRelationType,
    MemoryScopeFilter, MemorySearchOptions, MemoryType, MemoryTypeImportanceProfile,
};
use tau_session::{
    redo_session_head, resolve_session_navigation_head, session_message_preview, undo_session_head,
    SessionRuntime, SessionStore,
};

const BALANCED_COMMAND_ALLOWLIST: &[&str] = &[
    "awk", "cargo", "cat", "cp", "cut", "du", "echo", "env", "fd", "find", "git", "grep", "head",
    "ls", "mkdir", "mv", "printf", "pwd", "rg", "rm", "rustc", "rustup", "sed", "sleep", "sort",
    "stat", "tail", "touch", "tr", "uniq", "wc",
];

const STRICT_COMMAND_ALLOWLIST: &[&str] = &[
    "awk", "cat", "cut", "du", "echo", "env", "fd", "find", "grep", "head", "ls", "printf", "pwd",
    "rg", "sed", "sort", "stat", "tail", "tr", "uniq", "wc",
];

const MEMORY_SEARCH_DEFAULT_LIMIT: usize = 5;
const MEMORY_SEARCH_MAX_LIMIT: usize = 50;
const MEMORY_WRITE_MAX_SUMMARY_CHARS: usize = 1_200;
const MEMORY_WRITE_MAX_FACTS: usize = 32;
const MEMORY_WRITE_MAX_TAGS: usize = 32;
const WRITE_MANY_MAX_FILES: usize = 32;
const EDIT_MANY_MAX_EDITS: usize = 64;
const MEMORY_WRITE_MAX_FACT_CHARS: usize = 400;
const MEMORY_WRITE_MAX_TAG_CHARS: usize = 96;
const MEMORY_EMBEDDING_TIMEOUT_MS_DEFAULT: u64 = 10_000;
const MEMORY_BM25_K1_DEFAULT: f32 = 1.2;
const MEMORY_BM25_B_DEFAULT: f32 = 0.75;
const MEMORY_BM25_MIN_SCORE_DEFAULT: f32 = 0.0;
const MEMORY_RRF_K_DEFAULT: usize = 60;
const MEMORY_RRF_VECTOR_WEIGHT_DEFAULT: f32 = 1.0;
const MEMORY_RRF_LEXICAL_WEIGHT_DEFAULT: f32 = 1.0;
const JOBS_LIST_DEFAULT_LIMIT: usize = 20;
const JOBS_LIST_MAX_LIMIT: usize = 200;
const JOBS_DEFAULT_TIMEOUT_MS: u64 = 30_000;
const JOBS_MAX_TIMEOUT_MS: u64 = 900_000;
const JOBS_OUTPUT_PREVIEW_DEFAULT_BYTES: usize = 2_000;
const JOBS_OUTPUT_PREVIEW_MAX_BYTES: usize = 16_000;
const BRANCH_TOOL_MAX_PROMPT_CHARS: usize = 4_000;
const TOOL_RATE_LIMIT_WINDOW_MS_DEFAULT: u64 = 60_000;
const TOOL_RATE_LIMIT_MAX_REQUESTS_PERMISSIVE: u32 = 240;
const TOOL_RATE_LIMIT_MAX_REQUESTS_BALANCED: u32 = 120;
const TOOL_RATE_LIMIT_MAX_REQUESTS_STRICT: u32 = 60;
const TOOL_RATE_LIMIT_MAX_REQUESTS_HARDENED: u32 = 30;
const TOOL_BUILDER_MAX_ATTEMPTS_DEFAULT: usize = 3;
const TOOL_BUILDER_MAX_ATTEMPTS_MAX: usize = 8;
const TOOL_HTTP_TIMEOUT_MS_PERMISSIVE: u64 = 60_000;
const TOOL_HTTP_TIMEOUT_MS_BALANCED: u64 = 20_000;
const TOOL_HTTP_TIMEOUT_MS_STRICT: u64 = 15_000;
const TOOL_HTTP_TIMEOUT_MS_HARDENED: u64 = 10_000;
const TOOL_HTTP_MAX_RESPONSE_BYTES_PERMISSIVE: usize = 1_000_000;
const TOOL_HTTP_MAX_RESPONSE_BYTES_BALANCED: usize = 256_000;
const TOOL_HTTP_MAX_RESPONSE_BYTES_STRICT: usize = 128_000;
const TOOL_HTTP_MAX_RESPONSE_BYTES_HARDENED: usize = 64_000;
const TOOL_HTTP_MAX_REDIRECTS_PERMISSIVE: usize = 8;
const TOOL_HTTP_MAX_REDIRECTS_BALANCED: usize = 5;
const TOOL_HTTP_MAX_REDIRECTS_STRICT: usize = 3;
const TOOL_HTTP_MAX_REDIRECTS_HARDENED: usize = 2;
const DOCKER_SANDBOX_DEFAULT_IMAGE: &str = "debian:stable-slim";
const DOCKER_SANDBOX_DEFAULT_MEMORY_MB: u64 = 256;
const DOCKER_SANDBOX_DEFAULT_CPUS: f32 = 1.0;
const DOCKER_SANDBOX_DEFAULT_PIDS_LIMIT: u64 = 256;
const DOCKER_SANDBOX_TMPFS_SIZE_MB: u64 = 64;
const SANDBOX_REQUIRED_UNAVAILABLE_ERROR: &str =
    "OS sandbox policy mode 'required' is enabled but command would run without a sandbox launcher";
const SANDBOX_FORCE_UNAVAILABLE_ERROR: &str =
    "OS sandbox mode 'force' is enabled but no sandbox launcher is configured or available";
const SANDBOX_DOCKER_UNAVAILABLE_ERROR: &str =
    "OS sandbox Docker backend is enabled but Docker CLI is unavailable";
static MEMORY_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static BACKGROUND_JOB_RUNTIME_REGISTRY: OnceLock<
    Mutex<HashMap<PathBuf, Arc<BackgroundJobRuntime>>>,
> = OnceLock::new();
const DEFAULT_PROTECTED_RELATIVE_PATHS: &[&str] = &[
    "AGENTS.md",
    "SOUL.md",
    "USER.md",
    ".tau/AGENTS.md",
    ".tau/SOUL.md",
    ".tau/USER.md",
    ".tau/rbac-policy.json",
    ".tau/trust-roots.json",
    ".tau/channel-policy.json",
];
const BUILTIN_AGENT_TOOL_NAMES: &[&str] = &[
    "read",
    "write",
    "write_many",
    "edit",
    "edit_many",
    "memory_write",
    "memory_read",
    "memory_delete",
    "memory_search",
    "memory_tree",
    "sessions_list",
    "sessions_history",
    "sessions_search",
    "sessions_stats",
    "sessions_send",
    "jobs_create",
    "jobs_list",
    "jobs_status",
    "jobs_cancel",
    "branch",
    "undo",
    "redo",
    "send_file",
    "react",
    "skip",
    "grep",
    "glob",
    "ls",
    "http",
    "tool_builder",
    "bash",
];

mod bash_tool;
mod jobs_tools;
mod memory_tools;
mod registry_core;
mod runtime_helpers;
mod session_tools;

pub use bash_tool::BashTool;
use bash_tool::{
    evaluate_tool_approval_gate, evaluate_tool_rate_limit_gate, evaluate_tool_rbac_gate,
};
pub use jobs_tools::{JobsCancelTool, JobsCreateTool, JobsListTool, JobsStatusTool};
pub use memory_tools::{
    MemoryDeleteTool, MemoryReadTool, MemorySearchTool, MemoryTreeTool, MemoryWriteTool,
};
use registry_core::BashSandboxSpec;
pub use registry_core::{
    builtin_agent_tool_names, register_builtin_tools, register_extension_tools,
    tool_policy_preset_name, tool_rate_limit_behavior_name, BashCommandProfile,
    OsSandboxDockerNetwork, OsSandboxMode, OsSandboxPolicyMode, ToolPolicy, ToolPolicyPreset,
    ToolRateLimitCounters, ToolRateLimitExceededBehavior,
};
use runtime_helpers::*;
pub use runtime_helpers::{os_sandbox_docker_network_name, os_sandbox_policy_mode_name};
#[cfg(test)]
use session_tools::is_session_candidate_path;
pub use session_tools::{
    SessionsHistoryTool, SessionsListTool, SessionsSearchTool, SessionsSendTool, SessionsStatsTool,
};

/// Public struct `ToolBuilderTool` used across Tau components.
pub struct ToolBuilderTool {
    policy: Arc<ToolPolicy>,
}

impl ToolBuilderTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AgentTool for ToolBuilderTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "tool_builder".to_string(),
            description: "Generate, compile, persist, and register a wasm-backed extension tool"
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Generated tool name (lowercase alphanumeric, dash, underscore)" },
                    "description": { "type": "string", "description": "Generated tool description" },
                    "spec": { "type": "string", "description": "Natural-language tool behavior specification" },
                    "parameters": { "type": "object", "description": "JSON schema for generated tool arguments" },
                    "wat_source": { "type": "string", "description": "Optional initial WAT source candidate" },
                    "max_attempts": { "type": "integer", "minimum": 1, "maximum": 8 },
                    "timeout_ms": { "type": "integer", "minimum": 1 },
                    "fuel_limit": { "type": "integer", "minimum": 1 },
                    "memory_limit_bytes": { "type": "integer", "minimum": 1 },
                    "max_response_bytes": { "type": "integer", "minimum": 1 },
                    "filesystem_mode": { "type": "string", "enum": ["deny", "read-only", "read-write"] },
                    "network_mode": { "type": "string", "enum": ["deny", "allow"] },
                    "env_allowlist": {
                        "type": "array",
                        "items": { "type": "string" }
                    },
                    "output_root": { "type": "string", "description": "Optional override for generated artifact root" },
                    "extension_root": { "type": "string", "description": "Optional override for generated extension registration root" }
                },
                "required": ["name", "description", "spec"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        if !self.policy.tool_builder_enabled {
            return ToolExecutionResult::error(json!({
                "error": "tool_builder is disabled by policy",
                "reason_code": "tool_builder_disabled",
            }));
        }

        let name = match required_string(&arguments, "name") {
            Ok(value) => value,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "error": error,
                    "reason_code": "tool_builder_invalid_name",
                }));
            }
        };
        let description = match required_string(&arguments, "description") {
            Ok(value) => value,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "error": error,
                    "reason_code": "tool_builder_invalid_description",
                }));
            }
        };
        let spec = match required_string(&arguments, "spec") {
            Ok(value) => value,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "error": error,
                    "reason_code": "tool_builder_invalid_spec",
                }));
            }
        };
        let parameters = arguments.get("parameters").cloned().unwrap_or_else(
            || json!({"type":"object","properties":{},"additionalProperties":false}),
        );
        if !parameters.is_object() {
            return ToolExecutionResult::error(json!({
                "error": "field 'parameters' must be a JSON object",
                "reason_code": "tool_builder_invalid_parameters",
            }));
        }

        let max_attempts = match optional_positive_usize(&arguments, "max_attempts") {
            Ok(value) => value.unwrap_or(self.policy.tool_builder_max_attempts),
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "error": error,
                    "reason_code": "tool_builder_invalid_max_attempts",
                }));
            }
        }
        .clamp(1, TOOL_BUILDER_MAX_ATTEMPTS_MAX);
        let timeout_ms = match optional_positive_u64(&arguments, "timeout_ms") {
            Ok(value) => value.unwrap_or(WASM_SANDBOX_TIMEOUT_MS_DEFAULT),
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "error": error,
                    "reason_code": "tool_builder_invalid_timeout_ms",
                }));
            }
        };
        let fuel_limit = match optional_positive_u64(&arguments, "fuel_limit") {
            Ok(value) => value.unwrap_or(WASM_SANDBOX_FUEL_LIMIT_DEFAULT),
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "error": error,
                    "reason_code": "tool_builder_invalid_fuel_limit",
                }));
            }
        };
        let memory_limit_bytes = match optional_positive_u64(&arguments, "memory_limit_bytes") {
            Ok(value) => value.unwrap_or(WASM_SANDBOX_MEMORY_LIMIT_BYTES_DEFAULT),
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "error": error,
                    "reason_code": "tool_builder_invalid_memory_limit_bytes",
                }));
            }
        };
        let max_response_bytes = match optional_positive_usize(&arguments, "max_response_bytes") {
            Ok(value) => value.unwrap_or(WASM_SANDBOX_MAX_RESPONSE_BYTES_DEFAULT),
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "error": error,
                    "reason_code": "tool_builder_invalid_max_response_bytes",
                }));
            }
        };
        let filesystem_mode = match optional_string(&arguments, "filesystem_mode") {
            Ok(Some(mode)) => match mode.as_str() {
                "deny" => WasmSandboxFilesystemMode::Deny,
                "read-only" => WasmSandboxFilesystemMode::ReadOnly,
                "read-write" => WasmSandboxFilesystemMode::ReadWrite,
                _ => {
                    return ToolExecutionResult::error(json!({
                        "error": "field 'filesystem_mode' must be one of: deny, read-only, read-write",
                        "reason_code": "tool_builder_invalid_filesystem_mode",
                    }));
                }
            },
            Ok(None) => WasmSandboxFilesystemMode::Deny,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "error": error,
                    "reason_code": "tool_builder_invalid_filesystem_mode",
                }));
            }
        };
        let network_mode = match optional_string(&arguments, "network_mode") {
            Ok(Some(mode)) => match mode.as_str() {
                "deny" => WasmSandboxNetworkMode::Deny,
                "allow" => WasmSandboxNetworkMode::Allow,
                _ => {
                    return ToolExecutionResult::error(json!({
                        "error": "field 'network_mode' must be one of: deny, allow",
                        "reason_code": "tool_builder_invalid_network_mode",
                    }));
                }
            },
            Ok(None) => WasmSandboxNetworkMode::Deny,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "error": error,
                    "reason_code": "tool_builder_invalid_network_mode",
                }));
            }
        };
        let env_allowlist = match optional_string_array_unbounded(&arguments, "env_allowlist") {
            Ok(values) => values,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "error": error,
                    "reason_code": "tool_builder_invalid_env_allowlist",
                }));
            }
        };
        let provided_wat_source = match optional_string(&arguments, "wat_source") {
            Ok(value) => value,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "error": error,
                    "reason_code": "tool_builder_invalid_wat_source",
                }));
            }
        };
        let output_root = match optional_string(&arguments, "output_root") {
            Ok(value) => resolve_builder_root_path(value, &self.policy.tool_builder_output_root),
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "error": error,
                    "reason_code": "tool_builder_invalid_output_root",
                }));
            }
        };
        let extension_root = match optional_string(&arguments, "extension_root") {
            Ok(value) => resolve_builder_root_path(value, &self.policy.tool_builder_extension_root),
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "error": error,
                    "reason_code": "tool_builder_invalid_extension_root",
                }));
            }
        };

        let request = GeneratedToolBuildRequest {
            tool_name: name,
            description,
            spec,
            parameters,
            output_root,
            extension_root,
            max_attempts,
            timeout_ms,
            wasm_limits: WasmSandboxLimits {
                fuel_limit,
                memory_limit_bytes,
                timeout_ms,
                max_response_bytes,
            },
            wasm_capabilities: WasmSandboxCapabilityProfile {
                filesystem_mode,
                network_mode,
                env_allowlist,
            },
            provided_wat_source,
        };
        match build_generated_wasm_tool(request) {
            Ok(report) => ToolExecutionResult::ok(json!({
                "schema_version": report.schema_version,
                "tool_name": report.tool_name,
                "manifest_id": report.manifest_id,
                "manifest_path": report.manifest_path.display().to_string(),
                "module_path": report.module_path.display().to_string(),
                "source_path": report.source_path.display().to_string(),
                "metadata_path": report.metadata_path.display().to_string(),
                "attempts": report.attempts,
                "reason_codes": report.reason_codes,
                "diagnostics": report.diagnostics,
            })),
            Err(error) => ToolExecutionResult::error(json!({
                "error": error.message,
                "reason_code": error.reason_code,
                "diagnostics": error.diagnostics,
            })),
        }
    }
}

fn resolve_builder_root_path(override_path: Option<String>, default_path: &Path) -> PathBuf {
    let configured = override_path
        .map(PathBuf::from)
        .unwrap_or_else(|| default_path.to_path_buf());
    if configured.is_absolute() {
        configured
    } else {
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(configured),
            Err(_) => configured,
        }
    }
}

/// Public struct `ReadTool` used across Tau components.
pub struct ReadTool {
    policy: Arc<ToolPolicy>,
}

impl ReadTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AgentTool for ReadTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "read".to_string(),
            description: "Read a UTF-8 text file from disk".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to read" }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let path = match required_string(&arguments, "path") {
            Ok(path) => path,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };

        let resolved = match resolve_and_validate_path(&path, &self.policy, PathMode::Read) {
            Ok(path) => path,
            Err(error) => {
                return ToolExecutionResult::error(json!({ "path": path, "error": error }))
            }
        };
        if let Err(error) =
            validate_file_target(&resolved, PathMode::Read, self.policy.enforce_regular_files)
        {
            return ToolExecutionResult::error(json!({
                "path": resolved.display().to_string(),
                "error": error,
            }));
        }

        let metadata = match tokio::fs::metadata(&resolved).await {
            Ok(metadata) => metadata,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "path": resolved.display().to_string(),
                    "error": error.to_string(),
                }))
            }
        };

        if metadata.len() as usize > self.policy.max_file_read_bytes {
            return ToolExecutionResult::error(json!({
                "path": resolved.display().to_string(),
                "error": format!(
                    "file is too large ({} bytes), limit is {} bytes",
                    metadata.len(),
                    self.policy.max_file_read_bytes
                ),
            }));
        }

        match tokio::fs::read_to_string(&resolved).await {
            Ok(content) => ToolExecutionResult::ok(json!({
                "path": resolved.display().to_string(),
                "content": content,
            })),
            Err(error) => ToolExecutionResult::error(json!({
                "path": resolved.display().to_string(),
                "error": error.to_string(),
            })),
        }
    }
}

fn current_unix_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or_default()
}

fn evaluate_protected_path_gate(
    policy: &ToolPolicy,
    tool_name: &str,
    path: &Path,
) -> Option<ToolExecutionResult> {
    if policy.allow_protected_path_mutations {
        return None;
    }

    let normalized_path = normalize_policy_path(path);
    let matched_protected_path = policy
        .protected_paths
        .iter()
        .find(|candidate| **candidate == normalized_path)?;
    let path_display = normalized_path.display().to_string();
    let matched_display = matched_protected_path.display().to_string();

    Some(ToolExecutionResult::error(json!({
        "policy_rule": "protected_path",
        "decision": "deny",
        "reason_code": "protected_path_denied",
        "action": format!("tool:{tool_name}"),
        "path": path_display,
        "protected_path": matched_display,
        "error": "path is protected by tool policy",
        "hint": "set TAU_ALLOW_PROTECTED_PATH_MUTATIONS=1 to allow protected path mutations for controlled maintenance windows",
    })))
}

/// Public struct `WriteTool` used across Tau components.
pub struct WriteTool {
    policy: Arc<ToolPolicy>,
}

impl WriteTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AgentTool for WriteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "write".to_string(),
            description: "Write UTF-8 text to disk, creating parent directories if needed"
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["path", "content"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let path = match required_string(&arguments, "path") {
            Ok(path) => path,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };

        let content = match required_string(&arguments, "content") {
            Ok(content) => content,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        let content_size = content.len();
        if content_size > self.policy.max_file_write_bytes {
            return ToolExecutionResult::error(json!({
                "path": path,
                "error": format!(
                    "content is too large ({} bytes), limit is {} bytes",
                    content_size,
                    self.policy.max_file_write_bytes
                ),
            }));
        }

        let resolved = match resolve_and_validate_path(&path, &self.policy, PathMode::Write) {
            Ok(path) => path,
            Err(error) => {
                return ToolExecutionResult::error(json!({ "path": path, "error": error }))
            }
        };
        if let Err(error) = validate_file_target(
            &resolved,
            PathMode::Write,
            self.policy.enforce_regular_files,
        ) {
            return ToolExecutionResult::error(json!({
                "path": resolved.display().to_string(),
                "error": error,
            }));
        }
        if let Some(protected_path_result) =
            evaluate_protected_path_gate(&self.policy, "write", &resolved)
        {
            return protected_path_result;
        }

        if let Some(rbac_result) = evaluate_tool_rbac_gate(
            self.policy.rbac_principal.as_deref(),
            "write",
            self.policy.rbac_policy_path.as_deref(),
            json!({
                "path": resolved.display().to_string(),
                "content_bytes": content_size,
            }),
        ) {
            return rbac_result;
        }

        if let Some(approval_result) = evaluate_tool_approval_gate(ApprovalAction::ToolWrite {
            path: resolved.display().to_string(),
            content_bytes: content_size,
        }) {
            return approval_result;
        }

        if let Some(rate_limit_result) = evaluate_tool_rate_limit_gate(
            &self.policy,
            "write",
            json!({
                "path": resolved.display().to_string(),
                "content_bytes": content_size,
            }),
        ) {
            return rate_limit_result;
        }

        if let Some(parent) = resolved.parent() {
            if !parent.as_os_str().is_empty() {
                if let Err(error) = tokio::fs::create_dir_all(parent).await {
                    return ToolExecutionResult::error(json!({
                        "path": resolved.display().to_string(),
                        "error": format!("failed to create parent directory: {error}"),
                    }));
                }
            }
        }

        match tokio::fs::write(&resolved, content.as_bytes()).await {
            Ok(()) => ToolExecutionResult::ok(json!({
                "path": resolved.display().to_string(),
                "bytes_written": content.len(),
            })),
            Err(error) => ToolExecutionResult::error(json!({
                "path": resolved.display().to_string(),
                "error": error.to_string(),
            })),
        }
    }
}

struct WriteManyFilePlan {
    requested_path: String,
    resolved: PathBuf,
    content: String,
    content_size: usize,
}

/// Public struct `WriteManyTool` used across Tau components.
pub struct WriteManyTool {
    policy: Arc<ToolPolicy>,
}

impl WriteManyTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AgentTool for WriteManyTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "write_many".to_string(),
            description: "Write multiple UTF-8 files in one checked batch, creating parent directories after every path passes policy".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "files": {
                        "type": "array",
                        "minItems": 1,
                        "maxItems": WRITE_MANY_MAX_FILES,
                        "items": {
                            "type": "object",
                            "properties": {
                                "path": { "type": "string" },
                                "content": { "type": "string" }
                            },
                            "required": ["path", "content"],
                            "additionalProperties": false
                        }
                    }
                },
                "required": ["files"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let files = match arguments.get("files").and_then(Value::as_array) {
            Some(files) if !files.is_empty() => files,
            Some(_) => {
                return ToolExecutionResult::error(json!({
                    "error": "files must include at least one entry"
                }))
            }
            None => {
                return ToolExecutionResult::error(json!({
                    "error": "missing required array field 'files'"
                }))
            }
        };
        if files.len() > WRITE_MANY_MAX_FILES {
            return ToolExecutionResult::error(json!({
                "error": format!(
                    "files contains {} entries, limit is {}",
                    files.len(),
                    WRITE_MANY_MAX_FILES
                ),
            }));
        }

        let mut seen_paths = BTreeSet::new();
        let mut plans = Vec::with_capacity(files.len());
        for (index, file) in files.iter().enumerate() {
            let path = match required_string(file, "path") {
                Ok(path) => path,
                Err(error) => {
                    return ToolExecutionResult::error(json!({
                        "index": index,
                        "error": error,
                    }))
                }
            };
            let content = match required_string(file, "content") {
                Ok(content) => content,
                Err(error) => {
                    return ToolExecutionResult::error(json!({
                        "index": index,
                        "path": path,
                        "error": error,
                    }))
                }
            };
            let content_size = content.len();
            if content_size > self.policy.max_file_write_bytes {
                return ToolExecutionResult::error(json!({
                    "index": index,
                    "path": path,
                    "error": format!(
                        "content is too large ({} bytes), limit is {} bytes",
                        content_size,
                        self.policy.max_file_write_bytes
                    ),
                }));
            }
            let resolved = match resolve_and_validate_path(&path, &self.policy, PathMode::Write) {
                Ok(path) => path,
                Err(error) => {
                    return ToolExecutionResult::error(json!({
                        "index": index,
                        "path": path,
                        "error": error,
                    }))
                }
            };
            if !seen_paths.insert(resolved.clone()) {
                return ToolExecutionResult::error(json!({
                    "index": index,
                    "path": resolved.display().to_string(),
                    "error": "duplicate write_many path",
                }));
            }
            if let Err(error) = validate_file_target(
                &resolved,
                PathMode::Write,
                self.policy.enforce_regular_files,
            ) {
                return ToolExecutionResult::error(json!({
                    "index": index,
                    "path": resolved.display().to_string(),
                    "error": error,
                }));
            }
            if let Some(protected_path_result) =
                evaluate_protected_path_gate(&self.policy, "write_many", &resolved)
            {
                return protected_path_result;
            }
            if let Some(rbac_result) = evaluate_tool_rbac_gate(
                self.policy.rbac_principal.as_deref(),
                "write_many",
                self.policy.rbac_policy_path.as_deref(),
                json!({
                    "path": resolved.display().to_string(),
                    "content_bytes": content_size,
                }),
            ) {
                return rbac_result;
            }
            if let Some(approval_result) = evaluate_tool_approval_gate(ApprovalAction::ToolWrite {
                path: resolved.display().to_string(),
                content_bytes: content_size,
            }) {
                return approval_result;
            }
            if let Some(rate_limit_result) = evaluate_tool_rate_limit_gate(
                &self.policy,
                "write_many",
                json!({
                    "path": resolved.display().to_string(),
                    "content_bytes": content_size,
                }),
            ) {
                return rate_limit_result;
            }

            plans.push(WriteManyFilePlan {
                requested_path: path,
                resolved,
                content,
                content_size,
            });
        }

        let mut written = Vec::with_capacity(plans.len());
        let mut total_bytes = 0usize;
        for plan in plans {
            if let Some(parent) = plan.resolved.parent() {
                if !parent.as_os_str().is_empty() {
                    if let Err(error) = tokio::fs::create_dir_all(parent).await {
                        return ToolExecutionResult::error(json!({
                            "path": plan.resolved.display().to_string(),
                            "error": format!("failed to create parent directory: {error}"),
                        }));
                    }
                }
            }

            if let Err(error) = tokio::fs::write(&plan.resolved, plan.content.as_bytes()).await {
                return ToolExecutionResult::error(json!({
                    "path": plan.resolved.display().to_string(),
                    "error": error.to_string(),
                }));
            }
            total_bytes = total_bytes.saturating_add(plan.content_size);
            written.push(json!({
                "requested_path": plan.requested_path,
                "path": plan.resolved.display().to_string(),
                "bytes_written": plan.content_size,
            }));
        }

        ToolExecutionResult::ok(json!({
            "file_count": written.len(),
            "bytes_written": total_bytes,
            "files_written": written,
        }))
    }
}

/// Public struct `EditTool` used across Tau components.
pub struct EditTool {
    policy: Arc<ToolPolicy>,
}

impl EditTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AgentTool for EditTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "edit".to_string(),
            description: "Edit a file by replacing an existing string".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "find": { "type": "string" },
                    "replace": { "type": "string" },
                    "all": { "type": "boolean", "default": false }
                },
                "required": ["path", "find", "replace"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let path = match required_string(&arguments, "path") {
            Ok(path) => path,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };

        let find = match required_string(&arguments, "find") {
            Ok(find) => find,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };

        let replace = match required_string(&arguments, "replace") {
            Ok(replace) => replace,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };

        if find.is_empty() {
            return ToolExecutionResult::error(json!({
                "path": path,
                "error": "'find' must not be empty",
            }));
        }

        let resolved = match resolve_and_validate_path(&path, &self.policy, PathMode::Edit) {
            Ok(path) => path,
            Err(error) => {
                return ToolExecutionResult::error(json!({ "path": path, "error": error }))
            }
        };
        if let Err(error) =
            validate_file_target(&resolved, PathMode::Edit, self.policy.enforce_regular_files)
        {
            return ToolExecutionResult::error(json!({
                "path": resolved.display().to_string(),
                "error": error,
            }));
        }
        if let Some(protected_path_result) =
            evaluate_protected_path_gate(&self.policy, "edit", &resolved)
        {
            return protected_path_result;
        }

        if let Some(rbac_result) = evaluate_tool_rbac_gate(
            self.policy.rbac_principal.as_deref(),
            "edit",
            self.policy.rbac_policy_path.as_deref(),
            json!({
                "path": resolved.display().to_string(),
                "find": find,
                "replace_bytes": replace.len(),
            }),
        ) {
            return rbac_result;
        }

        if let Some(approval_result) = evaluate_tool_approval_gate(ApprovalAction::ToolEdit {
            path: resolved.display().to_string(),
            find: find.clone(),
            replace_bytes: replace.len(),
        }) {
            return approval_result;
        }

        if let Some(rate_limit_result) = evaluate_tool_rate_limit_gate(
            &self.policy,
            "edit",
            json!({
                "path": resolved.display().to_string(),
                "find": find.clone(),
                "replace_bytes": replace.len(),
            }),
        ) {
            return rate_limit_result;
        }

        let replace_all = arguments
            .get("all")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let source = match tokio::fs::read_to_string(&resolved).await {
            Ok(source) => source,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "path": resolved.display().to_string(),
                    "error": error.to_string(),
                }))
            }
        };

        let occurrences = source.matches(&find).count();
        if occurrences == 0 {
            return ToolExecutionResult::error(json!({
                "path": resolved.display().to_string(),
                "error": "target string not found",
            }));
        }

        let updated = if replace_all {
            source.replace(&find, &replace)
        } else {
            source.replacen(&find, &replace, 1)
        };
        if updated.len() > self.policy.max_file_write_bytes {
            return ToolExecutionResult::error(json!({
                "path": resolved.display().to_string(),
                "error": format!(
                    "edited content is too large ({} bytes), limit is {} bytes",
                    updated.len(),
                    self.policy.max_file_write_bytes
                ),
            }));
        }

        if let Err(error) = tokio::fs::write(&resolved, updated.as_bytes()).await {
            return ToolExecutionResult::error(json!({
                "path": resolved.display().to_string(),
                "error": error.to_string(),
            }));
        }

        let replacements = if replace_all { occurrences } else { 1 };
        ToolExecutionResult::ok(json!({
            "path": resolved.display().to_string(),
            "replacements": replacements,
        }))
    }
}

struct EditManyFilePlan {
    requested_path: String,
    resolved: PathBuf,
    updated: String,
    patches: Vec<Value>,
}

/// Public struct `EditManyTool` used across Tau components.
pub struct EditManyTool {
    policy: Arc<ToolPolicy>,
}

impl EditManyTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }

    async fn execute_unified_diff(&self, arguments: &Value) -> ToolExecutionResult {
        let diff = match required_string(arguments, "diff") {
            Ok(diff) => diff,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        if diff.trim().is_empty() {
            return ToolExecutionResult::error(json!({
                "error": "'diff' must not be empty",
            }));
        }

        let patches = match parse_edit_many_unified_diff(&diff) {
            Ok(patches) => patches,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "error": error,
                }))
            }
        };
        let hunk_count = patches.iter().map(|patch| patch.hunks.len()).sum::<usize>();
        if hunk_count == 0 {
            return ToolExecutionResult::error(json!({
                "error": "unified diff must include at least one hunk",
            }));
        }
        if hunk_count > EDIT_MANY_MAX_EDITS {
            return ToolExecutionResult::error(json!({
                "error": format!(
                    "diff contains {} hunks, limit is {}",
                    hunk_count,
                    EDIT_MANY_MAX_EDITS
                ),
            }));
        }

        let mut plans = BTreeMap::<PathBuf, EditManyFilePlan>::new();
        let mut total_added_lines = 0usize;
        let mut total_removed_lines = 0usize;
        for (index, patch) in patches.iter().enumerate() {
            let resolved =
                match resolve_and_validate_path(&patch.path, &self.policy, PathMode::Edit) {
                    Ok(path) => path,
                    Err(error) => {
                        return ToolExecutionResult::error(json!({
                            "index": index,
                            "path": patch.path,
                            "error": error,
                        }))
                    }
                };
            if plans.contains_key(&resolved) {
                return ToolExecutionResult::error(json!({
                    "index": index,
                    "path": resolved.display().to_string(),
                    "error": "duplicate unified diff file patch",
                }));
            }
            if let Err(error) =
                validate_file_target(&resolved, PathMode::Edit, self.policy.enforce_regular_files)
            {
                return ToolExecutionResult::error(json!({
                    "index": index,
                    "path": resolved.display().to_string(),
                    "error": error,
                }));
            }
            if let Some(protected_path_result) =
                evaluate_protected_path_gate(&self.policy, "edit_many", &resolved)
            {
                return protected_path_result;
            }
            if let Some(rbac_result) = evaluate_tool_rbac_gate(
                self.policy.rbac_principal.as_deref(),
                "edit_many",
                self.policy.rbac_policy_path.as_deref(),
                json!({
                    "path": resolved.display().to_string(),
                    "find": "<unified-diff>",
                    "replace_bytes": diff.len(),
                }),
            ) {
                return rbac_result;
            }
            if let Some(approval_result) = evaluate_tool_approval_gate(ApprovalAction::ToolEdit {
                path: resolved.display().to_string(),
                find: "<unified-diff>".to_string(),
                replace_bytes: diff.len(),
            }) {
                return approval_result;
            }
            if let Some(rate_limit_result) = evaluate_tool_rate_limit_gate(
                &self.policy,
                "edit_many",
                json!({
                    "path": resolved.display().to_string(),
                    "find": "<unified-diff>",
                    "replace_bytes": diff.len(),
                }),
            ) {
                return rate_limit_result;
            }

            let source = match tokio::fs::read_to_string(&resolved).await {
                Ok(source) => source,
                Err(error) => {
                    return ToolExecutionResult::error(json!({
                        "index": index,
                        "path": resolved.display().to_string(),
                        "error": error.to_string(),
                    }))
                }
            };
            let applied = match apply_edit_many_unified_file_patch(&source, patch) {
                Ok(applied) => applied,
                Err(error) => {
                    return ToolExecutionResult::error(json!({
                        "index": index,
                        "path": resolved.display().to_string(),
                        "error": error,
                    }))
                }
            };
            if applied.updated.len() > self.policy.max_file_write_bytes {
                return ToolExecutionResult::error(json!({
                    "index": index,
                    "path": resolved.display().to_string(),
                    "error": format!(
                        "edited content is too large ({} bytes), limit is {} bytes",
                        applied.updated.len(),
                        self.policy.max_file_write_bytes
                    ),
                }));
            }
            total_added_lines = total_added_lines.saturating_add(applied.added_lines);
            total_removed_lines = total_removed_lines.saturating_add(applied.removed_lines);
            plans.insert(
                resolved.clone(),
                EditManyFilePlan {
                    requested_path: patch.path.clone(),
                    resolved: resolved.clone(),
                    updated: applied.updated,
                    patches: vec![json!({
                        "index": index,
                        "requested_path": patch.path,
                        "path": resolved.display().to_string(),
                        "hunks": patch.hunks.len(),
                        "added_lines": applied.added_lines,
                        "removed_lines": applied.removed_lines,
                    })],
                },
            );
        }

        edit_many_write_plans(
            plans,
            hunk_count,
            total_added_lines.saturating_add(total_removed_lines),
        )
        .await
    }
}

#[async_trait]
impl AgentTool for EditManyTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "edit_many".to_string(),
            description: "Apply multiple exact-string edits or a unified diff across one or more existing files in one checked batch before writing any file".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "edits": {
                        "type": "array",
                        "minItems": 1,
                        "maxItems": EDIT_MANY_MAX_EDITS,
                        "items": {
                            "type": "object",
                            "properties": {
                                "path": { "type": "string" },
                                "find": { "type": "string" },
                                "replace": { "type": "string" },
                                "all": { "type": "boolean", "default": false }
                            },
                            "required": ["path", "find", "replace"],
                            "additionalProperties": false
                        }
                    },
                    "diff": {
                        "type": "string",
                        "description": "Unified diff text for existing-file modifications. New files and deletes are not supported here; use write_many for new files."
                    }
                },
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let has_edits = arguments.get("edits").is_some();
        let has_diff = arguments.get("diff").is_some();
        if has_edits == has_diff {
            return ToolExecutionResult::error(json!({
                "error": "provide exactly one of 'edits' or 'diff'",
            }));
        }
        if has_diff {
            return self.execute_unified_diff(&arguments).await;
        }

        let edits = match arguments.get("edits").and_then(Value::as_array) {
            Some(edits) if !edits.is_empty() => edits,
            Some(_) => {
                return ToolExecutionResult::error(json!({
                    "error": "edits must include at least one entry"
                }))
            }
            None => {
                return ToolExecutionResult::error(json!({
                    "error": "missing required array field 'edits'"
                }))
            }
        };
        if edits.len() > EDIT_MANY_MAX_EDITS {
            return ToolExecutionResult::error(json!({
                "error": format!(
                    "edits contains {} entries, limit is {}",
                    edits.len(),
                    EDIT_MANY_MAX_EDITS
                ),
            }));
        }

        let mut plans = BTreeMap::<PathBuf, EditManyFilePlan>::new();
        let mut edit_count = 0usize;
        let mut total_replacements = 0usize;

        for (index, edit) in edits.iter().enumerate() {
            let path = match required_string(edit, "path") {
                Ok(path) => path,
                Err(error) => {
                    return ToolExecutionResult::error(json!({
                        "index": index,
                        "error": error,
                    }))
                }
            };
            let find = match required_string(edit, "find") {
                Ok(find) => find,
                Err(error) => {
                    return ToolExecutionResult::error(json!({
                        "index": index,
                        "path": path,
                        "error": error,
                    }))
                }
            };
            let replace = match required_string(edit, "replace") {
                Ok(replace) => replace,
                Err(error) => {
                    return ToolExecutionResult::error(json!({
                        "index": index,
                        "path": path,
                        "error": error,
                    }))
                }
            };
            if find.is_empty() {
                return ToolExecutionResult::error(json!({
                    "index": index,
                    "path": path,
                    "error": "'find' must not be empty",
                }));
            }
            let replace_all = edit.get("all").and_then(Value::as_bool).unwrap_or(false);

            let resolved = match resolve_and_validate_path(&path, &self.policy, PathMode::Edit) {
                Ok(path) => path,
                Err(error) => {
                    return ToolExecutionResult::error(json!({
                        "index": index,
                        "path": path,
                        "error": error,
                    }))
                }
            };
            if let Err(error) =
                validate_file_target(&resolved, PathMode::Edit, self.policy.enforce_regular_files)
            {
                return ToolExecutionResult::error(json!({
                    "index": index,
                    "path": resolved.display().to_string(),
                    "error": error,
                }));
            }
            if let Some(protected_path_result) =
                evaluate_protected_path_gate(&self.policy, "edit_many", &resolved)
            {
                return protected_path_result;
            }
            if let Some(rbac_result) = evaluate_tool_rbac_gate(
                self.policy.rbac_principal.as_deref(),
                "edit_many",
                self.policy.rbac_policy_path.as_deref(),
                json!({
                    "path": resolved.display().to_string(),
                    "find": find,
                    "replace_bytes": replace.len(),
                }),
            ) {
                return rbac_result;
            }
            if let Some(approval_result) = evaluate_tool_approval_gate(ApprovalAction::ToolEdit {
                path: resolved.display().to_string(),
                find: find.clone(),
                replace_bytes: replace.len(),
            }) {
                return approval_result;
            }
            if let Some(rate_limit_result) = evaluate_tool_rate_limit_gate(
                &self.policy,
                "edit_many",
                json!({
                    "path": resolved.display().to_string(),
                    "find": find.clone(),
                    "replace_bytes": replace.len(),
                }),
            ) {
                return rate_limit_result;
            }

            if !plans.contains_key(&resolved) {
                let source = match tokio::fs::read_to_string(&resolved).await {
                    Ok(source) => source,
                    Err(error) => {
                        return ToolExecutionResult::error(json!({
                            "index": index,
                            "path": resolved.display().to_string(),
                            "error": error.to_string(),
                        }))
                    }
                };
                plans.insert(
                    resolved.clone(),
                    EditManyFilePlan {
                        requested_path: path.clone(),
                        resolved: resolved.clone(),
                        updated: source,
                        patches: Vec::new(),
                    },
                );
            }

            let plan = plans.get_mut(&resolved).expect("plan inserted");
            let occurrences = plan.updated.matches(&find).count();
            if occurrences == 0 {
                return ToolExecutionResult::error(json!({
                    "index": index,
                    "path": resolved.display().to_string(),
                    "error": "target string not found",
                }));
            }
            plan.updated = if replace_all {
                plan.updated.replace(&find, &replace)
            } else {
                plan.updated.replacen(&find, &replace, 1)
            };
            if plan.updated.len() > self.policy.max_file_write_bytes {
                return ToolExecutionResult::error(json!({
                    "index": index,
                    "path": resolved.display().to_string(),
                    "error": format!(
                        "edited content is too large ({} bytes), limit is {} bytes",
                        plan.updated.len(),
                        self.policy.max_file_write_bytes
                    ),
                }));
            }
            let replacements = if replace_all { occurrences } else { 1 };
            edit_count = edit_count.saturating_add(1);
            total_replacements = total_replacements.saturating_add(replacements);
            plan.patches.push(json!({
                "index": index,
                "requested_path": path,
                "path": resolved.display().to_string(),
                "replacements": replacements,
            }));
        }

        let mut edited_files = Vec::with_capacity(plans.len());
        let mut applied_edits = Vec::with_capacity(edit_count);
        for (_, plan) in plans {
            if let Err(error) = tokio::fs::write(&plan.resolved, plan.updated.as_bytes()).await {
                return ToolExecutionResult::error(json!({
                    "path": plan.resolved.display().to_string(),
                    "error": error.to_string(),
                }));
            }
            applied_edits.extend(plan.patches);
            edited_files.push(json!({
                "requested_path": plan.requested_path,
                "path": plan.resolved.display().to_string(),
                "bytes_written": plan.updated.len(),
            }));
        }

        ToolExecutionResult::ok(json!({
            "file_count": edited_files.len(),
            "edit_count": edit_count,
            "replacements": total_replacements,
            "files_edited": edited_files,
            "edits_applied": applied_edits,
        }))
    }
}

async fn edit_many_write_plans(
    plans: BTreeMap<PathBuf, EditManyFilePlan>,
    edit_count: usize,
    total_replacements: usize,
) -> ToolExecutionResult {
    let mut edited_files = Vec::with_capacity(plans.len());
    let mut applied_edits = Vec::with_capacity(edit_count);
    for (_, plan) in plans {
        if let Err(error) = tokio::fs::write(&plan.resolved, plan.updated.as_bytes()).await {
            return ToolExecutionResult::error(json!({
                "path": plan.resolved.display().to_string(),
                "error": error.to_string(),
            }));
        }
        applied_edits.extend(plan.patches);
        edited_files.push(json!({
            "requested_path": plan.requested_path,
            "path": plan.resolved.display().to_string(),
            "bytes_written": plan.updated.len(),
        }));
    }

    ToolExecutionResult::ok(json!({
        "file_count": edited_files.len(),
        "edit_count": edit_count,
        "replacements": total_replacements,
        "files_edited": edited_files,
        "edits_applied": applied_edits,
    }))
}

#[derive(Debug)]
struct EditManyUnifiedFilePatch {
    path: String,
    hunks: Vec<EditManyUnifiedHunk>,
}

#[derive(Debug)]
struct EditManyUnifiedHunk {
    old_start: usize,
    lines: Vec<EditManyUnifiedHunkLine>,
}

#[derive(Debug)]
enum EditManyUnifiedHunkLine {
    Context(String),
    Remove(String),
    Add(String),
}

#[derive(Debug)]
struct EditManyUnifiedApplyResult {
    updated: String,
    added_lines: usize,
    removed_lines: usize,
}

fn parse_edit_many_unified_diff(diff: &str) -> Result<Vec<EditManyUnifiedFilePatch>, String> {
    let lines = diff.lines().collect::<Vec<_>>();
    let mut patches = Vec::new();
    let mut index = 0usize;
    while index < lines.len() {
        if !is_unified_file_header(&lines, index) {
            index = index.saturating_add(1);
            continue;
        }

        let old_path = parse_unified_header_path(lines[index], "--- ")?;
        let new_path = parse_unified_header_path(lines[index + 1], "+++ ")?;
        if old_path == "/dev/null" || new_path == "/dev/null" {
            return Err("edit_many unified diff only supports existing-file modifications; use write_many for new files".to_string());
        }
        let old_normalized = normalize_unified_diff_path(&old_path)?;
        let path = normalize_unified_diff_path(&new_path)?;
        if old_normalized != path {
            return Err(format!(
                "edit_many unified diff does not support renames: '{}' -> '{}'",
                old_normalized, path
            ));
        }
        index = index.saturating_add(2);

        let mut hunks = Vec::new();
        while index < lines.len() {
            if is_unified_file_header(&lines, index) {
                break;
            }
            let line = lines[index];
            if !line.starts_with("@@") {
                index = index.saturating_add(1);
                continue;
            }

            let old_start = parse_unified_hunk_old_start(line)?;
            index = index.saturating_add(1);
            let mut hunk_lines = Vec::new();
            while index < lines.len()
                && !lines[index].starts_with("@@")
                && !is_unified_file_boundary(&lines, index)
            {
                let hunk_line = lines[index];
                if hunk_line.starts_with("\\ ") {
                    index = index.saturating_add(1);
                    continue;
                }
                let Some(prefix) = hunk_line.chars().next() else {
                    return Err("malformed unified diff hunk line without prefix".to_string());
                };
                let text = hunk_line
                    .get(prefix.len_utf8()..)
                    .unwrap_or_default()
                    .to_string();
                match prefix {
                    ' ' => hunk_lines.push(EditManyUnifiedHunkLine::Context(text)),
                    '-' => hunk_lines.push(EditManyUnifiedHunkLine::Remove(text)),
                    '+' => hunk_lines.push(EditManyUnifiedHunkLine::Add(text)),
                    _ => {
                        return Err(format!(
                            "malformed unified diff hunk line prefix '{}'",
                            prefix
                        ))
                    }
                }
                index = index.saturating_add(1);
            }
            if hunk_lines.is_empty() {
                return Err("unified diff hunk must include at least one line".to_string());
            }
            hunks.push(EditManyUnifiedHunk {
                old_start,
                lines: hunk_lines,
            });
        }

        if hunks.is_empty() {
            return Err(format!("unified diff file patch '{}' has no hunks", path));
        }
        patches.push(EditManyUnifiedFilePatch { path, hunks });
    }

    if patches.is_empty() {
        return Err("unified diff must include at least one file patch".to_string());
    }
    Ok(patches)
}

fn is_unified_file_header(lines: &[&str], index: usize) -> bool {
    lines
        .get(index)
        .is_some_and(|line| line.starts_with("--- "))
        && lines
            .get(index.saturating_add(1))
            .is_some_and(|line| line.starts_with("+++ "))
}

fn is_unified_file_boundary(lines: &[&str], index: usize) -> bool {
    lines
        .get(index)
        .is_some_and(|line| line.starts_with("diff --git "))
        || is_unified_file_header(lines, index)
}

fn parse_unified_header_path(line: &str, prefix: &str) -> Result<String, String> {
    let path = line
        .strip_prefix(prefix)
        .ok_or_else(|| format!("missing unified diff header prefix '{}'", prefix.trim()))?
        .split('\t')
        .next()
        .unwrap_or_default()
        .trim();
    if path.is_empty() {
        return Err("unified diff header path must not be empty".to_string());
    }
    Ok(path.to_string())
}

fn normalize_unified_diff_path(path: &str) -> Result<String, String> {
    let normalized = path
        .strip_prefix("a/")
        .or_else(|| path.strip_prefix("b/"))
        .unwrap_or(path)
        .trim();
    if normalized.is_empty() {
        return Err("unified diff path must not be empty".to_string());
    }
    Ok(normalized.to_string())
}

fn parse_unified_hunk_old_start(header: &str) -> Result<usize, String> {
    let body = header
        .strip_prefix("@@")
        .and_then(|rest| rest.split_once("@@").map(|(body, _)| body.trim()))
        .ok_or_else(|| format!("malformed unified diff hunk header '{}'", header))?;
    let mut ranges = body.split_whitespace();
    let old_range = ranges
        .next()
        .ok_or_else(|| format!("malformed unified diff hunk header '{}'", header))?;
    let new_range = ranges
        .next()
        .ok_or_else(|| format!("malformed unified diff hunk header '{}'", header))?;
    if !old_range.starts_with('-') || !new_range.starts_with('+') {
        return Err(format!("malformed unified diff hunk header '{}'", header));
    }
    let old_start = parse_unified_range_start(&old_range[1..], "old")?;
    parse_unified_range_start(&new_range[1..], "new")?;
    Ok(old_start)
}

fn parse_unified_range_start(range: &str, label: &str) -> Result<usize, String> {
    let (start, count) = range
        .split_once(',')
        .map_or((range, None), |(start, count)| (start, Some(count)));
    if start.is_empty() {
        return Err(format!("malformed unified diff {label} start"));
    }
    if let Some(count) = count {
        count
            .parse::<usize>()
            .map_err(|_| format!("malformed unified diff {label} count '{}'", count))?;
    }
    start
        .parse::<usize>()
        .map_err(|_| format!("malformed unified diff {label} start '{}'", start))
}

fn apply_edit_many_unified_file_patch(
    source: &str,
    patch: &EditManyUnifiedFilePatch,
) -> Result<EditManyUnifiedApplyResult, String> {
    let mut lines = source.split('\n').map(str::to_string).collect::<Vec<_>>();
    let mut offset: isize = 0;
    let mut added_lines = 0usize;
    let mut removed_lines = 0usize;

    for (hunk_index, hunk) in patch.hunks.iter().enumerate() {
        let base_index = if hunk.old_start == 0 {
            0isize
        } else {
            hunk.old_start as isize - 1
        };
        let target_index = base_index.saturating_add(offset);
        if target_index < 0 {
            return Err(format!(
                "unified diff hunk {} resolves before start of file",
                hunk_index
            ));
        }
        let start = target_index as usize;
        if start > lines.len() {
            return Err(format!(
                "unified diff hunk {} starts past end of file",
                hunk_index
            ));
        }

        let mut expected_old = Vec::new();
        let mut replacement = Vec::new();
        for line in &hunk.lines {
            match line {
                EditManyUnifiedHunkLine::Context(text) => {
                    expected_old.push(text.clone());
                    replacement.push(text.clone());
                }
                EditManyUnifiedHunkLine::Remove(text) => {
                    expected_old.push(text.clone());
                    removed_lines = removed_lines.saturating_add(1);
                }
                EditManyUnifiedHunkLine::Add(text) => {
                    replacement.push(text.clone());
                    added_lines = added_lines.saturating_add(1);
                }
            }
        }

        let end = start.saturating_add(expected_old.len());
        if end > lines.len() {
            return Err(format!(
                "unified diff hunk {} extends past end of file",
                hunk_index
            ));
        }
        if lines[start..end] != expected_old {
            return Err(format!(
                "unified diff hunk {} did not match current file contents",
                hunk_index
            ));
        }

        let expected_len = expected_old.len();
        let replacement_len = replacement.len();
        lines.splice(start..end, replacement);
        offset = offset.saturating_add(replacement_len as isize - expected_len as isize);
    }

    Ok(EditManyUnifiedApplyResult {
        updated: lines.join("\n"),
        added_lines,
        removed_lines,
    })
}

/// Public struct `BranchTool` used across Tau components.
pub struct BranchTool {
    policy: Arc<ToolPolicy>,
}

impl BranchTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AgentTool for BranchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "branch".to_string(),
            description: "Append a branch prompt to a session lineage with explicit parent control"
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to target session JSONL/SQLite file"
                    },
                    "prompt": {
                        "type": "string",
                        "description": format!(
                            "Branch prompt text (max {} characters)",
                            BRANCH_TOOL_MAX_PROMPT_CHARS
                        )
                    },
                    "parent_id": {
                        "type": "integer",
                        "description": "Optional parent entry id. Defaults to session head."
                    }
                },
                "required": ["path", "prompt"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let path = match required_string(&arguments, "path") {
            Ok(path) => path,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        let prompt = match required_string(&arguments, "prompt") {
            Ok(prompt) => prompt,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        let parent_id = match optional_u64(&arguments, "parent_id") {
            Ok(parent_id) => parent_id,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        if prompt.trim().is_empty() {
            return ToolExecutionResult::error(json!({
                "tool": "branch",
                "path": path,
                "reason_code": "branch_prompt_empty",
                "error": "prompt must not be empty",
            }));
        }
        if prompt.chars().count() > BRANCH_TOOL_MAX_PROMPT_CHARS {
            return ToolExecutionResult::error(json!({
                "tool": "branch",
                "path": path,
                "reason_code": "branch_prompt_too_large",
                "error": format!(
                    "prompt exceeds max length of {} characters",
                    BRANCH_TOOL_MAX_PROMPT_CHARS
                ),
            }));
        }

        let resolved = match resolve_and_validate_path(&path, &self.policy, PathMode::Write) {
            Ok(path) => path,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "path": path,
                    "error": error,
                }))
            }
        };
        if let Err(error) = validate_file_target(
            &resolved,
            PathMode::Write,
            self.policy.enforce_regular_files,
        ) {
            return ToolExecutionResult::error(json!({
                "path": resolved.display().to_string(),
                "error": error,
            }));
        }

        let mut store = match SessionStore::load(&resolved) {
            Ok(store) => store,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "branch",
                    "path": resolved.display().to_string(),
                    "reason_code": "session_branch_load_error",
                    "error": format!("failed to load session: {error}"),
                }))
            }
        };

        let before_entries = store.entries().len();
        let previous_head_id = store.head_id();
        let selected_parent_id = parent_id.or(previous_head_id);
        let branch_message = Message::user(prompt.clone());
        let branch_head_id = match store.append_messages(selected_parent_id, &[branch_message]) {
            Ok(Some(head)) => head,
            Ok(None) => {
                return ToolExecutionResult::error(json!({
                    "tool": "branch",
                    "path": resolved.display().to_string(),
                    "reason_code": "session_branch_append_noop",
                    "error": "branch append produced no new head",
                }))
            }
            Err(error) => {
                let error_string = error.to_string();
                let reason_code = if error_string.contains("parent id")
                    && error_string.contains("does not exist")
                {
                    "session_branch_parent_not_found"
                } else {
                    "session_branch_append_error"
                };
                return ToolExecutionResult::error(json!({
                    "tool": "branch",
                    "path": resolved.display().to_string(),
                    "reason_code": reason_code,
                    "parent_id": selected_parent_id,
                    "error": error_string,
                }));
            }
        };
        let after_entries = store.entries().len();

        ToolExecutionResult::ok(json!({
            "tool": "branch",
            "path": resolved.display().to_string(),
            "reason_code": "session_branch_created",
            "summary": "branch entry created",
            "selected_parent_id": selected_parent_id,
            "previous_head_id": previous_head_id,
            "branch_head_id": branch_head_id,
            "before_entries": before_entries,
            "after_entries": after_entries,
            "appended_entries": after_entries.saturating_sub(before_entries),
            "prompt_preview": session_message_preview(&Message::user(prompt)),
        }))
    }
}

/// Public struct `UndoTool` used across Tau components.
pub struct UndoTool {
    policy: Arc<ToolPolicy>,
}

impl UndoTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AgentTool for UndoTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "undo".to_string(),
            description:
                "Move a session's active navigation head backward using persisted undo history"
                    .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to target session JSONL file"
                    }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let path = match required_string(&arguments, "path") {
            Ok(path) => path,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };

        let resolved = match resolve_and_validate_path(&path, &self.policy, PathMode::Write) {
            Ok(path) => path,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "path": path,
                    "error": error,
                }))
            }
        };
        if let Err(error) = validate_file_target(
            &resolved,
            PathMode::Write,
            self.policy.enforce_regular_files,
        ) {
            return ToolExecutionResult::error(json!({
                "path": resolved.display().to_string(),
                "error": error,
            }));
        }

        let store = match SessionStore::load(&resolved) {
            Ok(store) => store,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "path": resolved.display().to_string(),
                    "reason_code": "session_navigation_load_error",
                    "error": format!("failed to load session: {error}"),
                }))
            }
        };

        let active_head = match resolve_session_navigation_head(&store) {
            Ok(active_head) => active_head,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "undo",
                    "path": resolved.display().to_string(),
                    "reason_code": "session_navigation_state_error",
                    "error": format!("failed to resolve navigation state: {error}"),
                }))
            }
        };
        let mut runtime = SessionRuntime { store, active_head };
        let transition = match undo_session_head(&mut runtime) {
            Ok(transition) => transition,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "undo",
                    "path": resolved.display().to_string(),
                    "reason_code": "session_navigation_state_error",
                    "error": format!("failed to execute undo: {error}"),
                }))
            }
        };

        if !transition.changed {
            return ToolExecutionResult::error(json!({
                "tool": "undo",
                "path": resolved.display().to_string(),
                "reason_code": "session_undo_empty_stack",
                "summary": "undo unavailable: no prior navigation target",
                "previous_head_id": transition.previous_head,
                "active_head_id": transition.active_head,
                "undo_depth": transition.undo_depth,
                "redo_depth": transition.redo_depth,
                "skipped_invalid_targets": transition.skipped_invalid_targets,
            }));
        }

        ToolExecutionResult::ok(json!({
            "tool": "undo",
            "path": resolved.display().to_string(),
            "reason_code": "session_undo_applied",
            "summary": "undo complete",
            "previous_head_id": transition.previous_head,
            "active_head_id": transition.active_head,
            "undo_depth": transition.undo_depth,
            "redo_depth": transition.redo_depth,
            "skipped_invalid_targets": transition.skipped_invalid_targets,
        }))
    }
}

/// Public struct `RedoTool` used across Tau components.
pub struct RedoTool {
    policy: Arc<ToolPolicy>,
}

impl RedoTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AgentTool for RedoTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "redo".to_string(),
            description:
                "Move a session's active navigation head forward using persisted redo history"
                    .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to target session JSONL file"
                    }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let path = match required_string(&arguments, "path") {
            Ok(path) => path,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };

        let resolved = match resolve_and_validate_path(&path, &self.policy, PathMode::Write) {
            Ok(path) => path,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "path": path,
                    "error": error,
                }))
            }
        };
        if let Err(error) = validate_file_target(
            &resolved,
            PathMode::Write,
            self.policy.enforce_regular_files,
        ) {
            return ToolExecutionResult::error(json!({
                "path": resolved.display().to_string(),
                "error": error,
            }));
        }

        let store = match SessionStore::load(&resolved) {
            Ok(store) => store,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "path": resolved.display().to_string(),
                    "reason_code": "session_navigation_load_error",
                    "error": format!("failed to load session: {error}"),
                }))
            }
        };

        let active_head = match resolve_session_navigation_head(&store) {
            Ok(active_head) => active_head,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "redo",
                    "path": resolved.display().to_string(),
                    "reason_code": "session_navigation_state_error",
                    "error": format!("failed to resolve navigation state: {error}"),
                }))
            }
        };
        let mut runtime = SessionRuntime { store, active_head };
        let transition = match redo_session_head(&mut runtime) {
            Ok(transition) => transition,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "redo",
                    "path": resolved.display().to_string(),
                    "reason_code": "session_navigation_state_error",
                    "error": format!("failed to execute redo: {error}"),
                }))
            }
        };

        if !transition.changed {
            return ToolExecutionResult::error(json!({
                "tool": "redo",
                "path": resolved.display().to_string(),
                "reason_code": "session_redo_empty_stack",
                "summary": "redo unavailable: no prior undone navigation target",
                "previous_head_id": transition.previous_head,
                "active_head_id": transition.active_head,
                "undo_depth": transition.undo_depth,
                "redo_depth": transition.redo_depth,
                "skipped_invalid_targets": transition.skipped_invalid_targets,
            }));
        }

        ToolExecutionResult::ok(json!({
            "tool": "redo",
            "path": resolved.display().to_string(),
            "reason_code": "session_redo_applied",
            "summary": "redo complete",
            "previous_head_id": transition.previous_head,
            "active_head_id": transition.active_head,
            "undo_depth": transition.undo_depth,
            "redo_depth": transition.redo_depth,
            "skipped_invalid_targets": transition.skipped_invalid_targets,
        }))
    }
}

/// Public struct `SkipTool` used across Tau components.
pub struct SkipTool {
    _policy: Arc<ToolPolicy>,
}

impl SkipTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { _policy: policy }
    }
}

#[async_trait]
impl AgentTool for SkipTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "skip".to_string(),
            description: "Suppress outbound user-facing response for the current turn".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "reason": {
                        "type": "string",
                        "description": "Optional audit/debug reason for suppressing the response"
                    }
                },
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let reason = match optional_string(&arguments, "reason") {
            Ok(reason) => reason.unwrap_or_default(),
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        ToolExecutionResult::ok(json!({
            "skip_response": true,
            "reason": reason,
            "reason_code": "skip_suppressed",
        }))
    }
}

/// Public struct `ReactTool` used across Tau components.
pub struct ReactTool {
    _policy: Arc<ToolPolicy>,
}

impl ReactTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { _policy: policy }
    }
}

#[async_trait]
impl AgentTool for ReactTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "react".to_string(),
            description: "Request emoji reaction delivery and suppress textual reply".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "emoji": {
                        "type": "string",
                        "description": "Emoji to dispatch to the target message"
                    },
                    "message_id": {
                        "type": "string",
                        "description": "Optional target message id; defaults to current event id in channel runtimes"
                    }
                },
                "required": ["emoji"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let emoji = match required_string(&arguments, "emoji") {
            Ok(value) => value.trim().to_string(),
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        if emoji.is_empty() {
            return ToolExecutionResult::error(json!({
                "error": "field 'emoji' must not be empty",
                "reason_code": "react_invalid_emoji",
            }));
        }
        let message_id = match optional_string(&arguments, "message_id") {
            Ok(value) => value
                .map(|raw| raw.trim().to_string())
                .filter(|value| !value.is_empty()),
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        ToolExecutionResult::ok(json!({
            "react_response": true,
            "emoji": emoji,
            "message_id": message_id,
            "reason_code": "react_requested",
            "suppress_response": true,
        }))
    }
}

/// Public struct `SendFileTool` used across Tau components.
pub struct SendFileTool {
    _policy: Arc<ToolPolicy>,
}

impl SendFileTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { _policy: policy }
    }
}

#[async_trait]
impl AgentTool for SendFileTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "send_file".to_string(),
            description: "Request file delivery and suppress textual reply".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path or URL identifying the file to deliver"
                    },
                    "message": {
                        "type": "string",
                        "description": "Optional caption/message to include with the file"
                    }
                },
                "required": ["file_path"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let file_path = match required_string(&arguments, "file_path") {
            Ok(value) => value.trim().to_string(),
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        if file_path.is_empty() {
            return ToolExecutionResult::error(json!({
                "error": "field 'file_path' must not be empty",
                "reason_code": "send_file_invalid_path",
            }));
        }
        let message = match optional_string(&arguments, "message") {
            Ok(value) => value
                .map(|raw| raw.trim().to_string())
                .filter(|value| !value.is_empty()),
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        ToolExecutionResult::ok(json!({
            "send_file_response": true,
            "file_path": file_path,
            "message": message,
            "reason_code": "send_file_requested",
            "suppress_response": true,
        }))
    }
}

/// Searches file contents using substring or regex matching, returning structured results.
pub struct GrepTool {
    policy: Arc<ToolPolicy>,
}

impl GrepTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AgentTool for GrepTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "grep".to_string(),
            description: "Search file contents using substring or regex patterns. Use this to find function definitions, usage sites, imports, or any text pattern across a codebase. Searches recursively, skipping hidden directories. Returns matching lines with file paths and line numbers.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Search pattern (substring or regex)" },
                    "path": { "type": "string", "description": "File or directory to search in" },
                    "include": { "type": "string", "description": "Glob pattern to filter files (e.g. '*.rs', '*.py')" },
                    "max_results": { "type": "integer", "description": "Maximum number of matches to return (default 50)" }
                },
                "required": ["pattern", "path"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let pattern = match required_string(&arguments, "pattern") {
            Ok(pattern) => pattern,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        let path = match required_string(&arguments, "path") {
            Ok(path) => path,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        let include = arguments
            .get("include")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let max_results = arguments
            .get("max_results")
            .and_then(Value::as_u64)
            .unwrap_or(50) as usize;

        let resolved = match resolve_and_validate_path(&path, &self.policy, PathMode::Read) {
            Ok(path) => path,
            Err(error) => {
                return ToolExecutionResult::error(json!({ "path": path, "error": error }))
            }
        };

        let regex = match regex::Regex::new(&pattern) {
            Ok(regex) => regex,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "pattern": pattern,
                    "error": format!("invalid regex pattern: {error}"),
                }))
            }
        };

        let include_glob = if include.is_empty() {
            None
        } else {
            match glob::Pattern::new(&include) {
                Ok(glob) => Some(glob),
                Err(error) => {
                    return ToolExecutionResult::error(json!({
                        "include": include,
                        "error": format!("invalid glob pattern: {error}"),
                    }))
                }
            }
        };

        let mut matches = Vec::new();
        let mut files_searched = 0usize;

        if resolved.is_file() {
            files_searched = 1;
            grep_file(&resolved, &regex, max_results, &mut matches);
        } else if resolved.is_dir() {
            grep_directory(
                &resolved,
                &regex,
                &include_glob,
                max_results,
                &mut matches,
                &mut files_searched,
            );
        } else {
            return ToolExecutionResult::error(json!({
                "path": resolved.display().to_string(),
                "error": "path is not a file or directory",
            }));
        }

        ToolExecutionResult::ok(json!({
            "pattern": pattern,
            "path": resolved.display().to_string(),
            "matches": matches,
            "total_matches": matches.len(),
            "files_searched": files_searched,
            "truncated": matches.len() >= max_results,
        }))
    }
}

fn grep_file(path: &Path, regex: &regex::Regex, max_results: usize, matches: &mut Vec<Value>) {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return,
    };
    for (line_number, line) in content.lines().enumerate() {
        if matches.len() >= max_results {
            break;
        }
        if regex.is_match(line) {
            matches.push(json!({
                "file": path.display().to_string(),
                "line": line_number + 1,
                "content": line,
            }));
        }
    }
}

fn grep_directory(
    dir: &Path,
    regex: &regex::Regex,
    include_glob: &Option<glob::Pattern>,
    max_results: usize,
    matches: &mut Vec<Value>,
    files_searched: &mut usize,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries {
        if matches.len() >= max_results {
            break;
        }
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden directories
            if path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with('.'))
            {
                continue;
            }
            grep_directory(
                &path,
                regex,
                include_glob,
                max_results,
                matches,
                files_searched,
            );
        } else if path.is_file() {
            if let Some(glob) = include_glob {
                if !glob.matches_path_with(
                    &path,
                    glob::MatchOptions {
                        case_sensitive: true,
                        require_literal_separator: false,
                        require_literal_leading_dot: true,
                    },
                ) {
                    // Also try matching just the file name
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                    if !glob.matches(&file_name) {
                        continue;
                    }
                }
            }
            *files_searched += 1;
            grep_file(&path, regex, max_results, matches);
        }
    }
}

/// Finds files matching a glob pattern, returning structured results.
pub struct GlobTool {
    policy: Arc<ToolPolicy>,
}

impl GlobTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AgentTool for GlobTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "glob".to_string(),
            description: "Find files matching a glob pattern. Use this to discover project structure, locate config files, or find all files of a type. Supports recursive patterns like 'src/**/*.rs'. Returns file paths with size metadata.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Glob pattern (e.g. 'src/**/*.rs', '*.toml')" },
                    "path": { "type": "string", "description": "Root directory to search from (default: working directory)" },
                    "max_results": { "type": "integer", "description": "Maximum number of results (default 100)" }
                },
                "required": ["pattern"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let pattern = match required_string(&arguments, "pattern") {
            Ok(pattern) => pattern,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        let max_results = arguments
            .get("max_results")
            .and_then(Value::as_u64)
            .unwrap_or(100) as usize;

        let base_path = if let Some(path) = arguments.get("path").and_then(Value::as_str) {
            match resolve_and_validate_path(path, &self.policy, PathMode::Read) {
                Ok(path) => path,
                Err(error) => {
                    return ToolExecutionResult::error(json!({ "path": path, "error": error }))
                }
            }
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        };

        let full_pattern = if pattern.starts_with('/') || pattern.starts_with("./") {
            pattern.clone()
        } else {
            format!("{}/{}", base_path.display(), pattern)
        };

        let glob_results = match glob::glob(&full_pattern) {
            Ok(paths) => paths,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "pattern": pattern,
                    "error": format!("invalid glob pattern: {error}"),
                }))
            }
        };

        let mut files = Vec::new();
        for entry in glob_results {
            if files.len() >= max_results {
                break;
            }
            let path = match entry {
                Ok(path) => path,
                Err(_) => continue,
            };
            let metadata = std::fs::metadata(&path).ok();
            let is_dir = metadata.as_ref().is_some_and(|m| m.is_dir());
            let size = metadata.as_ref().map_or(0, |m| m.len());
            files.push(json!({
                "path": path.display().to_string(),
                "is_dir": is_dir,
                "size": size,
            }));
        }

        ToolExecutionResult::ok(json!({
            "pattern": pattern,
            "base_path": base_path.display().to_string(),
            "files": files,
            "total": files.len(),
            "truncated": files.len() >= max_results,
        }))
    }
}

/// Lists directory contents with metadata.
pub struct ListDirectoryTool {
    policy: Arc<ToolPolicy>,
}

impl ListDirectoryTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AgentTool for ListDirectoryTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "ls".to_string(),
            description: "List directory contents with file names, types, and sizes. Use this to understand directory layout before navigating or modifying files. Supports recursive listing with configurable depth.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Directory path to list" },
                    "recursive": { "type": "boolean", "description": "List recursively (default false)" },
                    "max_depth": { "type": "integer", "description": "Maximum recursion depth (default 3)" }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let path = match required_string(&arguments, "path") {
            Ok(path) => path,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        let recursive = arguments
            .get("recursive")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let max_depth = arguments
            .get("max_depth")
            .and_then(Value::as_u64)
            .unwrap_or(3) as usize;

        let resolved = match resolve_and_validate_path(&path, &self.policy, PathMode::Read) {
            Ok(path) => path,
            Err(error) => {
                return ToolExecutionResult::error(json!({ "path": path, "error": error }))
            }
        };

        if !resolved.is_dir() {
            return ToolExecutionResult::error(json!({
                "path": resolved.display().to_string(),
                "error": "path is not a directory",
            }));
        }

        let mut entries = Vec::new();
        list_directory_entries(&resolved, recursive, max_depth, 0, &mut entries);

        ToolExecutionResult::ok(json!({
            "path": resolved.display().to_string(),
            "entries": entries,
            "total": entries.len(),
        }))
    }
}

fn list_directory_entries(
    dir: &Path,
    recursive: bool,
    max_depth: usize,
    current_depth: usize,
    entries: &mut Vec<Value>,
) {
    let dir_entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    let mut sorted_entries: Vec<_> = dir_entries.filter_map(|e| e.ok()).collect();
    sorted_entries.sort_by_key(|e| e.file_name());

    for entry in sorted_entries {
        if entries.len() >= 1000 {
            break;
        }
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let metadata = entry.metadata().ok();
        let is_dir = metadata.as_ref().is_some_and(|m| m.is_dir());
        let size = metadata.as_ref().map_or(0, |m| m.len());
        let file_type = if is_dir { "dir" } else { "file" };

        entries.push(json!({
            "name": file_name,
            "path": path.display().to_string(),
            "type": file_type,
            "size": size,
        }));

        if recursive && is_dir && current_depth < max_depth && !file_name.starts_with('.') {
            list_directory_entries(&path, recursive, max_depth, current_depth + 1, entries);
        }
    }
}

/// Public struct `HttpTool` used across Tau components.
pub struct HttpTool {
    policy: Arc<ToolPolicy>,
}

impl HttpTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }

    fn ssrf_guard(&self) -> SsrfGuard {
        SsrfGuard::new(SsrfProtectionConfig {
            enabled: true,
            allow_http: self.policy.http_allow_http,
            allow_private_network: self.policy.http_allow_private_network,
        })
    }

    fn ssrf_violation_result(
        &self,
        method: &Method,
        request_url: &str,
        endpoint: &str,
        violation: SsrfViolation,
    ) -> ToolExecutionResult {
        let retryable = violation.reason_code == "delivery_ssrf_dns_resolution_failed";
        ToolExecutionResult::error(json!({
            "policy_rule": "ssrf_guard",
            "tool": "http",
            "method": method.as_str(),
            "url": request_url,
            "final_url": endpoint,
            "reason_code": violation.reason_code,
            "retryable": retryable,
            "error": violation.detail,
        }))
    }
}

#[async_trait]
impl AgentTool for HttpTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "http".to_string(),
            description:
                "Send bounded outbound HTTP requests (GET/POST/PUT/DELETE) with SSRF guardrails"
                    .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "Absolute target URL for the outbound request"
                    },
                    "method": {
                        "type": "string",
                        "description": "HTTP method: GET, POST, PUT, or DELETE (defaults to GET)"
                    },
                    "headers": {
                        "type": "object",
                        "description": "Optional string headers forwarded to the request",
                        "additionalProperties": { "type": "string" }
                    },
                    "json": {
                        "description": "Optional JSON payload for POST/PUT/DELETE requests"
                    },
                    "timeout_ms": {
                        "type": "integer",
                        "description": "Optional per-request timeout (must be <= policy timeout cap)"
                    },
                    "max_response_bytes": {
                        "type": "integer",
                        "description": "Optional per-request response cap (must be <= policy response cap)"
                    }
                },
                "required": ["url"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let request_url = match required_string(&arguments, "url") {
            Ok(url) => url,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        let method = match parse_http_method(&arguments) {
            Ok(method) => method,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "http",
                    "url": request_url.as_str(),
                    "policy_rule": "http_method",
                    "reason_code": "http_invalid_method",
                    "error": error,
                }))
            }
        };
        let headers = match parse_http_headers(&arguments) {
            Ok(headers) => headers,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "http",
                    "method": method.as_str(),
                    "url": request_url.as_str(),
                    "policy_rule": "http_headers",
                    "reason_code": "http_invalid_headers",
                    "error": error,
                }))
            }
        };
        let header_names = headers
            .iter()
            .map(|(name, _value)| name.as_str().to_string())
            .collect::<Vec<_>>();
        let json_payload = arguments.get("json").cloned();
        if method == Method::GET && json_payload.is_some() {
            return ToolExecutionResult::error(json!({
                "tool": "http",
                "method": method.as_str(),
                "url": request_url.as_str(),
                "policy_rule": "http_method",
                "reason_code": "http_body_not_allowed",
                "error": "GET requests do not support a JSON payload",
            }));
        }

        let timeout_override_ms = match optional_positive_u64(&arguments, "timeout_ms") {
            Ok(value) => value,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };
        if let Some(timeout_override_ms) = timeout_override_ms {
            if timeout_override_ms > self.policy.http_timeout_ms {
                return ToolExecutionResult::error(json!({
                    "tool": "http",
                    "method": method.as_str(),
                    "url": request_url.as_str(),
                    "policy_rule": "http_timeout_ms",
                    "reason_code": "http_timeout_exceeds_policy",
                    "timeout_ms": timeout_override_ms,
                    "max_timeout_ms": self.policy.http_timeout_ms,
                    "error": format!(
                        "requested timeout {} ms exceeds policy cap {} ms",
                        timeout_override_ms,
                        self.policy.http_timeout_ms
                    ),
                }));
            }
        }
        let effective_timeout_ms = timeout_override_ms
            .unwrap_or(self.policy.http_timeout_ms)
            .max(1);

        let response_limit_override =
            match optional_positive_usize(&arguments, "max_response_bytes") {
                Ok(value) => value,
                Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
            };
        if let Some(limit) = response_limit_override {
            if limit > self.policy.http_max_response_bytes {
                return ToolExecutionResult::error(json!({
                    "tool": "http",
                    "method": method.as_str(),
                    "url": request_url.as_str(),
                    "policy_rule": "http_max_response_bytes",
                    "reason_code": "http_response_cap_exceeds_policy",
                    "max_response_bytes": limit,
                    "policy_max_response_bytes": self.policy.http_max_response_bytes,
                    "error": format!(
                        "requested response cap {} bytes exceeds policy cap {} bytes",
                        limit,
                        self.policy.http_max_response_bytes
                    ),
                }));
            }
        }
        let effective_response_limit = response_limit_override
            .unwrap_or(self.policy.http_max_response_bytes)
            .max(1);

        if let Some(rbac_result) = evaluate_tool_rbac_gate(
            self.policy.rbac_principal.as_deref(),
            "http",
            self.policy.rbac_policy_path.as_deref(),
            json!({
                "method": method.as_str(),
                "url": request_url.as_str(),
                "timeout_ms": effective_timeout_ms,
                "max_response_bytes": effective_response_limit,
                "headers": header_names.clone(),
                "has_json_payload": json_payload.is_some(),
            }),
        ) {
            return rbac_result;
        }

        if let Some(rate_limit_result) = evaluate_tool_rate_limit_gate(
            &self.policy,
            "http",
            json!({
                "method": method.as_str(),
                "url": request_url.as_str(),
                "timeout_ms": effective_timeout_ms,
                "max_response_bytes": effective_response_limit,
            }),
        ) {
            return rate_limit_result;
        }

        let client = match reqwest::Client::builder()
            .timeout(Duration::from_millis(effective_timeout_ms))
            .redirect(RedirectPolicy::none())
            .build()
        {
            Ok(client) => client,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "http",
                    "method": method.as_str(),
                    "url": request_url.as_str(),
                    "reason_code": "http_client_build_failed",
                    "error": format!("failed to initialize outbound HTTP client: {error}"),
                }))
            }
        };

        let ssrf_guard = self.ssrf_guard();
        let mut endpoint = match ssrf_guard.parse_and_validate_url(&request_url).await {
            Ok(url) => url,
            Err(violation) => {
                return self.ssrf_violation_result(
                    &method,
                    &request_url,
                    request_url.as_str(),
                    violation,
                )
            }
        };

        let started_at = Instant::now();
        let mut redirect_count = 0usize;
        loop {
            let mut request_builder = client.request(method.clone(), endpoint.clone());
            for (header_name, header_value) in &headers {
                request_builder = request_builder.header(header_name, header_value);
            }
            if let Some(payload) = &json_payload {
                request_builder = request_builder.json(payload);
            }

            let response = match request_builder.send().await {
                Ok(response) => response,
                Err(error) => {
                    let reason_code = if error.is_timeout() {
                        "http_request_timeout"
                    } else {
                        "http_transport_error"
                    };
                    let retryable = error.is_timeout() || error.is_connect();
                    return ToolExecutionResult::error(json!({
                        "tool": "http",
                        "method": method.as_str(),
                        "url": request_url.as_str(),
                        "final_url": endpoint.as_str(),
                        "reason_code": reason_code,
                        "retryable": retryable,
                        "error": error.to_string(),
                        "duration_ms": started_at.elapsed().as_millis(),
                    }));
                }
            };

            let status = response.status();
            if status.is_redirection() {
                if redirect_count >= self.policy.http_max_redirects {
                    return ToolExecutionResult::error(json!({
                        "tool": "http",
                        "method": method.as_str(),
                        "url": request_url.as_str(),
                        "final_url": endpoint.as_str(),
                        "policy_rule": "http_max_redirects",
                        "reason_code": "http_redirect_limit_exceeded",
                        "redirect_count": redirect_count,
                        "max_redirects": self.policy.http_max_redirects,
                        "http_status": status.as_u16(),
                        "error": format!(
                            "redirect count exceeded configured max_redirects={} for endpoint '{}'",
                            self.policy.http_max_redirects,
                            endpoint,
                        ),
                    }));
                }

                let location_header = match response.headers().get(reqwest::header::LOCATION) {
                    Some(location) => location,
                    None => {
                        return ToolExecutionResult::error(json!({
                            "tool": "http",
                            "method": method.as_str(),
                            "url": request_url.as_str(),
                            "final_url": endpoint.as_str(),
                            "reason_code": "http_redirect_missing_location",
                            "redirect_count": redirect_count,
                            "http_status": status.as_u16(),
                            "error": format!(
                                "received redirect status {} without a Location header",
                                status.as_u16()
                            ),
                        }))
                    }
                };

                let location = match location_header.to_str() {
                    Ok(value) => value,
                    Err(error) => {
                        return ToolExecutionResult::error(json!({
                            "tool": "http",
                            "method": method.as_str(),
                            "url": request_url.as_str(),
                            "final_url": endpoint.as_str(),
                            "reason_code": "http_redirect_invalid_location",
                            "redirect_count": redirect_count,
                            "http_status": status.as_u16(),
                            "error": format!("redirect Location header is not valid UTF-8: {error}"),
                        }))
                    }
                };

                let next_url = match endpoint.join(location) {
                    Ok(next_url) => next_url,
                    Err(error) => {
                        return ToolExecutionResult::error(json!({
                            "tool": "http",
                            "method": method.as_str(),
                            "url": request_url.as_str(),
                            "final_url": endpoint.as_str(),
                            "reason_code": "http_redirect_invalid_location",
                            "redirect_count": redirect_count,
                            "http_status": status.as_u16(),
                            "error": format!(
                                "redirect location '{}' could not be resolved against '{}': {error}",
                                location,
                                endpoint
                            ),
                        }))
                    }
                };

                if let Err(violation) = ssrf_guard.validate_url(&next_url).await {
                    return self.ssrf_violation_result(
                        &method,
                        &request_url,
                        next_url.as_str(),
                        violation,
                    );
                }

                endpoint = next_url;
                redirect_count = redirect_count.saturating_add(1);
                continue;
            }

            let response_headers = response.headers().clone();
            let mut response_bytes = Vec::new();
            let mut observed_bytes = 0usize;
            let mut response_stream = response;
            loop {
                let chunk = match response_stream.chunk().await {
                    Ok(chunk) => chunk,
                    Err(error) => {
                        return ToolExecutionResult::error(json!({
                            "tool": "http",
                            "method": method.as_str(),
                            "url": request_url.as_str(),
                            "final_url": endpoint.as_str(),
                            "reason_code": "http_response_read_error",
                            "retryable": true,
                            "error": error.to_string(),
                            "duration_ms": started_at.elapsed().as_millis(),
                        }));
                    }
                };
                let Some(chunk) = chunk else {
                    break;
                };
                observed_bytes = observed_bytes.saturating_add(chunk.len());
                if observed_bytes > effective_response_limit {
                    return ToolExecutionResult::error(json!({
                        "tool": "http",
                        "method": method.as_str(),
                        "url": request_url.as_str(),
                        "final_url": endpoint.as_str(),
                        "policy_rule": "http_max_response_bytes",
                        "reason_code": "http_response_too_large",
                        "response_bytes": observed_bytes,
                        "max_response_bytes": effective_response_limit,
                        "error": format!(
                            "response exceeded max_response_bytes cap of {} bytes",
                            effective_response_limit
                        ),
                    }));
                }
                response_bytes.extend_from_slice(&chunk);
            }

            let response_text = redact_secrets(&String::from_utf8_lossy(&response_bytes));
            let response_content_type = response_headers
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .map(ToString::to_string);
            let response_json = serde_json::from_slice::<Value>(&response_bytes).ok();
            let duration_ms = started_at.elapsed().as_millis() as u64;

            let mut payload = serde_json::Map::new();
            payload.insert("tool".to_string(), json!("http"));
            payload.insert("method".to_string(), json!(method.as_str()));
            payload.insert("url".to_string(), json!(request_url.as_str()));
            payload.insert("final_url".to_string(), json!(endpoint.as_str()));
            payload.insert("http_status".to_string(), json!(status.as_u16()));
            payload.insert("success".to_string(), json!(status.is_success()));
            payload.insert("redirect_count".to_string(), json!(redirect_count));
            payload.insert("duration_ms".to_string(), json!(duration_ms));
            payload.insert("timeout_ms".to_string(), json!(effective_timeout_ms));
            payload.insert(
                "max_response_bytes".to_string(),
                json!(effective_response_limit),
            );
            payload.insert("response_bytes".to_string(), json!(response_bytes.len()));
            payload.insert("request_header_names".to_string(), json!(header_names));
            payload.insert(
                "has_json_payload".to_string(),
                json!(json_payload.is_some()),
            );
            payload.insert("response_text".to_string(), json!(response_text));
            payload.insert(
                "ssrf_allow_http".to_string(),
                json!(self.policy.http_allow_http),
            );
            payload.insert(
                "ssrf_allow_private_network".to_string(),
                json!(self.policy.http_allow_private_network),
            );
            payload.insert(
                "max_redirects".to_string(),
                json!(self.policy.http_max_redirects),
            );
            if let Some(content_type) = response_content_type {
                payload.insert("content_type".to_string(), json!(content_type));
            } else {
                payload.insert("content_type".to_string(), Value::Null);
            }
            if let Some(response_json) = response_json {
                payload.insert("response_json".to_string(), response_json);
            }

            if status.is_success() {
                return ToolExecutionResult::ok(Value::Object(payload));
            }

            let (reason_code, retryable) = classify_http_status(status);
            payload.insert("reason_code".to_string(), json!(reason_code));
            payload.insert("retryable".to_string(), json!(retryable));
            payload.insert(
                "error".to_string(),
                json!(format!("request returned HTTP status {}", status.as_u16())),
            );
            return ToolExecutionResult::error(Value::Object(payload));
        }
    }
}

#[cfg(test)]
mod tests;
