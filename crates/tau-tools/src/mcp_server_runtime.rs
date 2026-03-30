//! Hosts Tau tools behind an MCP server transport with policy-aware dispatch.
//!
//! This runtime binds built-in tool contracts to MCP methods, preserves stable
//! error codes for protocol callers, and applies `ToolPolicy` enforcement
//! before command/file/network actions execute.

use std::{
    collections::{BTreeMap, BTreeSet},
    future::Future,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
};

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use tau_agent_core::{AgentTool, ToolExecutionResult};
use tau_cli::Cli;

use crate::tool_policy_config::build_tool_policy;
use crate::tools::{
    BashTool, EditTool, HttpTool, JobsCancelTool, JobsCreateTool, JobsListTool, JobsStatusTool,
    MemoryReadTool, MemorySearchTool, MemoryTreeTool, MemoryWriteTool, ReadTool, ToolPolicy,
    WriteTool,
};

const MCP_JSONRPC_VERSION: &str = "2.0";
const MCP_PROTOCOL_VERSION: &str = "2024-11-05";
const MCP_EXTERNAL_SERVER_SCHEMA_VERSION: u32 = 1;
const MCP_ERROR_PARSE: i64 = -32700;
const MCP_ERROR_INVALID_REQUEST: i64 = -32600;
const MCP_ERROR_METHOD_NOT_FOUND: i64 = -32601;
const MCP_ERROR_INVALID_PARAMS: i64 = -32602;
const MCP_TOOL_PREFIX_EXTERNAL: &str = "external.";
const MCP_TOOL_READ: &str = "tau.read";
const MCP_TOOL_WRITE: &str = "tau.write";
const MCP_TOOL_EDIT: &str = "tau.edit";
const MCP_TOOL_MEMORY_WRITE: &str = "tau.memory_write";
const MCP_TOOL_MEMORY_READ: &str = "tau.memory_read";
const MCP_TOOL_MEMORY_SEARCH: &str = "tau.memory_search";
const MCP_TOOL_MEMORY_TREE: &str = "tau.memory_tree";
const MCP_TOOL_JOBS_CREATE: &str = "tau.jobs_create";
const MCP_TOOL_JOBS_LIST: &str = "tau.jobs_list";
const MCP_TOOL_JOBS_STATUS: &str = "tau.jobs_status";
const MCP_TOOL_JOBS_CANCEL: &str = "tau.jobs_cancel";
const MCP_TOOL_HTTP: &str = "tau.http";
const MCP_TOOL_BASH: &str = "tau.bash";
// Session tools
const MCP_TOOL_SESSION_LIST: &str = "tau.session_list";
const MCP_TOOL_SESSION_RESUME: &str = "tau.session_resume";
const MCP_TOOL_SESSION_SEARCH: &str = "tau.session_search";
const MCP_TOOL_SESSION_STATS: &str = "tau.session_stats";
const MCP_TOOL_SESSION_EXPORT: &str = "tau.session_export";
// Orchestration tools
const MCP_TOOL_AGENT_SPAWN: &str = "tau.agent_spawn";
const MCP_TOOL_AGENT_STATUS: &str = "tau.agent_status";
const MCP_TOOL_AGENT_CANCEL: &str = "tau.agent_cancel";
// Learning tools
const MCP_TOOL_LEARN_STATUS: &str = "tau.learn_status";
const MCP_TOOL_LEARN_FAILURE_PATTERNS: &str = "tau.learn_failure_patterns";
const MCP_TOOL_LEARN_TOOL_RATES: &str = "tau.learn_tool_rates";
// Training tools
const MCP_TOOL_TRAINING_STATUS: &str = "tau.training_status";
const MCP_TOOL_TRAINING_TRIGGER: &str = "tau.training_trigger";
// Skills tools
const MCP_TOOL_SKILLS_LIST: &str = "tau.skills_list";
const MCP_TOOL_SKILLS_SEARCH: &str = "tau.skills_search";
const MCP_TOOL_SKILLS_INSTALL: &str = "tau.skills_install";
const MCP_TOOL_SKILLS_INFO: &str = "tau.skills_info";
const MCP_TOOL_CONTEXT_SESSION: &str = "tau.context.session";
const MCP_TOOL_CONTEXT_SKILLS: &str = "tau.context.skills";
const MCP_TOOL_CONTEXT_CHANNEL_STORE: &str = "tau.context.channel-store";
// Additional context providers
const MCP_TOOL_CONTEXT_LEARNING: &str = "tau.context.learning";
const MCP_TOOL_CONTEXT_TRAINING: &str = "tau.context.training";
const MCP_TOOL_CONTEXT_CONFIG: &str = "tau.context.config";
const MCP_CONTEXT_PROVIDER_SESSION: &str = "session";
const MCP_CONTEXT_PROVIDER_SKILLS: &str = "skills";
const MCP_CONTEXT_PROVIDER_CHANNEL_STORE: &str = "channel-store";
const MCP_EXTERNAL_INIT_REQUEST_ID: &str = "tau-ext-init";
const MCP_EXTERNAL_TOOLS_LIST_REQUEST_ID: &str = "tau-ext-tools-list";
const MCP_EXTERNAL_TOOLS_CALL_REQUEST_ID: &str = "tau-ext-tools-call";
const MCP_EXTERNAL_RESULT_TOOLS_FIELD: &str = "tools";
const MCP_CONTENT_TYPE_TEXT: &str = "text";
const RESERVED_MCP_TOOL_NAMES: &[&str] = &[
    MCP_TOOL_READ,
    MCP_TOOL_WRITE,
    MCP_TOOL_EDIT,
    MCP_TOOL_MEMORY_WRITE,
    MCP_TOOL_MEMORY_READ,
    MCP_TOOL_MEMORY_SEARCH,
    MCP_TOOL_MEMORY_TREE,
    MCP_TOOL_JOBS_CREATE,
    MCP_TOOL_JOBS_LIST,
    MCP_TOOL_JOBS_STATUS,
    MCP_TOOL_JOBS_CANCEL,
    MCP_TOOL_HTTP,
    MCP_TOOL_BASH,
    MCP_TOOL_SESSION_LIST,
    MCP_TOOL_SESSION_RESUME,
    MCP_TOOL_SESSION_SEARCH,
    MCP_TOOL_SESSION_STATS,
    MCP_TOOL_SESSION_EXPORT,
    MCP_TOOL_AGENT_SPAWN,
    MCP_TOOL_AGENT_STATUS,
    MCP_TOOL_AGENT_CANCEL,
    MCP_TOOL_LEARN_STATUS,
    MCP_TOOL_LEARN_FAILURE_PATTERNS,
    MCP_TOOL_LEARN_TOOL_RATES,
    MCP_TOOL_TRAINING_STATUS,
    MCP_TOOL_TRAINING_TRIGGER,
    MCP_TOOL_SKILLS_LIST,
    MCP_TOOL_SKILLS_SEARCH,
    MCP_TOOL_SKILLS_INSTALL,
    MCP_TOOL_SKILLS_INFO,
    MCP_TOOL_CONTEXT_SESSION,
    MCP_TOOL_CONTEXT_SKILLS,
    MCP_TOOL_CONTEXT_CHANNEL_STORE,
    MCP_TOOL_CONTEXT_LEARNING,
    MCP_TOOL_CONTEXT_TRAINING,
    MCP_TOOL_CONTEXT_CONFIG,
];

fn default_mcp_context_providers() -> Vec<String> {
    vec![
        MCP_CONTEXT_PROVIDER_SESSION.to_string(),
        MCP_CONTEXT_PROVIDER_SKILLS.to_string(),
        MCP_CONTEXT_PROVIDER_CHANNEL_STORE.to_string(),
    ]
}

fn default_external_server_enabled() -> bool {
    true
}

#[derive(Debug, Clone)]
/// Public struct `McpServeReport` used across Tau components.
pub struct McpServeReport {
    pub processed_frames: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct McpExternalServerConfigFile {
    schema_version: u32,
    #[serde(default)]
    servers: Vec<McpExternalServerConfig>,
}

#[derive(Debug, Clone, Deserialize)]
struct McpExternalServerConfig {
    name: String,
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: BTreeMap<String, String>,
    #[serde(default)]
    cwd: Option<PathBuf>,
    #[serde(default = "default_external_server_enabled")]
    enabled: bool,
}

#[derive(Debug, Clone)]
struct McpExternalDiscoveredTool {
    server_name: String,
    tool_name: String,
    description: String,
    input_schema: Value,
}

#[derive(Debug, Clone)]
struct McpServerState {
    tool_policy: ToolPolicy,
    session_path: PathBuf,
    skills_dir: PathBuf,
    channel_store_root: PathBuf,
    context_providers: BTreeSet<String>,
    external_servers: Vec<McpExternalServerConfig>,
    external_tools: Vec<McpExternalDiscoveredTool>,
}

#[derive(Debug, Clone)]
struct McpJsonRpcRequest {
    id: Value,
    method: String,
    params: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone)]
struct McpToolDescriptor {
    name: String,
    description: String,
    input_schema: Value,
}

#[derive(Debug, Clone)]
struct McpDispatchError {
    id: Value,
    code: i64,
    message: String,
}

impl McpDispatchError {
    fn new(id: Value, code: i64, message: impl Into<String>) -> Self {
        Self {
            id,
            code,
            message: message.into(),
        }
    }
}

pub fn resolve_mcp_context_providers(raw: &[String]) -> Result<Vec<String>> {
    if raw.is_empty() {
        return Ok(default_mcp_context_providers());
    }

    let mut resolved = Vec::new();
    for entry in raw {
        let normalized = entry.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            continue;
        }
        if !matches!(
            normalized.as_str(),
            MCP_CONTEXT_PROVIDER_SESSION
                | MCP_CONTEXT_PROVIDER_SKILLS
                | MCP_CONTEXT_PROVIDER_CHANNEL_STORE
        ) {
            bail!(
                "unsupported mcp context provider '{}'; supported values are session, skills, channel-store",
                entry
            );
        }
        if !resolved.iter().any(|existing| existing == &normalized) {
            resolved.push(normalized);
        }
    }

    if resolved.is_empty() {
        bail!("at least one valid --mcp-context-provider value is required");
    }
    Ok(resolved)
}

pub fn execute_mcp_server_command(cli: &Cli) -> Result<()> {
    if !cli.mcp_server {
        return Ok(());
    }

    let context_providers = resolve_mcp_context_providers(&cli.mcp_context_provider)?;
    let reserved_mcp_tool_names = reserved_builtin_mcp_tool_names();
    let external_servers = load_external_mcp_servers(cli.mcp_external_server_config.as_deref())?;
    let external_tools = discover_external_mcp_tools(&external_servers, &reserved_mcp_tool_names)?;
    let state = McpServerState {
        tool_policy: build_tool_policy(cli)?,
        session_path: cli.session.clone(),
        skills_dir: cli.skills_dir.clone(),
        channel_store_root: cli.channel_store_root.clone(),
        context_providers: context_providers.into_iter().collect(),
        external_servers,
        external_tools,
    };

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();
    let report = serve_mcp_jsonrpc_reader(&mut reader, &mut writer, &state)?;
    if report.error_count > 0 {
        bail!(
            "mcp server completed with {} error frame(s) after {} request(s)",
            report.error_count,
            report.processed_frames
        );
    }
    Ok(())
}

fn load_external_mcp_servers(path: Option<&Path>) -> Result<Vec<McpExternalServerConfig>> {
    let Some(path) = path else {
        return Ok(Vec::new());
    };

    let raw = std::fs::read_to_string(path).with_context(|| {
        format!(
            "failed to read mcp external server config {}",
            path.display()
        )
    })?;
    let parsed = serde_json::from_str::<McpExternalServerConfigFile>(&raw).with_context(|| {
        format!(
            "failed to parse mcp external server config {}",
            path.display()
        )
    })?;
    if parsed.schema_version != MCP_EXTERNAL_SERVER_SCHEMA_VERSION {
        bail!(
            "unsupported mcp external server config schema_version {} in {} (expected {})",
            parsed.schema_version,
            path.display(),
            MCP_EXTERNAL_SERVER_SCHEMA_VERSION
        );
    }

    let mut servers = Vec::new();
    let mut seen_names = BTreeSet::new();
    for server in parsed.servers {
        if !server.enabled {
            continue;
        }
        let name = normalize_external_server_name(&server.name)?;
        if !seen_names.insert(name.clone()) {
            bail!(
                "duplicate external mcp server name '{}' in {}",
                name,
                path.display()
            );
        }
        let command = server.command.trim();
        if command.is_empty() {
            bail!(
                "external mcp server '{}' in {} is missing a command",
                name,
                path.display()
            );
        }
        servers.push(McpExternalServerConfig {
            name,
            command: command.to_string(),
            args: server.args,
            env: server.env,
            cwd: server.cwd,
            enabled: true,
        });
    }
    Ok(servers)
}

fn normalize_external_server_name(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("external mcp server name must be non-empty");
    }
    if !trimmed
        .chars()
        .all(|value| value.is_ascii_alphanumeric() || matches!(value, '-' | '_'))
    {
        bail!(
            "external mcp server name '{}' must contain only ASCII letters, digits, '-' or '_'",
            raw
        );
    }
    Ok(trimmed.to_ascii_lowercase())
}

fn discover_external_mcp_tools(
    servers: &[McpExternalServerConfig],
    reserved_tool_names: &BTreeSet<String>,
) -> Result<Vec<McpExternalDiscoveredTool>> {
    let mut tools = Vec::new();
    let mut seen_qualified_names = BTreeSet::new();
    for server in servers {
        let init = jsonrpc_request_frame(
            Value::String(MCP_EXTERNAL_INIT_REQUEST_ID.to_string()),
            "initialize",
            json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {
                    "name": "tau-coding-agent",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        );
        let list = jsonrpc_request_frame(
            Value::String(MCP_EXTERNAL_TOOLS_LIST_REQUEST_ID.to_string()),
            "tools/list",
            json!({}),
        );
        let responses = call_external_mcp_server(server, &[init, list])?;
        let list_payload =
            external_response_result(&responses, MCP_EXTERNAL_TOOLS_LIST_REQUEST_ID, server)?;
        let list_tools = list_payload
            .get(MCP_EXTERNAL_RESULT_TOOLS_FIELD)
            .and_then(Value::as_array)
            .ok_or_else(|| {
                anyhow!(
                    "external mcp server '{}' returned invalid tools/list payload",
                    server.name
                )
            })?;
        for entry in list_tools {
            let object = entry.as_object().ok_or_else(|| {
                anyhow!(
                    "external mcp server '{}' returned non-object tool descriptor",
                    server.name
                )
            })?;
            let tool_name = object
                .get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    anyhow!(
                        "external mcp server '{}' returned tool with invalid name",
                        server.name
                    )
                })?
                .to_string();
            if reserved_tool_names.contains(tool_name.as_str()) {
                bail!(
                    "external mcp server '{}' returned reserved built-in tool name '{}'",
                    server.name,
                    tool_name
                );
            }
            let qualified_name = format!("{MCP_TOOL_PREFIX_EXTERNAL}{}.{}", server.name, tool_name);
            if !seen_qualified_names.insert(qualified_name.clone()) {
                bail!(
                    "external mcp server '{}' returned duplicate tool registration '{}'",
                    server.name,
                    qualified_name
                );
            }
            let description = object
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("external mcp tool")
                .trim()
                .to_string();
            let input_schema = object.get("inputSchema").cloned().unwrap_or_else(
                || json!({"type":"object","properties":{},"additionalProperties":true}),
            );
            tools.push(McpExternalDiscoveredTool {
                server_name: server.name.clone(),
                tool_name,
                description,
                input_schema,
            });
        }
    }
    Ok(tools)
}

fn reserved_builtin_mcp_tool_names() -> BTreeSet<String> {
    RESERVED_MCP_TOOL_NAMES
        .iter()
        .map(|name| (*name).to_string())
        .collect()
}

fn call_external_mcp_server(
    server: &McpExternalServerConfig,
    requests: &[Value],
) -> Result<Vec<Value>> {
    let mut command = Command::new(&server.command);
    command.args(&server.args);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    if let Some(cwd) = &server.cwd {
        command.current_dir(cwd);
    }
    for (key, value) in &server.env {
        command.env(key, value);
    }

    let mut child = command.spawn().with_context(|| {
        format!(
            "failed to spawn external mcp server '{}' command '{}'",
            server.name, server.command
        )
    })?;
    let mut child_stdin = child.stdin.take().ok_or_else(|| {
        anyhow!(
            "failed to open stdin for external mcp server '{}'",
            server.name
        )
    })?;
    for request in requests {
        let line = serde_json::to_string(request).context("failed to encode external request")?;
        writeln!(child_stdin, "{line}").with_context(|| {
            format!(
                "failed to write request to external mcp server '{}'",
                server.name
            )
        })?;
    }
    drop(child_stdin);

    let mut responses = Vec::new();
    {
        let child_stdout = child.stdout.take().ok_or_else(|| {
            anyhow!(
                "failed to open stdout for external mcp server '{}'",
                server.name
            )
        })?;
        let mut reader = BufReader::new(child_stdout);
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = reader.read_line(&mut line).with_context(|| {
                format!(
                    "failed to read response from external mcp server '{}'",
                    server.name
                )
            })?;
            if bytes == 0 {
                break;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let value = serde_json::from_str::<Value>(trimmed).with_context(|| {
                format!(
                    "external mcp server '{}' returned invalid JSON line '{}'",
                    server.name, trimmed
                )
            })?;
            responses.push(value);
        }
    }

    let status = child
        .wait()
        .with_context(|| format!("failed to wait for external mcp server '{}'", server.name))?;
    if !status.success() {
        let mut stderr = String::new();
        if let Some(mut handle) = child.stderr.take() {
            let _ = handle.read_to_string(&mut stderr);
        }
        bail!(
            "external mcp server '{}' exited with status {}{}",
            server.name,
            status,
            if stderr.trim().is_empty() {
                String::new()
            } else {
                format!(" stderr={}", stderr.trim())
            }
        );
    }

    Ok(responses)
}

fn external_response_result(
    responses: &[Value],
    request_id: &str,
    server: &McpExternalServerConfig,
) -> Result<Value> {
    for response in responses {
        let object = match response.as_object() {
            Some(object) => object,
            None => continue,
        };
        let Some(id) = object.get("id") else {
            continue;
        };
        let matches = id
            .as_str()
            .map(|value| value == request_id)
            .unwrap_or(false);
        if !matches {
            continue;
        }
        if let Some(error) = object.get("error") {
            bail!(
                "external mcp server '{}' returned error for request '{}': {}",
                server.name,
                request_id,
                error
            );
        }
        if let Some(result) = object.get("result") {
            return Ok(result.clone());
        }
        bail!(
            "external mcp server '{}' returned response without result for request '{}'",
            server.name,
            request_id
        );
    }

    bail!(
        "external mcp server '{}' returned no response for request '{}'",
        server.name,
        request_id
    )
}

fn serve_mcp_jsonrpc_reader<R, W>(
    reader: &mut R,
    writer: &mut W,
    state: &McpServerState,
) -> Result<McpServeReport>
where
    R: BufRead,
    W: Write,
{
    let mut processed_frames = 0usize;
    let mut error_count = 0usize;

    loop {
        let frame = match read_jsonrpc_content_length_frame(reader) {
            Ok(Some(value)) => value,
            Ok(None) => break,
            Err(error) => {
                let response = jsonrpc_error_frame(
                    Value::Null,
                    MCP_ERROR_PARSE,
                    format!("failed to read mcp frame: {error}"),
                );
                write_jsonrpc_content_length_frame(writer, &response)?;
                error_count = error_count.saturating_add(1);
                break;
            }
        };
        processed_frames = processed_frames.saturating_add(1);

        let response = match parse_jsonrpc_request(&frame) {
            Ok(request) => match dispatch_jsonrpc_request(&request, state) {
                Ok(result) => jsonrpc_result_frame(request.id, result),
                Err(error) => {
                    error_count = error_count.saturating_add(1);
                    jsonrpc_error_frame(error.id, error.code, error.message)
                }
            },
            Err(error) => {
                error_count = error_count.saturating_add(1);
                jsonrpc_error_frame(error.id, error.code, error.message)
            }
        };
        write_jsonrpc_content_length_frame(writer, &response)?;
    }

    Ok(McpServeReport {
        processed_frames,
        error_count,
    })
}

fn parse_jsonrpc_request(value: &Value) -> Result<McpJsonRpcRequest, McpDispatchError> {
    let Some(object) = value.as_object() else {
        return Err(McpDispatchError::new(
            Value::Null,
            MCP_ERROR_INVALID_REQUEST,
            "jsonrpc request must be an object",
        ));
    };
    let id = object.get("id").cloned().ok_or_else(|| {
        McpDispatchError::new(
            Value::Null,
            MCP_ERROR_INVALID_REQUEST,
            "jsonrpc request must include id",
        )
    })?;
    let jsonrpc = object
        .get("jsonrpc")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if jsonrpc != MCP_JSONRPC_VERSION {
        return Err(McpDispatchError::new(
            id,
            MCP_ERROR_INVALID_REQUEST,
            format!("jsonrpc must be '{}'", MCP_JSONRPC_VERSION),
        ));
    }
    let method = object
        .get("method")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            McpDispatchError::new(
                id.clone(),
                MCP_ERROR_INVALID_REQUEST,
                "jsonrpc request must include non-empty method",
            )
        })?;
    let params = match object.get("params") {
        Some(Value::Object(params)) => params.clone(),
        Some(_) => {
            return Err(McpDispatchError::new(
                id,
                MCP_ERROR_INVALID_PARAMS,
                "jsonrpc request params must be an object",
            ))
        }
        None => serde_json::Map::new(),
    };
    Ok(McpJsonRpcRequest {
        id,
        method: method.to_string(),
        params,
    })
}

fn dispatch_jsonrpc_request(
    request: &McpJsonRpcRequest,
    state: &McpServerState,
) -> Result<Value, McpDispatchError> {
    match request.method.as_str() {
        "initialize" => Ok(handle_initialize(state)),
        "tools/list" => Ok(handle_tools_list(state)),
        "tools/call" => handle_tools_call(state, &request.params).map_err(|error| {
            McpDispatchError::new(
                request.id.clone(),
                MCP_ERROR_INVALID_PARAMS,
                error.to_string(),
            )
        }),
        other => Err(McpDispatchError::new(
            request.id.clone(),
            MCP_ERROR_METHOD_NOT_FOUND,
            format!("unsupported method '{}'", other),
        )),
    }
}

fn handle_initialize(state: &McpServerState) -> Value {
    let context_providers = state
        .context_providers
        .iter()
        .map(|value| Value::String(value.clone()))
        .collect::<Vec<_>>();
    json!({
        "protocolVersion": MCP_PROTOCOL_VERSION,
        "serverInfo": {
            "name": "tau-coding-agent",
            "version": env!("CARGO_PKG_VERSION")
        },
        "capabilities": {
            "tools": {
                "listChanged": false
            },
            "experimental": {
                "contextProviders": context_providers
            }
        }
    })
}

fn handle_tools_list(state: &McpServerState) -> Value {
    let mut tools = builtin_mcp_tools(state);
    tools.extend(
        state
            .external_tools
            .iter()
            .map(|tool| McpToolDescriptor {
                name: format!(
                    "{MCP_TOOL_PREFIX_EXTERNAL}{}.{}",
                    tool.server_name, tool.tool_name
                ),
                description: format!(
                    "{} (external server {})",
                    tool.description, tool.server_name
                ),
                input_schema: tool.input_schema.clone(),
            })
            .collect::<Vec<_>>(),
    );
    tools.sort_by(|left, right| left.name.cmp(&right.name));
    json!({
        "tools": tools
            .into_iter()
            .map(|tool| {
                json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": tool.input_schema
                })
            })
            .collect::<Vec<_>>()
    })
}

fn handle_tools_call(
    state: &McpServerState,
    params: &serde_json::Map<String, Value>,
) -> Result<Value> {
    let tool_name = params
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("tools/call requires non-empty field 'name'"))?;
    let arguments = match params.get("arguments") {
        Some(Value::Object(arguments)) => Value::Object(arguments.clone()),
        Some(_) => bail!("tools/call field 'arguments' must be an object when provided"),
        None => Value::Object(serde_json::Map::new()),
    };

    // Handlers for tools that use McpServerState but not the full agent runtime
    if let Some(result) = handle_stateful_tool(tool_name, &arguments, state)? {
        return Ok(result);
    }

    // Check skill-registered tools before falling through to external tools.
    if let Some(skill_result) = try_dispatch_skill_tool(tool_name, &arguments, &state.skills_dir) {
        let is_error = skill_result
            .get("is_error")
            .and_then(Value::as_bool)
            .or_else(|| skill_result.get("error").map(|_| true))
            .unwrap_or(false);
        return Ok(mcp_tool_call_result(skill_result, is_error));
    }

    if let Some(qualified) = tool_name.strip_prefix(MCP_TOOL_PREFIX_EXTERNAL) {
        let mut parts = qualified.splitn(2, '.');
        let server_name = parts
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("external tool name must include server name"))?;
        let external_tool_name = parts
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("external tool name must include tool identifier"))?;
        let server = state
            .external_servers
            .iter()
            .find(|candidate| candidate.name == server_name)
            .ok_or_else(|| anyhow!("unknown external mcp server '{}'", server_name))?;
        let init = jsonrpc_request_frame(
            Value::String(MCP_EXTERNAL_INIT_REQUEST_ID.to_string()),
            "initialize",
            json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {
                    "name": "tau-coding-agent",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        );
        let call = jsonrpc_request_frame(
            Value::String(MCP_EXTERNAL_TOOLS_CALL_REQUEST_ID.to_string()),
            "tools/call",
            json!({
                "name": external_tool_name,
                "arguments": arguments
            }),
        );
        let responses = call_external_mcp_server(server, &[init, call])?;
        let result =
            external_response_result(&responses, MCP_EXTERNAL_TOOLS_CALL_REQUEST_ID, server)?;
        let is_error = result
            .get("isError")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        return Ok(mcp_tool_call_result(result, is_error));
    }

    if matches!(
        tool_name,
        MCP_TOOL_CONTEXT_SESSION | MCP_TOOL_CONTEXT_SKILLS | MCP_TOOL_CONTEXT_CHANNEL_STORE
    ) {
        let context_payload = execute_context_provider_tool(state, tool_name)?;
        return Ok(mcp_tool_call_result(context_payload, false));
    }

    let execution = execute_builtin_tool_call(tool_name, arguments, &state.tool_policy)?;
    Ok(mcp_tool_call_result(execution.content, execution.is_error))
}

fn execute_context_provider_tool(state: &McpServerState, tool_name: &str) -> Result<Value> {
    match tool_name {
        MCP_TOOL_CONTEXT_SESSION => {
            if !state
                .context_providers
                .contains(MCP_CONTEXT_PROVIDER_SESSION)
            {
                bail!(
                    "context provider '{}' is disabled",
                    MCP_CONTEXT_PROVIDER_SESSION
                );
            }
            let exists = state.session_path.exists();
            let (entries, storage_backend, backend_reason_code) = if exists {
                match tau_session::SessionStore::load(&state.session_path) {
                    Ok(store) => (
                        store.entries().len(),
                        store.storage_backend().label().to_string(),
                        store.storage_backend_reason_code().to_string(),
                    ),
                    Err(_) => (
                        0,
                        "unknown".to_string(),
                        "session_context_provider_load_failed".to_string(),
                    ),
                }
            } else {
                (
                    0,
                    "unknown".to_string(),
                    "session_context_provider_missing".to_string(),
                )
            };
            Ok(json!({
                "provider": MCP_CONTEXT_PROVIDER_SESSION,
                "path": state.session_path.display().to_string(),
                "exists": exists,
                "entries": entries,
                "storage_backend": storage_backend,
                "backend_reason_code": backend_reason_code,
            }))
        }
        MCP_TOOL_CONTEXT_SKILLS => {
            if !state
                .context_providers
                .contains(MCP_CONTEXT_PROVIDER_SKILLS)
            {
                bail!(
                    "context provider '{}' is disabled",
                    MCP_CONTEXT_PROVIDER_SKILLS
                );
            }
            let skills = list_skill_files(&state.skills_dir, 128)?;
            Ok(json!({
                "provider": MCP_CONTEXT_PROVIDER_SKILLS,
                "path": state.skills_dir.display().to_string(),
                "count": skills.len(),
                "files": skills,
            }))
        }
        MCP_TOOL_CONTEXT_CHANNEL_STORE => {
            if !state
                .context_providers
                .contains(MCP_CONTEXT_PROVIDER_CHANNEL_STORE)
            {
                bail!(
                    "context provider '{}' is disabled",
                    MCP_CONTEXT_PROVIDER_CHANNEL_STORE
                );
            }
            let channels_root = state.channel_store_root.join("channels");
            let channel_count = if channels_root.is_dir() {
                std::fs::read_dir(&channels_root)
                    .ok()
                    .into_iter()
                    .flatten()
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.path().is_dir())
                    .count()
            } else {
                0
            };
            Ok(json!({
                "provider": MCP_CONTEXT_PROVIDER_CHANNEL_STORE,
                "path": channels_root.display().to_string(),
                "channel_count": channel_count,
            }))
        }
        other => bail!("unknown context provider tool '{}'", other),
    }
}

fn list_skill_files(root: &Path, limit: usize) -> Result<Vec<String>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let entries = std::fs::read_dir(&path)
            .with_context(|| format!("failed to read skills directory {}", path.display()))?;
        for entry in entries {
            let entry = entry.with_context(|| format!("failed to read {}", path.display()))?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }
            if entry_path
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.eq_ignore_ascii_case("md"))
                .unwrap_or(false)
            {
                files.push(entry_path.display().to_string());
                if files.len() >= limit {
                    files.sort();
                    return Ok(files);
                }
            }
        }
    }

    files.sort();
    Ok(files)
}

/// Dispatch a tool call that requires `McpServerState` but not the full agent
/// runtime.  Returns `Ok(Some(result))` when the tool is handled, `Ok(None)`
/// when the tool is not in this dispatcher (fall through to built-in tools),
/// and `Err` on handler errors.
fn handle_stateful_tool(
    tool_name: &str,
    arguments: &Value,
    state: &McpServerState,
) -> Result<Option<Value>> {
    let result = match tool_name {
        MCP_TOOL_SESSION_LIST => Some(handle_session_list(arguments, state)?),
        MCP_TOOL_SESSION_RESUME => Some(handle_session_resume(arguments, state)?),
        MCP_TOOL_SESSION_SEARCH => Some(handle_session_search(arguments, state)?),
        MCP_TOOL_SESSION_STATS => Some(handle_session_stats(arguments, state)?),
        MCP_TOOL_SESSION_EXPORT => Some(handle_session_export(arguments, state)?),
        MCP_TOOL_LEARN_STATUS => Some(handle_learn_status(state)?),
        MCP_TOOL_LEARN_FAILURE_PATTERNS => Some(handle_learn_failure_patterns(arguments, state)?),
        MCP_TOOL_LEARN_TOOL_RATES => Some(handle_learn_tool_rates(arguments, state)?),
        MCP_TOOL_TRAINING_STATUS => Some(handle_training_status(state)?),
        MCP_TOOL_TRAINING_TRIGGER => Some(handle_training_trigger(arguments)?),
        MCP_TOOL_SKILLS_LIST => Some(handle_skills_list(arguments, state)?),
        MCP_TOOL_SKILLS_SEARCH => Some(handle_skills_search(arguments, state)?),
        MCP_TOOL_SKILLS_INFO => Some(handle_skills_info(arguments, state)?),
        MCP_TOOL_SKILLS_INSTALL => Some(handle_skills_install(arguments, state)?),
        MCP_TOOL_AGENT_SPAWN => Some(handle_agent_spawn(arguments)?),
        MCP_TOOL_AGENT_STATUS => Some(handle_agent_status(arguments)?),
        MCP_TOOL_AGENT_CANCEL => Some(handle_agent_cancel(arguments)?),
        MCP_TOOL_CONTEXT_LEARNING => Some(handle_context_learning(state)?),
        MCP_TOOL_CONTEXT_TRAINING => Some(handle_context_training(state)?),
        MCP_TOOL_CONTEXT_CONFIG => Some(handle_context_config(state)?),
        _ => None,
    };
    Ok(result)
}

// ---------------------------------------------------------------------------
// Session tool handlers
// ---------------------------------------------------------------------------

fn handle_session_list(arguments: &Value, state: &McpServerState) -> Result<Value> {
    let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(20) as usize;
    let offset = arguments.get("offset").and_then(Value::as_u64).unwrap_or(0) as usize;

    let session_dir = state
        .session_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let mut sessions: Vec<Value> = Vec::new();
    if session_dir.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(session_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "sqlite" || ext == "jsonl")
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by(|a, b| {
            let ma = a.metadata().and_then(|m| m.modified()).ok();
            let mb = b.metadata().and_then(|m| m.modified()).ok();
            mb.cmp(&ma)
        });
        for entry in entries.into_iter().skip(offset).take(limit) {
            let path = entry.path();
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            let modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());
            let entry_count = tau_session::SessionStore::load(&path)
                .map(|store| store.entries().len())
                .unwrap_or(0);
            sessions.push(json!({
                "name": name,
                "path": path.display().to_string(),
                "entries": entry_count,
                "modified_unix": modified,
            }));
        }
    }

    let total = sessions.len();
    Ok(mcp_tool_call_result(
        json!({
            "tool": MCP_TOOL_SESSION_LIST,
            "sessions": sessions,
            "total_discovered": total,
            "offset": offset,
            "limit": limit,
        }),
        false,
    ))
}

fn handle_session_resume(arguments: &Value, state: &McpServerState) -> Result<Value> {
    let session_id = arguments
        .get("session_id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("session_id is required"))?;

    let session_dir = state
        .session_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let candidate = session_dir.join(format!("{session_id}.sqlite"));
    let candidate_jsonl = session_dir.join(format!("{session_id}.jsonl"));
    let resolved = if candidate.exists() {
        Some(candidate)
    } else if candidate_jsonl.exists() {
        Some(candidate_jsonl)
    } else {
        None
    };

    match resolved {
        Some(path) => {
            let entry_count = tau_session::SessionStore::load(&path)
                .map(|store| store.entries().len())
                .unwrap_or(0);
            Ok(mcp_tool_call_result(
                json!({
                    "tool": MCP_TOOL_SESSION_RESUME,
                    "session_id": session_id,
                    "path": path.display().to_string(),
                    "entries": entry_count,
                    "status": "resolved",
                }),
                false,
            ))
        }
        None => Ok(mcp_tool_call_result(
            json!({
                "tool": MCP_TOOL_SESSION_RESUME,
                "session_id": session_id,
                "status": "not_found",
                "message": format!("No session file found for id '{session_id}'"),
            }),
            true,
        )),
    }
}

fn handle_session_search(arguments: &Value, state: &McpServerState) -> Result<Value> {
    let query = arguments
        .get("query")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("query is required"))?;
    let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;

    if !state.session_path.exists() {
        return Ok(mcp_tool_call_result(
            json!({
                "tool": MCP_TOOL_SESSION_SEARCH,
                "query": query,
                "matches": [],
                "total_matches": 0,
                "status": "no_session",
            }),
            false,
        ));
    }

    let store = tau_session::SessionStore::load(&state.session_path)
        .context("failed to load session for search")?;
    let (matches, total) = tau_session::search_session_entries(store.entries(), query, None, limit);
    let match_values: Vec<Value> = matches
        .iter()
        .map(|m| {
            json!({
                "id": m.id,
                "parent_id": m.parent_id,
                "role": m.role,
                "preview": m.preview,
            })
        })
        .collect();

    let returned = match_values.len();
    Ok(mcp_tool_call_result(
        json!({
            "tool": MCP_TOOL_SESSION_SEARCH,
            "query": query,
            "matches": match_values,
            "total_matches": total,
            "returned": returned,
        }),
        false,
    ))
}

fn handle_session_stats(_arguments: &Value, state: &McpServerState) -> Result<Value> {
    if !state.session_path.exists() {
        return Ok(mcp_tool_call_result(
            json!({
                "tool": MCP_TOOL_SESSION_STATS,
                "status": "no_session",
                "path": state.session_path.display().to_string(),
            }),
            false,
        ));
    }

    let store = tau_session::SessionStore::load(&state.session_path)
        .context("failed to load session for stats")?;
    let entries = store.entries();
    let usage = store.usage_summary();
    let entry_count = entries.len();
    let roots = entries.iter().filter(|e| e.parent_id.is_none()).count();
    let max_id = entries.iter().map(|e| e.id).max();

    let mut role_counts: BTreeMap<String, usize> = BTreeMap::new();
    for entry in entries {
        let role = tau_session::session_message_role(&entry.message);
        *role_counts.entry(role).or_insert(0) += 1;
    }

    Ok(mcp_tool_call_result(
        json!({
            "tool": MCP_TOOL_SESSION_STATS,
            "path": state.session_path.display().to_string(),
            "entries": entry_count,
            "roots": roots,
            "max_id": max_id,
            "role_counts": role_counts,
            "usage": {
                "input_tokens": usage.input_tokens,
                "output_tokens": usage.output_tokens,
                "total_tokens": usage.total_tokens,
                "estimated_cost_usd": usage.estimated_cost_usd,
            },
        }),
        false,
    ))
}

fn handle_session_export(arguments: &Value, state: &McpServerState) -> Result<Value> {
    let session_id = arguments
        .get("session_id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("session_id is required"))?;
    let format_name = arguments
        .get("format")
        .and_then(Value::as_str)
        .unwrap_or("json");

    let session_dir = state
        .session_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let candidate = session_dir.join(format!("{session_id}.sqlite"));
    let candidate_jsonl = session_dir.join(format!("{session_id}.jsonl"));
    let session_path = if candidate.exists() {
        candidate
    } else if candidate_jsonl.exists() {
        candidate_jsonl
    } else {
        return Ok(mcp_tool_call_result(
            json!({
                "tool": MCP_TOOL_SESSION_EXPORT,
                "session_id": session_id,
                "status": "not_found",
            }),
            true,
        ));
    };

    let store = tau_session::SessionStore::load(&session_path)
        .context("failed to load session for export")?;
    let entries = store.entries();

    let exported = match format_name {
        "markdown" => {
            let mut lines = Vec::new();
            lines.push(format!("# Session: {session_id}"));
            lines.push(String::new());
            for entry in entries {
                let role = tau_session::session_message_role(&entry.message);
                let preview = tau_session::session_message_preview(&entry.message);
                lines.push(format!("## [{role}] (id={})", entry.id));
                lines.push(preview);
                lines.push(String::new());
            }
            lines.join("\n")
        }
        _ => serde_json::to_string_pretty(
            &entries
                .iter()
                .map(|e| {
                    json!({
                        "id": e.id,
                        "parent_id": e.parent_id,
                        "role": tau_session::session_message_role(&e.message),
                        "preview": tau_session::session_message_preview(&e.message),
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap_or_default(),
    };

    let entry_count = entries.len();
    Ok(mcp_tool_call_result(
        json!({
            "tool": MCP_TOOL_SESSION_EXPORT,
            "session_id": session_id,
            "format": format_name,
            "entries": entry_count,
            "content": exported,
        }),
        false,
    ))
}

// ---------------------------------------------------------------------------
// Learning tool handlers
// ---------------------------------------------------------------------------

/// Build a learning report by scanning the action history JSONL file that
/// lives alongside the session.  Returns an empty report when no history
/// is available.
fn load_learn_report_from_session(state: &McpServerState) -> Value {
    let history_path = state.session_path.with_extension("actions.jsonl");
    if !history_path.exists() {
        return json!({
            "total_records": 0,
            "sessions_tracked": 0,
            "top_failure_patterns": [],
            "tool_success_rates": [],
        });
    }
    let content = match std::fs::read_to_string(&history_path) {
        Ok(c) => c,
        Err(_) => {
            return json!({
                "total_records": 0,
                "sessions_tracked": 0,
                "top_failure_patterns": [],
                "tool_success_rates": [],
            });
        }
    };

    let mut total_records = 0usize;
    let mut sessions = BTreeSet::new();
    let mut tool_success: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut failure_map: BTreeMap<(String, String), usize> = BTreeMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(record) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        total_records += 1;
        if let Some(sid) = record.get("session_id").and_then(Value::as_str) {
            sessions.insert(sid.to_string());
        }
        let tool_name = record
            .get("tool_name")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let is_error = record
            .get("is_error")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let entry = tool_success.entry(tool_name.clone()).or_insert((0, 0));
        entry.1 += 1;
        if !is_error {
            entry.0 += 1;
        } else {
            let error_msg = record
                .get("error_message")
                .and_then(Value::as_str)
                .unwrap_or("unknown error")
                .to_string();
            *failure_map
                .entry((tool_name.clone(), error_msg))
                .or_insert(0) += 1;
        }
    }

    let tool_success_rates: Vec<Value> = tool_success
        .iter()
        .map(|(name, (success, total))| {
            json!({
                "tool_name": name,
                "success_rate": if *total > 0 { *success as f64 / *total as f64 } else { 0.0 },
                "total_executions": total,
            })
        })
        .collect();

    let mut failure_patterns: Vec<((String, String), usize)> = failure_map.into_iter().collect();
    failure_patterns.sort_by(|a, b| b.1.cmp(&a.1));
    let top_failure_patterns: Vec<Value> = failure_patterns
        .into_iter()
        .map(|((tool, error), count)| {
            json!({
                "tool_name": tool,
                "common_error": error,
                "occurrence_count": count,
            })
        })
        .collect();

    json!({
        "total_records": total_records,
        "sessions_tracked": sessions.len(),
        "top_failure_patterns": top_failure_patterns,
        "tool_success_rates": tool_success_rates,
    })
}

fn handle_learn_status(state: &McpServerState) -> Result<Value> {
    let mut report = load_learn_report_from_session(state);
    report
        .as_object_mut()
        .unwrap()
        .insert("tool".into(), json!(MCP_TOOL_LEARN_STATUS));
    Ok(mcp_tool_call_result(report, false))
}

fn handle_learn_failure_patterns(arguments: &Value, state: &McpServerState) -> Result<Value> {
    let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(20) as usize;
    let min_occurrences = arguments
        .get("min_occurrences")
        .and_then(Value::as_u64)
        .unwrap_or(1) as usize;

    let report = load_learn_report_from_session(state);
    let all_patterns = report
        .get("top_failure_patterns")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let filtered: Vec<Value> = all_patterns
        .into_iter()
        .filter(|p| {
            p.get("occurrence_count")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize
                >= min_occurrences
        })
        .take(limit)
        .collect();

    let count = filtered.len();
    Ok(mcp_tool_call_result(
        json!({
            "tool": MCP_TOOL_LEARN_FAILURE_PATTERNS,
            "patterns": filtered,
            "count": count,
        }),
        false,
    ))
}

fn handle_learn_tool_rates(arguments: &Value, state: &McpServerState) -> Result<Value> {
    let tool_filter = arguments.get("tool_name").and_then(Value::as_str);

    let report = load_learn_report_from_session(state);
    let all_rates = report
        .get("tool_success_rates")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let rates: Vec<Value> = all_rates
        .into_iter()
        .filter(|r| {
            tool_filter
                .map(|f| {
                    r.get("tool_name")
                        .and_then(Value::as_str)
                        .map(|n| n.eq_ignore_ascii_case(f))
                        .unwrap_or(false)
                })
                .unwrap_or(true)
        })
        .collect();

    let count = rates.len();
    Ok(mcp_tool_call_result(
        json!({
            "tool": MCP_TOOL_LEARN_TOOL_RATES,
            "rates": rates,
            "count": count,
        }),
        false,
    ))
}

// ---------------------------------------------------------------------------
// Training tool handlers
// ---------------------------------------------------------------------------

fn load_training_report(state: &McpServerState) -> Value {
    let training_state_path = state.session_path.with_extension("training.json");
    if training_state_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&training_state_path) {
            if let Ok(report) = serde_json::from_str::<Value>(&content) {
                return report;
            }
        }
    }
    json!({
        "live_rl_enabled": false,
        "total_rollouts": 0,
        "last_reward_score": null,
        "apo_threshold": 0,
        "apo_runs_completed": 0,
        "current_prompt_version": null,
    })
}

fn handle_training_status(state: &McpServerState) -> Result<Value> {
    let mut report = load_training_report(state);
    report
        .as_object_mut()
        .unwrap()
        .insert("tool".into(), json!(MCP_TOOL_TRAINING_STATUS));
    Ok(mcp_tool_call_result(report, false))
}

fn runtime_unavailable_tool_result(
    tool_name: &str,
    subsystem: &str,
    extra_fields: serde_json::Map<String, Value>,
) -> Value {
    let mut content = serde_json::Map::new();
    content.insert("tool".into(), json!(tool_name));
    content.extend(extra_fields);
    content.insert("status".into(), json!("not_implemented"));
    content.insert("reason_code".into(), json!("runtime_unavailable"));
    content.insert(
        "message".into(),
        json!(format!(
            "{tool_name} requires a connected {subsystem} runtime. Standalone MCP server mode does not provide one."
        )),
    );
    mcp_tool_call_result(Value::Object(content), true)
}

fn handle_training_trigger(arguments: &Value) -> Result<Value> {
    let scope = arguments
        .get("scope")
        .and_then(Value::as_str)
        .unwrap_or("incremental");

    let mut extra = serde_json::Map::new();
    extra.insert("scope".into(), json!(scope));
    Ok(runtime_unavailable_tool_result(
        MCP_TOOL_TRAINING_TRIGGER,
        "training",
        extra,
    ))
}

// ---------------------------------------------------------------------------
// Skills tool handlers (delegated to tau-skills)
// ---------------------------------------------------------------------------

fn handle_skills_list(arguments: &Value, state: &McpServerState) -> Result<Value> {
    let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(50) as usize;
    let category_filter = arguments.get("category").and_then(Value::as_str);
    let category_query = category_filter.map(|value| value.to_ascii_lowercase());
    let skills = tau_skills::load_catalog(&state.skills_dir)?
        .into_iter()
        .filter(|skill| {
            category_query
                .as_ref()
                .map(|needle| skill.description.to_ascii_lowercase().contains(needle))
                .unwrap_or(true)
        })
        .take(limit)
        .map(|skill| {
            json!({
                "name": skill.name,
                "description": skill.description,
                "path": skill.path.display().to_string(),
            })
        })
        .collect::<Vec<_>>();

    let total = skills.len();
    Ok(mcp_tool_call_result(
        json!({
            "tool": MCP_TOOL_SKILLS_LIST,
            "skills": skills,
            "total": total,
            "skills_dir": state.skills_dir.display().to_string(),
        }),
        false,
    ))
}

fn handle_skills_search(arguments: &Value, state: &McpServerState) -> Result<Value> {
    let query = arguments
        .get("query")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("query is required"))?;
    let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
    let query_lower = query.to_ascii_lowercase();
    let results = tau_skills::load_catalog(&state.skills_dir)?
        .into_iter()
        .filter(|skill| {
            skill.name.to_ascii_lowercase().contains(&query_lower)
                || skill
                    .description
                    .to_ascii_lowercase()
                    .contains(&query_lower)
                || skill.content.to_ascii_lowercase().contains(&query_lower)
        })
        .take(limit)
        .map(|skill| {
            json!({
                "name": skill.name,
                "description": skill.description,
                "path": skill.path.display().to_string(),
            })
        })
        .collect::<Vec<_>>();

    let count = results.len();
    Ok(mcp_tool_call_result(
        json!({
            "tool": MCP_TOOL_SKILLS_SEARCH,
            "query": query,
            "results": results,
            "count": count,
        }),
        false,
    ))
}

fn handle_skills_info(arguments: &Value, state: &McpServerState) -> Result<Value> {
    let skill_name = arguments
        .get("skill_name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("skill_name is required"))?;
    let skill_lower = skill_name.to_ascii_lowercase();

    let found = tau_skills::load_catalog(&state.skills_dir)?
        .into_iter()
        .find(|skill| skill.name.to_ascii_lowercase() == skill_lower);

    match found {
        Some(skill) => {
            let preview = if skill.content.len() > 500 {
                format!("{}...", &skill.content[..500])
            } else {
                skill.content.clone()
            };
            Ok(mcp_tool_call_result(
                json!({
                    "tool": MCP_TOOL_SKILLS_INFO,
                    "name": skill.name,
                    "description": skill.description,
                    "path": skill.path.display().to_string(),
                    "content_length": skill.content.len(),
                    "content_preview": preview,
                }),
                false,
            ))
        }
        None => Ok(mcp_tool_call_result(
            json!({
                "tool": MCP_TOOL_SKILLS_INFO,
                "skill_name": skill_name,
                "status": "not_found",
                "message": format!("Skill '{}' not found in {}", skill_name, state.skills_dir.display()),
            }),
            true,
        )),
    }
}

fn handle_skills_install(arguments: &Value, state: &McpServerState) -> Result<Value> {
    let source = arguments
        .get("source")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("source is required"))?;
    let name_override = arguments.get("name").and_then(Value::as_str);

    let source_path = Path::new(source);
    if !source_path.exists() {
        return Ok(mcp_tool_call_result(
            json!({
                "tool": MCP_TOOL_SKILLS_INSTALL,
                "source": source,
                "status": "error",
                "message": format!("Source path '{}' does not exist", source),
            }),
            true,
        ));
    }

    let dest_name = name_override.map(|n| format!("{n}.md")).unwrap_or_else(|| {
        source_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("skill.md")
            .to_string()
    });
    let dest_path = state.skills_dir.join(&dest_name);

    let mut staged_root: Option<PathBuf> = None;
    let install_sources = if name_override.is_some() {
        let root = std::env::temp_dir().join(format!(
            "tau-mcp-skill-install-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&root)
            .with_context(|| format!("failed to create {}", root.display()))?;
        let staged_path = root.join(&dest_name);
        let content = std::fs::read_to_string(source_path)
            .with_context(|| format!("failed to read skill source {}", source_path.display()))?;
        std::fs::write(&staged_path, content)
            .with_context(|| format!("failed to stage skill source {}", staged_path.display()))?;
        staged_root = Some(root);
        vec![staged_path]
    } else {
        vec![source_path.to_path_buf()]
    };

    let install_result = tau_skills::install_skills(&install_sources, &state.skills_dir);
    if let Some(root) = staged_root {
        let _ = std::fs::remove_dir_all(root);
    }

    let report = match install_result {
        Ok(report) => report,
        Err(error) => {
            return Ok(mcp_tool_call_result(
                json!({
                    "tool": MCP_TOOL_SKILLS_INSTALL,
                    "source": source,
                    "name_override": name_override,
                    "status": "error",
                    "message": error.to_string(),
                }),
                true,
            ));
        }
    };

    let lock_hint = tau_skills::SkillLockHint {
        file: dest_name.clone(),
        source: tau_skills::SkillLockSource::Local {
            path: source_path.display().to_string(),
        },
    };
    let lock_path = tau_skills::default_skills_lock_path(&state.skills_dir);
    let lockfile =
        match tau_skills::write_skills_lockfile(&state.skills_dir, &lock_path, &[lock_hint]) {
            Ok(lockfile) => lockfile,
            Err(error) => {
                return Ok(mcp_tool_call_result(
                    json!({
                        "tool": MCP_TOOL_SKILLS_INSTALL,
                        "source": source,
                        "name_override": name_override,
                        "installed_path": dest_path.display().to_string(),
                        "status": "error",
                        "message": format!("skill installed but lockfile update failed: {error}"),
                    }),
                    true,
                ));
            }
        };

    let status = if report.installed > 0 {
        "installed"
    } else if report.updated > 0 {
        "updated"
    } else {
        "skipped"
    };

    Ok(mcp_tool_call_result(
        json!({
            "tool": MCP_TOOL_SKILLS_INSTALL,
            "source": source,
            "name_override": name_override,
            "installed_path": dest_path.display().to_string(),
            "installed": report.installed,
            "updated": report.updated,
            "skipped": report.skipped,
            "lockfile_path": lock_path.display().to_string(),
            "lockfile_entries": lockfile.entries.len(),
            "status": status,
        }),
        false,
    ))
}

// ---------------------------------------------------------------------------
// Orchestration tool handlers (structured response builders)
// ---------------------------------------------------------------------------

fn handle_agent_spawn(arguments: &Value) -> Result<Value> {
    let goal = arguments
        .get("goal")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("goal is required"))?;
    let model = arguments.get("model").and_then(Value::as_str);
    let max_turns = arguments.get("max_turns").and_then(Value::as_u64);
    let mut extra = serde_json::Map::new();
    extra.insert("goal".into(), json!(goal));
    extra.insert("model".into(), json!(model));
    extra.insert("max_turns".into(), json!(max_turns));
    Ok(runtime_unavailable_tool_result(
        MCP_TOOL_AGENT_SPAWN,
        "orchestration",
        extra,
    ))
}

fn handle_agent_status(arguments: &Value) -> Result<Value> {
    let agent_id = arguments
        .get("agent_id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("agent_id is required"))?;

    let mut extra = serde_json::Map::new();
    extra.insert("agent_id".into(), json!(agent_id));
    Ok(runtime_unavailable_tool_result(
        MCP_TOOL_AGENT_STATUS,
        "orchestration",
        extra,
    ))
}

fn handle_agent_cancel(arguments: &Value) -> Result<Value> {
    let agent_id = arguments
        .get("agent_id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("agent_id is required"))?;

    let mut extra = serde_json::Map::new();
    extra.insert("agent_id".into(), json!(agent_id));
    Ok(runtime_unavailable_tool_result(
        MCP_TOOL_AGENT_CANCEL,
        "orchestration",
        extra,
    ))
}

// ---------------------------------------------------------------------------
// Context provider handlers
// ---------------------------------------------------------------------------

fn handle_context_learning(state: &McpServerState) -> Result<Value> {
    let report = load_learn_report_from_session(state);
    let total = report
        .get("total_records")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let sessions = report
        .get("sessions_tracked")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let pattern_count = report
        .get("top_failure_patterns")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    let rate_count = report
        .get("tool_success_rates")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);

    let summary = format!(
        "learning: total_records={} sessions_tracked={} failure_patterns={} tool_rates={}",
        total, sessions, pattern_count, rate_count,
    );

    Ok(mcp_tool_call_result(
        json!({
            "provider": "learning",
            "summary": summary,
            "total_records": total,
            "sessions_tracked": sessions,
            "failure_pattern_count": pattern_count,
            "tool_rate_count": rate_count,
        }),
        false,
    ))
}

fn handle_context_training(state: &McpServerState) -> Result<Value> {
    let report = load_training_report(state);
    let live_rl = report
        .get("live_rl_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let rollouts = report
        .get("total_rollouts")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let apo_runs = report
        .get("apo_runs_completed")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let apo_threshold = report
        .get("apo_threshold")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    let summary = format!(
        "training: live_rl={} rollouts={} apo_runs={}/{}",
        if live_rl { "enabled" } else { "disabled" },
        rollouts,
        apo_runs,
        apo_threshold,
    );

    Ok(mcp_tool_call_result(
        json!({
            "provider": "training",
            "summary": summary,
            "live_rl_enabled": live_rl,
            "total_rollouts": rollouts,
            "apo_runs_completed": apo_runs,
        }),
        false,
    ))
}

fn handle_context_config(state: &McpServerState) -> Result<Value> {
    let skills_count = list_skill_files(&state.skills_dir, 1024)
        .map(|f| f.len())
        .unwrap_or(0);
    let context_providers: Vec<&str> = state.context_providers.iter().map(|s| s.as_str()).collect();
    let external_server_names: Vec<&str> = state
        .external_servers
        .iter()
        .map(|s| s.name.as_str())
        .collect();

    Ok(mcp_tool_call_result(
        json!({
            "provider": "config",
            "session_path": state.session_path.display().to_string(),
            "skills_dir": state.skills_dir.display().to_string(),
            "skills_count": skills_count,
            "channel_store_root": state.channel_store_root.display().to_string(),
            "context_providers": context_providers,
            "external_servers": external_server_names,
            "external_tools_count": state.external_tools.len(),
        }),
        false,
    ))
}

/// Attempt to dispatch a tool call to a skill-registered tool.
///
/// Loads the skill catalog from `skills_dir`, checks whether any loaded skill
/// declares a tool matching `tool_name`, and dispatches via
/// `skill_runtime::dispatch_skill_tool()` if found.
///
/// Returns `Some(result)` if a skill handled the tool, or `None` if no skill
/// declares a tool with the given name.
fn try_dispatch_skill_tool(tool_name: &str, params: &Value, skills_dir: &Path) -> Option<Value> {
    let catalog = match tau_skills::load_catalog(skills_dir) {
        Ok(catalog) => catalog,
        Err(_) => return None,
    };

    for skill in &catalog {
        let Some(tools) = skill.tools.as_ref() else {
            continue;
        };
        if !tools.iter().any(|t| t.name == tool_name) {
            continue;
        }
        match tau_skills::skill_runtime::dispatch_skill_tool(skill, tool_name, params.clone()) {
            Ok(result) => return Some(result),
            Err(error) => {
                return Some(json!({
                    "error": format!("skill tool dispatch failed: {error}"),
                    "skill": skill.name,
                    "tool": tool_name
                }));
            }
        }
    }

    None
}

fn execute_builtin_tool_call(
    tool_name: &str,
    arguments: Value,
    policy: &ToolPolicy,
) -> Result<ToolExecutionResult> {
    let policy = Arc::new(policy.clone());
    match tool_name {
        MCP_TOOL_READ => Ok(block_on_tool_future(
            ReadTool::new(policy).execute(arguments),
        )),
        MCP_TOOL_WRITE => Ok(block_on_tool_future(
            WriteTool::new(policy).execute(arguments),
        )),
        MCP_TOOL_EDIT => Ok(block_on_tool_future(
            EditTool::new(policy).execute(arguments),
        )),
        MCP_TOOL_MEMORY_WRITE => Ok(block_on_tool_future(
            MemoryWriteTool::new(policy).execute(arguments),
        )),
        MCP_TOOL_MEMORY_READ => Ok(block_on_tool_future(
            MemoryReadTool::new(policy).execute(arguments),
        )),
        MCP_TOOL_MEMORY_SEARCH => Ok(block_on_tool_future(
            MemorySearchTool::new(policy).execute(arguments),
        )),
        MCP_TOOL_MEMORY_TREE => Ok(block_on_tool_future(
            MemoryTreeTool::new(policy).execute(arguments),
        )),
        MCP_TOOL_JOBS_CREATE => Ok(block_on_tool_future(
            JobsCreateTool::new(policy).execute(arguments),
        )),
        MCP_TOOL_JOBS_LIST => Ok(block_on_tool_future(
            JobsListTool::new(policy).execute(arguments),
        )),
        MCP_TOOL_JOBS_STATUS => Ok(block_on_tool_future(
            JobsStatusTool::new(policy).execute(arguments),
        )),
        MCP_TOOL_JOBS_CANCEL => Ok(block_on_tool_future(
            JobsCancelTool::new(policy).execute(arguments),
        )),
        MCP_TOOL_HTTP => Ok(block_on_tool_future(
            HttpTool::new(policy).execute(arguments),
        )),
        MCP_TOOL_BASH => Ok(block_on_tool_future(
            BashTool::new(policy).execute(arguments),
        )),
        other => bail!("unknown mcp tool '{}'", other),
    }
}

fn block_on_tool_future<F>(future: F) -> ToolExecutionResult
where
    F: Future<Output = ToolExecutionResult>,
{
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(future)),
        Err(_) => {
            match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime.block_on(future),
                Err(error) => ToolExecutionResult::error(json!({
                    "error": format!("failed to create temporary tokio runtime for mcp tool execution: {error}")
                })),
            }
        }
    }
}

fn builtin_mcp_tools(state: &McpServerState) -> Vec<McpToolDescriptor> {
    let policy = Arc::new(state.tool_policy.clone());
    let mut tools = vec![
        agent_tool_descriptor(MCP_TOOL_READ, &ReadTool::new(policy.clone())),
        agent_tool_descriptor(MCP_TOOL_WRITE, &WriteTool::new(policy.clone())),
        agent_tool_descriptor(MCP_TOOL_EDIT, &EditTool::new(policy.clone())),
        agent_tool_descriptor(MCP_TOOL_MEMORY_WRITE, &MemoryWriteTool::new(policy.clone())),
        agent_tool_descriptor(MCP_TOOL_MEMORY_READ, &MemoryReadTool::new(policy.clone())),
        agent_tool_descriptor(
            MCP_TOOL_MEMORY_SEARCH,
            &MemorySearchTool::new(policy.clone()),
        ),
        agent_tool_descriptor(MCP_TOOL_MEMORY_TREE, &MemoryTreeTool::new(policy.clone())),
        agent_tool_descriptor(MCP_TOOL_JOBS_CREATE, &JobsCreateTool::new(policy.clone())),
        agent_tool_descriptor(MCP_TOOL_JOBS_LIST, &JobsListTool::new(policy.clone())),
        agent_tool_descriptor(MCP_TOOL_JOBS_STATUS, &JobsStatusTool::new(policy.clone())),
        agent_tool_descriptor(MCP_TOOL_JOBS_CANCEL, &JobsCancelTool::new(policy.clone())),
        agent_tool_descriptor(MCP_TOOL_HTTP, &HttpTool::new(policy.clone())),
        agent_tool_descriptor(MCP_TOOL_BASH, &BashTool::new(policy)),
    ];

    // Session tools
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_SESSION_LIST.to_string(),
        description: "List available sessions with optional filtering".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "limit": { "type": "integer", "description": "Maximum number of sessions to return", "default": 20 },
                "offset": { "type": "integer", "description": "Offset for pagination", "default": 0 },
                "sort_by": { "type": "string", "enum": ["created", "updated", "name"], "description": "Field to sort by", "default": "updated" }
            },
            "additionalProperties": false
        }),
    });
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_SESSION_RESUME.to_string(),
        description: "Resume a previous session by identifier".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string", "description": "Identifier of the session to resume" }
            },
            "required": ["session_id"],
            "additionalProperties": false
        }),
    });
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_SESSION_SEARCH.to_string(),
        description: "Search session history by query string".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query to match against session content" },
                "limit": { "type": "integer", "description": "Maximum number of results", "default": 10 }
            },
            "required": ["query"],
            "additionalProperties": false
        }),
    });
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_SESSION_STATS.to_string(),
        description: "Return statistics about session usage".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string", "description": "Optional session identifier; omit for aggregate stats" }
            },
            "additionalProperties": false
        }),
    });
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_SESSION_EXPORT.to_string(),
        description: "Export a session to a portable format".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string", "description": "Identifier of the session to export" },
                "format": { "type": "string", "enum": ["json", "markdown"], "description": "Export format", "default": "json" }
            },
            "required": ["session_id"],
            "additionalProperties": false
        }),
    });

    // Orchestration tools
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_AGENT_SPAWN.to_string(),
        description: "Request a new agent task from a connected orchestration runtime; standalone MCP mode returns not_implemented".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "goal": { "type": "string", "description": "The objective for the spawned agent" },
                "model": { "type": "string", "description": "Optional model override for the agent" },
                "max_turns": { "type": "integer", "description": "Maximum number of turns before auto-cancel" }
            },
            "required": ["goal"],
            "additionalProperties": false
        }),
    });
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_AGENT_STATUS.to_string(),
        description: "Query an agent through a connected orchestration runtime; standalone MCP mode returns not_implemented".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "agent_id": { "type": "string", "description": "Identifier of the agent to query" }
            },
            "required": ["agent_id"],
            "additionalProperties": false
        }),
    });
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_AGENT_CANCEL.to_string(),
        description: "Cancel an agent through a connected orchestration runtime; standalone MCP mode returns not_implemented".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "agent_id": { "type": "string", "description": "Identifier of the agent to cancel" }
            },
            "required": ["agent_id"],
            "additionalProperties": false
        }),
    });

    // Learning tools
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_LEARN_STATUS.to_string(),
        description: "Return current learning subsystem status and statistics".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }),
    });
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_LEARN_FAILURE_PATTERNS.to_string(),
        description: "List learned failure patterns and their frequencies".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "limit": { "type": "integer", "description": "Maximum number of patterns to return", "default": 20 },
                "min_occurrences": { "type": "integer", "description": "Minimum occurrence count to include", "default": 1 }
            },
            "additionalProperties": false
        }),
    });
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_LEARN_TOOL_RATES.to_string(),
        description: "Return success/failure rates per tool".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "tool_name": { "type": "string", "description": "Optional filter by tool name" }
            },
            "additionalProperties": false
        }),
    });

    // Training tools
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_TRAINING_STATUS.to_string(),
        description: "Return training pipeline status and recent runs".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }),
    });
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_TRAINING_TRIGGER.to_string(),
        description: "Request a training pipeline run from a connected training runtime; standalone MCP mode returns not_implemented".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "scope": { "type": "string", "enum": ["full", "incremental"], "description": "Training scope", "default": "incremental" }
            },
            "additionalProperties": false
        }),
    });

    // Skills tools
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_SKILLS_LIST.to_string(),
        description: "List installed skills with metadata".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "limit": { "type": "integer", "description": "Maximum number of skills to return", "default": 50 },
                "category": { "type": "string", "description": "Optional category filter" }
            },
            "additionalProperties": false
        }),
    });
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_SKILLS_SEARCH.to_string(),
        description: "Search skills by keyword or capability".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query for skill discovery" },
                "limit": { "type": "integer", "description": "Maximum number of results", "default": 10 }
            },
            "required": ["query"],
            "additionalProperties": false
        }),
    });
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_SKILLS_INSTALL.to_string(),
        description: "Install a skill from a registry or local path".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "source": { "type": "string", "description": "Skill source URI or local path" },
                "name": { "type": "string", "description": "Optional name override for the installed skill" }
            },
            "required": ["source"],
            "additionalProperties": false
        }),
    });
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_SKILLS_INFO.to_string(),
        description: "Return detailed information about a specific skill".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "skill_name": { "type": "string", "description": "Name of the skill to inspect" }
            },
            "required": ["skill_name"],
            "additionalProperties": false
        }),
    });

    // Additional context providers
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_CONTEXT_LEARNING.to_string(),
        description: "Return learning subsystem context summary".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }),
    });
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_CONTEXT_TRAINING.to_string(),
        description: "Return training pipeline context summary".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }),
    });
    tools.push(McpToolDescriptor {
        name: MCP_TOOL_CONTEXT_CONFIG.to_string(),
        description: "Return current Tau configuration context".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }),
    });

    if state
        .context_providers
        .contains(MCP_CONTEXT_PROVIDER_SESSION)
    {
        tools.push(McpToolDescriptor {
            name: MCP_TOOL_CONTEXT_SESSION.to_string(),
            description: "Summarize configured Tau session context".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        });
    }
    if state
        .context_providers
        .contains(MCP_CONTEXT_PROVIDER_SKILLS)
    {
        tools.push(McpToolDescriptor {
            name: MCP_TOOL_CONTEXT_SKILLS.to_string(),
            description: "List discovered skills context files".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        });
    }
    if state
        .context_providers
        .contains(MCP_CONTEXT_PROVIDER_CHANNEL_STORE)
    {
        tools.push(McpToolDescriptor {
            name: MCP_TOOL_CONTEXT_CHANNEL_STORE.to_string(),
            description: "Summarize channel-store context state".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        });
    }

    tools
}

fn agent_tool_descriptor<T: AgentTool>(name: &str, tool: &T) -> McpToolDescriptor {
    let definition = tool.definition();
    McpToolDescriptor {
        name: name.to_string(),
        description: definition.description,
        input_schema: definition.parameters,
    }
}

fn mcp_tool_call_result(content: Value, is_error: bool) -> Value {
    let text = serde_json::to_string_pretty(&content)
        .unwrap_or_else(|_| "{\"error\":\"failed to serialize tool result\"}".to_string());
    json!({
        "content": [{
            "type": MCP_CONTENT_TYPE_TEXT,
            "text": text
        }],
        "isError": is_error,
        "structuredContent": content,
    })
}

fn read_jsonrpc_content_length_frame<R>(reader: &mut R) -> Result<Option<Value>>
where
    R: BufRead,
{
    let mut content_length: Option<usize> = None;
    let mut saw_header = false;
    loop {
        let mut line = String::new();
        let bytes = reader
            .read_line(&mut line)
            .context("failed to read mcp frame header line")?;
        if bytes == 0 {
            if saw_header {
                bail!("unexpected eof while reading mcp frame headers");
            }
            return Ok(None);
        }
        saw_header = true;
        if line == "\n" || line == "\r\n" {
            break;
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        let (name, value) = trimmed.split_once(':').ok_or_else(|| {
            anyhow!(
                "invalid mcp header '{}': expected 'Name: value' format",
                trimmed
            )
        })?;
        if name.trim().eq_ignore_ascii_case("content-length") {
            let parsed = value
                .trim()
                .parse::<usize>()
                .context("invalid Content-Length header value")?;
            content_length = Some(parsed);
        }
    }

    let content_length =
        content_length.ok_or_else(|| anyhow!("mcp frame is missing Content-Length header"))?;
    let mut body = vec![0_u8; content_length];
    reader
        .read_exact(&mut body)
        .context("failed to read mcp frame body bytes")?;
    let value = serde_json::from_slice::<Value>(&body).context("failed to parse mcp JSON frame")?;
    Ok(Some(value))
}

fn write_jsonrpc_content_length_frame<W>(writer: &mut W, value: &Value) -> Result<()>
where
    W: Write,
{
    let encoded = serde_json::to_vec(value).context("failed to encode mcp jsonrpc response")?;
    write!(writer, "Content-Length: {}\r\n\r\n", encoded.len())
        .context("failed to write mcp frame header")?;
    writer
        .write_all(&encoded)
        .context("failed to write mcp frame body")?;
    writer.flush().context("failed to flush mcp frame output")?;
    Ok(())
}

fn jsonrpc_request_frame(id: Value, method: &str, params: Value) -> Value {
    json!({
        "jsonrpc": MCP_JSONRPC_VERSION,
        "id": id,
        "method": method,
        "params": params,
    })
}

fn jsonrpc_result_frame(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": MCP_JSONRPC_VERSION,
        "id": id,
        "result": result,
    })
}

fn jsonrpc_error_frame(id: Value, code: i64, message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": MCP_JSONRPC_VERSION,
        "id": id,
        "error": {
            "code": code,
            "message": message.into(),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{
        execute_context_provider_tool, jsonrpc_request_frame, normalize_external_server_name,
        resolve_mcp_context_providers, serve_mcp_jsonrpc_reader, McpExternalServerConfig,
        McpServerState, MCP_CONTEXT_PROVIDER_CHANNEL_STORE, MCP_CONTEXT_PROVIDER_SESSION,
        MCP_ERROR_INVALID_REQUEST, MCP_ERROR_METHOD_NOT_FOUND, MCP_JSONRPC_VERSION,
    };
    use crate::tools::ToolPolicy;
    use serde::Deserialize;
    use serde_json::{json, Value};
    use std::collections::BTreeMap;
    use std::io::{BufRead, Read};
    use std::path::Path;
    use tempfile::tempdir;

    fn encode_frames(frames: &[Value]) -> Vec<u8> {
        let mut encoded = Vec::new();
        for frame in frames {
            let payload = serde_json::to_vec(frame).expect("encode frame");
            encoded
                .extend_from_slice(format!("Content-Length: {}\r\n\r\n", payload.len()).as_bytes());
            encoded.extend_from_slice(&payload);
        }
        encoded
    }

    fn decode_frames(raw: &[u8]) -> Vec<Value> {
        let mut frames = Vec::new();
        let mut cursor = std::io::Cursor::new(raw);
        let mut reader = std::io::BufReader::new(&mut cursor);
        loop {
            let mut header = String::new();
            let bytes = reader.read_line(&mut header).expect("header");
            if bytes == 0 {
                break;
            }
            if header.trim().is_empty() {
                continue;
            }
            let length = header
                .split_once(':')
                .and_then(|(_, value)| value.trim().parse::<usize>().ok())
                .expect("content length");
            let mut separator = String::new();
            reader.read_line(&mut separator).expect("separator");
            let mut body = vec![0_u8; length];
            reader.read_exact(&mut body).expect("body");
            let frame = serde_json::from_slice::<Value>(&body).expect("json frame");
            frames.push(frame);
        }
        frames
    }

    fn test_state() -> McpServerState {
        let temp = tempdir().expect("tempdir");
        let tau_root = temp.path().join(".tau");
        std::fs::create_dir_all(tau_root.join("skills")).expect("create skills");
        std::fs::create_dir_all(tau_root.join("sessions")).expect("create sessions");
        std::fs::create_dir_all(tau_root.join("channel-store/channels"))
            .expect("create channel store");
        let session_path = tau_root.join("sessions/default.sqlite");
        let mut store = tau_session::SessionStore::load(&session_path).expect("load session");
        store
            .append_messages(None, &[tau_ai::Message::system("mcp-session-seed")])
            .expect("seed session");
        McpServerState {
            tool_policy: ToolPolicy::new(vec![temp.path().to_path_buf()]),
            session_path,
            skills_dir: tau_root.join("skills"),
            channel_store_root: tau_root.join("channel-store"),
            context_providers: resolve_mcp_context_providers(&[])
                .expect("providers")
                .into_iter()
                .collect(),
            external_servers: Vec::new(),
            external_tools: Vec::new(),
        }
    }

    fn test_state_from_root(root: &Path) -> McpServerState {
        let tau_root = root.join(".tau");
        std::fs::create_dir_all(tau_root.join("skills")).expect("create skills");
        std::fs::create_dir_all(tau_root.join("sessions")).expect("create sessions");
        std::fs::create_dir_all(tau_root.join("channel-store/channels"))
            .expect("create channel store");
        let session_path = tau_root.join("sessions/default.sqlite");
        let mut store = tau_session::SessionStore::load(&session_path).expect("load session");
        store
            .append_messages(None, &[tau_ai::Message::system("mcp-session-seed")])
            .expect("seed session");
        McpServerState {
            tool_policy: ToolPolicy::new(vec![root.to_path_buf()]),
            session_path,
            skills_dir: tau_root.join("skills"),
            channel_store_root: tau_root.join("channel-store"),
            context_providers: resolve_mcp_context_providers(&[])
                .expect("providers")
                .into_iter()
                .collect(),
            external_servers: Vec::new(),
            external_tools: Vec::new(),
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    struct McpProtocolFixture {
        schema_version: u32,
        name: String,
        requests: Vec<Value>,
        expected_response_ids: Vec<String>,
        expected_methods: Vec<String>,
    }

    fn load_mcp_protocol_fixture(name: &str) -> McpProtocolFixture {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join("mcp-protocol")
            .join(name);
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        serde_json::from_str::<McpProtocolFixture>(&raw)
            .unwrap_or_else(|error| panic!("invalid fixture {}: {error}", path.display()))
    }

    #[test]
    fn unit_resolve_mcp_context_providers_defaults_and_validation() {
        let defaults = resolve_mcp_context_providers(&[]).expect("default providers");
        assert_eq!(defaults, vec!["session", "skills", "channel-store"]);

        let selected = resolve_mcp_context_providers(&[
            "skills".to_string(),
            "session".to_string(),
            "skills".to_string(),
        ])
        .expect("selected providers");
        assert_eq!(selected, vec!["skills", "session"]);

        let error = resolve_mcp_context_providers(&["bad-provider".to_string()])
            .expect_err("invalid provider should fail");
        assert!(error
            .to_string()
            .contains("unsupported mcp context provider"));
    }

    #[test]
    fn unit_normalize_external_server_name_rejects_invalid_tokens() {
        assert_eq!(
            normalize_external_server_name("Server_01").expect("name"),
            "server_01"
        );
        let error =
            normalize_external_server_name("bad name").expect_err("spaces should be rejected");
        assert!(error
            .to_string()
            .contains("must contain only ASCII letters"));
    }

    #[test]
    fn functional_mcp_server_initialize_and_tools_list_roundtrip() {
        let state = test_state();
        let request_frames = vec![
            jsonrpc_request_frame(
                Value::String("req-init".to_string()),
                "initialize",
                serde_json::json!({}),
            ),
            jsonrpc_request_frame(
                Value::String("req-tools".to_string()),
                "tools/list",
                serde_json::json!({}),
            ),
        ];
        let raw = encode_frames(&request_frames);
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(raw));
        let mut writer = Vec::new();
        let report = serve_mcp_jsonrpc_reader(&mut reader, &mut writer, &state)
            .expect("serve should succeed");
        assert_eq!(report.processed_frames, 2);
        assert_eq!(report.error_count, 0);

        let responses = decode_frames(&writer);
        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0]["jsonrpc"], MCP_JSONRPC_VERSION);
        assert_eq!(responses[0]["id"], "req-init");
        assert_eq!(responses[0]["result"]["protocolVersion"], "2024-11-05");
        let tools = responses[1]["result"]["tools"]
            .as_array()
            .expect("tools array");
        assert!(tools.iter().any(|tool| tool["name"] == "tau.read"));
        assert!(tools.iter().any(|tool| tool["name"] == "tau.memory_write"));
        assert!(tools.iter().any(|tool| tool["name"] == "tau.memory_read"));
        assert!(tools.iter().any(|tool| tool["name"] == "tau.memory_search"));
        assert!(tools.iter().any(|tool| tool["name"] == "tau.memory_tree"));
        assert!(tools.iter().any(|tool| tool["name"] == "tau.jobs_create"));
        assert!(tools.iter().any(|tool| tool["name"] == "tau.jobs_list"));
        assert!(tools.iter().any(|tool| tool["name"] == "tau.jobs_status"));
        assert!(tools.iter().any(|tool| tool["name"] == "tau.jobs_cancel"));
        assert!(tools.iter().any(|tool| tool["name"] == "tau.http"));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "tau.context.session"));
    }

    #[test]
    fn integration_mcp_protocol_fixture_initialize_tools_list_roundtrip() {
        let fixture = load_mcp_protocol_fixture("initialize-tools-list.json");
        assert_eq!(fixture.schema_version, 1);
        assert_eq!(fixture.name, "initialize-tools-list");
        assert_eq!(
            fixture.expected_methods,
            vec!["initialize".to_string(), "tools/list".to_string()]
        );

        let state = test_state();
        let raw = encode_frames(&fixture.requests);
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(raw));
        let mut writer = Vec::new();
        let report = serve_mcp_jsonrpc_reader(&mut reader, &mut writer, &state)
            .expect("serve should succeed");
        assert_eq!(report.processed_frames, fixture.requests.len());
        assert_eq!(report.error_count, 0);

        let responses = decode_frames(&writer);
        let ids = responses
            .iter()
            .map(|response| {
                response["id"]
                    .as_str()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| response["id"].to_string())
            })
            .collect::<Vec<_>>();
        assert_eq!(ids, fixture.expected_response_ids);
    }

    #[test]
    fn integration_tools_call_context_provider_returns_structured_payload() {
        let mut state = test_state();
        state.context_providers = [MCP_CONTEXT_PROVIDER_SESSION.to_string()]
            .into_iter()
            .collect();
        let result = execute_context_provider_tool(&state, "tau.context.session")
            .expect("context provider call should succeed");
        assert_eq!(result["provider"], MCP_CONTEXT_PROVIDER_SESSION);
        assert!(result["exists"].is_boolean());
        assert!(result["entries"].is_number());
        assert!(result["storage_backend"].is_string());
        assert!(result["backend_reason_code"].is_string());
    }

    #[test]
    fn integration_tools_call_write_denies_protected_paths() {
        let temp = tempdir().expect("tempdir");
        let tau_root = temp.path().join(".tau");
        std::fs::create_dir_all(tau_root.join("skills")).expect("create skills");
        std::fs::create_dir_all(tau_root.join("sessions")).expect("create sessions");
        std::fs::create_dir_all(tau_root.join("channel-store/channels"))
            .expect("create channel store");
        let session_path = tau_root.join("sessions/default.sqlite");
        let mut store = tau_session::SessionStore::load(&session_path).expect("load session");
        store
            .append_messages(None, &[tau_ai::Message::system("mcp-session-seed")])
            .expect("seed session");
        let state = McpServerState {
            tool_policy: ToolPolicy::new(vec![temp.path().to_path_buf()]),
            session_path,
            skills_dir: tau_root.join("skills"),
            channel_store_root: tau_root.join("channel-store"),
            context_providers: resolve_mcp_context_providers(&[])
                .expect("providers")
                .into_iter()
                .collect(),
            external_servers: Vec::new(),
            external_tools: Vec::new(),
        };
        let request_frames = vec![jsonrpc_request_frame(
            Value::String("req-write".to_string()),
            "tools/call",
            serde_json::json!({
                "name": "tau.write",
                "arguments": {
                    "path": temp.path().join("AGENTS.md").display().to_string(),
                    "content": "blocked"
                }
            }),
        )];
        let raw = encode_frames(&request_frames);
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(raw));
        let mut writer = Vec::new();
        let report = serve_mcp_jsonrpc_reader(&mut reader, &mut writer, &state)
            .expect("serve should succeed");
        assert_eq!(report.processed_frames, 1);
        assert_eq!(report.error_count, 0);

        let responses = decode_frames(&writer);
        assert_eq!(responses[0]["id"], "req-write");
        assert_eq!(responses[0]["result"]["isError"], true);
        assert_eq!(
            responses[0]["result"]["structuredContent"]["policy_rule"],
            "protected_path"
        );
        assert_eq!(
            responses[0]["result"]["structuredContent"]["reason_code"],
            "protected_path_denied"
        );
    }

    #[test]
    fn integration_tools_call_http_blocks_plain_http_scheme_by_default() {
        let state = test_state();
        let request_frames = vec![jsonrpc_request_frame(
            Value::String("req-http".to_string()),
            "tools/call",
            serde_json::json!({
                "name": "tau.http",
                "arguments": {
                    "url": "http://example.com",
                    "method": "GET"
                }
            }),
        )];
        let raw = encode_frames(&request_frames);
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(raw));
        let mut writer = Vec::new();
        let report = serve_mcp_jsonrpc_reader(&mut reader, &mut writer, &state)
            .expect("serve should succeed");
        assert_eq!(report.processed_frames, 1);
        assert_eq!(report.error_count, 0);

        let responses = decode_frames(&writer);
        assert_eq!(responses[0]["id"], "req-http");
        assert_eq!(responses[0]["result"]["isError"], true);
        assert_eq!(
            responses[0]["result"]["structuredContent"]["reason_code"],
            "delivery_ssrf_blocked_scheme"
        );
    }

    #[test]
    fn regression_invalid_request_and_unknown_method_return_jsonrpc_errors() {
        let state = test_state();
        let request_frames = vec![
            serde_json::json!({
                "jsonrpc": MCP_JSONRPC_VERSION,
                "method": "initialize",
                "params": {}
            }),
            jsonrpc_request_frame(
                Value::String("req-unknown".to_string()),
                "method/unknown",
                serde_json::json!({}),
            ),
        ];
        let raw = encode_frames(&request_frames);
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(raw));
        let mut writer = Vec::new();
        let report = serve_mcp_jsonrpc_reader(&mut reader, &mut writer, &state)
            .expect("serve should return report");
        assert_eq!(report.processed_frames, 2);
        assert_eq!(report.error_count, 2);

        let responses = decode_frames(&writer);
        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0]["error"]["code"], MCP_ERROR_INVALID_REQUEST);
        assert_eq!(responses[1]["error"]["code"], MCP_ERROR_METHOD_NOT_FOUND);
    }

    #[test]
    fn regression_context_provider_guard_rejects_disabled_provider() {
        let mut state = test_state();
        state.context_providers = [MCP_CONTEXT_PROVIDER_CHANNEL_STORE.to_string()]
            .into_iter()
            .collect();
        let error = execute_context_provider_tool(&state, "tau.context.session")
            .expect_err("disabled provider should fail");
        assert!(error.to_string().contains("is disabled"));
    }

    #[test]
    fn integration_external_discovery_and_call_via_line_jsonrpc_server() {
        let temp = tempdir().expect("tempdir");
        let script = temp.path().join("mock-external-mcp.sh");
        std::fs::write(
            &script,
            r#"#!/bin/sh
set -eu
while IFS= read -r line; do
  if [ -z "$line" ]; then
    continue
  fi
  method=$(printf '%s' "$line" | sed -n 's/.*"method":"\([^"]*\)".*/\1/p')
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  if [ "$method" = "initialize" ]; then
    printf '{"jsonrpc":"2.0","id":"%s","result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{"listChanged":false}}}}\n' "$id"
    continue
  fi
  if [ "$method" = "tools/list" ]; then
    printf '{"jsonrpc":"2.0","id":"%s","result":{"tools":[{"name":"echo","description":"echo tool","inputSchema":{"type":"object","properties":{"value":{"type":"string"}},"required":["value"]}}]}}\n' "$id"
    continue
  fi
  if [ "$method" = "tools/call" ]; then
    printf '{"jsonrpc":"2.0","id":"%s","result":{"content":[{"type":"text","text":"external-ok"}],"isError":false,"structuredContent":{"ok":true}}}\n' "$id"
    continue
  fi
done
"#,
        )
        .expect("write script");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script).expect("metadata").permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script, perms).expect("chmod");
        }

        let config = McpExternalServerConfig {
            name: "mock".to_string(),
            command: script.display().to_string(),
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: None,
            enabled: true,
        };
        let discovered = super::discover_external_mcp_tools(
            std::slice::from_ref(&config),
            &super::reserved_builtin_mcp_tool_names(),
        )
        .expect("discover external tool");
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].tool_name, "echo");

        let state = McpServerState {
            tool_policy: ToolPolicy::new(vec![temp.path().to_path_buf()]),
            session_path: temp.path().join(".tau/sessions/default.sqlite"),
            skills_dir: temp.path().join(".tau/skills"),
            channel_store_root: temp.path().join(".tau/channel-store"),
            context_providers: resolve_mcp_context_providers(&[])
                .expect("providers")
                .into_iter()
                .collect(),
            external_servers: vec![config],
            external_tools: discovered,
        };

        let request_frames = vec![jsonrpc_request_frame(
            Value::String("req-call".to_string()),
            "tools/call",
            serde_json::json!({
                "name": "external.mock.echo",
                "arguments": {"value":"hello"}
            }),
        )];
        let raw = encode_frames(&request_frames);
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(raw));
        let mut writer = Vec::new();
        let report = serve_mcp_jsonrpc_reader(&mut reader, &mut writer, &state)
            .expect("serve should succeed");
        assert_eq!(report.processed_frames, 1);
        assert_eq!(report.error_count, 0);
        let responses = decode_frames(&writer);
        assert_eq!(responses[0]["id"], "req-call");
        assert_eq!(responses[0]["result"]["isError"], false);
        assert_eq!(
            responses[0]["result"]["structuredContent"]["isError"],
            false
        );
    }

    #[test]
    fn unit_reserved_builtin_mcp_tool_names_contains_catalog_entries() {
        let names = super::reserved_builtin_mcp_tool_names();
        assert!(names.contains(super::MCP_TOOL_READ));
        assert!(names.contains(super::MCP_TOOL_WRITE));
        assert!(names.contains(super::MCP_TOOL_EDIT));
        assert!(names.contains(super::MCP_TOOL_MEMORY_WRITE));
        assert!(names.contains(super::MCP_TOOL_MEMORY_READ));
        assert!(names.contains(super::MCP_TOOL_MEMORY_SEARCH));
        assert!(names.contains(super::MCP_TOOL_MEMORY_TREE));
        assert!(names.contains(super::MCP_TOOL_HTTP));
        assert!(names.contains(super::MCP_TOOL_BASH));
        assert!(names.contains(super::MCP_TOOL_CONTEXT_SESSION));
        assert!(names.contains(super::MCP_TOOL_CONTEXT_SKILLS));
        assert!(names.contains(super::MCP_TOOL_CONTEXT_CHANNEL_STORE));
    }

    #[test]
    fn regression_external_discovery_rejects_reserved_builtin_name() {
        let temp = tempdir().expect("tempdir");
        let script = temp.path().join("mock-external-mcp-reserved.sh");
        std::fs::write(
            &script,
            r#"#!/bin/sh
set -eu
while IFS= read -r line; do
  if [ -z "$line" ]; then
    continue
  fi
  method=$(printf '%s' "$line" | sed -n 's/.*"method":"\([^"]*\)".*/\1/p')
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  if [ "$method" = "initialize" ]; then
    printf '{"jsonrpc":"2.0","id":"%s","result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{"listChanged":false}}}}\n' "$id"
    continue
  fi
  if [ "$method" = "tools/list" ]; then
    printf '{"jsonrpc":"2.0","id":"%s","result":{"tools":[{"name":"tau.read","description":"reserved","inputSchema":{"type":"object","properties":{}}}]}}\n' "$id"
    continue
  fi
done
"#,
        )
        .expect("write script");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script).expect("metadata").permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script, perms).expect("chmod");
        }

        let config = McpExternalServerConfig {
            name: "mock".to_string(),
            command: script.display().to_string(),
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: None,
            enabled: true,
        };
        let error = super::discover_external_mcp_tools(
            std::slice::from_ref(&config),
            &super::reserved_builtin_mcp_tool_names(),
        )
        .expect_err("reserved tool names must be rejected");
        assert!(error
            .to_string()
            .contains("reserved built-in tool name 'tau.read'"));
    }

    #[test]
    fn regression_external_discovery_rejects_duplicate_qualified_names() {
        let temp = tempdir().expect("tempdir");
        let script = temp.path().join("mock-external-mcp-duplicate.sh");
        std::fs::write(
            &script,
            r#"#!/bin/sh
set -eu
while IFS= read -r line; do
  if [ -z "$line" ]; then
    continue
  fi
  method=$(printf '%s' "$line" | sed -n 's/.*"method":"\([^"]*\)".*/\1/p')
  id=$(printf '%s' "$line" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  if [ "$method" = "initialize" ]; then
    printf '{"jsonrpc":"2.0","id":"%s","result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{"listChanged":false}}}}\n' "$id"
    continue
  fi
  if [ "$method" = "tools/list" ]; then
    printf '{"jsonrpc":"2.0","id":"%s","result":{"tools":[{"name":"echo","description":"first","inputSchema":{"type":"object","properties":{}}},{"name":"echo","description":"second","inputSchema":{"type":"object","properties":{}}}]}}\n' "$id"
    continue
  fi
done
"#,
        )
        .expect("write script");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script).expect("metadata").permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script, perms).expect("chmod");
        }

        let config = McpExternalServerConfig {
            name: "mock".to_string(),
            command: script.display().to_string(),
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: None,
            enabled: true,
        };
        let error = super::discover_external_mcp_tools(
            std::slice::from_ref(&config),
            &super::reserved_builtin_mcp_tool_names(),
        )
        .expect_err("duplicate names must fail");
        assert!(error
            .to_string()
            .contains("duplicate tool registration 'external.mock.echo'"));
    }

    #[test]
    fn unit_new_tool_constants_are_defined() {
        // Session tools
        assert_eq!(super::MCP_TOOL_SESSION_LIST, "tau.session_list");
        assert_eq!(super::MCP_TOOL_SESSION_RESUME, "tau.session_resume");
        assert_eq!(super::MCP_TOOL_SESSION_SEARCH, "tau.session_search");
        assert_eq!(super::MCP_TOOL_SESSION_STATS, "tau.session_stats");
        assert_eq!(super::MCP_TOOL_SESSION_EXPORT, "tau.session_export");
        // Orchestration tools
        assert_eq!(super::MCP_TOOL_AGENT_SPAWN, "tau.agent_spawn");
        assert_eq!(super::MCP_TOOL_AGENT_STATUS, "tau.agent_status");
        assert_eq!(super::MCP_TOOL_AGENT_CANCEL, "tau.agent_cancel");
        // Learning tools
        assert_eq!(super::MCP_TOOL_LEARN_STATUS, "tau.learn_status");
        assert_eq!(
            super::MCP_TOOL_LEARN_FAILURE_PATTERNS,
            "tau.learn_failure_patterns"
        );
        assert_eq!(super::MCP_TOOL_LEARN_TOOL_RATES, "tau.learn_tool_rates");
        // Training tools
        assert_eq!(super::MCP_TOOL_TRAINING_STATUS, "tau.training_status");
        assert_eq!(super::MCP_TOOL_TRAINING_TRIGGER, "tau.training_trigger");
        // Skills tools
        assert_eq!(super::MCP_TOOL_SKILLS_LIST, "tau.skills_list");
        assert_eq!(super::MCP_TOOL_SKILLS_SEARCH, "tau.skills_search");
        assert_eq!(super::MCP_TOOL_SKILLS_INSTALL, "tau.skills_install");
        assert_eq!(super::MCP_TOOL_SKILLS_INFO, "tau.skills_info");
        // Context providers
        assert_eq!(super::MCP_TOOL_CONTEXT_LEARNING, "tau.context.learning");
        assert_eq!(super::MCP_TOOL_CONTEXT_TRAINING, "tau.context.training");
        assert_eq!(super::MCP_TOOL_CONTEXT_CONFIG, "tau.context.config");
    }

    #[test]
    fn unit_tool_list_includes_all_new_tools_and_meets_minimum_count() {
        let state = test_state();
        let request_frames = vec![
            jsonrpc_request_frame(
                Value::String("req-init".to_string()),
                "initialize",
                serde_json::json!({}),
            ),
            jsonrpc_request_frame(
                Value::String("req-tools".to_string()),
                "tools/list",
                serde_json::json!({}),
            ),
        ];
        let raw = encode_frames(&request_frames);
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(raw));
        let mut writer = Vec::new();
        let report = serve_mcp_jsonrpc_reader(&mut reader, &mut writer, &state)
            .expect("serve should succeed");
        assert_eq!(report.error_count, 0);

        let responses = decode_frames(&writer);
        let tools = responses[1]["result"]["tools"]
            .as_array()
            .expect("tools array");

        // Verify minimum count of 30+ tools
        assert!(
            tools.len() >= 30,
            "expected at least 30 tools, got {}",
            tools.len()
        );

        let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

        // Session tools
        assert!(
            tool_names.contains(&"tau.session_list"),
            "missing session_list"
        );
        assert!(
            tool_names.contains(&"tau.session_resume"),
            "missing session_resume"
        );
        assert!(
            tool_names.contains(&"tau.session_search"),
            "missing session_search"
        );
        assert!(
            tool_names.contains(&"tau.session_stats"),
            "missing session_stats"
        );
        assert!(
            tool_names.contains(&"tau.session_export"),
            "missing session_export"
        );
        // Orchestration tools
        assert!(
            tool_names.contains(&"tau.agent_spawn"),
            "missing agent_spawn"
        );
        assert!(
            tool_names.contains(&"tau.agent_status"),
            "missing agent_status"
        );
        assert!(
            tool_names.contains(&"tau.agent_cancel"),
            "missing agent_cancel"
        );
        // Learning tools
        assert!(
            tool_names.contains(&"tau.learn_status"),
            "missing learn_status"
        );
        assert!(
            tool_names.contains(&"tau.learn_failure_patterns"),
            "missing learn_failure_patterns"
        );
        assert!(
            tool_names.contains(&"tau.learn_tool_rates"),
            "missing learn_tool_rates"
        );
        // Training tools
        assert!(
            tool_names.contains(&"tau.training_status"),
            "missing training_status"
        );
        assert!(
            tool_names.contains(&"tau.training_trigger"),
            "missing training_trigger"
        );
        // Skills tools
        assert!(
            tool_names.contains(&"tau.skills_list"),
            "missing skills_list"
        );
        assert!(
            tool_names.contains(&"tau.skills_search"),
            "missing skills_search"
        );
        assert!(
            tool_names.contains(&"tau.skills_install"),
            "missing skills_install"
        );
        assert!(
            tool_names.contains(&"tau.skills_info"),
            "missing skills_info"
        );
        // Context providers
        assert!(
            tool_names.contains(&"tau.context.learning"),
            "missing context.learning"
        );
        assert!(
            tool_names.contains(&"tau.context.training"),
            "missing context.training"
        );
        assert!(
            tool_names.contains(&"tau.context.config"),
            "missing context.config"
        );
    }

    #[test]
    fn unit_each_new_tool_has_valid_schema() {
        let state = test_state();
        let request_frames = vec![
            jsonrpc_request_frame(
                Value::String("req-init".to_string()),
                "initialize",
                serde_json::json!({}),
            ),
            jsonrpc_request_frame(
                Value::String("req-tools".to_string()),
                "tools/list",
                serde_json::json!({}),
            ),
        ];
        let raw = encode_frames(&request_frames);
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(raw));
        let mut writer = Vec::new();
        serve_mcp_jsonrpc_reader(&mut reader, &mut writer, &state).expect("serve should succeed");

        let responses = decode_frames(&writer);
        let tools = responses[1]["result"]["tools"]
            .as_array()
            .expect("tools array");

        let new_tool_names = [
            "tau.session_list",
            "tau.session_resume",
            "tau.session_search",
            "tau.session_stats",
            "tau.session_export",
            "tau.agent_spawn",
            "tau.agent_status",
            "tau.agent_cancel",
            "tau.learn_status",
            "tau.learn_failure_patterns",
            "tau.learn_tool_rates",
            "tau.training_status",
            "tau.training_trigger",
            "tau.skills_list",
            "tau.skills_search",
            "tau.skills_install",
            "tau.skills_info",
            "tau.context.learning",
            "tau.context.training",
            "tau.context.config",
        ];

        for expected_name in &new_tool_names {
            let tool = tools
                .iter()
                .find(|t| t["name"].as_str() == Some(expected_name))
                .unwrap_or_else(|| panic!("tool '{}' not found in tools list", expected_name));

            // Verify it has a name, description, and inputSchema
            assert!(tool["name"].is_string(), "{} missing name", expected_name);
            assert!(
                tool["description"].is_string()
                    && !tool["description"].as_str().unwrap().is_empty(),
                "{} missing or empty description",
                expected_name
            );
            let schema = &tool["inputSchema"];
            assert!(schema.is_object(), "{} missing inputSchema", expected_name);
            assert_eq!(
                schema["type"].as_str(),
                Some("object"),
                "{} inputSchema type must be 'object'",
                expected_name
            );
            assert!(
                schema["properties"].is_object(),
                "{} inputSchema missing properties",
                expected_name
            );
        }
    }

    #[test]
    fn unit_stateful_tools_return_structured_response() {
        let state = test_state();
        let tools_and_args: Vec<(&str, Value)> = vec![
            ("tau.session_list", json!({})),
            ("tau.learn_status", json!({})),
            ("tau.training_status", json!({})),
            ("tau.skills_list", json!({})),
            ("tau.context.learning", json!({})),
            ("tau.context.training", json!({})),
            ("tau.context.config", json!({})),
            ("tau.agent_spawn", json!({"goal": "test"})),
            ("tau.agent_status", json!({"agent_id": "test-1"})),
            ("tau.agent_cancel", json!({"agent_id": "test-1"})),
            ("tau.session_search", json!({"query": "test"})),
            ("tau.session_stats", json!({})),
            ("tau.learn_failure_patterns", json!({})),
            ("tau.learn_tool_rates", json!({})),
        ];
        for (tool_name, args) in &tools_and_args {
            let request_frames = vec![jsonrpc_request_frame(
                Value::String(format!("req-{}", tool_name)),
                "tools/call",
                json!({
                    "name": tool_name,
                    "arguments": args
                }),
            )];
            let raw = encode_frames(&request_frames);
            let mut reader = std::io::BufReader::new(std::io::Cursor::new(raw));
            let mut writer = Vec::new();
            let report = serve_mcp_jsonrpc_reader(&mut reader, &mut writer, &state)
                .expect("serve should succeed");
            assert_eq!(report.error_count, 0, "error calling {}", tool_name);

            let responses = decode_frames(&writer);
            let result = &responses[0]["result"];
            assert!(
                result.is_object(),
                "{} should return a result object",
                tool_name
            );
            let structured = &result["structuredContent"];
            assert!(
                structured.is_object(),
                "{} should return structuredContent",
                tool_name
            );
            // Verify no response has "not_yet_implemented" status
            assert_ne!(
                structured.get("status").and_then(Value::as_str),
                Some("not_yet_implemented"),
                "{} should not return not_yet_implemented",
                tool_name
            );
        }
    }

    #[test]
    fn regression_training_trigger_reports_runtime_unavailable_error() {
        let state = test_state();
        let request_frames = vec![jsonrpc_request_frame(
            Value::String("req-training-trigger".to_string()),
            "tools/call",
            json!({
                "name": "tau.training_trigger",
                "arguments": { "scope": "incremental" }
            }),
        )];
        let raw = encode_frames(&request_frames);
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(raw));
        let mut writer = Vec::new();
        let report = serve_mcp_jsonrpc_reader(&mut reader, &mut writer, &state)
            .expect("serve should succeed");
        assert_eq!(report.error_count, 0);

        let responses = decode_frames(&writer);
        let result = &responses[0]["result"];
        assert_eq!(result["isError"], true);
        assert_eq!(
            result["structuredContent"]["status"].as_str(),
            Some("not_implemented")
        );
        assert_eq!(
            result["structuredContent"]["reason_code"].as_str(),
            Some("runtime_unavailable")
        );
    }

    #[test]
    fn regression_agent_lifecycle_tools_report_runtime_unavailable_errors() {
        let state = test_state();
        let cases = [
            ("tau.agent_spawn", json!({ "goal": "test" })),
            ("tau.agent_status", json!({ "agent_id": "agent-1" })),
            ("tau.agent_cancel", json!({ "agent_id": "agent-1" })),
        ];

        for (tool_name, arguments) in cases {
            let request_frames = vec![jsonrpc_request_frame(
                Value::String(format!("req-{tool_name}")),
                "tools/call",
                json!({
                    "name": tool_name,
                    "arguments": arguments
                }),
            )];
            let raw = encode_frames(&request_frames);
            let mut reader = std::io::BufReader::new(std::io::Cursor::new(raw));
            let mut writer = Vec::new();
            let report = serve_mcp_jsonrpc_reader(&mut reader, &mut writer, &state)
                .expect("serve should succeed");
            assert_eq!(report.error_count, 0, "error calling {}", tool_name);

            let responses = decode_frames(&writer);
            let result = &responses[0]["result"];
            assert_eq!(result["isError"], true, "{} should be an error", tool_name);
            assert_eq!(
                result["structuredContent"]["status"].as_str(),
                Some("not_implemented"),
                "{} should return an explicit unsupported status",
                tool_name
            );
            assert_eq!(
                result["structuredContent"]["reason_code"].as_str(),
                Some("runtime_unavailable"),
                "{} should expose runtime_unavailable reason",
                tool_name
            );
        }
    }

    #[test]
    fn regression_skills_list_and_info_follow_tau_skills_catalog_resolution() {
        let temp = tempdir().expect("tempdir");
        let state = test_state_from_root(temp.path());
        let nested_skill_dir = state.skills_dir.join("ops-helper");
        std::fs::create_dir_all(&nested_skill_dir).expect("mkdir nested skill");
        std::fs::write(
            nested_skill_dir.join("SKILL.md"),
            "---\ndescription: Handles operational runbooks\n---\nUse this skill for ops work.\n",
        )
        .expect("write nested skill");

        let list_frames = vec![jsonrpc_request_frame(
            Value::String("req-skills-list".to_string()),
            "tools/call",
            json!({
                "name": "tau.skills_list",
                "arguments": {}
            }),
        )];
        let raw = encode_frames(&list_frames);
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(raw));
        let mut writer = Vec::new();
        serve_mcp_jsonrpc_reader(&mut reader, &mut writer, &state).expect("skills list succeeds");
        let responses = decode_frames(&writer);
        let skills = responses[0]["result"]["structuredContent"]["skills"]
            .as_array()
            .expect("skills array");
        assert!(
            skills.iter().any(|entry| entry["name"] == "ops-helper"),
            "directory-backed SKILL.md entries should resolve to the directory name"
        );

        let info_frames = vec![jsonrpc_request_frame(
            Value::String("req-skills-info".to_string()),
            "tools/call",
            json!({
                "name": "tau.skills_info",
                "arguments": { "skill_name": "ops-helper" }
            }),
        )];
        let raw = encode_frames(&info_frames);
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(raw));
        let mut writer = Vec::new();
        serve_mcp_jsonrpc_reader(&mut reader, &mut writer, &state).expect("skills info succeeds");
        let responses = decode_frames(&writer);
        assert_eq!(responses[0]["result"]["isError"], false);
        assert_eq!(
            responses[0]["result"]["structuredContent"]["name"].as_str(),
            Some("ops-helper")
        );
    }

    #[test]
    fn regression_skills_install_writes_skills_lockfile_metadata() {
        let temp = tempdir().expect("tempdir");
        let state = test_state_from_root(temp.path());
        let source = temp.path().join("release-notes.md");
        std::fs::write(
            &source,
            "---\ndescription: Release note helper\n---\nTurn notes into release summaries.\n",
        )
        .expect("write source skill");

        let request_frames = vec![jsonrpc_request_frame(
            Value::String("req-skills-install".to_string()),
            "tools/call",
            json!({
                "name": "tau.skills_install",
                "arguments": { "source": source.display().to_string() }
            }),
        )];
        let raw = encode_frames(&request_frames);
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(raw));
        let mut writer = Vec::new();
        serve_mcp_jsonrpc_reader(&mut reader, &mut writer, &state)
            .expect("skills install succeeds");
        let responses = decode_frames(&writer);
        let structured = &responses[0]["result"]["structuredContent"];
        let lockfile_path = structured["lockfile_path"]
            .as_str()
            .expect("lockfile path should be returned");
        let lockfile = tau_skills::load_skills_lockfile(Path::new(lockfile_path))
            .expect("lockfile should be readable");
        assert!(
            lockfile
                .entries
                .iter()
                .any(|entry| entry.file == "release-notes.md"),
            "installed skill should be recorded in the lockfile"
        );
    }

    #[test]
    fn unit_reserved_names_include_new_tools() {
        let names = super::reserved_builtin_mcp_tool_names();
        let new_names = [
            super::MCP_TOOL_SESSION_LIST,
            super::MCP_TOOL_SESSION_RESUME,
            super::MCP_TOOL_SESSION_SEARCH,
            super::MCP_TOOL_SESSION_STATS,
            super::MCP_TOOL_SESSION_EXPORT,
            super::MCP_TOOL_AGENT_SPAWN,
            super::MCP_TOOL_AGENT_STATUS,
            super::MCP_TOOL_AGENT_CANCEL,
            super::MCP_TOOL_LEARN_STATUS,
            super::MCP_TOOL_LEARN_FAILURE_PATTERNS,
            super::MCP_TOOL_LEARN_TOOL_RATES,
            super::MCP_TOOL_TRAINING_STATUS,
            super::MCP_TOOL_TRAINING_TRIGGER,
            super::MCP_TOOL_SKILLS_LIST,
            super::MCP_TOOL_SKILLS_SEARCH,
            super::MCP_TOOL_SKILLS_INSTALL,
            super::MCP_TOOL_SKILLS_INFO,
            super::MCP_TOOL_CONTEXT_LEARNING,
            super::MCP_TOOL_CONTEXT_TRAINING,
            super::MCP_TOOL_CONTEXT_CONFIG,
        ];
        for name in &new_names {
            assert!(
                names.contains(*name),
                "reserved names should include '{}'",
                name
            );
        }
        // Total reserved should be at least 36 (16 original + 20 new)
        assert!(
            names.len() >= 36,
            "expected at least 36 reserved names, got {}",
            names.len()
        );
    }

    #[test]
    fn try_dispatch_skill_tool_returns_none_for_nonexistent_dir() {
        let result = super::try_dispatch_skill_tool(
            "some.tool",
            &json!({}),
            std::path::Path::new("/nonexistent/dir"),
        );
        assert!(result.is_none());
    }

    #[test]
    fn try_dispatch_skill_tool_returns_none_for_empty_catalog() {
        let tmp = tempfile::tempdir().expect("tmp dir");
        let result = super::try_dispatch_skill_tool("some.tool", &json!({}), tmp.path());
        assert!(result.is_none());
    }
}
