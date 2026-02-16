//! Background jobs tool definitions and execution wiring.

use super::*;

/// Public struct `JobsCreateTool` used across Tau components.
pub struct JobsCreateTool {
    policy: Arc<ToolPolicy>,
}

impl JobsCreateTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AgentTool for JobsCreateTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "jobs_create".to_string(),
            description:
                "Queue a background job command for asynchronous execution and persisted tracking"
                    .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Executable command (e.g. cargo, sh, cmd)"
                    },
                    "args": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional command arguments"
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Optional working directory constrained to allowed roots"
                    },
                    "timeout_ms": {
                        "type": "integer",
                        "description": format!(
                            "Optional timeout in milliseconds (1..={})",
                            self.policy.jobs_max_timeout_ms
                        )
                    },
                    "env": {
                        "type": "object",
                        "description": "Optional environment map. Values must be strings.",
                        "additionalProperties": { "type": "string" }
                    },
                    "channel_transport": {
                        "type": "string",
                        "description": "Optional channel transport for channel-store trace emission"
                    },
                    "channel_id": {
                        "type": "string",
                        "description": "Optional channel id for channel-store trace emission"
                    },
                    "session_path": {
                        "type": "string",
                        "description": "Optional session path override for trace message append"
                    }
                },
                "required": ["command"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        if !self.policy.jobs_enabled {
            return ToolExecutionResult::error(json!({
                "tool": "jobs_create",
                "reason_code": "jobs_disabled",
                "error": "background jobs tooling is disabled by policy",
            }));
        }

        let command = match required_string(&arguments, "command") {
            Ok(command) => command.trim().to_string(),
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_create",
                    "reason_code": "jobs_invalid_arguments",
                    "error": error,
                }))
            }
        };
        if command.is_empty() {
            return ToolExecutionResult::error(json!({
                "tool": "jobs_create",
                "reason_code": "jobs_invalid_arguments",
                "error": "command must not be empty",
            }));
        }

        let args = match optional_string_array_unbounded(&arguments, "args") {
            Ok(args) => args,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_create",
                    "reason_code": "jobs_invalid_arguments",
                    "error": error,
                }))
            }
        };

        let timeout_ms = match optional_positive_u64(&arguments, "timeout_ms") {
            Ok(timeout_ms) => timeout_ms,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_create",
                    "reason_code": "jobs_invalid_arguments",
                    "error": error,
                }))
            }
        };

        if let Some(timeout_ms) = timeout_ms {
            if timeout_ms > self.policy.jobs_max_timeout_ms {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_create",
                    "reason_code": "jobs_invalid_arguments",
                    "error": format!(
                        "timeout_ms exceeds policy maximum {}",
                        self.policy.jobs_max_timeout_ms
                    ),
                }));
            }
        }

        let env = match optional_string_map(&arguments, "env") {
            Ok(env) => env,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_create",
                    "reason_code": "jobs_invalid_arguments",
                    "error": error,
                }))
            }
        };

        let cwd = match optional_string(&arguments, "cwd") {
            Ok(cwd) => cwd,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_create",
                    "reason_code": "jobs_invalid_arguments",
                    "error": error,
                }))
            }
        };
        let cwd = match cwd {
            Some(cwd) => match resolve_and_validate_path(&cwd, &self.policy, PathMode::Read) {
                Ok(cwd) => Some(cwd),
                Err(error) => {
                    return ToolExecutionResult::error(json!({
                        "tool": "jobs_create",
                        "reason_code": "jobs_invalid_arguments",
                        "error": error,
                    }))
                }
            },
            None => None,
        };

        let session_path_override = match optional_string(&arguments, "session_path") {
            Ok(path) => path,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_create",
                    "reason_code": "jobs_invalid_arguments",
                    "error": error,
                }))
            }
        };
        let session_path = match session_path_override {
            Some(path) => match resolve_and_validate_path(&path, &self.policy, PathMode::Write) {
                Ok(path) => Some(path),
                Err(error) => {
                    return ToolExecutionResult::error(json!({
                        "tool": "jobs_create",
                        "reason_code": "jobs_invalid_arguments",
                        "error": error,
                    }))
                }
            },
            None => self.policy.jobs_default_session_path.clone(),
        };

        let channel_transport = match optional_string(&arguments, "channel_transport") {
            Ok(value) => value,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_create",
                    "reason_code": "jobs_invalid_arguments",
                    "error": error,
                }))
            }
        };
        let channel_id = match optional_string(&arguments, "channel_id") {
            Ok(value) => value,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_create",
                    "reason_code": "jobs_invalid_arguments",
                    "error": error,
                }))
            }
        };
        if (channel_transport.is_some() && channel_id.is_none())
            || (channel_transport.is_none() && channel_id.is_some())
        {
            return ToolExecutionResult::error(json!({
                "tool": "jobs_create",
                "reason_code": "jobs_invalid_arguments",
                "error": "channel_transport and channel_id must both be provided when tracing to channel-store",
            }));
        }

        if let Some(rbac_result) = evaluate_tool_rbac_gate(
            self.policy.rbac_principal.as_deref(),
            "jobs_create",
            self.policy.rbac_policy_path.as_deref(),
            json!({
                "command": command.clone(),
                "args_count": args.len(),
                "cwd": cwd.as_ref().map(|path| path.display().to_string()),
                "timeout_ms": timeout_ms.unwrap_or(self.policy.jobs_default_timeout_ms),
            }),
        ) {
            return rbac_result;
        }
        if let Some(approval_result) = evaluate_tool_approval_gate(ApprovalAction::ToolBash {
            command: command.clone(),
            cwd: cwd.as_ref().map(|path| path.display().to_string()),
        }) {
            return approval_result;
        }
        if let Some(rate_limit_result) = evaluate_tool_rate_limit_gate(
            &self.policy,
            "jobs_create",
            json!({
                "command": command.clone(),
                "args_count": args.len(),
                "timeout_ms": timeout_ms.unwrap_or(self.policy.jobs_default_timeout_ms),
            }),
        ) {
            return rate_limit_result;
        }

        let runtime = match resolve_background_job_runtime(&self.policy) {
            Ok(runtime) => runtime,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_create",
                    "reason_code": "jobs_runtime_unavailable",
                    "error": error,
                }))
            }
        };

        let request = BackgroundJobCreateRequest {
            command,
            args,
            env,
            cwd,
            timeout_ms,
            trace: BackgroundJobTraceContext {
                channel_store_root: Some(self.policy.jobs_channel_store_root.clone()),
                channel_transport,
                channel_id,
                session_path,
            },
        };
        let record = match runtime.create_job(request).await {
            Ok(record) => record,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_create",
                    "reason_code": "jobs_runtime_error",
                    "error": error.to_string(),
                }))
            }
        };
        let health = runtime.inspect_health().await;

        ToolExecutionResult::ok(json!({
            "tool": "jobs_create",
            "reason_code": "job_queued",
            "job": background_job_record_payload(&record, false),
            "health": background_job_health_payload(&health),
        }))
    }
}

/// Public struct `JobsListTool` used across Tau components.
pub struct JobsListTool {
    policy: Arc<ToolPolicy>,
}

impl JobsListTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AgentTool for JobsListTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "jobs_list".to_string(),
            description:
                "List persisted background jobs with optional status filtering and runtime health"
                    .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": format!(
                            "Max jobs returned (default {}, max {})",
                            self.policy.jobs_list_default_limit,
                            self.policy.jobs_list_max_limit
                        )
                    },
                    "status": {
                        "type": "string",
                        "description": "Optional status filter: queued, running, succeeded, failed, cancelled, terminal"
                    }
                },
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        if !self.policy.jobs_enabled {
            return ToolExecutionResult::error(json!({
                "tool": "jobs_list",
                "reason_code": "jobs_disabled",
                "error": "background jobs tooling is disabled by policy",
            }));
        }

        let limit = match optional_usize(
            &arguments,
            "limit",
            self.policy.jobs_list_default_limit,
            self.policy
                .jobs_list_max_limit
                .max(self.policy.jobs_list_default_limit),
        ) {
            Ok(limit) => limit,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_list",
                    "reason_code": "jobs_invalid_arguments",
                    "error": error,
                }))
            }
        };

        let status = match optional_string(&arguments, "status") {
            Ok(status) => status,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_list",
                    "reason_code": "jobs_invalid_arguments",
                    "error": error,
                }))
            }
        };
        let status_filter = match status.as_deref() {
            Some(raw) => match BackgroundJobStatusFilter::parse(raw) {
                Some(filter) => Some(filter),
                None => {
                    return ToolExecutionResult::error(json!({
                        "tool": "jobs_list",
                        "reason_code": "jobs_invalid_arguments",
                        "error": "status must be one of queued,running,succeeded,failed,cancelled,terminal",
                    }))
                }
            },
            None => None,
        };

        let runtime = match resolve_background_job_runtime(&self.policy) {
            Ok(runtime) => runtime,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_list",
                    "reason_code": "jobs_runtime_unavailable",
                    "error": error,
                }))
            }
        };
        let jobs = match runtime.list_jobs(limit, status_filter).await {
            Ok(jobs) => jobs,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_list",
                    "reason_code": "jobs_runtime_error",
                    "error": error.to_string(),
                }))
            }
        };
        let health = runtime.inspect_health().await;
        ToolExecutionResult::ok(json!({
            "tool": "jobs_list",
            "reason_code": "jobs_list_ok",
            "limit": limit,
            "status": status,
            "count": jobs.len(),
            "jobs": jobs.iter().map(|job| background_job_record_payload(job, false)).collect::<Vec<_>>(),
            "health": background_job_health_payload(&health),
        }))
    }
}

/// Public struct `JobsStatusTool` used across Tau components.
pub struct JobsStatusTool {
    policy: Arc<ToolPolicy>,
}

impl JobsStatusTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AgentTool for JobsStatusTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "jobs_status".to_string(),
            description: "Fetch one background job manifest and bounded stdout/stderr previews"
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "job_id": {
                        "type": "string",
                        "description": "Background job id returned by jobs_create"
                    },
                    "output_preview_bytes": {
                        "type": "integer",
                        "description": format!(
                            "Optional output preview byte cap (default {}, max {})",
                            JOBS_OUTPUT_PREVIEW_DEFAULT_BYTES,
                            JOBS_OUTPUT_PREVIEW_MAX_BYTES
                        )
                    }
                },
                "required": ["job_id"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        if !self.policy.jobs_enabled {
            return ToolExecutionResult::error(json!({
                "tool": "jobs_status",
                "reason_code": "jobs_disabled",
                "error": "background jobs tooling is disabled by policy",
            }));
        }
        let job_id = match required_string(&arguments, "job_id") {
            Ok(job_id) => job_id.trim().to_string(),
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_status",
                    "reason_code": "jobs_invalid_arguments",
                    "error": error,
                }))
            }
        };
        if job_id.is_empty() {
            return ToolExecutionResult::error(json!({
                "tool": "jobs_status",
                "reason_code": "jobs_invalid_arguments",
                "error": "job_id must not be empty",
            }));
        }
        let preview_bytes = match optional_usize(
            &arguments,
            "output_preview_bytes",
            JOBS_OUTPUT_PREVIEW_DEFAULT_BYTES,
            JOBS_OUTPUT_PREVIEW_MAX_BYTES,
        ) {
            Ok(preview_bytes) => preview_bytes,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_status",
                    "reason_code": "jobs_invalid_arguments",
                    "error": error,
                }))
            }
        };

        let runtime = match resolve_background_job_runtime(&self.policy) {
            Ok(runtime) => runtime,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_status",
                    "reason_code": "jobs_runtime_unavailable",
                    "error": error,
                }))
            }
        };
        let Some(record) = (match runtime.get_job(job_id.as_str()).await {
            Ok(record) => record,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_status",
                    "reason_code": "jobs_runtime_error",
                    "error": error.to_string(),
                }))
            }
        }) else {
            return ToolExecutionResult::error(json!({
                "tool": "jobs_status",
                "reason_code": "job_not_found",
                "job_id": job_id,
                "error": "background job id does not exist",
            }));
        };

        let stdout_preview =
            read_output_preview(record.stdout_path.as_path(), preview_bytes).unwrap_or_default();
        let stderr_preview =
            read_output_preview(record.stderr_path.as_path(), preview_bytes).unwrap_or_default();

        ToolExecutionResult::ok(json!({
            "tool": "jobs_status",
            "reason_code": "jobs_status_ok",
            "job": background_job_record_payload(&record, true),
            "stdout_preview": stdout_preview,
            "stderr_preview": stderr_preview,
            "output_preview_bytes": preview_bytes,
        }))
    }
}

/// Public struct `JobsCancelTool` used across Tau components.
pub struct JobsCancelTool {
    policy: Arc<ToolPolicy>,
}

impl JobsCancelTool {
    pub fn new(policy: Arc<ToolPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AgentTool for JobsCancelTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "jobs_cancel".to_string(),
            description: "Cancel one background job by id".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "job_id": {
                        "type": "string",
                        "description": "Background job id returned by jobs_create"
                    }
                },
                "required": ["job_id"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        if !self.policy.jobs_enabled {
            return ToolExecutionResult::error(json!({
                "tool": "jobs_cancel",
                "reason_code": "jobs_disabled",
                "error": "background jobs tooling is disabled by policy",
            }));
        }
        let job_id = match required_string(&arguments, "job_id") {
            Ok(job_id) => job_id.trim().to_string(),
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_cancel",
                    "reason_code": "jobs_invalid_arguments",
                    "error": error,
                }))
            }
        };
        if job_id.is_empty() {
            return ToolExecutionResult::error(json!({
                "tool": "jobs_cancel",
                "reason_code": "jobs_invalid_arguments",
                "error": "job_id must not be empty",
            }));
        }

        if let Some(rbac_result) = evaluate_tool_rbac_gate(
            self.policy.rbac_principal.as_deref(),
            "jobs_cancel",
            self.policy.rbac_policy_path.as_deref(),
            json!({ "job_id": job_id }),
        ) {
            return rbac_result;
        }
        if let Some(rate_limit_result) =
            evaluate_tool_rate_limit_gate(&self.policy, "jobs_cancel", json!({ "job_id": job_id }))
        {
            return rate_limit_result;
        }

        let runtime = match resolve_background_job_runtime(&self.policy) {
            Ok(runtime) => runtime,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_cancel",
                    "reason_code": "jobs_runtime_unavailable",
                    "error": error,
                }))
            }
        };

        let Some(record) = (match runtime.cancel_job(job_id.as_str()).await {
            Ok(record) => record,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "tool": "jobs_cancel",
                    "reason_code": "jobs_runtime_error",
                    "error": error.to_string(),
                }))
            }
        }) else {
            return ToolExecutionResult::error(json!({
                "tool": "jobs_cancel",
                "reason_code": "job_not_found",
                "job_id": job_id,
                "error": "background job id does not exist",
            }));
        };
        let health = runtime.inspect_health().await;
        ToolExecutionResult::ok(json!({
            "tool": "jobs_cancel",
            "reason_code": "jobs_cancel_ok",
            "job": background_job_record_payload(&record, false),
            "health": background_job_health_payload(&health),
        }))
    }
}
