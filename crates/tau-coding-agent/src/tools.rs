//! Built-in tool facade for coding-agent command/runtime use.
//!
//! Re-exports tool types and registration helpers from `tau-tools` to keep
//! tool dispatch contracts uniform across startup and runtime loops.
//!
//! The coding-agent ships a *superset* of tau-tools' built-in registry: in
//! addition to the generic tools, agents constructed via this module's
//! [`register_builtin_tools`] also receive
//! [`crate::self_modification_tool::SelfModificationProposeTool`]. That tool
//! is fail-closed — `execute()` refuses with
//! `reason_code: "autonomous_self_mod_disabled"` unless the operator sets
//! `TAU_AUTONOMOUS_SELF_MOD=1`. Visible-but-refused is deliberate: the model
//! observes a structured error instead of silently missing capability.

pub use tau_tools::tools::*;

use std::sync::Arc;

use tau_agent_core::Agent;
use tau_ai::LlmClient;

use tau_coding_agent::self_modification_synthesis_tool::SelfModificationSynthesizeTool;
use tau_coding_agent::self_modification_tool::SelfModificationProposeTool;

/// Registers the full coding-agent built-in tool set on `agent`.
///
/// Thin wrapper around [`tau_tools::tools::register_builtin_tools`] that
/// *optionally* also registers [`SelfModificationProposeTool`].
///
/// The self-modification tool is gated by two independent switches:
///
/// 1. **Policy flag** ([`ToolPolicy::self_modification_propose_enabled`],
///    defaults to `false`). Controls *registration*: when false, the tool
///    is not registered on the agent at all — the model never sees it.
/// 2. **Runtime env gate** (`TAU_AUTONOMOUS_SELF_MOD=1`). Controls
///    *invocation*: when unset the tool refuses with
///    `reason_code: "autonomous_self_mod_disabled"` even if registered.
///
/// This two-layer design lets operators:
/// - Disable autonomous self-modification completely (policy=false).
/// - Expose the tool in an "advertised but refusing" state (policy=true,
///   env unset) so the model can learn the surface without being able to
///   actually invoke changes.
/// - Fully enable autonomous dry-runs (policy=true, env=1).
///
/// This function shadows the glob-imported
/// `tau_tools::tools::register_builtin_tools` name. Call sites that want the
/// *base* tau-tools registry without the coding-agent superset should refer
/// to it by its fully-qualified path.
pub fn register_builtin_tools(agent: &mut Agent, policy: ToolPolicy) {
    let self_mod_enabled = policy.self_modification_propose_enabled;
    tau_tools::tools::register_builtin_tools(agent, policy);
    if self_mod_enabled {
        agent.register_tool(SelfModificationProposeTool::new());
    }
}

/// Registers the `self_modification_synthesize` tool on `agent`.
///
/// Unlike [`register_builtin_tools`] which is a drop-in replacement for the
/// base tau-tools registry, this helper must be called *separately* because
/// synthesis requires an [`LlmClient`] and a model string. Call sites that
/// have those in scope (e.g. the coding-agent runtime in `events.rs`) should
/// invoke this immediately after `register_builtin_tools`.
///
/// Both gates apply: registration is skipped when
/// `policy.self_modification_synthesize_enabled` is false, and at execute
/// time the tool refuses unless `TAU_AUTONOMOUS_SELF_MOD=1`.
pub fn register_self_modification_synthesis(
    agent: &mut Agent,
    policy: &ToolPolicy,
    client: Arc<dyn LlmClient>,
    model: impl Into<String>,
) {
    if policy.self_modification_synthesize_enabled {
        agent.register_tool(SelfModificationSynthesizeTool::new(client, model));
    }
}
