//! `AgentTool` implementation that synthesizes a self-modification proposal
//! from a natural-language intent via an `LlmClient`.
//!
//! ## Design
//!
//! This tool is the *synthesis* half of the two-tool autonomous-self-modification
//! pair. It takes a natural-language intent (e.g. "add tracing to the
//! policy-denial paths") and returns a structured proposal that can be fed
//! to [`crate::self_modification_tool::SelfModificationProposeTool`] for a
//! dry-run. This tool **never** invokes the dry-run pipeline itself — it is
//! a pure synthesis step. The LLM (or another agent) decides whether to
//! chain its output into the propose tool.
//!
//! ## Safety model — four independent gates
//!
//! 1. **Policy flag** ([`tau_tools::tools::ToolPolicy::self_modification_synthesize_enabled`]).
//!    When `false`, the tool is not registered on the agent. Model cannot see it.
//! 2. **Env gate** (`TAU_AUTONOMOUS_SELF_MOD=1`). Same gate as the propose
//!    tool. When unset, `execute()` refuses with
//!    `reason_code: "autonomous_self_mod_disabled"` before calling the LLM.
//! 3. **Output validation**. The LLM's JSON output is parsed through a strict
//!    schema; any `target` path is rejected if it contains `..`, starts with
//!    `/`, or fails the ASCII allowlist.
//! 4. **Dry-run boundary**. Even if the synthesized proposal is garbage, it
//!    only becomes dangerous if the LLM chains it through the propose tool,
//!    which is itself dry-run-only and gated by the same env flag.
//!
//! ## Inputs
//!
//! ```json
//! {
//!   "intent":          "Add tracing to the policy-denial paths",   // required
//!   "workspace_root":  "/path/to/workspace"                        // optional
//! }
//! ```
//!
//! ## Outputs
//!
//! On success:
//! ```json
//! {
//!   "target":           "crates/tau-coding-agent/src/self_modification_runtime.rs",
//!   "change_type":      "source",
//!   "rationale":        "...",
//!   "proposed_diff":    "...",           // may be null
//!   "policy_projected": "would_be_blocked"
//! }
//! ```
//!
//! On failure, an error payload with `reason_code` and `message`.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tau_agent_core::{AgentTool, ToolExecutionResult};
use tau_ai::{ChatRequest, LlmClient, Message, ToolDefinition};
use tracing::{info, warn};

use crate::self_modification_runtime::{
    can_auto_apply, classify_modification_target, SelfModificationConfig,
};
use crate::self_modification_tool::AUTONOMOUS_SELF_MOD_ENV;

/// Canonical tool name exposed to the agent.
pub const TOOL_NAME: &str = "self_modification_synthesize";

/// System prompt asking the LLM for strict JSON output.
const SYSTEM_PROMPT: &str = r#"You propose code changes as JSON objects. Given an INTENT describing a desired change, return a single JSON object with these fields:

{
  "target": "<workspace-relative path the change would modify>",
  "change_type": "skill" | "prompt" | "config" | "source" | "other",
  "rationale": "<one or two sentences justifying the change>",
  "proposed_diff": "<unified diff text, or null>"
}

Rules:
- `target` MUST be a forward-slash, workspace-relative path. Never absolute. Never contain `..`. ASCII [A-Za-z0-9/._-] only.
- `change_type` MUST be one of the five literals above. Use "source" for any .rs / .py / .ts / compiled-language file.
- `proposed_diff` SHOULD be provided when you have enough context to draft one; otherwise null.
- Respond with ONLY the JSON object. No prose before or after. No markdown fences."#;

/// Maximum bytes accepted from the model before we assume it's run away.
const MAX_LLM_OUTPUT_BYTES: usize = 32 * 1024;

/// Hard cap on intent length — prevents prompt-injection bombs.
const MAX_INTENT_CHARS: usize = 4096;

#[derive(Debug, Deserialize)]
struct SynthesizeArgs {
    intent: String,
    #[serde(default)]
    workspace_root: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LlmProposal {
    target: String,
    change_type: String,
    rationale: String,
    #[serde(default)]
    proposed_diff: Option<String>,
}

/// Synthesis tool backed by an `LlmClient`.
pub struct SelfModificationSynthesizeTool {
    client: Arc<dyn LlmClient>,
    model: String,
}

impl SelfModificationSynthesizeTool {
    pub fn new(client: Arc<dyn LlmClient>, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
        }
    }

    fn autonomous_enabled() -> bool {
        std::env::var(AUTONOMOUS_SELF_MOD_ENV)
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }
}

fn validate_target_path(target: &str) -> Result<(), String> {
    if target.is_empty() {
        return Err("target path is empty".to_string());
    }
    if target.len() > 4096 {
        return Err("target path exceeds 4096 bytes".to_string());
    }
    if target.starts_with('/') || target.starts_with('\\') {
        return Err("target path must be workspace-relative, not absolute".to_string());
    }
    for segment in target.split(['/', '\\']) {
        if segment == ".." {
            return Err("target path contains parent-directory traversal".to_string());
        }
    }
    for ch in target.chars() {
        let ok = ch.is_ascii_alphanumeric()
            || matches!(ch, '/' | '.' | '_' | '-' | '\\');
        if !ok {
            return Err(format!(
                "target path contains disallowed character: {ch:?}"
            ));
        }
    }
    Ok(())
}

fn parse_llm_json(raw: &str) -> Result<LlmProposal, String> {
    // Strip leading/trailing whitespace and tolerate fenced code blocks even
    // though the system prompt forbids them — defensive parsing.
    let trimmed = raw.trim();
    let stripped = if let Some(rest) = trimmed.strip_prefix("```json") {
        rest.trim_end_matches("```").trim()
    } else if let Some(rest) = trimmed.strip_prefix("```") {
        rest.trim_end_matches("```").trim()
    } else {
        trimmed
    };
    serde_json::from_str::<LlmProposal>(stripped)
        .map_err(|e| format!("LLM output is not a valid proposal JSON: {e}"))
}

fn classify_for_projection(
    _workspace_root: &std::path::Path,
    target: &str,
    config: &SelfModificationConfig,
) -> &'static str {
    let classified = classify_modification_target(target);
    if can_auto_apply(config, &classified) {
        "auto_apply"
    } else {
        // Covers both ModificationTarget::Other and the auto-apply-disabled
        // categories (source, config, skill when the corresponding flag is
        // false). The caller gets a single unambiguous signal.
        "would_be_blocked"
    }
}

#[async_trait]
impl AgentTool for SelfModificationSynthesizeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: TOOL_NAME.to_string(),
            description: "Synthesize a structured self-modification proposal from a \
                          natural-language intent. Returns { target, change_type, \
                          rationale, proposed_diff, policy_projected }. This tool \
                          does NOT apply any change; pair with \
                          self_modification_propose to dry-run it. Requires \
                          TAU_AUTONOMOUS_SELF_MOD=1 to be enabled."
                .to_string(),
            parameters: json!({
                "type": "object",
                "required": ["intent"],
                "properties": {
                    "intent": {
                        "type": "string",
                        "description": "Natural-language description of the desired change. Max 4096 chars."
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
                "autonomous synthesis invoked while disabled; refusing"
            );
            return ToolExecutionResult::error(json!({
                "reason_code": "autonomous_self_mod_disabled",
                "message": format!(
                    "autonomous self-modification is disabled; set {AUTONOMOUS_SELF_MOD_ENV}=1 \
                     to enable synthesis"
                ),
            }));
        }

        let args: SynthesizeArgs = match serde_json::from_value(arguments) {
            Ok(a) => a,
            Err(err) => {
                return ToolExecutionResult::error(json!({
                    "reason_code": "invalid_arguments",
                    "message": format!("failed to parse tool arguments: {err}"),
                }));
            }
        };

        if args.intent.chars().count() > MAX_INTENT_CHARS {
            return ToolExecutionResult::error(json!({
                "reason_code": "intent_too_large",
                "message": format!("intent must be ≤ {MAX_INTENT_CHARS} characters"),
            }));
        }
        if args.intent.trim().is_empty() {
            return ToolExecutionResult::error(json!({
                "reason_code": "invalid_arguments",
                "message": "intent must not be empty",
            }));
        }

        let workspace_root = args
            .workspace_root
            .unwrap_or_else(|| PathBuf::from("."));

        info!(
            tool = TOOL_NAME,
            intent_chars = args.intent.chars().count(),
            model = %self.model,
            "autonomous self-modification synthesis starting"
        );

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                Message::system(SYSTEM_PROMPT),
                Message::user(&args.intent),
            ],
            tools: Vec::new(),
            tool_choice: None,
            json_mode: true,
            max_tokens: Some(2048),
            temperature: Some(0.0),
            prompt_cache: Default::default(),
        };

        let response = match self.client.complete(request).await {
            Ok(r) => r,
            Err(err) => {
                warn!(tool = TOOL_NAME, error = %err, "LLM synthesis call failed");
                return ToolExecutionResult::error(json!({
                    "reason_code": "llm_call_failed",
                    "message": format!("{err}"),
                }));
            }
        };

        let text = response.message.text_content();
        if text.len() > MAX_LLM_OUTPUT_BYTES {
            return ToolExecutionResult::error(json!({
                "reason_code": "llm_output_too_large",
                "message": format!(
                    "LLM output exceeds {MAX_LLM_OUTPUT_BYTES} bytes ({} bytes received)",
                    text.len()
                ),
            }));
        }

        let proposal = match parse_llm_json(&text) {
            Ok(p) => p,
            Err(err) => {
                warn!(tool = TOOL_NAME, error = %err, "LLM output rejected");
                return ToolExecutionResult::error(json!({
                    "reason_code": "llm_output_malformed",
                    "message": err,
                }));
            }
        };

        if let Err(err) = validate_target_path(&proposal.target) {
            warn!(
                tool = TOOL_NAME,
                target = %proposal.target,
                error = %err,
                "synthesized target rejected"
            );
            return ToolExecutionResult::error(json!({
                "reason_code": "invalid_target_path",
                "message": err,
            }));
        }

        let allowed_change_types = ["skill", "prompt", "config", "source", "other"];
        if !allowed_change_types.contains(&proposal.change_type.as_str()) {
            return ToolExecutionResult::error(json!({
                "reason_code": "invalid_change_type",
                "message": format!(
                    "change_type {:?} is not one of {:?}",
                    proposal.change_type, allowed_change_types
                ),
            }));
        }

        let config = SelfModificationConfig::default();
        let policy_projected =
            classify_for_projection(&workspace_root, &proposal.target, &config);

        info!(
            tool = TOOL_NAME,
            target = %proposal.target,
            change_type = %proposal.change_type,
            policy_projected,
            input_tokens = response.usage.input_tokens,
            output_tokens = response.usage.output_tokens,
            "autonomous self-modification synthesis succeeded"
        );

        ToolExecutionResult::ok(json!({
            "target": proposal.target,
            "change_type": proposal.change_type,
            "rationale": proposal.rationale,
            "proposed_diff": proposal.proposed_diff,
            "policy_projected": policy_projected,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use tau_ai::{ChatResponse, ChatUsage, Message, TauAiError};
    use tokio::runtime::Builder;
    use tokio::sync::Mutex as AsyncMutex;

    // Tests mutate process env; serialize them via the crate-level lock in
    // `lib.rs` so tests across modules (e.g. self_modification_tool) cannot
    // interleave env mutations and produce a torn state.
    use crate::AUTONOMOUS_SELF_MOD_ENV_LOCK as ENV_LOCK;

    fn run_with_env<F, Fut, R>(value: Option<&str>, body: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var(AUTONOMOUS_SELF_MOD_ENV).ok();
        // SAFETY: serialized via ENV_LOCK; no concurrent env mutation inside
        // this test binary can observe a torn state.
        unsafe {
            match value {
                Some(v) => std::env::set_var(AUTONOMOUS_SELF_MOD_ENV, v),
                None => std::env::remove_var(AUTONOMOUS_SELF_MOD_ENV),
            }
        }
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        let result = rt.block_on(body());
        unsafe {
            match prev {
                Some(v) => std::env::set_var(AUTONOMOUS_SELF_MOD_ENV, v),
                None => std::env::remove_var(AUTONOMOUS_SELF_MOD_ENV),
            }
        }
        result
    }

    struct QueueClient {
        responses: AsyncMutex<VecDeque<Result<ChatResponse, TauAiError>>>,
    }

    impl QueueClient {
        fn with_text_response(text: &str) -> Arc<Self> {
            let resp = ChatResponse {
                message: Message::assistant_text(text),
                finish_reason: Some("stop".to_string()),
                usage: ChatUsage::default(),
            };
            Arc::new(Self {
                responses: AsyncMutex::new(VecDeque::from([Ok(resp)])),
            })
        }

        fn with_error(err: TauAiError) -> Arc<Self> {
            Arc::new(Self {
                responses: AsyncMutex::new(VecDeque::from([Err(err)])),
            })
        }
    }

    #[async_trait]
    impl LlmClient for QueueClient {
        async fn complete(
            &self,
            _request: ChatRequest,
        ) -> Result<ChatResponse, TauAiError> {
            let mut q = self.responses.lock().await;
            q.pop_front()
                .unwrap_or(Err(TauAiError::InvalidResponse("queue empty".into())))
        }
    }

    fn tool(client: Arc<dyn LlmClient>) -> SelfModificationSynthesizeTool {
        SelfModificationSynthesizeTool::new(client, "test-model")
    }

    #[test]
    fn definition_uses_canonical_tool_name() {
        let t = tool(QueueClient::with_text_response("{}"));
        assert_eq!(t.definition().name, "self_modification_synthesize");
    }

    #[test]
    fn execute_refuses_when_env_gate_is_off() {
        let result = run_with_env(None, || async {
            let t = tool(QueueClient::with_text_response("{}"));
            t.execute(json!({ "intent": "anything" })).await
        });
        assert!(result.is_error);
        let payload = &result.content;
        assert_eq!(payload["reason_code"], "autonomous_self_mod_disabled");
    }

    #[test]
    fn execute_rejects_empty_intent_when_enabled() {
        let result = run_with_env(Some("1"), || async {
            let t = tool(QueueClient::with_text_response("{}"));
            t.execute(json!({ "intent": "   " })).await
        });
        assert!(result.is_error);
        let payload = &result.content;
        assert_eq!(payload["reason_code"], "invalid_arguments");
    }

    #[test]
    fn execute_returns_structured_proposal_on_happy_path() {
        let llm_reply = r#"{
            "target": "skills/foo.md",
            "change_type": "skill",
            "rationale": "foo skill needs a bar section",
            "proposed_diff": null
        }"#;
        let result = run_with_env(Some("1"), || async {
            let t = tool(QueueClient::with_text_response(llm_reply));
            t.execute(json!({ "intent": "add bar section to foo skill" }))
                .await
        });
        assert!(!result.is_error, "expected success, got: {}", result.content);
        let payload = &result.content;
        assert_eq!(payload["target"], "skills/foo.md");
        assert_eq!(payload["change_type"], "skill");
        assert!(payload["policy_projected"].is_string());
    }

    #[test]
    fn execute_rejects_path_traversal_in_llm_output() {
        let llm_reply = r#"{
            "target": "../../etc/passwd",
            "change_type": "source",
            "rationale": "evil",
            "proposed_diff": null
        }"#;
        let result = run_with_env(Some("1"), || async {
            let t = tool(QueueClient::with_text_response(llm_reply));
            t.execute(json!({ "intent": "hostile intent" })).await
        });
        assert!(result.is_error);
        let payload = &result.content;
        assert_eq!(payload["reason_code"], "invalid_target_path");
    }

    #[test]
    fn execute_rejects_absolute_path_in_llm_output() {
        let llm_reply = r#"{
            "target": "/etc/passwd",
            "change_type": "source",
            "rationale": "also evil",
            "proposed_diff": null
        }"#;
        let result = run_with_env(Some("1"), || async {
            let t = tool(QueueClient::with_text_response(llm_reply));
            t.execute(json!({ "intent": "another hostile intent" })).await
        });
        assert!(result.is_error);
        let payload = &result.content;
        assert_eq!(payload["reason_code"], "invalid_target_path");
    }

    #[test]
    fn execute_rejects_invalid_change_type() {
        let llm_reply = r#"{
            "target": "skills/foo.md",
            "change_type": "wingding",
            "rationale": "r",
            "proposed_diff": null
        }"#;
        let result = run_with_env(Some("1"), || async {
            let t = tool(QueueClient::with_text_response(llm_reply));
            t.execute(json!({ "intent": "x" })).await
        });
        assert!(result.is_error);
        let payload = &result.content;
        assert_eq!(payload["reason_code"], "invalid_change_type");
    }

    #[test]
    fn execute_surfaces_malformed_json_as_llm_output_malformed() {
        let result = run_with_env(Some("1"), || async {
            let t = tool(QueueClient::with_text_response("not json at all"));
            t.execute(json!({ "intent": "x" })).await
        });
        assert!(result.is_error);
        let payload = &result.content;
        assert_eq!(payload["reason_code"], "llm_output_malformed");
    }

    #[test]
    fn execute_surfaces_llm_transport_error() {
        let result = run_with_env(Some("1"), || async {
            let t = tool(QueueClient::with_error(
                TauAiError::InvalidResponse("upstream 503".into()),
            ));
            t.execute(json!({ "intent": "x" })).await
        });
        assert!(result.is_error);
        let payload = &result.content;
        assert_eq!(payload["reason_code"], "llm_call_failed");
    }

    #[test]
    fn execute_tolerates_markdown_fenced_json() {
        let llm_reply = "```json\n{\n  \"target\": \"skills/a.md\",\n  \"change_type\": \"skill\",\n  \"rationale\": \"r\",\n  \"proposed_diff\": null\n}\n```";
        let result = run_with_env(Some("1"), || async {
            let t = tool(QueueClient::with_text_response(llm_reply));
            t.execute(json!({ "intent": "x" })).await
        });
        assert!(!result.is_error, "expected success, got: {}", result.content);
    }
}
