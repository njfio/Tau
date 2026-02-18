# Spec #2523 - Epic: G14 send-file tool-contract closure + validation

Status: Implemented

## Problem Statement
Tau has `/tau send-file` command handling in multi-channel runtime, but no built-in `send_file` tool contract in `tau-tools`. This prevents deterministic LLM tool-call-based file-delivery intent and consistent suppression semantics.

## Acceptance Criteria
### AC-1 Tool contract exists
Given built-in tool registration, when names are enumerated, then `send_file` is present with stable JSON schema and deterministic response payload.

### AC-2 Turn suppression for send-file directives
Given a successful `send_file` tool execution during a prompt run, when the run completes, then no follow-up assistant text reply is emitted for that turn.

### AC-3 Diagnostics preserve file-delivery intent
Given a successful `send_file` directive, when outbound event diagnostics are written, then payload includes file metadata and stable reason code.

### AC-4 Existing send-file command delivery remains green
Given existing `/tau send-file` runtime conformance coverage, when tests run, then dispatch behavior remains unchanged.

## Scope
In scope:
- `tau-tools` send_file tool contract + registration.
- `tau-agent-core` send-file directive extraction and suppression wiring.
- `tau-coding-agent` send-file diagnostics payload.
- Verification against existing multi-channel send-file command coverage.

Out of scope:
- New transport providers.

## Conformance Cases
- C-01 (AC-1, functional): `spec_2525_c01_builtin_agent_tool_name_registry_includes_send_file_tool`
- C-02 (AC-1, functional): `spec_2525_c02_send_file_tool_returns_structured_file_delivery_payload`
- C-03 (AC-2, integration): `integration_spec_2525_c03_prompt_send_file_tool_call_terminates_run_without_follow_up_model_turn`
- C-04 (AC-2, unit): `spec_2525_c04_extract_send_file_request_detects_valid_send_file_tool_payload`
- C-05 (AC-3, integration): `integration_spec_2525_c05_runner_persists_send_file_payload_and_suppresses_text_reply`
- C-06 (AC-4, functional): `functional_runner_executes_tau_send_file_command_and_records_delivery`

## Success Metrics
- All C-01..C-06 pass.
- Scoped mutation in touched code paths has no missed mutants.
- Live validation script passes.
