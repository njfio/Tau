//! Session runtime orchestration for OpenResponses requests, response streaming, and persistence.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tau_agent_core::{extract_skip_response_reason, Agent, AgentCostSnapshot};
use tau_ai::{Message, MessageRole};
use tau_session::{SessionStore, SessionUsageSummary};

#[derive(Debug)]
pub(super) struct SessionRuntime {
    pub(super) store: SessionStore,
    pub(super) active_head: Option<u64>,
}

pub(super) fn persist_messages(
    session_runtime: &mut Option<SessionRuntime>,
    new_messages: &[Message],
) -> Result<()> {
    let Some(runtime) = session_runtime.as_mut() else {
        return Ok(());
    };

    runtime.active_head = runtime
        .store
        .append_messages(runtime.active_head, new_messages)?;
    Ok(())
}

pub(super) fn persist_session_usage_delta(
    session_runtime: &mut Option<SessionRuntime>,
    pre_prompt_cost: &AgentCostSnapshot,
    post_prompt_cost: &AgentCostSnapshot,
) -> Result<()> {
    let Some(runtime) = session_runtime.as_mut() else {
        return Ok(());
    };

    let delta = SessionUsageSummary {
        input_tokens: post_prompt_cost
            .input_tokens
            .saturating_sub(pre_prompt_cost.input_tokens),
        output_tokens: post_prompt_cost
            .output_tokens
            .saturating_sub(pre_prompt_cost.output_tokens),
        total_tokens: post_prompt_cost
            .total_tokens
            .saturating_sub(pre_prompt_cost.total_tokens),
        estimated_cost_usd: (post_prompt_cost.estimated_cost_usd
            - pre_prompt_cost.estimated_cost_usd)
            .max(0.0),
    };
    runtime.store.record_usage_delta(delta)
}

pub(super) fn gateway_session_path(state_dir: &Path, session_key: &str) -> PathBuf {
    state_dir
        .join("openresponses")
        .join("sessions")
        .join(format!("{session_key}.jsonl"))
}

pub(super) fn initialize_gateway_session_runtime(
    session_path: &Path,
    system_prompt: &str,
    lock_wait_ms: u64,
    lock_stale_ms: u64,
    agent: &mut Agent,
) -> Result<SessionRuntime> {
    if let Some(parent) = session_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }
    let mut store = SessionStore::load(session_path)?;
    store.set_lock_policy(lock_wait_ms.max(1), lock_stale_ms);
    let active_head = store.ensure_initialized(system_prompt)?;
    let lineage = store.lineage_messages(active_head)?;
    if !lineage.is_empty() {
        agent.replace_messages(lineage);
    }
    Ok(SessionRuntime { store, active_head })
}

pub(super) fn collect_assistant_reply(messages: &[Message]) -> String {
    if extract_skip_response_reason(messages).is_some() {
        return String::new();
    }
    let content = messages
        .iter()
        .filter(|message| message.role == MessageRole::Assistant)
        .map(Message::text_content)
        .filter(|text| !text.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    if content.trim().is_empty() {
        "I couldn't generate a textual response for this request.".to_string()
    } else {
        content
    }
}

#[cfg(test)]
mod tests {
    use super::collect_assistant_reply;
    use tau_ai::Message;

    #[test]
    fn integration_spec_c05_collect_assistant_reply_suppresses_output_when_skip_tool_result_present(
    ) {
        let messages = vec![Message::tool_result(
            "call_skip_1",
            "skip",
            r#"{"skip_response":true,"reason":"already acknowledged","reason_code":"skip_suppressed"}"#,
            false,
        )];
        let reply = collect_assistant_reply(&messages);
        assert!(reply.is_empty());
    }
}
