use super::*;
use async_trait::async_trait;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tau_agent_core::{AgentTool, ToolExecutionResult};
use tau_ai::{Message, ToolDefinition};

#[derive(Clone)]
struct FixturePipelineTool {
    name: &'static str,
    root: PathBuf,
    state_dir: PathBuf,
    read_delay_ms: u64,
    memories: Arc<Mutex<Vec<String>>>,
    jobs: Arc<Mutex<BTreeMap<String, String>>>,
    successful_bash_commands: Arc<BTreeSet<String>>,
}

impl FixturePipelineTool {
    fn is_protected_path(path: &str) -> bool {
        path.starts_with('/') || path.contains("..")
    }

    fn load_navigation_runtime(
        &self,
        session_key: &str,
    ) -> Result<tau_session::SessionRuntime, ToolExecutionResult> {
        let normalized_session_key = sanitize_session_key(session_key);
        let session_path = gateway_session_path(&self.state_dir, normalized_session_key.as_str());
        let store = tau_session::SessionStore::load(&session_path).map_err(|error| {
            ToolExecutionResult::error(json!({
                "code": "session_load_error",
                "message": error.to_string(),
                "session_key": normalized_session_key,
            }))
        })?;
        let resolved_active_head = tau_session::resolve_session_navigation_head(&store)
            .map_err(|error| {
                ToolExecutionResult::error(json!({
                    "code": "navigation_state_error",
                    "message": error.to_string(),
                    "session_key": normalized_session_key,
                }))
            })?
            .or(store.head_id());
        Ok(tau_session::SessionRuntime {
            store,
            active_head: resolved_active_head,
        })
    }
}

#[async_trait]
impl AgentTool for FixturePipelineTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name.to_string(),
            description: format!("fixture pipeline tool {}", self.name),
            parameters: json!({
                "type": "object",
                "properties": {},
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        match self.name {
            "read" => {
                let Some(path) = arguments.get("path").and_then(Value::as_str) else {
                    return ToolExecutionResult::error(json!({"code":"invalid_args"}));
                };
                if self.read_delay_ms > 0 {
                    tokio::time::sleep(Duration::from_millis(self.read_delay_ms)).await;
                }
                let resolved = self.root.join(path);
                match std::fs::read_to_string(&resolved) {
                    Ok(content) => ToolExecutionResult::ok(json!({
                        "path": path,
                        "content": content
                    })),
                    Err(error) => ToolExecutionResult::error(json!({
                        "code": "read_error",
                        "message": error.to_string()
                    })),
                }
            }
            "write" => {
                let Some(path) = arguments.get("path").and_then(Value::as_str) else {
                    return ToolExecutionResult::error(json!({"code":"invalid_args"}));
                };
                if Self::is_protected_path(path) {
                    return ToolExecutionResult::error(json!({
                        "code": "protected_path",
                        "path": path
                    }));
                }
                let content = arguments
                    .get("content")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let resolved = self.root.join(path);
                if let Some(parent) = resolved.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                match std::fs::write(&resolved, content) {
                    Ok(()) => ToolExecutionResult::ok(json!({
                        "path": path,
                        "bytes_written": content.len()
                    })),
                    Err(error) => ToolExecutionResult::error(json!({
                        "code": "write_error",
                        "message": error.to_string()
                    })),
                }
            }
            "edit" => {
                let Some(path) = arguments.get("path").and_then(Value::as_str) else {
                    return ToolExecutionResult::error(json!({"code":"invalid_args"}));
                };
                if Self::is_protected_path(path) {
                    return ToolExecutionResult::error(json!({
                        "code": "protected_path",
                        "path": path
                    }));
                }
                let find = arguments
                    .get("find")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let replace = arguments
                    .get("replace")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let resolved = self.root.join(path);
                let source = match std::fs::read_to_string(&resolved) {
                    Ok(content) => content,
                    Err(error) => {
                        return ToolExecutionResult::error(json!({
                            "code": "read_error",
                            "message": error.to_string()
                        }));
                    }
                };
                let updated = if find.is_empty() {
                    source.clone()
                } else {
                    source.replacen(find, replace, 1)
                };
                match std::fs::write(&resolved, updated.as_str()) {
                    Ok(()) => ToolExecutionResult::ok(json!({
                        "path": path,
                        "updated": true
                    })),
                    Err(error) => ToolExecutionResult::error(json!({
                        "code": "write_error",
                        "message": error.to_string()
                    })),
                }
            }
            "branch" => {
                let source_session_key = arguments
                    .get("source_session_key")
                    .or_else(|| arguments.get("session_key"))
                    .and_then(Value::as_str)
                    .unwrap_or("default");
                let target_session_key = arguments
                    .get("target_session_key")
                    .and_then(Value::as_str)
                    .unwrap_or("branch-tool-target");
                let branch_prompt = arguments
                    .get("prompt")
                    .and_then(Value::as_str)
                    .unwrap_or("branch prompt")
                    .trim()
                    .to_string();
                if branch_prompt.is_empty() {
                    return ToolExecutionResult::error(json!({
                        "code": "invalid_args",
                        "message": "prompt must be non-empty"
                    }));
                }

                let source_session_key = sanitize_session_key(source_session_key);
                let target_session_key = sanitize_session_key(target_session_key);
                let source_path =
                    gateway_session_path(&self.state_dir, source_session_key.as_str());
                let target_path =
                    gateway_session_path(&self.state_dir, target_session_key.as_str());
                let target_navigation_path =
                    tau_session::session_navigation_path_for_session(&target_path);

                let source_store = match tau_session::SessionStore::load(&source_path) {
                    Ok(store) => store,
                    Err(error) => {
                        return ToolExecutionResult::error(json!({
                            "code": "session_load_error",
                            "message": error.to_string(),
                            "session_key": source_session_key,
                        }));
                    }
                };
                let source_head = source_store.head_id();
                let source_lineage = match source_store.lineage_messages(source_head) {
                    Ok(lineage) => lineage,
                    Err(error) => {
                        return ToolExecutionResult::error(json!({
                            "code": "session_lineage_error",
                            "message": error.to_string(),
                            "session_key": source_session_key,
                        }));
                    }
                };
                let copied_messages = source_lineage
                    .into_iter()
                    .filter(|message| message.role != tau_ai::MessageRole::System)
                    .collect::<Vec<_>>();

                if target_path.exists() {
                    let _ = std::fs::remove_file(&target_path);
                }
                if target_navigation_path.exists() {
                    let _ = std::fs::remove_file(&target_navigation_path);
                }

                let mut target_store = match tau_session::SessionStore::load(&target_path) {
                    Ok(store) => store,
                    Err(error) => {
                        return ToolExecutionResult::error(json!({
                            "code": "session_load_error",
                            "message": error.to_string(),
                            "session_key": target_session_key,
                        }));
                    }
                };
                target_store.set_lock_policy(500, 10_000);
                let mut target_head = match target_store.ensure_initialized("You are Tau.") {
                    Ok(head) => head,
                    Err(error) => {
                        return ToolExecutionResult::error(json!({
                            "code": "session_init_error",
                            "message": error.to_string(),
                            "session_key": target_session_key,
                        }));
                    }
                };
                if !copied_messages.is_empty() {
                    target_head = match target_store
                        .append_messages(target_head, copied_messages.as_slice())
                    {
                        Ok(head) => head,
                        Err(error) => {
                            return ToolExecutionResult::error(json!({
                                "code": "session_copy_error",
                                "message": error.to_string(),
                                "session_key": target_session_key,
                            }));
                        }
                    };
                }
                let before_branch_head = target_head;
                target_head = match target_store
                    .append_messages(target_head, &[Message::user(branch_prompt)])
                {
                    Ok(head) => head,
                    Err(error) => {
                        return ToolExecutionResult::error(json!({
                            "code": "session_branch_error",
                            "message": error.to_string(),
                            "session_key": target_session_key,
                        }));
                    }
                };

                let resolved_active_head =
                    tau_session::resolve_session_navigation_head(&target_store)
                        .unwrap_or(None)
                        .or(target_head);
                let mut runtime = tau_session::SessionRuntime {
                    store: target_store,
                    active_head: resolved_active_head,
                };
                if let (Some(previous_head), Some(branch_head)) = (before_branch_head, target_head)
                {
                    if previous_head != branch_head {
                        let _ =
                            tau_session::navigate_session_head(&mut runtime, Some(previous_head));
                        let _ = tau_session::navigate_session_head(&mut runtime, Some(branch_head));
                    }
                }

                ToolExecutionResult::ok(json!({
                    "tool": "branch",
                    "reason_code": "session_branch_created",
                    "source_session_key": source_session_key,
                    "target_session_key": target_session_key,
                    "before_branch_head": before_branch_head,
                    "branch_head": target_head,
                    "entry_count": runtime.store.entries().len(),
                }))
            }
            "undo" => {
                let session_key = arguments
                    .get("session_key")
                    .and_then(Value::as_str)
                    .unwrap_or("default");
                let mut runtime = match self.load_navigation_runtime(session_key) {
                    Ok(runtime) => runtime,
                    Err(result) => return result,
                };
                let transition = match tau_session::undo_session_head(&mut runtime) {
                    Ok(transition) => transition,
                    Err(error) => {
                        return ToolExecutionResult::error(json!({
                            "code": "session_undo_error",
                            "message": error.to_string(),
                            "session_key": sanitize_session_key(session_key),
                        }));
                    }
                };
                ToolExecutionResult::ok(json!({
                    "tool": "undo",
                    "reason_code": if transition.changed { "session_undo_applied" } else { "session_undo_empty_stack" },
                    "session_key": sanitize_session_key(session_key),
                    "changed": transition.changed,
                    "previous_head": transition.previous_head,
                    "active_head": transition.active_head,
                    "undo_depth": transition.undo_depth,
                    "redo_depth": transition.redo_depth,
                }))
            }
            "redo" => {
                let session_key = arguments
                    .get("session_key")
                    .and_then(Value::as_str)
                    .unwrap_or("default");
                let mut runtime = match self.load_navigation_runtime(session_key) {
                    Ok(runtime) => runtime,
                    Err(result) => return result,
                };
                let transition = match tau_session::redo_session_head(&mut runtime) {
                    Ok(transition) => transition,
                    Err(error) => {
                        return ToolExecutionResult::error(json!({
                            "code": "session_redo_error",
                            "message": error.to_string(),
                            "session_key": sanitize_session_key(session_key),
                        }));
                    }
                };
                ToolExecutionResult::ok(json!({
                    "tool": "redo",
                    "reason_code": if transition.changed { "session_redo_applied" } else { "session_redo_empty_stack" },
                    "session_key": sanitize_session_key(session_key),
                    "changed": transition.changed,
                    "previous_head": transition.previous_head,
                    "active_head": transition.active_head,
                    "undo_depth": transition.undo_depth,
                    "redo_depth": transition.redo_depth,
                }))
            }
            "memory_write" => {
                let summary = arguments
                    .get("summary")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                if let Ok(mut memories) = self.memories.lock() {
                    memories.push(summary.clone());
                }
                ToolExecutionResult::ok(json!({
                    "stored": true,
                    "summary": summary
                }))
            }
            "memory_search" => {
                let query = arguments
                    .get("query")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let matches = self
                    .memories
                    .lock()
                    .map(|memories| {
                        memories
                            .iter()
                            .filter(|entry| entry.contains(query.as_str()))
                            .cloned()
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                ToolExecutionResult::ok(json!({
                    "returned": matches.len(),
                    "matches": matches
                }))
            }
            "jobs_create" => {
                let id = arguments
                    .get("job_id")
                    .and_then(Value::as_str)
                    .unwrap_or("job-001")
                    .to_string();
                let name = arguments
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("fixture-job")
                    .to_string();
                if let Ok(mut jobs) = self.jobs.lock() {
                    jobs.insert(id.clone(), "queued".to_string());
                }
                ToolExecutionResult::ok(json!({
                    "job_id": id,
                    "name": name,
                    "status": "queued"
                }))
            }
            "jobs_status" => {
                let id = arguments
                    .get("job_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let status = self
                    .jobs
                    .lock()
                    .ok()
                    .and_then(|jobs| jobs.get(id.as_str()).cloned())
                    .unwrap_or_else(|| "not_found".to_string());
                ToolExecutionResult::ok(json!({
                    "job_id": id,
                    "status": status
                }))
            }
            "http" => ToolExecutionResult::ok(json!({
                "status": 200,
                "body": "fixture-http-ok"
            })),
            "bash" => {
                let command = arguments
                    .get("command")
                    .or_else(|| arguments.get("cmd"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                if self.successful_bash_commands.contains(command.as_str()) {
                    ToolExecutionResult::ok(json!({
                        "command": command,
                        "code": "ok",
                        "stdout": "fixture bash succeeded"
                    }))
                } else {
                    ToolExecutionResult::error(json!({
                        "code": "policy_blocked",
                        "message": "bash is disabled by fixture policy"
                    }))
                }
            }
            _ => ToolExecutionResult::error(json!({"code":"unknown_tool"})),
        }
    }
}

#[derive(Clone)]
pub(super) struct FixturePipelineToolRegistrar {
    root: PathBuf,
    state_dir: PathBuf,
    read_delay_ms: u64,
    memories: Arc<Mutex<Vec<String>>>,
    jobs: Arc<Mutex<BTreeMap<String, String>>>,
    successful_bash_commands: Arc<BTreeSet<String>>,
}

impl FixturePipelineToolRegistrar {
    pub(super) fn new(root: PathBuf, state_dir: PathBuf) -> Self {
        Self {
            root,
            state_dir,
            read_delay_ms: 0,
            memories: Arc::new(Mutex::new(Vec::new())),
            jobs: Arc::new(Mutex::new(BTreeMap::new())),
            successful_bash_commands: Arc::new(BTreeSet::new()),
        }
    }

    pub(super) fn with_read_delay_ms(mut self, read_delay_ms: u64) -> Self {
        self.read_delay_ms = read_delay_ms;
        self
    }

    pub(super) fn with_successful_bash_commands<I, S>(mut self, commands: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.successful_bash_commands = Arc::new(
            commands
                .into_iter()
                .map(Into::into)
                .collect::<BTreeSet<_>>(),
        );
        self
    }
}

impl GatewayToolRegistrar for FixturePipelineToolRegistrar {
    fn register(&self, agent: &mut Agent) {
        for tool_name in [
            "read",
            "write",
            "edit",
            "memory_write",
            "memory_search",
            "http",
            "bash",
            "jobs_create",
            "jobs_status",
            "branch",
            "undo",
            "redo",
        ] {
            agent.register_tool(FixturePipelineTool {
                name: tool_name,
                root: self.root.clone(),
                state_dir: self.state_dir.clone(),
                read_delay_ms: self.read_delay_ms,
                memories: Arc::clone(&self.memories),
                jobs: Arc::clone(&self.jobs),
                successful_bash_commands: Arc::clone(&self.successful_bash_commands),
            });
        }
    }
}
