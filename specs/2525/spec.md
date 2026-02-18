# Spec #2525 - Task: implement and validate G14 SendFileTool pathway

Status: Implemented

## Problem Statement
G14 checklist remains open because there is no built-in `send_file` tool contract in `tau-tools`, despite existing command-path delivery behavior.

## Acceptance Criteria
### AC-1 send_file tool registry and schema
Given built-in tools are registered, when names are enumerated, then `send_file` is present; when executed with valid args, then response includes normalized file metadata and reason code.

### AC-2 turn suppression behavior
Given successful `send_file` tool result, when prompt run completes, then run terminates without follow-up assistant text.

### AC-3 outbound diagnostics
Given successful send-file directive, when event outbound payload is recorded, then payload includes send-file metadata and explicit reason code.

### AC-4 existing command compatibility
Given existing multi-channel send-file command tests, when executed, then behavior remains green.

## Scope
In scope:
- Add `SendFileTool` to `tau-tools` and register as built-in.
- Add extraction + suppression handling in `tau-agent-core`.
- Add send-file metadata to `tau-coding-agent` outbound event payload.
- Verify against existing multi-channel send-file command tests.

Out of scope:
- Net-new transport implementations.

## Conformance Cases
- C-01 (AC-1, functional): `spec_2525_c01_builtin_agent_tool_name_registry_includes_send_file_tool`
- C-02 (AC-1, functional): `spec_2525_c02_send_file_tool_returns_structured_file_delivery_payload`
- C-03 (AC-2, integration): `integration_spec_2525_c03_prompt_send_file_tool_call_terminates_run_without_follow_up_model_turn`
- C-04 (AC-2, unit): `spec_2525_c04_extract_send_file_request_detects_valid_send_file_tool_payload`
- C-05 (AC-3, integration): `integration_spec_2525_c05_runner_persists_send_file_payload_and_suppresses_text_reply`
- C-06 (AC-4, functional): `functional_runner_executes_tau_send_file_command_and_records_delivery`

## Success Metrics
- All C-01..C-06 pass.
- Diff-scoped mutation has zero missed mutants.
- Live validation script passes.
