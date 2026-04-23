//! End-to-end composability test for the autonomous self-modification
//! tool pair.
//!
//! Exercises the chain:
//!   LLM → SelfModificationSynthesizeTool → SelfModificationProposeTool
//!
//! Both tools are driven via their public `AgentTool::execute` surface; the
//! synthesis call uses a fake `LlmClient` that returns a canned proposal.
//! The propose tool's output is asserted to be a structured
//! `SelfModificationResult` consistent with the target the synthesis tool
//! produced. This closes the "each tool passes in isolation, but are they
//! actually composable?" gap.
//!
//! This is an integration test (lives under `tests/`) so it exercises only
//! the public crate surface — catches any accidental
//! `pub(crate)` slip that would break external consumers.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{json, Value};
use tau_agent_core::AgentTool;
use tau_ai::{
    ChatRequest, ChatResponse, ChatUsage, LlmClient, Message, TauAiError,
};
use tau_coding_agent::self_modification_synthesis_tool::SelfModificationSynthesizeTool;
use tau_coding_agent::self_modification_tool::{
    SelfModificationProposeTool, AUTONOMOUS_SELF_MOD_ENV,
};
use tempfile::TempDir;
use tokio::sync::Mutex as AsyncMutex;

/// Tests in this file mutate `TAU_AUTONOMOUS_SELF_MOD`. The in-binary
/// modules in the same crate already coordinate via the lib-level
/// `AUTONOMOUS_SELF_MOD_ENV_LOCK`, but that static is `pub(crate)` and is
/// therefore NOT reachable from this integration test binary. So this file
/// uses its own process-local lock AND runs with `#[test]` (single-threaded
/// per test) — acceptable because only one test here touches the env var.
static LOCAL_ENV_LOCK: Mutex<()> = Mutex::new(());

struct FakeLlm {
    responses: AsyncMutex<VecDeque<Result<ChatResponse, TauAiError>>>,
}

impl FakeLlm {
    fn with_text(text: &str) -> Arc<Self> {
        let resp = ChatResponse {
            message: Message::assistant_text(text),
            finish_reason: Some("stop".to_string()),
            usage: ChatUsage::default(),
        };
        Arc::new(Self {
            responses: AsyncMutex::new(VecDeque::from([Ok(resp)])),
        })
    }
}

#[async_trait]
impl LlmClient for FakeLlm {
    async fn complete(
        &self,
        _request: ChatRequest,
    ) -> Result<ChatResponse, TauAiError> {
        let mut q = self.responses.lock().await;
        q.pop_front()
            .unwrap_or(Err(TauAiError::InvalidResponse("empty queue".into())))
    }
}

fn with_env_set<F: FnOnce() -> R, R>(value: &str, body: F) -> R {
    let _guard = LOCAL_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let prev = std::env::var(AUTONOMOUS_SELF_MOD_ENV).ok();
    // SAFETY: serialized via LOCAL_ENV_LOCK; no other test in this binary
    // mutates this env var concurrently.
    unsafe {
        std::env::set_var(AUTONOMOUS_SELF_MOD_ENV, value);
    }
    let result = body();
    unsafe {
        match prev {
            Some(v) => std::env::set_var(AUTONOMOUS_SELF_MOD_ENV, v),
            None => std::env::remove_var(AUTONOMOUS_SELF_MOD_ENV),
        }
    }
    result
}

#[test]
fn synthesize_output_is_directly_consumable_by_propose_tool() {
    let workspace = TempDir::new().expect("tempdir");
    let workspace_str = workspace.path().to_string_lossy().to_string();

    let llm_reply = r#"{
        "target": "skills/autonomy/improve-retry-loop.md",
        "change_type": "skill",
        "rationale": "retry logic needs explicit backoff guidance",
        "proposed_diff": null
    }"#;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");

    let (synthesized_target, propose_result) = with_env_set("1", || {
        rt.block_on(async {
            // Step 1: synthesis tool produces a structured proposal.
            let synth = SelfModificationSynthesizeTool::new(
                FakeLlm::with_text(llm_reply),
                "fake-model",
            );
            let synth_result = synth
                .execute(json!({
                    "intent": "Improve the retry loop in the autonomy skill.",
                    "workspace_root": workspace_str,
                }))
                .await;
            assert!(
                !synth_result.is_error,
                "synthesis must succeed: {}",
                synth_result.content
            );

            // Extract the target synthesis produced and feed it to propose.
            let target = synth_result.content["target"]
                .as_str()
                .expect("synthesis output has a target string")
                .to_string();

            // Step 2: propose tool dry-runs against the same workspace.
            let propose = SelfModificationProposeTool::new();
            let propose_result = propose
                .execute(json!({
                    "target": target,
                    "workspace_root": workspace_str,
                }))
                .await;

            (target, propose_result)
        })
    });

    assert_eq!(synthesized_target, "skills/autonomy/improve-retry-loop.md");
    assert!(
        !propose_result.is_error,
        "propose must accept synthesis output: {}",
        propose_result.content
    );

    // The propose tool should classify this as a skill target, which is the
    // auto-apply-allowed category in the default config.
    let safety: &Value = &propose_result.content["safety_evaluation"];
    assert_eq!(
        safety["allowed"],
        Value::Bool(true),
        "skill target must be allowed by default config: {}",
        propose_result.content
    );
    assert_eq!(
        propose_result.content["applied"],
        Value::Bool(false),
        "propose tool must never apply (dry-run only)"
    );
}

#[test]
fn synthesize_output_with_would_be_blocked_target_still_chains_cleanly_to_propose() {
    // When synthesis produces a target whose projected policy is
    // "would_be_blocked" (e.g. a source file with default config), the
    // propose tool should still execute the pipeline and return a
    // structured denial — no panic, no silent drop.
    let workspace = TempDir::new().expect("tempdir");
    let workspace_str = workspace.path().to_string_lossy().to_string();

    let llm_reply = r#"{
        "target": "crates/tau-ops/src/main.rs",
        "change_type": "source",
        "rationale": "add a new CLI flag",
        "proposed_diff": null
    }"#;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");

    let propose_result = with_env_set("1", || {
        rt.block_on(async {
            let synth = SelfModificationSynthesizeTool::new(
                FakeLlm::with_text(llm_reply),
                "fake-model",
            );
            let synth_result = synth
                .execute(json!({
                    "intent": "Add a --dry-run flag to tau-ops.",
                    "workspace_root": workspace_str,
                }))
                .await;
            assert!(!synth_result.is_error);
            assert_eq!(
                synth_result.content["policy_projected"],
                "would_be_blocked",
                "source target must project as blocked under default config"
            );

            let target = synth_result.content["target"]
                .as_str()
                .expect("target present")
                .to_string();
            let propose = SelfModificationProposeTool::new();
            propose
                .execute(json!({
                    "target": target,
                    "workspace_root": workspace_str,
                }))
                .await
        })
    });

    // Propose does NOT return is_error for a policy-denied target; it
    // returns a successful response whose safety_evaluation.allowed is
    // false. This is the documented contract.
    assert!(!propose_result.is_error);
    assert_eq!(
        propose_result.content["safety_evaluation"]["allowed"],
        Value::Bool(false),
        "source target must be denied"
    );
    let blocked_by = propose_result.content["safety_evaluation"]["blocked_by"]
        .as_array()
        .expect("blocked_by array");
    assert!(
        blocked_by
            .iter()
            .any(|v| v.as_str() == Some("auto_apply_source_disabled")),
        "expected auto_apply_source_disabled reason; got {blocked_by:?}"
    );
}
