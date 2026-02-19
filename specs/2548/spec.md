# Spec #2548 - Task: prompt-template hot-reload bridge for local runtime turns

Status: Implemented

## Problem Statement
Startup prompt templates are currently composed once during startup. Workspace edits to prompt templates require runtime restart and do not satisfy Spacebot-comparison `G17` hot-reload expectations.

## Acceptance Criteria
### AC-1 Local runtime applies workspace template updates without restart
Given local runtime is running with workspace startup prompt templates, when template content changes, then subsequent prompt turns use the recomposed system prompt without restarting the process.

### AC-2 Invalid template edits fail closed
Given a valid system prompt is active, when a template edit introduces invalid template content, then runtime keeps the last-known-good system prompt and emits deterministic invalid-update diagnostics.

### AC-3 No-op edits do not churn system prompt state
Given template source resolves to the same effective system prompt, when bridge evaluation runs, then runtime does not rewrite/apply a new system prompt and emits a deterministic no-change outcome.

### AC-4 Bridge lifecycle integrates with local runtime start/stop
Given local runtime starts with prompt hot-reload enabled, when runtime shuts down, then bridge task exits cleanly without leaked background work.

## Scope
In scope:
- `crates/tau-coding-agent` local runtime bridge/wiring for prompt-template hot reload.
- System prompt update path for runtime turns.
- Deterministic diagnostics + conformance/regression tests.

Out of scope:
- Non-local runtime transports.
- Template language/variable-surface expansion.

## Conformance Cases
- C-01 (AC-1): `integration_spec_2548_c01_prompt_template_hot_reload_applies_updated_system_prompt_without_restart`
- C-02 (AC-2): `regression_spec_2548_c02_prompt_template_invalid_update_preserves_last_good_system_prompt`
- C-03 (AC-3): `regression_spec_2548_c03_prompt_template_noop_update_emits_no_change_without_reapply`
- C-04 (AC-4): `integration_spec_2548_c04_prompt_template_hot_reload_bridge_start_and_shutdown_are_clean`

## Success Metrics
- C-01..C-04 pass.
- `cargo fmt --check`, `cargo clippy -- -D warnings`, scoped `tau-coding-agent` tests, mutation in diff, live validation, and workspace `cargo test -j 1` pass.
