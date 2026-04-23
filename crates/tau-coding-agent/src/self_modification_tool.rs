//! `AgentTool` implementation that lets an autonomous agent request a
//! self-modification *dry-run* via the standard tool-call path.
//!
//! ## Safety model
//!
//! This tool is **dry-run only** — it never applies changes, never writes
//! outside `<workspace>/.tau/self-mod-worktrees/`, and refuses to run at all
//! unless explicitly enabled by the operator via the
//! `TAU_AUTONOMOUS_SELF_MOD=1` environment variable.
//!
//! When the env gate is off, the tool still appears in the agent's tool list
//! but every `execute()` call returns an error result with
//! `reason_code: "autonomous_self_mod_disabled"`. That design choice is
//! deliberate: hiding the tool would make the failure invisible to the agent
//! and encourage it to invent workarounds. Visible-but-refused is safer.
//!
//! ## Inputs
//!
//! ```json
//! {
//!   "target":              "skills/foo/manifest.toml",   // required
//!   "proposal_id":         "optional-explicit-id",        // optional
//!   "workspace_root":      "/path/to/workspace"           // optional, defaults to CWD
//! }
//! ```
//!
//! ## Outputs
//!
//! The tool returns the full `SelfModificationResult` JSON on success, and
//! an error payload with `reason_code` + `message` on any failure (env gate,
//! validation, pipeline error).

use std::path::PathBuf;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tau_agent_core::{AgentTool, ToolExecutionResult};
use tau_ai::ToolDefinition;
use tracing::{info, warn};

use crate::self_modification_pipeline::run_dry_run_pipeline;
use crate::self_modification_runtime::SelfModificationConfig;

/// Environment flag operators set to `1` to enable autonomous dry-runs.
pub const AUTONOMOUS_SELF_MOD_ENV: &str = "TAU_AUTONOMOUS_SELF_MOD";

/// Canonical tool name exposed to the agent.
pub const TOOL_NAME: &str = "self_modification_propose";

#[derive(Debug, Deserialize)]
struct ProposeArgs {
    target: String,
    #[serde(default)]
    proposal_id: Option<String>,
    #[serde(default)]
    workspace_root: Option<PathBuf>,
}

/// Default `AgentTool` implementation backed by the production pipeline.
pub struct SelfModificationProposeTool;

impl SelfModificationProposeTool {
    pub fn new() -> Self {
        Self
    }

    fn autonomous_enabled() -> bool {
        std::env::var(AUTONOMOUS_SELF_MOD_ENV)
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }
}

impl Default for SelfModificationProposeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentTool for SelfModificationProposeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: TOOL_NAME.to_string(),
            description: "Propose a dry-run self-modification. Never mutates \
                          source. Returns a SelfModificationResult JSON with \
                          policy verdict. Requires TAU_AUTONOMOUS_SELF_MOD=1 \
                          to be enabled."
                .to_string(),
            parameters: json!({
                "type": "object",
                "required": ["target"],
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "Workspace-relative path the proposal would modify."
                    },
                    "proposal_id": {
                        "type": "string",
                        "description": "Optional deterministic id. ASCII [A-Za-z0-9._-] only."
                    },
                    "workspace_root": {
                        "type": "string",
                        "description": "Absolute workspace root. Defaults to current working dir."
                    }
                }
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        if !Self::autonomous_enabled() {
            warn!(
                tool = TOOL_NAME,
                env = AUTONOMOUS_SELF_MOD_ENV,
                "autonomous self-modification invoked while disabled; refusing"
            );
            return ToolExecutionResult::error(json!({
                "reason_code": "autonomous_self_mod_disabled",
                "message": format!(
                    "autonomous self-modification is disabled; set {AUTONOMOUS_SELF_MOD_ENV}=1 \
                     to enable dry-runs"
                ),
            }));
        }

        let args: ProposeArgs = match serde_json::from_value(arguments) {
            Ok(a) => a,
            Err(err) => {
                return ToolExecutionResult::error(json!({
                    "reason_code": "invalid_arguments",
                    "message": format!("failed to parse tool arguments: {err}"),
                }));
            }
        };

        let workspace_root = args
            .workspace_root
            .unwrap_or_else(|| PathBuf::from("."));
        let config = SelfModificationConfig::default();

        info!(
            tool = TOOL_NAME,
            target = %args.target,
            proposal_id_override = args.proposal_id.as_deref().unwrap_or("<none>"),
            "autonomous self-modification dry-run starting"
        );

        match run_dry_run_pipeline(
            &workspace_root,
            &args.target,
            args.proposal_id.as_deref(),
            &config,
        ) {
            Ok(result) => match serde_json::to_value(&result) {
                Ok(value) => ToolExecutionResult::ok(value),
                Err(err) => ToolExecutionResult::error(json!({
                    "reason_code": "serialization_failed",
                    "message": format!("failed to serialize result: {err}"),
                })),
            },
            Err(err) => {
                warn!(
                    tool = TOOL_NAME,
                    target = %args.target,
                    error = %err,
                    "autonomous self-modification dry-run failed"
                );
                ToolExecutionResult::error(json!({
                    "reason_code": "pipeline_failed",
                    "message": format!("{err:#}"),
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::runtime::Builder;

    // Tests mutate process env; serialize them via the crate-level lock in
    // `lib.rs` so tests across modules (e.g. self_modification_synthesis_tool)
    // cannot interleave env mutations and produce a torn state.
    use crate::AUTONOMOUS_SELF_MOD_ENV_LOCK as ENV_LOCK;

    fn run_with_env<F, Fut, R>(value: Option<&str>, body: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var(AUTONOMOUS_SELF_MOD_ENV).ok();
        // SAFETY: serialized via ENV_LOCK; no concurrent env mutation inside
        // this test binary can observe a torn state. Other binaries in the
        // workspace do not touch this env var.
        unsafe {
            match value {
                Some(v) => std::env::set_var(AUTONOMOUS_SELF_MOD_ENV, v),
                None => std::env::remove_var(AUTONOMOUS_SELF_MOD_ENV),
            }
        }
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime builds");
        let out = rt.block_on(body());
        unsafe {
            match prev {
                Some(v) => std::env::set_var(AUTONOMOUS_SELF_MOD_ENV, v),
                None => std::env::remove_var(AUTONOMOUS_SELF_MOD_ENV),
            }
        }
        out
    }

    #[test]
    fn definition_uses_canonical_tool_name() {
        let tool = SelfModificationProposeTool::new();
        let def = tool.definition();
        assert_eq!(def.name, TOOL_NAME);
        assert!(def.parameters.get("required").is_some());
    }

    #[test]
    fn execute_refuses_when_env_gate_is_off() {
        let tool = SelfModificationProposeTool::new();
        let result = run_with_env(None, || async {
            tool.execute(json!({ "target": "skills/foo.md" })).await
        });
        assert!(result.is_error);
        assert_eq!(
            result.content.get("reason_code").and_then(|v| v.as_str()),
            Some("autonomous_self_mod_disabled")
        );
    }

    #[test]
    fn execute_refuses_when_env_gate_is_literal_zero() {
        let tool = SelfModificationProposeTool::new();
        let result = run_with_env(Some("0"), || async {
            tool.execute(json!({ "target": "skills/foo.md" })).await
        });
        assert!(result.is_error);
        assert_eq!(
            result.content.get("reason_code").and_then(|v| v.as_str()),
            Some("autonomous_self_mod_disabled")
        );
    }

    #[test]
    fn execute_rejects_invalid_arguments_when_enabled() {
        let tool = SelfModificationProposeTool::new();
        let result = run_with_env(Some("1"), || async {
            tool.execute(json!({ "nope": 1 })).await
        });
        assert!(result.is_error);
        assert_eq!(
            result.content.get("reason_code").and_then(|v| v.as_str()),
            Some("invalid_arguments")
        );
    }

    #[test]
    fn execute_runs_pipeline_when_enabled_for_allowed_target() {
        let workspace = tempfile::TempDir::new().unwrap();
        let tool = SelfModificationProposeTool::new();
        let workspace_str = workspace.path().to_string_lossy().to_string();

        let result = run_with_env(Some("1"), || async {
            tool.execute(json!({
                "target": "skills/foo/manifest.toml",
                "workspace_root": workspace_str,
            }))
            .await
        });

        assert!(
            !result.is_error,
            "expected ok, got error: {}",
            result.as_text()
        );
        assert_eq!(result.content["applied"], Value::Bool(false));
        assert_eq!(
            result.content["safety_evaluation"]["allowed"],
            Value::Bool(true)
        );
    }

    #[test]
    fn execute_surfaces_policy_denial_for_source_target() {
        let workspace = tempfile::TempDir::new().unwrap();
        let tool = SelfModificationProposeTool::new();
        let workspace_str = workspace.path().to_string_lossy().to_string();

        let result = run_with_env(Some("1"), || async {
            tool.execute(json!({
                "target": "crates/tau-ops/src/main.rs",
                "workspace_root": workspace_str,
            }))
            .await
        });

        assert!(!result.is_error);
        assert_eq!(
            result.content["safety_evaluation"]["allowed"],
            Value::Bool(false)
        );
        let blocked_by = result.content["safety_evaluation"]["blocked_by"]
            .as_array()
            .expect("blocked_by is an array");
        assert!(
            blocked_by
                .iter()
                .any(|v| v.as_str() == Some("auto_apply_source_disabled")),
            "expected auto_apply_source_disabled; got {blocked_by:?}"
        );
    }
}
