use crate::{collapse_whitespace, runtime_safety_memory::assistant_text_suggests_failure};
use tau_ai::{Message, MessageRole};

const IMPLEMENTATION_ACTION_TERMS: &[&str] = &[
    "create",
    "build",
    "implement",
    "scaffold",
    "make",
    "develop",
    "fix",
    "add",
    "edit",
    "update",
    "refactor",
];

const IMPLEMENTATION_TARGET_TERMS: &[&str] = &[
    "game",
    "app",
    "application",
    "site",
    "website",
    "page",
    "component",
    "feature",
    "ui",
    "scene",
    "phaser",
    "phaserjs",
    "project",
    "prototype",
    "script",
];

const IMPLEMENTATION_PROGRESS_MARKERS: &[&str] = &[
    "going well",
    "core systems are in place",
    "systems are in place",
    "implemented",
    "built",
    "created",
    "scaffolded",
    "wired up",
    "hooked up",
    "finishing",
    "wrapping up",
    "completed",
];

const IMPLEMENTATION_COMPLETION_MARKERS: &[&str] = &[
    "built",
    "implemented",
    "created",
    "completed",
    "finished",
    "done",
    "ready",
    "playable",
];

const MUTATING_TOOL_NAMES: &[&str] = &["write", "edit"];

pub(crate) fn assistant_text_suggests_unverified_implementation_progress(
    user_prompt: &str,
    assistant_text: &str,
) -> bool {
    let normalized_prompt = collapse_whitespace(&user_prompt.to_lowercase());
    let normalized_assistant = collapse_whitespace(&assistant_text.to_lowercase());
    if normalized_prompt.trim().is_empty() || normalized_assistant.trim().is_empty() {
        return false;
    }
    if assistant_text_suggests_failure(&normalized_assistant) {
        return false;
    }
    if !user_prompt_requests_workspace_implementation(&normalized_prompt) {
        return false;
    }
    assistant_claims_implementation_progress(&normalized_assistant)
}

pub(crate) fn assistant_text_suggests_unverified_implementation_completion(
    user_prompt: &str,
    assistant_text: &str,
) -> bool {
    let normalized_prompt = collapse_whitespace(&user_prompt.to_lowercase());
    let normalized_assistant = collapse_whitespace(&assistant_text.to_lowercase());
    if normalized_prompt.trim().is_empty() || normalized_assistant.trim().is_empty() {
        return false;
    }
    if assistant_text_suggests_failure(&normalized_assistant) {
        return false;
    }
    if !user_prompt_requests_workspace_implementation(&normalized_prompt) {
        return false;
    }
    assistant_claims_implementation_completion(&normalized_assistant)
}

pub(crate) fn messages_include_successful_mutating_tool_result(messages: &[Message]) -> bool {
    messages.iter().any(|message| {
        message.role == MessageRole::Tool
            && !message.is_error
            && message
                .tool_name
                .as_deref()
                .is_some_and(|tool_name| MUTATING_TOOL_NAMES.contains(&tool_name))
    })
}

fn user_prompt_requests_workspace_implementation(normalized_prompt: &str) -> bool {
    let has_action = IMPLEMENTATION_ACTION_TERMS
        .iter()
        .any(|term| normalized_prompt.contains(term));
    has_action
        && IMPLEMENTATION_TARGET_TERMS
            .iter()
            .any(|term| normalized_prompt.contains(term))
}

fn assistant_claims_implementation_progress(normalized_assistant: &str) -> bool {
    IMPLEMENTATION_PROGRESS_MARKERS
        .iter()
        .any(|term| normalized_assistant.contains(term))
}

fn assistant_claims_implementation_completion(normalized_assistant: &str) -> bool {
    IMPLEMENTATION_COMPLETION_MARKERS
        .iter()
        .any(|term| normalized_assistant.contains(term))
}
