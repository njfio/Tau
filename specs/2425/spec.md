# Spec: Issue #2425 - G12 phase-2 SkipTool implementation and validation

Status: Implemented

## Problem Statement
Tau supports `/tau skip` command suppression in `tau-multi-channel`, but does not expose a
first-class built-in `skip` tool that the model can invoke through normal tool-calling flows.
This leaves G12 phase-2 incomplete: the agent cannot explicitly end a turn and suppress outbound
text through tool orchestration.

## Acceptance Criteria

### AC-1 Built-in tool registry includes `skip`
Given Tau registers built-in tools via `register_builtin_tools`,
When the tool catalog is inspected,
Then `skip` is present in `builtin_agent_tool_names()` and can be resolved as a registered tool.

### AC-2 `skip` tool returns a structured suppression payload
Given the model invokes `skip` with optional `reason`,
When the tool executes,
Then it returns a success payload that explicitly marks response suppression and includes the
normalized reason for audit/debug use.

### AC-3 Agent run loop terminates after successful `skip` tool directive
Given an assistant message that requests `skip`,
When the tool call executes successfully with suppression marker,
Then the agent loop ends the run without requesting another model turn.

### AC-4 User-facing output collectors suppress textual fallback on skip directive
Given new messages include a successful `skip` tool result marker,
When user-facing reply collectors/renderers run,
Then they suppress textual output rather than emitting fallback strings.

### AC-5 Existing `/tau skip` command path remains functional
Given current command-mode skip behavior in `tau-multi-channel`,
When regression tests execute,
Then `/tau skip` suppression behavior remains unchanged.

## Scope

### In Scope
- Add `skip` built-in tool implementation in `tau-tools`.
- Add skip directive detection in `tau-agent-core` run loop.
- Add shared skip directive extraction helpers for output suppression.
- Update gateway/runtime output collectors to suppress text for skip directives.
- Add conformance/regression tests across touched crates.

### Out of Scope
- Replacing `/tau skip` command behavior with tool-only behavior.
- New provider integrations or routing policy changes.
- G13/G14 reaction/file tool orchestration.

## Conformance Cases
- C-01 (AC-1, unit): `spec_c01_builtin_agent_tool_name_registry_includes_skip_tool`
- C-02 (AC-2, functional): `spec_c02_skip_tool_returns_structured_suppression_payload`
- C-03 (AC-3, integration): `integration_spec_c03_prompt_skip_tool_call_terminates_run_without_follow_up_model_turn`
- C-04 (AC-4, unit): `spec_c04_extract_skip_response_reason_detects_valid_skip_tool_payload`
- C-05 (AC-4, integration): `integration_spec_c05_collect_assistant_reply_suppresses_output_when_skip_tool_result_present`
- C-06 (AC-5, regression): `functional_runner_executes_tau_skip_command_without_outbound_delivery`

## Success Metrics / Observable Signals
- Conformance tests C-01..C-06 pass.
- `cargo fmt --check`, scoped `clippy`, and scoped crate tests pass.
- No regression in existing `/tau skip` command behavior.
